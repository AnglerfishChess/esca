"""The `attacks` group, fact by fact.

Every expectation is worked out from the definitions in `docs/features.md` §1
for the named position above it. The cases mirror `tests/facts_attacks.rs`.
"""

from __future__ import annotations

from collections.abc import Callable

import esca
import pytest

#: The untouched array: every unit but the two rooks stands on a defended square.
START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

#: The checked king stops the rook's ray on it, and the pawn covers two squares.
RAYS = "4k3/8/8/8/8/1N6/1p6/r2K4 w - - 0 1"

#: Rooks face each other down an open file; a knight and a bishop stand before pawns.
LOOSE = "3r2k1/7p/6p1/p4N2/1b2P3/2P5/1P4PP/3R2K1 w - - 0 1"

#: The checking rook is defended, the rook that defends the checked king is not.
CHECKED = "4k3/8/6b1/8/8/8/4r3/1R2K1n1 w - - 0 1"

#: Two white units may not leave the line to their king, and one black unit may not.
PINS = "4k3/5p2/2n5/1B2q3/1b6/2N1N2Q/8/4K3 w - - 0 1"

#: The same placement with Black to move: every fact of it changes sides.
PINS_THEIRS = "4k3/5p2/2n5/1B2q3/1b6/2N1N2Q/8/4K3 b - - 0 1"

#: A rook and a bishop each look through a piece at a cheaper one behind it.
SKEWERS = "1r1r2k1/6b1/8/1N1qN3/8/8/1P6/3R2K1 w - - 0 1"

#: Sliders looking through four units at what stands behind, of every value.
BEHIND = "2n3k1/Rrr5/b6b/8/q7/8/3K4/2Q5 w - - 0 1"

#: A castled middlegame: the pinned f7-pawn is neither hanging nor en prise.
CASTLED = "3q1rk1/5ppp/3p1n2/8/1bBP4/2N1P3/5PPP/3Q1RK1 w - - 0 1"

#: A Chess960 middlegame: three loose pawns a side, and a bishop loose on b3.
NINE_SIXTY = "nnqrkr1b/1p1pp3/3p4/p4ppp/PP5P/1b2PP2/2PPK1P1/N1QR1RBB w fd - 0 10"

#: The role order `by_role` is written in.
PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING = range(6)

#: The helpers `conftest.py` hands over.
FactsOf = Callable[..., esca.Facts]
Squares = Callable[[str], set[str]]


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (
            START,
            "a2 a3 b1 b2 b3 c1 c2 c3 d1 d2 d3 e1 e2 e3 f1 f2 f3 g1 g2 g3 h2 h3",
            "a6 a7 b6 b7 b8 c6 c7 c8 d6 d7 d8 e6 e7 e8 f6 f7 f8 g6 g7 g8 h6 h7",
        ),
        (
            RAYS,
            "a1 a5 c1 c2 c5 d2 d4 e1 e2",
            "a1 a2 a3 a4 a5 a6 a7 a8 b1 c1 d1 d7 d8 e7 f7 f8",
        ),
        (
            CHECKED,
            "a1 b2 b3 b4 b5 b6 b7 b8 c1 d1 d2 e1 e2 f1 f2",
            "a2 b1 b2 c2 d2 d3 d7 d8 e1 e2 e3 e4 e5 e6 e7 e8 f2 f3 f5 f7 f8 g2 h2 h3 h5 h7",
        ),
    ],
    ids=["start", "rays", "checked"],
)
def test_a_side_attacks_every_square_one_of_its_units_could_capture_on(
    fen: str, us: str, them: str, facts_of: FactsOf, squares: Squares
) -> None:
    attacks = facts_of(fen).attacks
    assert set(attacks.by[esca.US]) == squares(us)
    assert set(attacks.by[esca.THEM]) == squares(them)


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, 22, 22),
        (RAYS, 9, 16),
        (LOOSE, 29, 26),
        (CHECKED, 15, 26),
        (SKEWERS, 26, 34),
        (BEHIND, 23, 43),
        (CASTLED, 34, 29),
    ],
    ids=["start", "rays", "loose", "checked", "skewers", "behind", "castled"],
)
def test_the_attacked_square_count_is_the_size_of_that_map(fen: str, us: int, them: int, facts_of: FactsOf) -> None:
    attacks = facts_of(fen).attacks
    assert len(attacks.by[esca.US]) == us
    assert len(attacks.by[esca.THEM]) == them


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "a3 b3 c3 d3 e3 f3 g3 h3", "a6 b6 c6 d6 e6 f6 g6 h6"),
        (RAYS, "", "a1 c1"),
        (LOOSE, "a3 b4 c3 d4 d5 f3 f5 g3 h3", "b4 f5 g6 h5"),
        (CASTLED, "c5 d4 e3 e5 f3 f4 g3 h3", "c5 e5 e6 f6 g6 h6"),
    ],
    ids=["start", "rays", "loose", "castled"],
)
def test_a_pawn_attacks_the_two_squares_diagonally_ahead_of_it_and_no_other(
    fen: str, us: str, them: str, facts_of: FactsOf, squares: Squares
) -> None:
    attacks = facts_of(fen).attacks
    assert set(attacks.by_pawns[esca.US]) == squares(us)
    assert set(attacks.by_pawns[esca.THEM]) == squares(them)


