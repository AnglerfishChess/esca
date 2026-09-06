"""The `material` group, fact by fact.

Every expectation is worked out from the definitions in `docs/features.md` for
the named position above it; a per-side case writes White first. The cases
mirror `tests/facts_material.rs`.
"""

from __future__ import annotations

from collections.abc import Callable

import esca
import pytest

#: The untouched array: a full set a side, and a phase of exactly 1.
START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

#: A queen too many a side: 28 phase points, capped back to a full opening.
TWO_QUEENS_EACH = "r1bqk2r/ppp1qppp/2n5/8/8/2N5/PPP1QPPP/R1BQK2R w KQkq - 0 1"

#: Queens, all four rooks and three minors: nineteen points, still an opening.
HEAVY_AND_A_KNIGHT = "r3k2r/pppq1ppp/2n5/8/8/8/PPPQ1PPP/RNB1K2R w KQkq - 0 1"

#: One minor fewer: eighteen points, the top of the middlegame bucket.
HEAVY = "r3k2r/pppq1ppp/2n5/8/8/8/PPPQ1PPP/R1B1K2R w KQkq - 0 1"

#: The array with both queens gone: two thirds of a full set.
QUEENLESS = "rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNB1KBNR w KQkq - 0 1"

#: A whole army against a bare king: both differences run past their scale.
ARMY_AGAINST_A_KING = "4k3/8/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ - 0 1"

#: A queen and five pawns against a rook and three: six points, the bottom of
#: the middlegame bucket.
QUEEN_FOR_A_ROOK = "r3k3/5ppp/8/8/8/8/PP3PPP/3QK3 w - - 0 1"

#: The same placement read from the other side: every difference changes sign.
QUEEN_FOR_A_ROOK_THEIRS = "r3k3/5ppp/8/8/8/8/PP3PPP/3QK3 b - - 0 1"

#: A knight and a pawn up in a rook ending: five points is already an endgame.
ROOK_AND_KNIGHT = "r3k3/pp3ppp/8/8/8/8/PPP2PPP/4K1NR w Kq - 0 1"

#: Two dark-squared bishops against two knights: only the bishops cannot mate.
SAME_COLOUR_BISHOPS = "4k3/8/8/1n1n4/8/2B1B3/8/4K3 w - - 0 1"

#: Bishops of both colours against a lone knight: only the knight cannot mate.
BISHOP_PAIR = "4k3/6n1/8/8/8/8/8/2B1KB2 w - - 0 1"

#: The same placement read from the other side: the pair is theirs.
BISHOP_PAIR_THEIRS = "4k3/6n1/8/8/8/8/8/2B1KB2 b - - 0 1"

#: A bishop and a knight against two bishops: the same piece value, the pair on
#: one side only.
ONE_BISHOP_AGAINST_TWO = "2b1kb2/8/8/8/8/8/6N1/4K1B1 w - - 0 1"

#: A bare knight against a rook and a pawn.
LONE_KNIGHT = "r3k3/5p2/8/8/8/5N2/8/4K3 w - - 0 1"

#: Kings and pawns, one pawn apart, and nothing to count phase with.
PAWN_ENDING = "4k3/pp4p1/8/8/8/8/P4PPP/4K3 w - - 0 1"

#: Nothing but the kings: every count zero, and neither side able to mate.
BARE_KINGS = "8/8/4k3/8/8/4K3/8/8 w - - 0 1"

#: A Chess960 middlegame; no material fact reads the back rank.
NINE_SIXTY = "nnqrkr1b/1p1pp3/3p4/p4ppp/PP5P/1b2PP2/2PPK1P1/N1QR1RBB w fd - 0 10"

#: The helpers `conftest.py` hands over.
FactsOf = Callable[..., esca.Facts]


