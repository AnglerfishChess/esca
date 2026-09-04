"""The protocol as values: what a command writes, and what one line of engine
output reads as.

The cases mirror `tests/uci_protocol.rs`, including the shapes engines are
known to write in the wild: a keyword after junk, missing arguments, extra
spaces, names and values of several words.
"""

from __future__ import annotations

import esca
import pytest
from esca import uci
from esca.uci import protocol
from esca.uci.protocol import Command, Session, parse

#: A Chess960 endgame: the white king on b1 with its own rook beside it on c1,
#: so that castling short leaves the king's origin behind.
BESIDE_ROOK = "4k3/8/8/8/8/8/8/1KR5 w C - 0 1"


def played(*moves: str, fen: str | None = None, variant: esca.Variant | None = None) -> esca.Game:
    """The game those moves reach, from `fen` or from the standard array."""
    game = esca.Game(variant=variant) if fen is None else esca.Game.from_fen(fen, variant=variant)
    for move in moves:
        game.play(move)
    return game


# -- Commands ---------------------------------------------------------------


@pytest.mark.parametrize(
    ("command", "line"),
    [
        (Command.uci(), "uci"),
        (Command.debug(True), "debug on"),
        (Command.debug(False), "debug off"),
        (Command.isready(), "isready"),
        (Command.ucinewgame(), "ucinewgame"),
        (Command.stop(), "stop"),
        (Command.ponderhit(), "ponderhit"),
        (Command.quit(), "quit"),
    ],
)
def test_a_plain_command_is_its_keyword(command: Command, line: str) -> None:
    assert command.to_line() == line
    assert command.keyword == line.split()[0]


@pytest.mark.parametrize(
    ("value", "line"),
    [
        ("64", "setoption name Hash value 64"),
        (None, "setoption name Hash"),  # a button carries no value
    ],
)
def test_setting_an_option_names_it_and_its_value(value: str | None, line: str) -> None:
    assert Command.setoption("Hash", value).to_line() == line


def test_an_option_name_and_value_may_be_several_words() -> None:
    assert Command.setoption("Clear Hash", "two words").to_line() == "setoption name Clear Hash value two words"


def test_a_position_names_where_it_starts() -> None:
    assert Command.position(esca.Game()).to_line() == "position startpos"
    fen = "4k3/8/8/8/8/8/8/4K2R w K - 0 1"
    assert Command.position(esca.Game.from_fen(fen)).to_line() == f"position fen {fen}"


def test_a_position_lists_the_moves_played_onto_it() -> None:
    game = played("e2e4", "e7e5")
    assert Command.position(game).to_line() == "position startpos moves e2e4 e7e5"


def test_classic_castling_is_written_as_two_squares() -> None:
    fen = "4k3/8/8/8/8/8/8/4K2R w K - 0 1"
    game = played("e1h1", fen=fen)
    assert Command.position(game, esca.KING_TWO_SQUARES).to_line() == f"position fen {fen} moves e1g1"


def test_chess960_castling_is_written_king_to_rook() -> None:
    game = played("b1c1", fen=BESIDE_ROOK, variant=esca.CHESS960)
    assert Command.position(game, esca.KING_TO_ROOK).to_line() == f"position fen {BESIDE_ROOK} moves b1c1"


@pytest.mark.parametrize(
    ("limits", "line"),
    [
        (None, "go"),
        (uci.Limits(), "go"),
        (uci.Limits(infinite=True), "go infinite"),
        (uci.Limits(depth=12), "go depth 12"),
        (uci.Limits(nodes=50_000), "go nodes 50000"),
        (uci.Limits(mate=3), "go mate 3"),
        (uci.Limits(movetime=1.5), "go movetime 1500"),
        (uci.Limits(depth=4, ponder=True), "go ponder depth 4"),
    ],
)
def test_a_go_names_the_limits_it_has(limits: uci.Limits | None, line: str) -> None:
    assert Command.go(limits).to_line() == line


def test_a_clock_is_written_in_milliseconds() -> None:
    limits = uci.Limits(white_time=60.0, black_time=45.5, white_increment=0.6, black_increment=0.6, moves_to_go=20)
    assert Command.go(limits).to_line() == "go wtime 60000 btime 45500 winc 600 binc 600 movestogo 20"


def test_searchmoves_comes_after_every_other_limit() -> None:
    limits = uci.Limits(depth=6, search_moves=["e2e4", "d2d4"])
    assert Command.go(limits).to_line() == "go depth 6 searchmoves e2e4 d2d4"


# -- Engine lines -----------------------------------------------------------