def test_the_map_is_kept_per_role_as_well(facts_of: FactsOf, squares: Squares) -> None:
    """The whole map is the union of the six role maps, so a role a side has
    none of contributes nothing."""
    attacks = facts_of(LOOSE).attacks

    assert set(attacks.by_role[esca.US][KNIGHT]) == squares("d4 d6 e3 e7 g3 g7 h4 h6")
    assert set(attacks.by_role[esca.US][ROOK]) == squares("a1 b1 c1 d2 d3 d4 d5 d6 d7 d8 e1 f1 g1")
    assert set(attacks.by_role[esca.US][KING]) == squares("f1 f2 g2 h1 h2")
    assert set(attacks.by_role[esca.THEM][BISHOP]) == squares("a3 a5 c3 c5 d6 e7 f8")
    assert not attacks.by_role[esca.THEM][QUEEN]

    for side in (esca.US, esca.THEM):
        union: set[str] = set()
        for role in (PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING):
            union |= set(attacks.by_role[side][role])
        assert union == set(attacks.by[side])
        assert set(attacks.by_pawns[side]) == set(attacks.by_role[side][PAWN])


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "", ""),
        (RAYS, "", ""),
        (LOOSE, "d1", "d8"),
        (CHECKED, "b1", ""),
        (PINS, "c3", "c6"),
        (PINS_THEIRS, "c6", "c3"),
        (SKEWERS, "b5 d1 e5", ""),
        (BEHIND, "a7", ""),
        (CASTLED, "c3", ""),
    ],
    ids=["start", "rays", "loose", "checked", "pins", "pins_theirs", "skewers", "behind", "castled"],
)
def test_a_hanging_unit_is_attacked_and_undefended_and_is_never_a_king(
    fen: str, us: str, them: str, facts_of: FactsOf, squares: Squares
) -> None:
    attacks = facts_of(fen).attacks
    assert set(attacks.hanging[esca.US]) == squares(us)
    assert set(attacks.hanging[esca.THEM]) == squares(them)


@pytest.mark.parametrize(
    ("fen", "value"),
    [
        (START, (0, 0)),
        (RAYS, (0, 0)),
        (LOOSE, (5, 5)),
        (CHECKED, (5, 0)),
        (PINS, (3, 3)),
        (SKEWERS, (11, 0)),
        (BEHIND, (5, 0)),
        (CASTLED, (3, 0)),
    ],
    ids=["start", "rays", "loose", "checked", "pins", "skewers", "behind", "castled"],
)
def test_the_hanging_value_adds_up_what_the_hanging_units_are_worth(
    fen: str, value: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).attacks.hanging_value == value


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "", ""),
        (RAYS, "", "a1"),
        (LOOSE, "d1 f5", "b4 d8"),
        (CHECKED, "b1", ""),
        (PINS, "c3", "c6"),
        (SKEWERS, "b5 d1 e5", "d5"),
        (BEHIND, "a7 c1", ""),
        (CASTLED, "c3", ""),
    ],
    ids=["start", "rays", "loose", "checked", "pins", "skewers", "behind", "castled"],
)
def test_a_unit_is_en_prise_when_it_hangs_or_a_cheaper_unit_attacks_it(
    fen: str, us: str, them: str, facts_of: FactsOf, squares: Squares
) -> None:
    attacks = facts_of(fen).attacks
    assert set(attacks.en_prise[esca.US]) == squares(us)
    assert set(attacks.en_prise[esca.THEM]) == squares(them)


