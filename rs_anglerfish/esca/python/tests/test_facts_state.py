"""The `state` group, fact by fact.

Every expectation is worked out from the definitions in `docs/features.md`
§2.1 for the named position above it. Repetition and history are facts of a
game, so those cases play the moves that make them true. The cases mirror
`tests/facts_state.rs`.
"""

from __future__ import annotations

from collections.abc import Callable, Sequence

import esca
import pytest

#: The untouched array: every right, a fresh clock, nothing to repeat.
START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

#: The same array as the evaluation dump writes it: four fields, no clocks.
START_NO_CLOCKS = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -"

#: A queen down the open e-file: one checker, and nothing else to say.
QUEEN_CHECK = "4k3/8/8/4q3/8/8/8/4K3 w - - 6 40"

#: The same queen with a knight in the way: aimed at the king, not checking.
BLOCKED = "4k3/8/8/4q3/8/4N3/8/4K3 w - - 6 40"

#: A pawn on d2 checks e1 the only way a pawn can, diagonally.
PAWN_CHECK = "4k3/8/8/8/8/8/3p4/4K3 w - - 0 40"

#: A rook down the e-file and a knight on d3: two checkers at once.
ROOK_AND_KNIGHT = "k3r3/8/8/8/8/3n4/8/4K3 w - - 8 40"

#: A bishop bearing from a5 down to e1 and a knight on f3: two checkers again.
BISHOP_AND_KNIGHT = "4k3/8/8/b7/8/5n2/8/4K3 w - - 12 45"

#: Kings and rooks at home, but only White's short and Black's long right left.
RIGHTS_SPLIT = "r3k2r/8/8/8/8/8/8/R3K2R w Kq - 4 12"

#: The same array with Black to move and Black's long right gone.
RIGHTS_BLACK_TO_MOVE = "r3k2r/8/8/8/8/8/8/R3K2R b KQk - 2 9"

#: Kings and rooks at home with every right spent.
RIGHTS_NONE = "r3k2r/8/8/8/8/8/8/R3K2R b - - 10 30"

#: After 1.e4 c5 2.e5 d5: e5 stands beside the pawn that has just run past it.
EP_TAKEABLE = "rnbqkbnr/pp2pppp/8/2ppP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3"

#: The FEN names e3, and the one black pawn stands two files away from it.
EP_UNREACHED = "4k3/8/8/8/2p1P3/8/8/4K3 b - e3 0 1"

#: c4xd3 e.p. would empty the fourth rank between h4 and the black king.
EP_PINNED = "8/8/8/8/k1pP3R/8/8/4K3 b - d3 0 1"

#: Pawns on either side of d4: two legal moves take d3 en passant.
EP_TWO_TAKERS = "4k3/8/8/8/2pPp3/8/8/4K3 b - d3 0 1"

#: A rook endgame stripped of its clocks; the clock cases append their own.
NO_CLOCKS = "4k3/8/8/8/8/8/R7/4K3 w - -"

#: The same endgame 45 plies into a shuffle.
CLOCK_45 = "4k3/8/8/8/8/8/R7/4K3 w - - 45 60"

#: Kings and a rook with room to walk: the repetition cases play from here.
SHUFFLE = "k6r/8/8/8/8/8/8/K7 w - - 0 1"

#: Chess960: kings on g between rooks on f and h, and a pawn just past b5.
NINE_SIXTY = "bqnb1rkr/pp1ppppp/8/1Pp5/8/8/P1PPPPPP/BQNB1RKR w HFhf c6 0 3"

#: The same rights in the `KQkq` dialect: the outermost rook of each wing.
NINE_SIXTY_OUTERMOST = "bqnb1rkr/pp1ppppp/8/1Pp5/8/8/P1PPPPPP/BQNB1RKR w KQkq c6 0 3"

#: Only White's h-rook and Black's f-rook are still free, and Black is to move.
NINE_SIXTY_SPLIT = "bqnb1rkr/pp1ppppp/8/1Pp5/8/8/P1PPPPPP/BQNB1RKR b Hf - 4 5"

#: Kings on b between rooks on a and c: only the c-rook may still be castled with.
NINE_SIXTY_INNER = "rkrbbnnq/pppppppp/8/8/8/8/PPPPPPPP/RKRBBNNQ w Cc - 0 1"

