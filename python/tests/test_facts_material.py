"""The `material` group, fact by fact.

Every expectation is worked out from the definitions in `docs/features.md` §1
and §2.2 for the named position above it. `piece_count_diff`,
`material_balance` and `phase_bucket` are derived at encoding time and are read
off the group's own values. The cases mirror `tests/facts_material.rs`.
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


def material_layout() -> dict[str, slice]:
    """Where each `material` feature sits inside its group, per the schema's own text."""
    layout: dict[str, slice] = {}
    at = 0
    inside = False
    for line in esca.SCHEMA.canonical().splitlines():
        if not line.startswith(" "):
            inside = line.startswith("material:")
        elif inside:
            name, width, _encoding = line.split(":")
            layout[name.strip()] = slice(at, at + int(width))
            at += int(width)
    return layout


#: The offset and width of every `material` feature.
MATERIAL = material_layout()


def encoded(fen: str, feature: str) -> list[float]:
    """The values `material.<feature>` encodes to for `fen`."""
    row = esca.encode([fen], groups=["material"])[0]
    return [float(value) for value in row[MATERIAL[feature]]]


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, [8, 2, 2, 2, 1], [8, 2, 2, 2, 1]),
        (TWO_QUEENS_EACH, [6, 1, 1, 2, 2], [6, 1, 1, 2, 2]),
        (HEAVY, [6, 0, 1, 2, 1], [6, 1, 0, 2, 1]),
        (ARMY_AGAINST_A_KING, [8, 2, 2, 2, 1], [0, 0, 0, 0, 0]),
        (QUEEN_FOR_A_ROOK, [5, 0, 0, 0, 1], [3, 0, 0, 1, 0]),
        (QUEEN_FOR_A_ROOK_THEIRS, [3, 0, 0, 1, 0], [5, 0, 0, 0, 1]),
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
def test_the_units_of_a_side_are_counted_by_role(fen: str, us: list[int], them: list[int], facts_of: FactsOf) -> None:
    material = facts_of(fen).material
    assert material.count[esca.US] == us
    assert material.count[esca.THEM] == them


@pytest.mark.parametrize(
    ("fen", "diff"),
    [
        (START, [0.0, 0.0, 0.0, 0.0, 0.0]),
        (HEAVY_AND_A_KNIGHT, [0.0, 0.0, 0.25, 0.0, 0.0]),
        (HEAVY, [0.0, -0.25, 0.25, 0.0, 0.0]),
        (ARMY_AGAINST_A_KING, [1.0, 0.5, 0.5, 0.5, 0.25]),
        (QUEEN_FOR_A_ROOK, [0.5, 0.0, 0.0, -0.25, 0.25]),
        (QUEEN_FOR_A_ROOK_THEIRS, [-0.5, 0.0, 0.0, 0.25, -0.25]),
        (ROOK_AND_KNIGHT, [0.25, 0.25, 0.0, 0.0, 0.0]),
        (SAME_COLOUR_BISHOPS, [0.0, -0.5, 0.5, 0.0, 0.0]),
        (LONE_KNIGHT, [-0.25, 0.25, 0.0, -0.25, 0.0]),
        (PAWN_ENDING, [0.25, 0.0, 0.0, 0.0, 0.0]),
    ],
    ids=[
        "start",
        "heavy_and_a_knight",
        "heavy",
        "army_against_a_king",
        "queen_for_a_rook",
        "queen_for_a_rook_theirs",
        "rook_and_knight",
        "same_colour_bishops",
        "lone_knight",
        "pawn_ending",
    ],
)
def test_the_count_difference_is_ours_less_theirs_by_role(fen: str, diff: list[float]) -> None:
    assert encoded(fen, "piece_count_diff") == pytest.approx(diff)


