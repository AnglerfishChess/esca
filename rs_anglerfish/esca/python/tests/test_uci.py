"""Talking to an engine, over the scripted doubles in `tests/fixtures/`.

The cases mirror `tests/uci_engine.rs`: a normal game, a slow engine, one
writing garbage, one dying mid-search, one that never answers, and the two
Chess960 handshakes.
"""

from __future__ import annotations

import ast
import asyncio
import shutil
import sys
from collections.abc import Iterator, Sequence
from pathlib import Path

import esca
import pytest
from esca import uci

#: The scripted engine double, shared with the Rust tests.
FAKE = Path(__file__).resolve().parents[2] / "tests" / "fixtures" / "fake_engine.py"

#: Long enough for a subprocess to answer, short enough to fail a test fast.
TIMEOUT = 5.0

#: A Chess960 endgame: the white king on b1 with its own rook beside it on c1.
BESIDE_ROOK = "4k3/8/8/8/8/8/8/1KR5 w C - 0 1"


def fake(*flags: str, timeout: float = TIMEOUT) -> uci.Engine:
    """A started double, misbehaving as the flags ask."""
    return uci.Engine(sys.executable, [str(FAKE), *flags], timeout=timeout)


@pytest.fixture
def engine() -> Iterator[uci.Engine]:
    """A double that has identified itself."""
    with fake() as started:
        started.handshake()
        yield started


def identified(*flags: str, timeout: float = TIMEOUT) -> uci.Engine:
    """A started double that has identified itself."""
    started = fake(*flags, timeout=timeout)
    started.handshake()
    return started


def log_of(path: Path) -> list[str]:
    """The commands a double wrote to its log, in order."""
    return path.read_text(encoding="utf-8").splitlines() if path.exists() else []


# -- A normal game ----------------------------------------------------------


def test_an_engine_names_itself_and_lists_what_it_offers(engine: uci.Engine) -> None:
    assert engine.name == "Fake Engine 1.0"
    assert engine.author == "The esca test suite"
    assert engine.state == "idle"

    hash_option = engine.options["Hash"]
    assert (hash_option.type, hash_option.default, hash_option.min, hash_option.max) == (
        "spin",
        16,
        1,
        1024,
    )
    assert engine.options["Style"].vars == ["Solid", "Wild"]
    assert engine.options["Clear Hash"].default is None
    # Engines match option names without regard to case, and so does esca.
    assert engine.option("hash") is not None


def test_a_search_reports_and_then_answers(engine: uci.Engine) -> None:
    engine.new_game()
    game = esca.Game()
    engine.set_position(game)

    search = engine.go(uci.Limits(depth=2))
    reports = list(search)
    assert [report.depth for report in reports] == [1, 2, None]
    assert [move.uci for move in reports[0].pv] == ["e2e4"]
    assert [move.uci for move in reports[1].pv] == ["e2e4", "e7e5"]
    assert reports[1].cp == 25
    assert reports[1].bound == "lowerbound"
    assert reports[2].string == "thinking about it: hard"

    answer = search.answer()
    assert answer.best is not None
    assert game.move_to_san(answer.best) == "e4"
    assert answer.ponder is not None
    assert answer.ponder.uci == "e7e5"
    assert engine.state == "idle"


def test_playing_sets_the_position_and_answers(engine: uci.Engine) -> None:
    answer = engine.play(esca.Game(), uci.Limits(movetime=0.01))
    assert answer.best is not None
    assert answer.best.uci == "e2e4"


def test_analysing_keeps_the_deepest_report_of_each_variation(engine: uci.Engine) -> None:
    reports = engine.analyse(esca.Game(), uci.Limits(depth=2))
    assert len(reports) == 1
    assert reports[0].depth == 2
    assert [move.uci for move in reports[0].pv] == ["e2e4", "e7e5"]


def test_an_engine_with_no_move_answers_with_none() -> None:
    with identified("--no-move") as engine:
        answer = engine.play(esca.Game(), uci.Limits(depth=1))
        assert answer.best is None
        assert answer.ponder is None


def test_a_search_that_waits_is_ended_by_stop(engine: uci.Engine) -> None:
    engine.set_position(esca.Game())
    search = engine.go(uci.Limits(infinite=True))
    search.stop()
    assert search.answer().best is not None


def test_a_ponder_becomes_a_search_on_a_ponderhit(engine: uci.Engine) -> None:
    engine.set_position(esca.Game())
    search = engine.go(uci.Limits(infinite=True, ponder=True))
    assert engine.state == "pondering"
    search.ponderhit()
    assert search.answer().best is not None