@pytest.mark.parametrize(
    ("line", "kind"),
    [
        ("uciok", "uciok"),
        ("readyok", "readyok"),
        ("registration checking", "registration"),
        ("copyprotection ok", "copyprotection"),
        ("info depth 3", "info"),
        ("bestmove e2e4", "bestmove"),
        ("id name Fake", "id"),
        ("option name Hash type spin default 16", "option"),
    ],
)
def test_a_plain_line_is_its_keyword(line: str, kind: str) -> None:
    message = parse(line)
    assert message.kind == kind
    assert message.line == line


@pytest.mark.parametrize(
    "line",
    [
        "",
        "   ",
        "Fake engine 1.0, not a real one",
        "bestmove",  # a bestmove with no move at all
        "option name Hash type nonsense",
    ],
)
def test_a_line_that_is_not_understood_is_kept_whole(line: str) -> None:
    message = parse(line)
    assert message.kind == "raw"
    assert message.line == line


@pytest.mark.parametrize(
    ("line", "key", "value"),
    [
        ("id name Fake Engine 1.0", "name", "Fake Engine 1.0"),
        ("id author The esca test suite", "author", "The esca test suite"),
        ("id copyright 2026 nobody", "copyright", "2026 nobody"),
    ],
)
def test_an_id_keeps_the_rest_of_its_line(line: str, key: str, value: str) -> None:
    message = parse(line)
    assert (message.key, message.value) == (key, value)


@pytest.mark.parametrize(
    ("line", "type_name", "default", "extra"),
    [
        ("option name Ponder type check default false", "check", False, {}),
        ("option name Hash type spin default 16 min 1 max 1024", "spin", 16, {"min": 1, "max": 1024}),
        (
            "option name Style type combo default Solid var Solid var Wild",
            "combo",
            "Solid",
            {"vars": ["Solid", "Wild"]},
        ),
        ("option name Clear Hash type button", "button", None, {}),
        ("option name Debug Log File type string default <empty>", "string", "", {}),
    ],
)
def test_an_option_line_declares_a_name_a_type_and_a_domain(
    line: str, type_name: str, default: object, extra: dict[str, object]
) -> None:
    option = parse(line).option
    assert option is not None
    assert option.type == type_name
    assert option.default == default
    for name, expected in extra.items():
        assert getattr(option, name) == expected


def test_a_name_and_a_value_may_both_be_several_words() -> None:
    option = parse("option name Debug Log File type string default out of the box").option
    assert option is not None
    assert (option.name, option.default) == ("Debug Log File", "out of the box")


def test_an_info_reads_the_counters_of_a_search() -> None:
    info = parse("info depth 12 seldepth 20 nodes 1234 nps 5000 time 250 hashfull 30 tbhits 7 multipv 2").info
    assert info is not None
    assert (info.depth, info.seldepth, info.multipv) == (12, 20, 2)
    assert (info.nodes, info.nps, info.hashfull, info.tbhits) == (1234, 5000, 30, 7)
    assert info.time == pytest.approx(0.25)


@pytest.mark.parametrize(
    ("line", "cp", "mate", "bound"),
    [
        ("info score cp 34", 34, None, None),
        ("info score cp -12 lowerbound", -12, None, "lowerbound"),
        ("info score cp 8 upperbound", 8, None, "upperbound"),
        ("info score mate -3", None, -3, None),
    ],
)
def test_a_score_is_read_with_its_bound(line: str, cp: int | None, mate: int | None, bound: str | None) -> None:
    info = parse(line).info
    assert info is not None
    assert (info.cp, info.mate, info.bound) == (cp, mate, bound)


def test_a_win_draw_loss_estimate_is_three_numbers() -> None:
    info = parse("info score cp 20 wdl 350 600 50").info
    assert info is not None
    assert info.wdl == (350, 600, 50)


def test_an_info_string_is_the_rest_of_the_line() -> None:
    info = parse("info string weird: colons: and   spacing").info
    assert info is not None
    assert info.string == "weird: colons: and   spacing"


@pytest.mark.parametrize(
    ("line", "unknown"),
    [
        ("info depth", ["depth"]),  # a keyword with nothing after it
        ("info depth 3 wobble 4", ["wobble", "4"]),  # its value is a token too
        ("info score", ["score"]),
    ],
)
def test_a_field_that_is_not_understood_is_kept_as_a_token(line: str, unknown: list[str]) -> None:
    info = parse(line).info
    assert info is not None
    assert info.unknown == unknown


def test_spacing_and_leading_junk_do_not_change_what_a_line_says() -> None:
    message = parse("  Fake 1.0   info   depth 4  score cp 7 ")
    assert message.kind == "info"
    assert message.info is not None
    assert (message.info.depth, message.info.cp) == (4, 7)


# -- Reading move text against a game ---------------------------------------