@pytest.mark.parametrize(
    ("fen", "non_pawn_value"),
    [
        (START, (31, 31)),
        (TWO_QUEENS_EACH, (34, 34)),
        (HEAVY_AND_A_KNIGHT, (25, 22)),
        (QUEENLESS, (22, 22)),
        (ARMY_AGAINST_A_KING, (31, 0)),
        (QUEEN_FOR_A_ROOK, (9, 5)),
        (QUEEN_FOR_A_ROOK_THEIRS, (5, 9)),
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
    assert facts_of(fen).material.non_pawn_value == non_pawn_value


@pytest.mark.parametrize(
    ("fen", "value", "balance"),
    [
        (START, (39, 39), 0.0),
        (TWO_QUEENS_EACH, (40, 40), 0.0),
        (HEAVY_AND_A_KNIGHT, (31, 28), 0.15),
        (ARMY_AGAINST_A_KING, (39, 0), 1.0),
        (QUEEN_FOR_A_ROOK, (14, 8), 0.3),
        (QUEEN_FOR_A_ROOK_THEIRS, (8, 14), -0.3),
        (ROOK_AND_KNIGHT, (14, 10), 0.2),
        (LONE_KNIGHT, (3, 6), -0.15),
        (PAWN_ENDING, (4, 3), 0.05),
        (BARE_KINGS, (0, 0), 0.0),
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
def test_the_balance_is_our_value_sum_less_theirs(
    fen: str, value: tuple[int, int], balance: float, facts_of: FactsOf
) -> None:
    assert facts_of(fen).material.value == value
    assert encoded(fen, "material_balance") == pytest.approx([balance])


@pytest.mark.parametrize(
    ("fen", "phase"),
    [
        (START, 1.0),
        (TWO_QUEENS_EACH, 1.0),
        (HEAVY_AND_A_KNIGHT, 19 / 24),
        (HEAVY, 0.75),
        (QUEENLESS, 16 / 24),
        (ARMY_AGAINST_A_KING, 0.5),
        (QUEEN_FOR_A_ROOK, 0.25),
        (ROOK_AND_KNIGHT, 5 / 24),
        (SAME_COLOUR_BISHOPS, 4 / 24),
        (BISHOP_PAIR, 0.125),
        (PAWN_ENDING, 0.0),
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
def test_phase_weighs_what_is_left_against_a_full_opening_set(fen: str, phase: float, facts_of: FactsOf) -> None:
    assert facts_of(fen).material.phase == pytest.approx(phase)


@pytest.mark.parametrize(
    ("fen", "bucket"),
    [
        (START, [1.0, 0.0, 0.0]),
        (TWO_QUEENS_EACH, [1.0, 0.0, 0.0]),
        (HEAVY_AND_A_KNIGHT, [1.0, 0.0, 0.0]),
        (HEAVY, [0.0, 1.0, 0.0]),
        (QUEENLESS, [0.0, 1.0, 0.0]),
        (ARMY_AGAINST_A_KING, [0.0, 1.0, 0.0]),
        (QUEEN_FOR_A_ROOK, [0.0, 1.0, 0.0]),
        (ROOK_AND_KNIGHT, [0.0, 0.0, 1.0]),
        (BISHOP_PAIR, [0.0, 0.0, 1.0]),
        (PAWN_ENDING, [0.0, 0.0, 1.0]),
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
        "bishop_pair",
        "pawn_ending",
    ],
)
def test_the_phase_bucket_keeps_both_its_boundaries_in_the_middlegame(fen: str, bucket: list[float]) -> None:
    assert encoded(fen, "phase_bucket") == pytest.approx(bucket)


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
    assert facts_of(fen).material.insufficient == insufficient


@pytest.mark.parametrize(
    ("fen", "imbalance"),
    [
        (START, 0),
        (QUEENLESS, 0),
        (SAME_COLOUR_BISHOPS, 0),
        (ARMY_AGAINST_A_KING, 1),
        (BISHOP_PAIR, 1),
        (BISHOP_PAIR_THEIRS, -1),
        (ONE_BISHOP_AGAINST_TWO, -1),
        (LONE_KNIGHT, 0),
        (BARE_KINGS, 0),
    ],
    ids=[
        "start",
        "queenless",
        "same_colour_bishops",
        "army_against_a_king",
        "bishop_pair",
        "bishop_pair_theirs",
        "one_bishop_against_two",
        "lone_knight",
        "bare_kings",
    ],
)
def test_only_bishops_of_both_colours_are_a_pair(fen: str, imbalance: int, facts_of: FactsOf) -> None:
    assert facts_of(fen).material.bishop_pair_imbalance == imbalance
    assert encoded(fen, "bishop_pair_imbalance") == pytest.approx([imbalance])


@pytest.mark.parametrize(
    ("fen", "difference"),
    [
        (START, 0.0),
        (TWO_QUEENS_EACH, 0.0),
        (HEAVY_AND_A_KNIGHT, 3 / 20),
        (ARMY_AGAINST_A_KING, 1.0),
        (QUEEN_FOR_A_ROOK, 4 / 20),
        (QUEEN_FOR_A_ROOK_THEIRS, -4 / 20),
        (ROOK_AND_KNIGHT, 3 / 20),
        (ONE_BISHOP_AGAINST_TWO, 0.0),
        (LONE_KNIGHT, -2 / 20),
        (PAWN_ENDING, 0.0),
    ],
    ids=[
        "start",
        "two_queens_each",
        "heavy_and_a_knight",
        "army_against_a_king",
        "queen_for_a_rook",
        "queen_for_a_rook_theirs",
        "rook_and_knight",
        "one_bishop_against_two",
        "lone_knight",
        "pawn_ending",
    ],
)
def test_the_piece_value_difference_runs_past_its_scale_only_with_a_whole_army(fen: str, difference: float) -> None:
    assert encoded(fen, "non_pawn_material_diff") == pytest.approx([difference])


def test_the_material_facts_of_a_chess960_position_are_the_classic_ones(facts_of: FactsOf) -> None:
    """No `material` fact is one of the four `features.md` §4 defines for
    classic chess only, so a Chess960 position answers as the same placement
    would."""
    material = facts_of(NINE_SIXTY, esca.CHESS960).material
    assert material.count[esca.US] == [8, 1, 2, 2, 1]
    assert material.count[esca.THEM] == [8, 2, 2, 2, 1]
    assert material.non_pawn_value == (28, 31)
    assert material.value == (36, 39)
    assert material.phase == pytest.approx(23 / 24)
    assert material.both_queens
    assert not material.pawns_only
    assert material.insufficient == (False, False)
    assert material.bishop_pair_imbalance == 0

    classic = facts_of("nnqrkr1b/1p1pp3/3p4/p4ppp/PP5P/1b2PP2/2PPK1P1/N1QR1RBB w - - 0 10").material
    assert classic.count == material.count
    assert classic.value == material.value
    assert classic.phase == material.phase
