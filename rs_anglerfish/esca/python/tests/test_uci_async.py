"""Talking to an engine over asyncio, with the scripted doubles in `tests/fixtures/`.

The cases mirror `tests/uci_tokio.rs` — a normal game, a slow engine, one
writing garbage, one dying mid-search, one that never answers, and the two
Chess960 handshakes — and add what only an async client can be asked: a wait
that is given up on, and a search cancelled mid-flight.
"""

from __future__ import annotations

import asyncio
import functools
import sys
from collections.abc import Callable, Coroutine
from pathlib import Path
from typing import Any

import esca
import pytest
from esca import uci

#: The scripted engine double, shared with the Rust tests.
FAKE = Path(__file__).resolve().parents[2] / "tests" / "fixtures" / "fake_engine.py"

#: How long a double is given to answer: a bound on the machine, not on the
#: engine, since starting an interpreter on a loaded runner takes as long as it
#: takes. A case that is about a wait running out names its own short budget.
TIMEOUT = 60.0

#: A Chess960 endgame: the white king on b1 with its own rook beside it on c1.
BESIDE_ROOK = "4k3/8/8/8/8/8/8/1KR5 w C - 0 1"


def asyncio_test[**P, T](case: Callable[P, Coroutine[Any, Any, T]]) -> Callable[P, T]:
    """Runs one async case on an event loop of its own, as a program would."""

    @functools.wraps(case)
    def run(*args: P.args, **kwargs: P.kwargs) -> T:
        return asyncio.run(case(*args, **kwargs))

    return run


def fake(*flags: str, timeout: float = TIMEOUT) -> uci.AsyncEngine:
    """A double, misbehaving as the flags ask, ready to be spoken to."""
    return uci.AsyncEngine(sys.executable, [str(FAKE), *flags], timeout=timeout)


async def identified(*flags: str, timeout: float = TIMEOUT) -> uci.AsyncEngine:
    """A double that has identified itself."""
    engine = fake(*flags, timeout=timeout)
    await engine.handshake()
    return engine


def log_of(path: Path) -> list[str]:
    """The commands a double wrote to its log, in order."""
    return path.read_text(encoding="utf-8").splitlines() if path.exists() else []


async def until_searching(engine: uci.AsyncEngine) -> None:
    """Waits for the engine to be searching, so a case can act on the search."""
    for _ in range(500):
        if engine.state in ("searching", "pondering"):
            return
        await asyncio.sleep(0.01)
    raise AssertionError(f"the engine is {engine.state}, not searching")


# -- A normal game ----------------------------------------------------------


@asyncio_test
async def test_an_engine_names_itself_and_lists_what_it_offers() -> None:
    async with fake() as engine:
        await engine.handshake()
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


@asyncio_test
async def test_a_search_reports_and_then_answers() -> None:
    async with await identified() as engine:
        await engine.new_game()
        game = esca.Game()
        await engine.set_position(game)

        search = await engine.go(uci.Limits(depth=2))
        reports = [report async for report in search]
        assert [report.depth for report in reports] == [1, 2, None]
        assert [move.uci for move in reports[0].pv] == ["e2e4"]
        assert [move.uci for move in reports[1].pv] == ["e2e4", "e7e5"]
        assert reports[1].cp == 25
        assert reports[1].bound == "lowerbound"
        assert reports[2].string == "thinking about it: hard"

        answer = await search.answer()
        assert answer.best is not None
        assert game.move_to_san(answer.best) == "e4"
        assert answer.ponder is not None
        assert answer.ponder.uci == "e7e5"
        assert engine.state == "idle"


@asyncio_test
async def test_the_moves_played_go_out_with_the_position(tmp_path: Path) -> None:
    log = tmp_path / "moves.log"
    async with fake(f"--log={log}") as engine:
        await engine.handshake()
        game = esca.Game()
        game.play_san("e4")
        game.play_san("e5")
        await engine.set_position(game)
        await engine.is_ready()
        sent = log_of(log)

    assert sent == ["uci", "position startpos moves e2e4 e7e5", "isready"]


@asyncio_test
async def test_playing_sets_the_position_and_answers() -> None:
    async with await identified() as engine:
        answer = await engine.play(esca.Game(), uci.Limits(movetime=0.01))
        assert answer.best is not None
        assert answer.best.uci == "e2e4"


@asyncio_test
async def test_analysing_keeps_the_deepest_report_of_each_variation() -> None:
    async with await identified() as engine:
        reports = await engine.analyse(esca.Game(), uci.Limits(depth=2))
        assert len(reports) == 1
        assert reports[0].depth == 2
        assert [move.uci for move in reports[0].pv] == ["e2e4", "e7e5"]