@pytest.mark.parametrize(
    ("fen", "value"),
    [
        (START, (0, 0)),
        (RAYS, (0, 5)),
        (LOOSE, (5, 5)),
        (CHECKED, (5, 0)),
        (PINS, (3, 3)),
        (SKEWERS, (5, 9)),
        (BEHIND, (9, 0)),
        (CASTLED, (3, 0)),
    ],
    ids=["start", "rays", "loose", "checked", "pins", "skewers", "behind", "castled"],
)
def test_the_en_prise_maximum_is_the_largest_value_standing_en_prise(
    fen: str, value: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).attacks.en_prise_max_value == value


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "", ""),
        (RAYS, "", ""),
        (LOOSE, "", ""),
        (CHECKED, "", ""),
        (PINS, "c3 e3", "c6"),
        (PINS_THEIRS, "c6", "c3 e3"),
        (BEHIND, "", ""),
        (CASTLED, "", "f7"),
    ],
    ids=["start", "rays", "loose", "checked", "pins", "pins_theirs", "behind", "castled"],
)
def test_a_pinned_unit_is_the_only_thing_between_a_slider_and_its_own_king(
    fen: str, us: str, them: str, facts_of: FactsOf, squares: Squares
) -> None:
    attacks = facts_of(fen).attacks
    assert set(attacks.pinned[esca.US]) == squares(us)
    assert set(attacks.pinned[esca.THEM]) == squares(them)


@pytest.mark.parametrize(
    ("fen", "skewers"),
    [
        (START, (0, 0)),
        (LOOSE, (0, 0)),
        (CHECKED, (0, 0)),
        (PINS, (0, 0)),
        (SKEWERS, (1, 2)),
        (BEHIND, (2, 1)),
        (CASTLED, (0, 0)),
    ],
    ids=["start", "loose", "checked", "pins", "skewers", "behind", "castled"],
)
def test_a_skewer_is_counted_once_per_slider_front_unit_and_cheaper_unit_behind(
    fen: str, skewers: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).attacks.skewer_candidates == skewers


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (
            START,
            "a2 b1 b2 c1 c2 d1 d2 e1 e2 f1 f2 g1 g2 h2",
            "a7 b7 b8 c7 c8 d7 d8 e7 e8 f7 f8 g7 g8 h7",
        ),
        (RAYS, "", "a1"),
        (LOOSE, "c3 f5 g1 g2 h2", "a5 b4 g6 g8 h7"),
        (CHECKED, "e1", "e2 e8"),
        (PINS, "b5 e3", "b4 e5 e8 f7"),
        (SKEWERS, "g1", "b8 d5 d8 g7 g8"),
        (BEHIND, "c1 d2", "a6 b7 c7 c8"),
        (CASTLED, "d1 d4 e3 f1 f2 g1 g2 h2", "d6 d8 f6 f7 f8 g7 g8 h7"),
    ],
    ids=["start", "rays", "loose", "checked", "pins", "skewers", "behind", "castled"],
)
def test_a_defended_unit_stands_on_a_square_its_own_side_attacks(
    fen: str, us: str, them: str, facts_of: FactsOf, squares: Squares
) -> None:
    attacks = facts_of(fen).attacks
    assert set(attacks.defended[esca.US]) == squares(us)
    assert set(attacks.defended[esca.THEM]) == squares(them)


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (RAYS, "b3 d1", "a1 b2 e8"),
        (LOOSE, "b2 c3 d1 e4 f5 g1 g2 h2", "a5 b4 d8 g6 g8 h7"),
        (SKEWERS, "b2 b5 d1 e5 g1", "b8 d5 d8 g7 g8"),
        (BEHIND, "a7 c1 d2", "a4 a6 b7 c7 c8 g8 h6"),
        (PINS_THEIRS, "b4 c6 e5 e8 f7", "b5 c3 e1 e3 h3"),
    ],
    ids=["rays", "loose", "skewers", "behind", "pins_theirs"],
)
def test_the_units_of_each_side_are_listed_us_first(
    fen: str, us: str, them: str, facts_of: FactsOf, squares: Squares
) -> None:
    attacks = facts_of(fen).attacks
    assert set(attacks.units(esca.US)) == squares(us)
    assert set(attacks.units(esca.THEM)) == squares(them)