def test_leaving_a_search_stops_it_and_leaves_the_engine_idle(engine: uci.Engine) -> None:
    engine.set_position(esca.Game())
    with engine.go(uci.Limits(infinite=True)):
        pass
    assert engine.state == "idle"
    engine.is_ready()


def test_quitting_reaps_the_process() -> None:
    engine = identified()
    assert engine.quit() == 0
    assert not engine.is_alive


def test_an_engine_that_will_not_quit_is_killed() -> None:
    engine = identified("--zombie")
    engine.timeout = 0.3
    engine.quit()
    assert not engine.is_alive


# -- Unhappy paths ----------------------------------------------------------


def test_a_silent_engine_times_out_rather_than_hangs() -> None:
    with identified("--no-readyok") as engine:
        engine.timeout = 0.2
        with pytest.raises(uci.EngineTimeout):
            engine.is_ready()


def test_an_engine_that_never_ends_its_identification_times_out() -> None:
    with fake("--no-uciok", timeout=1.0) as engine, pytest.raises(uci.EngineTimeout):
        engine.handshake()


def test_a_search_that_outlasts_its_budget_is_a_timeout() -> None:
    with identified("--slow") as engine:
        engine.set_position(esca.Game())
        search = engine.go(uci.Limits(depth=2), timeout=0.05)
        with pytest.raises(uci.EngineTimeout):
            search.answer()


def test_a_slow_engine_answers_within_a_budget_that_fits() -> None:
    with identified("--slow") as engine:
        assert engine.play(esca.Game(), uci.Limits(depth=2)).best is not None


def test_garbage_never_derails_a_search() -> None:
    with identified("--garbage") as engine:
        assert engine.play(esca.Game(), uci.Limits(depth=2)).best is not None
        engine.is_ready()


def test_an_answer_too_many_is_ignored() -> None:
    with identified("--twice") as engine:
        engine.play(esca.Game(), uci.Limits(depth=1))
        engine.is_ready()
        assert engine.state == "idle"


def test_an_engine_that_dies_mid_search_is_reported_as_dead() -> None:
    with identified("--die-on-go") as engine:
        engine.set_position(esca.Game())
        search = engine.go(uci.Limits(depth=2))
        with pytest.raises(uci.EngineDied):
            search.answer()
        # Every call after says the same thing.
        with pytest.raises(uci.EngineDied):
            engine.is_ready()
        assert not engine.is_alive


def test_a_command_the_conversation_has_no_room_for_is_refused(engine: uci.Engine) -> None:
    with pytest.raises(uci.ProtocolError):
        engine.stop()
    engine.is_ready()


def test_every_error_is_a_uci_error() -> None:
    for error in (uci.EngineTimeout, uci.EngineDied, uci.ProtocolError):
        assert issubclass(error, uci.UciError)


# -- Options ----------------------------------------------------------------


def test_an_option_is_set_by_the_name_the_engine_declared(tmp_path: Path) -> None:
    log = tmp_path / "options.log"
    with fake(f"--log={log}") as engine:
        engine.handshake()
        engine.set_option("multipv", 3)
        engine.set_option("Clear Hash")
        engine.set_option("Debug Log File", "")
        engine.is_ready()
        sent = log_of(log)

    assert "setoption name MultiPV value 3" in sent
    assert "setoption name Clear Hash" in sent
    assert "setoption name Debug Log File value <empty>" in sent


@pytest.mark.parametrize(
    ("name", "value", "complaint"),
    [
        ("Contempt", 10, "no option"),  # the double offers no Contempt
        ("MultiPV", 99, "outside"),  # outside the declared range
        ("MultiPV", True, "an integer"),  # True is an int in Python, not a count
        ("Style", "Wild wild", "none of"),  # none of the declared vars
        ("Clear Hash", 1, "no value"),  # a button takes no value
    ],
)
def test_a_value_the_engine_did_not_declare_is_refused(
    engine: uci.Engine, name: str, value: object, complaint: str
) -> None:
    with pytest.raises(ValueError, match=complaint):
        engine.set_option(name, value)


# -- Chess960 ---------------------------------------------------------------


def test_a_chess960_game_puts_the_engine_into_chess960(tmp_path: Path) -> None:
    log = tmp_path / "chess960.log"
    with fake(f"--log={log}") as engine:
        engine.handshake()
        game = esca.Game.from_fen(BESIDE_ROOK, variant=esca.CHESS960)
        game.play("b1c1")
        engine.set_position(game)
        engine.is_ready()
        sent = log_of(log)

    assert sent == [
        "uci",
        "setoption name UCI_Chess960 value true",
        "isready",
        f"position fen {BESIDE_ROOK} moves b1c1",
        "isready",
    ]