@asyncio_test
async def test_an_engine_analyses_several_positions_in_turn() -> None:
    async with await identified() as engine:
        scores = [(await engine.analyse(esca.Game(), uci.Limits(depth=2)))[0].cp for _ in range(3)]
        assert scores == [25, 25, 25]


@asyncio_test
async def test_an_engine_with_no_move_answers_with_none() -> None:
    async with await identified("--no-move") as engine:
        answer = await engine.play(esca.Game(), uci.Limits(depth=1))
        assert answer.best is None
        assert answer.ponder is None


@asyncio_test
async def test_a_search_that_waits_is_ended_by_stop() -> None:
    async with await identified() as engine:
        await engine.set_position(esca.Game())
        search = await engine.go(uci.Limits(infinite=True))
        await search.stop()
        assert (await search.answer()).best is not None


@asyncio_test
async def test_a_ponder_becomes_a_search_on_a_ponderhit() -> None:
    async with await identified() as engine:
        await engine.set_position(esca.Game())
        search = await engine.go(uci.Limits(infinite=True, ponder=True))
        assert engine.state == "pondering"
        await search.ponderhit()
        assert (await search.answer()).best is not None


@asyncio_test
async def test_leaving_a_search_stops_it_and_leaves_the_engine_idle() -> None:
    async with await identified() as engine:
        await engine.set_position(esca.Game())
        async with await engine.go(uci.Limits(infinite=True)):
            pass
        assert engine.state == "idle"
        await engine.is_ready()


@asyncio_test
async def test_quitting_reaps_the_process() -> None:
    engine = await identified()
    assert await engine.quit() == 0


@asyncio_test
async def test_an_engine_that_will_not_quit_is_killed() -> None:
    engine = await identified("--zombie")
    engine.timeout = 0.3
    assert await engine.quit() is not None


# -- Unhappy paths ----------------------------------------------------------


@asyncio_test
async def test_a_silent_engine_times_out_rather_than_hangs() -> None:
    async with await identified("--no-readyok") as engine:
        engine.timeout = 0.2
        with pytest.raises(uci.EngineTimeout):
            await engine.is_ready()


@asyncio_test
async def test_an_engine_that_never_ends_its_identification_times_out() -> None:
    async with fake("--no-uciok", timeout=1.0) as engine:
        with pytest.raises(uci.EngineTimeout):
            await engine.handshake()


@asyncio_test
async def test_a_search_that_outlasts_its_budget_is_a_timeout() -> None:
    async with await identified("--slow") as engine:
        await engine.set_position(esca.Game())
        search = await engine.go(uci.Limits(depth=2), timeout=0.05)
        with pytest.raises(uci.EngineTimeout):
            await search.answer()


@asyncio_test
async def test_a_slow_engine_answers_within_a_budget_that_fits() -> None:
    async with await identified("--slow") as engine:
        assert (await engine.play(esca.Game(), uci.Limits(depth=2))).best is not None


@asyncio_test
async def test_garbage_never_derails_a_search() -> None:
    async with await identified("--garbage") as engine:
        assert (await engine.play(esca.Game(), uci.Limits(depth=2))).best is not None
        await engine.is_ready()


@asyncio_test
async def test_an_answer_too_many_is_ignored() -> None:
    async with await identified("--twice") as engine:
        await engine.play(esca.Game(), uci.Limits(depth=1))
        await engine.is_ready()
        assert engine.state == "idle"


@asyncio_test
async def test_an_engine_that_dies_mid_search_is_reported_as_dead() -> None:
    engine = await identified("--die-on-go")
    try:
        await engine.set_position(esca.Game())
        search = await engine.go(uci.Limits(depth=2))
        with pytest.raises(uci.EngineDied):
            await search.answer()
        # Every call after says the same thing.
        with pytest.raises(uci.EngineDied):
            await engine.is_ready()
    finally:
        engine.kill()


@asyncio_test
async def test_a_command_the_conversation_has_no_room_for_is_refused() -> None:
    async with await identified() as engine:
        with pytest.raises(uci.ProtocolError):
            await engine.stop()
        await engine.is_ready()


# -- Giving up on a wait ----------------------------------------------------


@asyncio_test
async def test_a_wait_given_up_on_leaves_the_engine_usable() -> None:
    async with await identified("--slow") as engine:
        game = esca.Game()
        with pytest.raises(TimeoutError):
            await asyncio.wait_for(engine.play(game, uci.Limits(depth=2)), 0.05)

        await engine.is_ready()
        assert engine.state == "idle"
        assert (await engine.play(game, uci.Limits(depth=2))).best is not None