@pytest.mark.parametrize(
    ("fen", "square", "us", "them"),
    [
        (LOOSE, "d4", "c3 d1 f5", "d8"),
        (LOOSE, "f5", "e4", "g6"),
        (BEHIND, "c1", "d2", "c7"),
        (BEHIND, "b7", "a7", "a6 c7"),
        (CHECKED, "e1", "b1", "e2"),
        (PINS, "e3", "h3", "e5"),
        (SKEWERS, "d5", "d1", "d8"),
    ],
    ids=[
        "three_at_the_centre",
        "pawn_against_pawn",
        "a_king_is_an_attacker_too",
        "two_on_one_square",
        "the_checked_square",
        "through_no_one",
        "down_the_open_file",
    ],
)
def test_the_attackers_of_a_square_are_the_units_of_a_side_that_bear_on_it(
    fen: str, square: str, us: str, them: str, facts_of: FactsOf, squares: Squares
) -> None:
    attacks = facts_of(fen).attacks
    assert set(attacks.attackers_of(square, esca.US)) == squares(us)
    assert set(attacks.attackers_of(square, esca.THEM)) == squares(them)


@pytest.mark.parametrize(
    ("fen", "square", "hanging"),
    [
        (LOOSE, "d1", True),
        (LOOSE, "d8", True),
        (LOOSE, "f5", False),
        (LOOSE, "g1", False),
        (SKEWERS, "b5", True),
        (SKEWERS, "d5", False),
    ],
    ids=["ours", "theirs", "defended", "unattacked", "one_of_three", "en_prise_but_defended"],
)
def test_a_unit_of_either_colour_is_asked_whether_it_hangs(
    fen: str, square: str, hanging: bool, facts_of: FactsOf
) -> None:
    assert facts_of(fen).attacks.is_hanging(square) is hanging


def test_the_attack_facts_of_a_chess960_position_are_the_classic_ones(facts_of: FactsOf, squares: Squares) -> None:
    """No `attacks` fact is one of the four `features.md` §4 defines for classic
    chess only, so a Chess960 position answers as the same placement would."""
    attacks = facts_of(NINE_SIXTY, esca.CHESS960).attacks
    assert len(attacks.by[esca.US]) == 28
    assert len(attacks.by[esca.THEM]) == 36
    assert set(attacks.hanging[esca.US]) == squares("a4 b4 h4")
    assert set(attacks.hanging[esca.THEM]) == squares("a5 b3 g5")
    assert attacks.hanging_value == (3, 5)
    assert set(attacks.en_prise[esca.THEM]) == squares("a5 b3 g5")
    assert attacks.en_prise_max_value == (1, 3)
    assert set(attacks.defended[esca.US]) == squares("a1 c1 c2 d1 d2 e3 f1 f3 g1 g2")
    assert not attacks.pinned[esca.US]
    assert not attacks.pinned[esca.THEM]
    assert attacks.skewer_candidates == (0, 0)

    classic = facts_of("nnqrkr1b/1p1pp3/3p4/p4ppp/PP5P/1b2PP2/2PPK1P1/N1QR1RBB w - - 0 10").attacks
    assert set(classic.by[esca.THEM]) == set(attacks.by[esca.THEM])
    assert set(classic.hanging[esca.US]) == set(attacks.hanging[esca.US])
    assert classic.hanging_value == attacks.hanging_value