@pytest.mark.parametrize(
    ("fen", "white", "black"),
    [
        (START, [8, 2, 2, 2, 1], [8, 2, 2, 2, 1]),
        (TWO_QUEENS_EACH, [6, 1, 1, 2, 2], [6, 1, 1, 2, 2]),
        (HEAVY, [6, 0, 1, 2, 1], [6, 1, 0, 2, 1]),
        (ARMY_AGAINST_A_KING, [8, 2, 2, 2, 1], [0, 0, 0, 0, 0]),
        (QUEEN_FOR_A_ROOK, [5, 0, 0, 0, 1], [3, 0, 0, 1, 0]),
        (QUEEN_FOR_A_ROOK_THEIRS, [5, 0, 0, 0, 1], [3, 0, 0, 1, 0]),
        (ROOK_AND_KNIGHT, [6, 1, 0, 1, 0], [5, 0, 0, 1, 0]),
        (PAWN_ENDING, [4, 0, 0, 0, 0], [3, 0, 0, 0, 0]),
        (BARE_KINGS, [0, 0, 0, 0, 0], [0, 0, 0, 0, 0]),
    ],
    ids=[
        "start",
        "two_queens_each",
        "heavy",
        "army_against_a_king",
        "queen_for_a_rook",
        "queen_for_a_rook_theirs",
        "rook_and_knight",
        "pawn_ending",
        "bare_kings",
    ],
)
def test_the_units_of_a_side_are_counted_by_role(
    fen: str, white: list[int], black: list[int], facts_of: FactsOf
) -> None:
    material = facts_of(fen).material
    assert material.pawns == esca.ByColour(white[0], black[0])
    assert material.knights == esca.ByColour(white[1], black[1])
    assert material.bishops == esca.ByColour(white[2], black[2])
    assert material.rooks == esca.ByColour(white[3], black[3])
    assert material.queens == esca.ByColour(white[4], black[4])


@pytest.mark.parametrize(
    ("fen", "non_pawn_value"),
    [
        (START, (31, 31)),
        (TWO_QUEENS_EACH, (34, 34)),
        (HEAVY_AND_A_KNIGHT, (25, 22)),
        (QUEENLESS, (22, 22)),
        (ARMY_AGAINST_A_KING, (31, 0)),
        (QUEEN_FOR_A_ROOK, (9, 5)),
        (QUEEN_FOR_A_ROOK_THEIRS, (9, 5)),
        (ROOK_AND_KNIGHT, (8, 5)),
        (PAWN_ENDING, (0, 0)),
    ],
    ids=[
        "start",
        "two_queens_each",
        "heavy_and_a_knight",
        "queenless",
        "army_against_a_king",
        "queen_for_a_rook",
        "queen_for_a_rook_theirs",
        "rook_and_knight",
        "pawn_ending",
    ],
)
def test_non_pawn_material_leaves_out_the_pawns_and_the_king(
    fen: str, non_pawn_value: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).material.non_pawn_value == esca.ByColour(*non_pawn_value)


@pytest.mark.parametrize(
    ("fen", "value"),
    [
        (START, (39, 39)),
        (TWO_QUEENS_EACH, (40, 40)),
        (HEAVY_AND_A_KNIGHT, (31, 28)),
        (ARMY_AGAINST_A_KING, (39, 0)),
        (QUEEN_FOR_A_ROOK, (14, 8)),
        (QUEEN_FOR_A_ROOK_THEIRS, (14, 8)),
        (ROOK_AND_KNIGHT, (14, 10)),
        (LONE_KNIGHT, (3, 6)),
        (PAWN_ENDING, (4, 3)),
        (BARE_KINGS, (0, 0)),
    ],
    ids=[
        "start",
        "two_queens_each",
        "heavy_and_a_knight",
        "army_against_a_king",
        "queen_for_a_rook",
        "queen_for_a_rook_theirs",
        "rook_and_knight",
        "lone_knight",
        "pawn_ending",
        "bare_kings",
    ],
)
def test_the_value_of_a_side_is_the_sum_of_its_units_but_the_king(
    fen: str, value: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).material.value == esca.ByColour(*value)