#: Where the `halfmove_bucket` one-hot starts in the 29-wide `state` row: after
#: `in_check`, `double_check`, the four castling bits, `ep_available`, the eight
#: `ep_file` bits and `ep_capture_legal` (`features.md` §2.1).
HALFMOVE_BUCKET_AT = 16

#: The helper `conftest.py` hands over.
FactsOf = Callable[..., esca.Facts]


def game_of(fen: str, moves: Sequence[str]) -> esca.Game:
    """The classic game `fen` starts, after the UCI `moves`."""
    game = esca.Game.from_fen(fen)
    for uci in moves:
        game.play(uci)
    return game


def halfmove_bucket(fen: str) -> int:
    """The bucket the encoded `state` row's one-hot sets for `fen`."""
    row = esca.encode([fen], groups=["state"])[0]
    return list(row[HALFMOVE_BUCKET_AT : HALFMOVE_BUCKET_AT + 8]).index(1.0)


@pytest.mark.parametrize(
    ("fen", "in_check"),
    [
        (START, False),
        (BLOCKED, False),
        (RIGHTS_NONE, False),
        (QUEEN_CHECK, True),
        (PAWN_CHECK, True),
        (ROOK_AND_KNIGHT, True),
        (BISHOP_AND_KNIGHT, True),
    ],
    ids=["start", "blocked", "rights_none", "queen_check", "pawn_check", "rook_and_knight", "bishop_and_knight"],
)
def test_a_check_is_the_side_to_move_standing_under_attack(fen: str, in_check: bool, facts_of: FactsOf) -> None:
    assert facts_of(fen).state.in_check == in_check


@pytest.mark.parametrize(
    ("fen", "double_check"),
    [
        (START, False),
        (BLOCKED, False),
        (QUEEN_CHECK, False),
        (PAWN_CHECK, False),
        (ROOK_AND_KNIGHT, True),
        (BISHOP_AND_KNIGHT, True),
    ],
    ids=["start", "blocked", "queen_check", "pawn_check", "rook_and_knight", "bishop_and_knight"],
)
def test_two_checkers_at_once_make_the_check_a_double_one(fen: str, double_check: bool, facts_of: FactsOf) -> None:
    assert facts_of(fen).state.double_check == double_check


@pytest.mark.parametrize(
    ("variant", "fen", "short", "long"),
    [
        (esca.CLASSIC, START, (True, True), (True, True)),
        (esca.CLASSIC, RIGHTS_SPLIT, (True, False), (False, True)),
        (esca.CLASSIC, RIGHTS_BLACK_TO_MOVE, (True, True), (False, True)),
        (esca.CLASSIC, RIGHTS_NONE, (False, False), (False, False)),
        (esca.CHESS960, NINE_SIXTY, (True, True), (True, True)),
        (esca.CHESS960, NINE_SIXTY_OUTERMOST, (True, True), (True, True)),
        (esca.CHESS960, NINE_SIXTY_SPLIT, (False, True), (True, False)),
        (esca.CHESS960, NINE_SIXTY_INNER, (True, True), (False, False)),
    ],
    ids=[
        "start",
        "rights_split",
        "rights_black_to_move",
        "rights_none",
        "nine_sixty",
        "nine_sixty_outermost",
        "nine_sixty_split",
        "nine_sixty_inner",
    ],
)
def test_a_castling_right_survives_for_the_side_and_wing_the_fen_still_names(
    variant: esca.Variant, fen: str, short: tuple[bool, bool], long: tuple[bool, bool], facts_of: FactsOf
) -> None:
    state = facts_of(fen, variant).state
    assert state.castle_short == short
    assert state.castle_long == long


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, False),
        (CLOCK_45, False),
        (EP_TAKEABLE, True),
        (EP_UNREACHED, True),
        (EP_PINNED, True),
        (EP_TWO_TAKERS, True),
    ],
    ids=["start", "clock_45", "ep_takeable", "ep_unreached", "ep_pinned", "ep_two_takers"],
)
def test_the_en_passant_bit_says_only_that_the_fen_named_a_target(fen: str, available: bool, facts_of: FactsOf) -> None:
    assert (facts_of(fen).state.en_passant is not None) == available