def test_a_variation_reads_as_the_moves_of_the_game_it_was_searched_from() -> None:
    game = esca.Game()
    info = parse("info pv e2e4 e7e5 g1f3", game).info
    assert info is not None
    assert [move.uci for move in info.pv] == ["e2e4", "e7e5", "g1f3"]
    assert game.move_to_san(info.pv[0]) == "e4"


def test_a_variation_stops_at_the_first_move_that_is_not_legal() -> None:
    info = parse("info pv e2e4 e7e5 e1e8", esca.Game()).info
    assert info is not None
    assert len(info.pv) == 2


def test_a_bestmove_reads_as_a_move_of_the_position_searched() -> None:
    game = esca.Game()
    answer = parse("bestmove e2e4 ponder e7e5", game).answer
    assert answer is not None
    assert answer.best is not None
    assert game.move_to_san(answer.best) == "e4"
    assert answer.ponder is not None
    assert answer.ponder.uci == "e7e5"


def test_a_bestmove_of_none_names_no_move() -> None:
    answer = parse("bestmove (none)", esca.Game()).answer
    assert answer is not None
    assert answer.best is None


def test_chess960_castling_is_read_king_to_rook() -> None:
    game = esca.Game.from_fen(BESIDE_ROOK, variant=esca.CHESS960)
    answer = parse("bestmove b1c1", game).answer
    assert answer is not None
    assert answer.best is not None
    assert answer.best.is_castling
    assert game.move_to_san(answer.best) == "O-O"


# -- The state machine ------------------------------------------------------


def identified() -> Session:
    """A session that has seen the whole identification."""
    session = Session()
    session.sent(Command.uci())
    session.received(parse("uciok"))
    return session


def test_a_session_follows_the_engine_from_identification_to_an_answer() -> None:
    session = Session()
    assert session.state == "started"

    session.sent(Command.uci())
    assert session.state == "identifying"
    session.received(parse("id name Fake Engine 1.0"))
    session.received(parse("option name Hash type spin default 16"))
    session.received(parse("uciok"))
    assert session.state == "idle"

    session.sent(Command.isready())
    assert session.pending_ready == 1
    session.received(parse("readyok"))
    assert session.pending_ready == 0

    session.sent(Command.position(esca.Game()))
    session.sent(Command.go(uci.Limits(depth=4)))
    assert session.state == "searching"
    session.received(parse("info depth 1"))
    session.received(parse("bestmove e2e4"))
    assert session.state == "idle"


def test_pondering_becomes_searching_on_a_ponderhit() -> None:
    session = identified()
    session.sent(Command.go(uci.Limits(infinite=True, ponder=True)))
    assert session.state == "pondering"
    session.sent(Command.ponderhit())
    assert session.state == "searching"


@pytest.mark.parametrize(
    "command",
    [Command.position(esca.Game()), Command.ucinewgame(), Command.setoption("Hash", "64"), Command.uci()],
)
def test_a_searching_engine_takes_no_new_work(command: Command) -> None:
    session = identified()
    session.sent(Command.go(uci.Limits(depth=4)))
    with pytest.raises(uci.ProtocolError):
        session.sent(command)


@pytest.mark.parametrize("command", [Command.stop(), Command.ponderhit()])
def test_an_idle_engine_has_no_search_to_end(command: Command) -> None:
    with pytest.raises(uci.ProtocolError):
        identified().sent(command)


def test_an_answer_that_no_search_asked_for_is_refused() -> None:
    session = identified()
    session.sent(Command.go(uci.Limits(depth=1)))
    session.received(parse("bestmove e2e4"))
    with pytest.raises(uci.ProtocolError):
        session.received(parse("bestmove d2d4"))


def test_a_readyok_that_no_isready_asked_for_is_refused() -> None:
    with pytest.raises(uci.ProtocolError):
        identified().received(parse("readyok"))


def test_options_and_names_belong_to_the_identification() -> None:
    session = identified()
    for line in ("id name Fake Engine 1.0", "option name Hash type spin default 16"):
        with pytest.raises(uci.ProtocolError):
            session.received(parse(line))


def test_info_and_raw_lines_are_welcome_at_any_time() -> None:
    session = Session()
    for line in ("info depth 1", "Fake engine 1.0, not a real one", "copyprotection ok"):
        session.received(parse(line))
    assert session.state == "started"


# -- The module and its stub ------------------------------------------------


def test_the_client_offers_the_protocol_as_a_submodule() -> None:
    assert "protocol" in uci.__all__
    assert uci.protocol is protocol


def test_every_exported_name_exists() -> None:
    assert [name for name in protocol.__all__ if not hasattr(protocol, name)] == []