@pytest.mark.parametrize(
    ("fen", "phase_points"),
    [
        (START, 24),
        (TWO_QUEENS_EACH, 28),
        (HEAVY_AND_A_KNIGHT, 19),
        (HEAVY, 18),
        (QUEENLESS, 16),
        (ARMY_AGAINST_A_KING, 12),
        (QUEEN_FOR_A_ROOK, 6),
        (ROOK_AND_KNIGHT, 5),
        (SAME_COLOUR_BISHOPS, 4),
        (BISHOP_PAIR, 3),
        (PAWN_ENDING, 0),
    ],
    ids=[
        "start",
        "two_queens_each",
        "heavy_and_a_knight",
        "heavy",
        "queenless",
        "army_against_a_king",
        "queen_for_a_rook",
        "rook_and_knight",
        "same_colour_bishops",
        "bishop_pair",
        "pawn_ending",
    ],
)
def test_the_phase_points_count_what_is_left_of_a_full_opening_set(
    fen: str, phase_points: int, facts_of: FactsOf
) -> None:
    assert facts_of(fen).material.phase_points == phase_points


@pytest.mark.parametrize(
    ("fen", "both_queens"),
    [
        (START, True),
        (TWO_QUEENS_EACH, True),
        (HEAVY, True),
        (QUEENLESS, False),
        (ARMY_AGAINST_A_KING, False),
        (QUEEN_FOR_A_ROOK, False),
        (QUEEN_FOR_A_ROOK_THEIRS, False),
        (BARE_KINGS, False),
    ],
    ids=[
        "start",
        "two_queens_each",
        "heavy",
        "queenless",
        "army_against_a_king",
        "queen_for_a_rook",
        "queen_for_a_rook_theirs",
        "bare_kings",
    ],
)
def test_both_queens_asks_for_a_queen_on_each_side(fen: str, both_queens: bool, facts_of: FactsOf) -> None:
    assert facts_of(fen).material.both_queens is both_queens


@pytest.mark.parametrize(
    ("fen", "pawns_only"),
    [
        (START, False),
        (QUEENLESS, False),
        (ARMY_AGAINST_A_KING, False),
        (ROOK_AND_KNIGHT, False),
        (LONE_KNIGHT, False),
        (PAWN_ENDING, True),
        (BARE_KINGS, True),
    ],
    ids=[
        "start",
        "queenless",
        "army_against_a_king",
        "rook_and_knight",
        "lone_knight",
        "pawn_ending",
        "bare_kings",
    ],
)
def test_pawns_only_leaves_the_board_to_the_kings_and_the_pawns(fen: str, pawns_only: bool, facts_of: FactsOf) -> None:
    assert facts_of(fen).material.pawns_only is pawns_only


@pytest.mark.parametrize(
    ("fen", "insufficient"),
    [
        (START, (False, False)),
        (QUEEN_FOR_A_ROOK, (False, False)),
        (ARMY_AGAINST_A_KING, (False, True)),
        (SAME_COLOUR_BISHOPS, (True, False)),
        (BISHOP_PAIR, (False, True)),
        (LONE_KNIGHT, (True, False)),
        (PAWN_ENDING, (False, False)),
        (BARE_KINGS, (True, True)),
    ],
    ids=[
        "start",
        "queen_for_a_rook",
        "army_against_a_king",
        "same_colour_bishops",
        "bishop_pair",
        "lone_knight",
        "pawn_ending",
        "bare_kings",
    ],
)
def test_a_side_holding_at_most_a_minor_or_bishops_of_one_colour_cannot_mate(
    fen: str, insufficient: tuple[bool, bool], facts_of: FactsOf
) -> None:
    assert facts_of(fen).material.insufficient_material == esca.ByColour(*insufficient)


def test_the_material_facts_of_a_chess960_position_are_the_classic_ones(facts_of: FactsOf) -> None:
    """No `material` fact reads the back rank, so a Chess960 position answers
    as the same placement would."""
    material = facts_of(NINE_SIXTY, esca.CHESS960).material
    assert material.pawns == esca.ByColour(8, 8)
    assert material.knights == esca.ByColour(1, 2)
    assert material.non_pawn_value == esca.ByColour(28, 31)
    assert material.value == esca.ByColour(36, 39)
    assert material.phase_points == 23
    assert material.both_queens
    assert not material.pawns_only
    assert material.insufficient_material == esca.ByColour(False, False)

    classic = facts_of("nnqrkr1b/1p1pp3/3p4/p4ppp/PP5P/1b2PP2/2PPK1P1/N1QR1RBB w - - 0 10").material
    assert classic.pawns == material.pawns
    assert classic.value == material.value
    assert classic.phase_points == material.phase_points