def test_an_engine_that_cannot_play_chess960_is_refused_the_game() -> None:
    with identified("--no-chess960") as engine:
        game = esca.Game.from_fen(BESIDE_ROOK, variant=esca.CHESS960)
        with pytest.raises(ValueError, match="UCI_Chess960"):
            engine.set_position(game)


def test_a_classic_game_is_sent_without_touching_the_option(tmp_path: Path) -> None:
    log = tmp_path / "classic.log"
    with fake(f"--log={log}") as engine:
        engine.handshake()
        game = esca.Game.from_fen("4k3/8/8/8/8/8/8/4K2R w K - 0 1")
        game.play("e1h1")
        engine.set_position(game)
        engine.is_ready()
        sent = log_of(log)

    assert sent == [
        "uci",
        "position fen 4k3/8/8/8/8/8/8/4K2R w K - 0 1 moves e1g1",
        "isready",
    ]


# -- The raw interface ------------------------------------------------------


def test_lines_can_be_written_and_read_as_they_are() -> None:
    with fake() as engine:
        engine.send_line("uci")
        seen: list[str] = []
        while (line := engine.next_line(TIMEOUT)) is not None:
            seen.append(line)
            if line == "uciok":
                break
        assert seen[0] == "id name Fake Engine 1.0"
        assert seen[-1] == "uciok"


def test_a_read_that_finds_nothing_answers_with_nothing(engine: uci.Engine) -> None:
    assert engine.next_line(0.05) is None


# -- The asyncio surface ----------------------------------------------------


def test_an_async_engine_plays_a_move() -> None:
    async def play() -> tuple[str | None, list[str], str | None]:
        async with uci.AsyncEngine(sys.executable, [str(FAKE)]) as engine:
            await engine.handshake()
            await engine.new_game()
            game = esca.Game()
            await engine.set_position(game)
            search = await engine.go(uci.Limits(depth=2))
            depths = [str(report.depth) async for report in search]
            answer = await search.answer()
            return engine.name, depths, answer.best.uci if answer.best else None

    name, depths, best = asyncio.run(play())
    assert name == "Fake Engine 1.0"
    assert depths == ["1", "2", "None"]
    assert best == "e2e4"


def test_an_async_engine_analyses_several_positions_in_turn() -> None:
    async def analyse() -> list[int | None]:
        async with uci.AsyncEngine(sys.executable, [str(FAKE)]) as engine:
            await engine.handshake()
            return [(await engine.analyse(esca.Game(), uci.Limits(depth=2)))[0].cp for _ in range(3)]

    assert asyncio.run(analyse()) == [25, 25, 25]


def test_an_async_engine_reports_a_death_as_the_blocking_one_does() -> None:
    async def die() -> None:
        engine = uci.AsyncEngine(sys.executable, [str(FAKE), "--die-on-go"])
        try:
            await engine.handshake()
            await engine.set_position(esca.Game())
            search = await engine.go(uci.Limits(depth=2))
            await search.answer()
        finally:
            engine.kill()

    with pytest.raises(uci.EngineDied):
        asyncio.run(die())


# -- The module and its stub ------------------------------------------------


def stub_all(name: str) -> list[str]:
    """The `__all__` a module declares."""
    source = (Path(esca.__file__).resolve().parent / name).read_text()
    for node in ast.walk(ast.parse(source)):
        if isinstance(node, ast.Assign) and any(
            isinstance(target, ast.Name) and target.id == "__all__" for target in node.targets
        ):
            assert isinstance(node.value, ast.List)
            return [ast.literal_eval(element) for element in node.value.elts]
    raise AssertionError(f"{name} declares no __all__")


def test_the_package_offers_the_client_as_a_submodule() -> None:
    assert "uci" in esca.__all__
    assert esca.uci is uci


def test_every_exported_name_exists() -> None:
    assert [name for name in uci.__all__ if not hasattr(uci, name)] == []


def test_the_module_and_its_stub_agree() -> None:
    assert uci.__all__ == stub_all("uci.py")


# -- Real engines -----------------------------------------------------------


def real_engines() -> Sequence[str]:
    """The engines to try: the well-known ones on PATH."""
    found = [shutil.which(name) for name in ("stockfish", "lc0", "anglerfry", "anglerfish")]
    return [path for path in found if path]


@pytest.mark.parametrize("path", real_engines())
def test_a_real_engine_identifies_itself_and_plays(path: str) -> None:
    with uci.Engine(path, timeout=20.0) as engine:
        engine.handshake()
        assert engine.name
        engine.new_game()
        game = esca.Game()
        answer = engine.play(game, uci.Limits(movetime=0.2), timeout=20.0)
        assert answer.best in game.legal_moves()