@pytest.mark.parametrize(
    ("fen", "file"),
    [
        (START, None),
        (CLOCK_45, None),
        (EP_TAKEABLE, "d"),
        (EP_UNREACHED, "e"),
        (EP_PINNED, "d"),
        (EP_TWO_TAKERS, "d"),
    ],
    ids=["start", "clock_45", "ep_takeable", "ep_unreached", "ep_pinned", "ep_two_takers"],
)
def test_the_en_passant_file_is_the_one_the_target_square_stands_on(
    fen: str, file: str | None, facts_of: FactsOf
) -> None:
    assert facts_of(fen).state.en_passant == file


@pytest.mark.parametrize(
    ("fen", "legal"),
    [
        (START, False),
        (EP_UNREACHED, False),
        (EP_PINNED, False),
        (EP_TAKEABLE, True),
        (EP_TWO_TAKERS, True),
    ],
    ids=["start", "ep_unreached", "ep_pinned", "ep_takeable", "ep_two_takers"],
)
def test_an_en_passant_capture_counts_only_when_a_legal_move_makes_it(fen: str, legal: bool, facts_of: FactsOf) -> None:
    assert facts_of(fen).state.ep_capture_legal == legal


@pytest.mark.parametrize(
    ("clock", "bucket"),
    [
        (0, 0),
        (1, 1),
        (3, 1),
        (4, 2),
        (9, 2),
        (10, 3),
        (19, 3),
        (20, 4),
        (39, 4),
        (40, 5),
        (69, 5),
        (70, 6),
        (89, 6),
        (90, 7),
        (100, 7),
    ],
    ids=[
        "clock_0",
        "clock_1",
        "clock_3",
        "clock_4",
        "clock_9",
        "clock_10",
        "clock_19",
        "clock_20",
        "clock_39",
        "clock_40",
        "clock_69",
        "clock_70",
        "clock_89",
        "clock_90",
        "clock_100",
    ],
)
def test_the_halfmove_clock_falls_in_the_bucket_whose_range_holds_it(
    clock: int, bucket: int, facts_of: FactsOf
) -> None:
    fen = f"{NO_CLOCKS} {clock} 60"
    assert facts_of(fen).state.halfmove_clock == clock
    assert halfmove_bucket(fen) == bucket


@pytest.mark.parametrize(
    ("fen", "known", "clock"),
    [
        (START, True, 0),
        (START_NO_CLOCKS, False, 0),
        (NO_CLOCKS, False, 0),
        (CLOCK_45, True, 45),
        (RIGHTS_NONE, True, 10),
    ],
    ids=["start", "start_no_clocks", "no_clocks", "clock_45", "rights_none"],
)
def test_a_clock_is_known_only_from_a_fen_that_carries_one(
    fen: str, known: bool, clock: int, facts_of: FactsOf
) -> None:
    state = facts_of(fen).state
    assert state.halfmove_known == known
    assert state.halfmove_clock == clock


@pytest.mark.parametrize(
    ("fen", "moves", "seen"),
    [
        (START, [], False),
        (START, ["g1f3"], False),
        (START, ["g1f3", "g8f6", "f3g1", "f6g8"], True),
        (SHUFFLE, [], False),
        (SHUFFLE, ["a1b1", "h8h7", "b1a1", "h7h6"], False),
        (SHUFFLE, ["a1b1", "h8h7", "b1a1", "h7h8"], True),
        (SHUFFLE, ["a1b1", "h8h7", "b1a1", "h7h8", "a1b1", "h8h7", "b1a1", "h7h8"], True),
    ],
    ids=[
        "start_fresh",
        "start_knight_out",
        "start_knights_home",
        "shuffle_fresh",
        "shuffle_rook_h6",
        "shuffle_rook_back",
        "shuffle_rook_back_twice",
    ],
)
def test_a_position_the_game_has_already_held_is_a_repetition(fen: str, moves: Sequence[str], seen: bool) -> None:
    assert game_of(fen, moves).facts().state.repetition_seen == seen


