"""The `history` group, fact by fact.

Every expectation is worked out from the definitions in `docs/features.md`
§2.13 for the named position above it. Everything but the halfmove clock is a
fact of a game, so those cases play the moves that make them true. The cases
mirror `tests/facts_history.rs`.
"""

from __future__ import annotations

from collections.abc import Callable, Sequence

import esca
import pytest

#: The untouched array: a fresh clock and nothing to repeat.
START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

#: The same array as the evaluation dump writes it: four fields, no clocks.
START_NO_CLOCKS = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -"

#: A rook endgame stripped of its clocks; the clock cases append their own.
NO_CLOCKS = "4k3/8/8/8/8/8/R7/4K3 w - -"

#: The same endgame 45 plies into a shuffle.
CLOCK_45 = "4k3/8/8/8/8/8/R7/4K3 w - - 45 60"

#: Kings and rooks at home with every right spent, ten plies into a shuffle.
RIGHTS_NONE = "r3k2r/8/8/8/8/8/8/R3K2R b - - 10 30"

#: A queen down the open e-file: a check, six plies into the fifty-move count.
QUEEN_CHECK = "4k3/8/8/4q3/8/8/8/4K3 w - - 6 40"

#: Kings and a rook with room to walk: the repetition cases play from here.
SHUFFLE = "k6r/8/8/8/8/8/8/K7 w - - 0 1"

#: Chess960: kings on g between rooks on f and h, and a pawn just past b5.
NINE_SIXTY = "bqnb1rkr/pp1ppppp/8/1Pp5/8/8/P1PPPPPP/BQNB1RKR w HFhf c6 0 3"

#: The `halfmove_bucket` one-hot opens the `history` row (`features.md` §2.13).
HALFMOVE_BUCKET_AT = 0

#: The helper `conftest.py` hands over.
FactsOf = Callable[..., esca.Facts]


def game_of(fen: str, moves: Sequence[str]) -> esca.Game:
    """The classic game `fen` starts, after the UCI `moves`."""
    game = esca.Game.from_fen(fen)
    for uci in moves:
        game.play(uci)
    return game


def halfmove_bucket(fen: str) -> int:
    """The bucket the encoded `history` row's one-hot sets for `fen`."""
    row = esca.encode([fen], groups=["history"])[0]
    return list(row[HALFMOVE_BUCKET_AT : HALFMOVE_BUCKET_AT + 8]).index(1.0)


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
    assert facts_of(fen).history.halfmove_clock == clock
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
    history = facts_of(fen).history
    assert history.halfmove_known == known
    assert history.halfmove_clock == clock


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
    assert game_of(fen, moves).facts().history.repetition_seen == seen


@pytest.mark.parametrize(
    ("fen", "moves", "available"),
    [
        (START, [], False),
        (START, ["g1f3"], False),
        (START, ["g1f3", "g8f6", "f3g1"], True),
        (START, ["g1f3", "g8f6", "f3g1", "f6g8"], True),
        (SHUFFLE, ["a1b1", "h8h7", "b1a1"], True),
        (SHUFFLE, ["a1b1", "h8h7", "b1a1", "h7h8"], True),
        (SHUFFLE, ["a1b1", "h8h7", "b1a1", "h7h6"], False),
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
def test_a_repetition_is_available_when_one_of_our_moves_reaches_the_history(
    fen: str, moves: Sequence[str], available: bool
) -> None:
    assert game_of(fen, moves).facts().history.repetition_available == available


@pytest.mark.parametrize(
    "fen",
    [START, SHUFFLE, CLOCK_45, QUEEN_CHECK],
    ids=["start", "shuffle", "clock_45", "queen_check"],
)
def test_a_history_is_known_only_to_the_game_that_holds_it(fen: str, facts_of: FactsOf) -> None:
    assert not facts_of(fen).history.known
    assert game_of(fen, []).facts().history.known


@pytest.mark.parametrize(
    ("fen", "moves"),
    [
        (START, ["g1f3", "g8f6", "f3g1", "f6g8"]),
        (SHUFFLE, ["a1b1", "h8h7", "b1a1", "h7h8"]),
        (SHUFFLE, ["a1b1", "h8h7", "b1a1"]),
    ],
    ids=["start_knights_home", "shuffle_rook_back", "shuffle_three_plies"],
)
def test_a_position_on_its_own_carries_none_of_the_repetition_facts(
    fen: str, moves: Sequence[str], facts_of: FactsOf
) -> None:
    game = game_of(fen, moves)
    played = game.facts().history
    bare = facts_of(game.position.fen).history

    assert played.repetition_seen or played.repetition_available, "the game sees something to repeat here"
    assert not bare.known
    assert not bare.repetition_seen
    assert not bare.repetition_available


def test_the_history_facts_of_a_chess960_position_are_the_clock_it_carries(facts_of: FactsOf) -> None:
    """No `history` fact is one of the four `features.md` §4 defines for classic
    chess only, and a Chess960 position carries its clock like any other."""
    history = facts_of(NINE_SIXTY, esca.CHESS960).history
    assert history.halfmove_clock == 0
    assert history.halfmove_known
    assert not history.known
    assert not history.repetition_seen
    assert not history.repetition_available