@asyncio_test
async def test_a_cancelled_search_is_stopped_and_the_engine_settles() -> None:
    async with await identified() as engine:
        playing = asyncio.ensure_future(engine.play(esca.Game(), uci.Limits(infinite=True)))
        await until_searching(engine)
        playing.cancel()
        with pytest.raises(asyncio.CancelledError):
            await playing

        await engine.is_ready()
        assert engine.state == "idle"
        assert (await engine.play(esca.Game(), uci.Limits(depth=2))).best is not None


@asyncio_test
async def test_a_search_given_up_on_is_stopped() -> None:
    async with await identified() as engine:
        await engine.set_position(esca.Game())
        search = await engine.go(uci.Limits(infinite=True))
        with pytest.raises(TimeoutError):
            await asyncio.wait_for(search.answer(), 0.05)

        await engine.is_ready()
        assert engine.state == "idle"


# -- The line buffer --------------------------------------------------------


@asyncio_test
async def test_a_flood_of_reports_drops_the_oldest_and_keeps_the_answer() -> None:
    async with await identified("--flood") as engine:
        await engine.set_position(esca.Game())
        search = await engine.go(uci.Limits(depth=2), timeout=TIMEOUT)

        # Let the double outrun the client, which reads nothing until it does.
        for _ in range(int(TIMEOUT / 0.01)):
            if engine.dropped_lines > 0:
                break
            await asyncio.sleep(0.01)
        assert engine.dropped_lines > 0, "the double floods the client"

        assert (await search.answer()).best is not None, "the answer is never dropped"
        assert engine.state == "idle"


@asyncio_test
async def test_a_client_that_keeps_up_drops_nothing() -> None:
    async with await identified() as engine:
        await engine.play(esca.Game(), uci.Limits(depth=2))
        assert engine.dropped_lines == 0


# -- Options ----------------------------------------------------------------


@asyncio_test
async def test_an_option_is_set_by_the_name_the_engine_declared(tmp_path: Path) -> None:
    log = tmp_path / "options.log"
    async with fake(f"--log={log}") as engine:
        await engine.handshake()
        await engine.set_option("multipv", 3)
        await engine.set_option("Clear Hash")
        await engine.set_option("Debug Log File", "")
        await engine.is_ready()
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
@asyncio_test
async def test_a_value_the_engine_did_not_declare_is_refused(name: str, value: object, complaint: str) -> None:
    async with await identified() as engine:
        with pytest.raises(ValueError, match=complaint):
            await engine.set_option(name, value)


@asyncio_test
async def test_options_are_unknown_until_the_engine_has_listed_them() -> None:
    async with fake() as engine:
        with pytest.raises(uci.ProtocolError):
            await engine.set_option("Hash", 32)


# -- Chess960 ---------------------------------------------------------------


@asyncio_test
async def test_a_chess960_game_puts_the_engine_into_chess960(tmp_path: Path) -> None:
    log = tmp_path / "chess960.log"
    async with fake(f"--log={log}") as engine:
        await engine.handshake()
        game = esca.Game.from_fen(BESIDE_ROOK, variant=esca.CHESS960)
        game.play("b1c1")
        await engine.set_position(game)
        await engine.is_ready()
        sent = log_of(log)

    assert sent == [
        "uci",
        "setoption name UCI_Chess960 value true",
        "isready",
        f"position fen {BESIDE_ROOK} moves b1c1",
        "isready",
    ]


@asyncio_test
async def test_an_engine_that_cannot_play_chess960_is_refused_the_game() -> None:
    async with await identified("--no-chess960") as engine:
        game = esca.Game.from_fen(BESIDE_ROOK, variant=esca.CHESS960)
        with pytest.raises(ValueError, match="UCI_Chess960"):
            await engine.set_position(game)


@asyncio_test
async def test_a_classic_game_is_sent_without_touching_the_option(tmp_path: Path) -> None:
    log = tmp_path / "classic.log"
    async with fake(f"--log={log}") as engine:
        await engine.handshake()
        game = esca.Game.from_fen("4k3/8/8/8/8/8/8/4K2R w K - 0 1")
        game.play("e1h1")
        await engine.set_position(game)
        await engine.is_ready()
        sent = log_of(log)

    assert sent == [
        "uci",
        "position fen 4k3/8/8/8/8/8/8/4K2R w K - 0 1 moves e1g1",
        "isready",
    ]


# -- The raw interface ------------------------------------------------------


@asyncio_test
async def test_lines_can_be_written_and_read_as_they_are() -> None:
    async with fake() as engine:
        await engine.send_line("uci")
        seen: list[str] = []
        while (line := await engine.next_line(TIMEOUT)) is not None:
            seen.append(line)
            if line == "uciok":
                break
        assert seen[0] == "id name Fake Engine 1.0"
        assert seen[-1] == "uciok"


@asyncio_test
async def test_a_read_that_finds_nothing_answers_with_nothing() -> None:
    async with await identified() as engine:
        assert await engine.next_line(0.05) is None