@pytest.mark.parametrize(
    ("fen", "moves", "available"),
    [
        (START, [], (False, False)),
        (START, ["g1f3"], (False, False)),
        (START, ["g1f3", "g8f6", "f3g1"], (True, False)),
        (START, ["g1f3", "g8f6", "f3g1", "f6g8"], (True, False)),
        (SHUFFLE, ["a1b1", "h8h7", "b1a1"], (True, False)),
        (SHUFFLE, ["a1b1", "h8h7", "b1a1", "h7h8"], (True, False)),
        (SHUFFLE, ["a1b1", "h8h7", "b1a1", "h7h6"], (False, True)),
    ],
    ids=[
        "start_fresh",
        "start_knight_out",
        "start_three_of_four",
        "start_knights_home",
        "shuffle_three_plies",
        "shuffle_rook_back",
        "shuffle_rook_h6",
    ],
)
def test_a_repetition_is_available_to_us_when_one_of_our_moves_reaches_the_history(
    fen: str, moves: Sequence[str], available: tuple[bool, bool]
) -> None:
    state = game_of(fen, moves).facts().state
    assert state.repetition_available == available
    assert state.repetition_available[esca.US] == available[0]


@pytest.mark.parametrize(
    ("fen", "moves", "available"),
    [
        (START, [], (False, False)),
        (START, ["g1f3", "g8f6", "f3g1", "f6g8"], (True, False)),
        (SHUFFLE, ["a1b1", "h8h7", "b1a1", "h7h6"], (False, True)),
        (SHUFFLE, ["a1b1", "h8h7", "b1a1", "h7h6", "a1b1", "h6h5"], (False, True)),
        (SHUFFLE, ["a1b1", "h8b8"], (False, False)),
    ],
    ids=["start_fresh", "start_knights_home", "shuffle_rook_h6", "shuffle_rook_h5", "shuffle_checked"],
)
def test_their_repetition_is_read_after_a_null_move_and_a_check_leaves_none(
    fen: str, moves: Sequence[str], available: tuple[bool, bool]
) -> None:
    state = game_of(fen, moves).facts().state
    assert state.repetition_available == available
    assert state.repetition_available[esca.THEM] == available[1]
    assert not state.in_check or not available[1], "no null move exists"


@pytest.mark.parametrize(
    "fen",
    [START, SHUFFLE, CLOCK_45, QUEEN_CHECK],
    ids=["start", "shuffle", "clock_45", "queen_check"],
)
def test_a_history_is_known_only_to_the_game_that_holds_it(fen: str, facts_of: FactsOf) -> None:
    assert not facts_of(fen).state.history_known
    assert game_of(fen, []).facts().state.history_known


@pytest.mark.parametrize(
    ("fen", "moves"),
    [
        (START, ["g1f3", "g8f6", "f3g1", "f6g8"]),
        (SHUFFLE, ["a1b1", "h8h7", "b1a1", "h7h8"]),
        (SHUFFLE, ["a1b1", "h8h7", "b1a1", "h7h6"]),
    ],
    ids=["start_knights_home", "shuffle_rook_back", "shuffle_rook_h6"],
)
def test_a_position_on_its_own_carries_none_of_the_repetition_facts(
    fen: str, moves: Sequence[str], facts_of: FactsOf
) -> None:
    game = game_of(fen, moves)
    played = game.facts().state
    bare = facts_of(game.position.fen).state

    assert played.repetition_seen or played.repetition_available != (False, False), (
        "the game sees something to repeat here"
    )
    assert not bare.history_known
    assert not bare.repetition_seen
    assert bare.repetition_available == (False, False)


def test_the_state_facts_of_a_chess960_position_read_the_rooks_the_rights_name(facts_of: FactsOf) -> None:
    """No `state` fact is one of the four `features.md` §4 defines for classic
    chess only, and castling rights are read from the rook files either dialect
    names."""
    state = facts_of(NINE_SIXTY, esca.CHESS960).state
    assert not state.in_check
    assert not state.double_check
    assert state.castle_short == (True, True)
    assert state.castle_long == (True, True)
    assert state.en_passant == "c"
    assert state.ep_capture_legal, "b5 takes c6 en passant"
    assert state.halfmove_clock == 0
    assert state.halfmove_known
    assert not state.history_known
    assert not state.repetition_seen
    assert state.repetition_available == (False, False)

    outermost = facts_of(NINE_SIXTY_OUTERMOST, esca.CHESS960).state
    assert outermost.castle_short == state.castle_short, "`KQkq` names the same two rooks"
    assert outermost.castle_long == state.castle_long
    assert outermost.en_passant == state.en_passant
