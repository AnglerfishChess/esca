"""The `tactics` group, fact by fact.

Every expectation is worked out from the definitions in `docs/features.md` §1
for the named position above it. The `them` block of a position is the `us`
block of the same placement with the other side to move, which is what the null
move of `features.md` §1 makes it. The cases mirror `tests/facts_tactics.rs`.
"""

from __future__ import annotations

from collections.abc import Callable

import esca
import pytest

#: The untouched array: twenty moves a side, and not a tactic among them.
START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

#: White's king has one square left, and Black has a move that stalemates him.
ONLY_MOVE = "8/8/8/8/8/p1k5/P7/K7 w - - 0 1"

#: White stands in check, so the null move their block needs does not exist.
IN_CHECK = "4k3/8/8/8/8/8/4r3/4K3 w - - 0 1"

#: Black is already without a move; most of White's moves leave it that way.
STALEMATE = "7k/5K2/8/8/8/8/8/1B6 w - - 0 1"

#: Ra8 mates: the eighth rank is sealed and the black rook cannot come back.
MATE = "6k1/5ppp/8/8/8/7r/5PPP/R5K1 w - - 0 1"

#: The same board a tempo later, so the mate is the one they have.
MATE_THEIRS = "6k1/5ppp/8/8/8/7r/5PPP/R5K1 b - - 0 1"

#: Knight and rook checks a side; two of White's fork the king and a rook.
CHECKS = "4k3/8/8/5N1r/1n5R/8/8/4K3 w - - 0 1"

#: The same board with Black to move, so each block changes sides.
CHECKS_THEIRS = "4k3/8/8/5N1r/1n5R/8/8/4K3 b - - 0 1"

#: Two promotions a side, one of each guarded by the enemy knight.
PROMOTIONS = "8/1P3P2/n6k/8/7K/4N3/1p3p2/8 w - - 0 1"

#: Each side may promote by pushing or by taking the rook that stands in reach.
PROMOTION_CAPTURES = "r7/1P6/8/8/2K4k/8/6p1/5R2 w - - 0 1"

#: A knight fork for White and a rook fork for Black, both on loose pieces.
FORKS = "2r3k1/5r2/8/8/4N3/8/7K/1R3N2 w - - 0 1"

#: The rook's file pins the knight to the queen; taking it skewers her instead.
PINS = "3r4/3q4/7k/3n4/8/8/1B6/3R3K w - - 0 1"

#: The same board with Black to move.
PINS_THEIRS = "3r4/3q4/7k/3n4/8/8/1B6/3R3K b - - 0 1"

#: Every knight move uncovers the rook's check, and two of them check twice.
DISCOVERY = "r3k3/7p/8/6n1/4N3/8/3N4/2B1R2K w - - 0 1"

#: The same board with Black to move.
DISCOVERY_THEIRS = "r3k3/7p/8/6n1/4N3/8/3N4/2B1R2K b - - 0 1"

#: Chess960: castling long lands the king on c1 and the rook on d1, in check.
NINE_SIXTY = "3k3r/8/8/8/8/8/8/RK6 w A - 0 1"

#: The helper `conftest.py` hands over.
FactsOf = Callable[..., esca.Facts]


def roles(letters: str) -> list[bool]:
    """The five counted roles, named by their letters in the order the schema
    writes them: `roles("nr")` is a knight and a rook."""
    return [letter in letters for letter in "pnbrq"]


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (MATE, [True, False]),
        (MATE_THEIRS, [False, True]),
        (CHECKS, [True, True]),
        (DISCOVERY, [True, False]),
    ],
    ids=["start", "mate", "mate_theirs", "checks", "discovery"],
)
def test_a_check_is_available_when_a_legal_move_leaves_the_enemy_king_attacked(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].check_available == available[0]
    assert tactics[esca.THEM].check_available == available[1]


@pytest.mark.parametrize(
    ("fen", "checks"),
    [
        (START, [0, 0]),
        (MATE, [1, 0]),
        (CHECKS, [3, 2]),
        (CHECKS_THEIRS, [2, 3]),
        (PROMOTIONS, [4, 0]),
        (DISCOVERY, [7, 0]),
    ],
    ids=["start", "mate", "checks", "checks_theirs", "promotions", "discovery"],
)
def test_every_checking_move_is_counted_once(fen: str, checks: list[int], facts_of: FactsOf) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].check_count == checks[0]
    assert tactics[esca.THEM].check_count == checks[1]


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "", ""),
        (MATE, "r", ""),
        (CHECKS, "nr", "n"),
        (PROMOTIONS, "pn", ""),
        (PROMOTION_CAPTURES, "r", "pr"),
        (PINS, "b", "q"),
    ],
    ids=["start", "mate", "checks", "promotions", "promotion_captures", "pins"],
)
def test_a_checking_move_is_recorded_against_the_role_that_makes_it(
    fen: str, us: str, them: str, facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].check_by_role == roles(us)
    assert tactics[esca.THEM].check_by_role == roles(them)


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (MATE, [True, False]),
        (FORKS, [False, True]),
        (CHECKS, [True, True]),
        (PROMOTION_CAPTURES, [True, True]),
    ],
    ids=["start", "mate", "forks", "checks", "promotion_captures"],
)
def test_a_safe_check_is_a_check_whose_destination_the_enemy_cannot_profitably_take(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].safe_check_available == available[0]
    assert tactics[esca.THEM].safe_check_available == available[1]


@pytest.mark.parametrize(
    ("fen", "checks"),
    [
        (START, [0, 0]),
        (FORKS, [0, 2]),
        (CHECKS, [3, 2]),
        (PROMOTION_CAPTURES, [1, 3]),
        (PINS, [1, 1]),
        (DISCOVERY, [7, 0]),
    ],
    ids=["start", "forks", "checks", "promotion_captures", "pins", "discovery"],
)
def test_only_the_checks_with_a_safe_destination_are_counted_safe(
    fen: str, checks: list[int], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].safe_check_count == checks[0]
    assert tactics[esca.THEM].safe_check_count == checks[1]


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "", ""),
        (FORKS, "", "r"),
        (CHECKS, "nr", "n"),
        (PROMOTIONS, "pn", ""),
        (PROMOTION_CAPTURES, "r", "pr"),
        (PINS, "b", "q"),
    ],
    ids=["start", "forks", "checks", "promotions", "promotion_captures", "pins"],
)
def test_a_safe_check_is_recorded_against_the_role_that_makes_it(
    fen: str, us: str, them: str, facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].safe_check_by_role == roles(us)
    assert tactics[esca.THEM].safe_check_by_role == roles(them)


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (CHECKS, [False, False]),
        (DISCOVERY, [True, False]),
        (DISCOVERY_THEIRS, [False, True]),
    ],
    ids=["start", "checks", "discovery", "discovery_theirs"],
)
def test_a_double_check_is_a_move_that_leaves_two_units_giving_check(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].double_check_available == available[0]
    assert tactics[esca.THEM].double_check_available == available[1]


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (CHECKS, [False, False]),
        (FORKS, [False, False]),
        (DISCOVERY, [True, False]),
        (DISCOVERY_THEIRS, [False, True]),
    ],
    ids=["start", "checks", "forks", "discovery", "discovery_theirs"],
)
def test_a_discovered_check_comes_from_a_unit_that_did_not_move(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].discovered_check_available == available[0]
    assert tactics[esca.THEM].discovered_check_available == available[1]


@pytest.mark.parametrize(
    ("fen", "mate"),
    [
        (START, [False, False]),
        (STALEMATE, [False, False]),
        (CHECKS, [False, False]),
        (MATE, [True, False]),
        (MATE_THEIRS, [False, True]),
    ],
    ids=["start", "stalemate", "checks", "mate", "mate_theirs"],
)
def test_a_mate_in_1_is_a_legal_move_that_leaves_the_opponent_checkmated(
    fen: str, mate: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].mate_in_1 == mate[0]
    assert tactics[esca.THEM].mate_in_1 == mate[1]


@pytest.mark.parametrize(
    ("fen", "stalemate"),
    [
        (START, [False, False]),
        (MATE, [False, False]),
        (CHECKS, [False, False]),
        (STALEMATE, [True, False]),
        (ONLY_MOVE, [False, True]),
    ],
    ids=["start", "mate", "checks", "stalemate", "only_move"],
)
def test_a_stalemate_in_1_is_a_legal_move_that_leaves_the_opponent_without_one(
    fen: str, stalemate: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].stalemate_in_1 == stalemate[0]
    assert tactics[esca.THEM].stalemate_in_1 == stalemate[1]


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (CHECKS, [False, False]),
        (PROMOTIONS, [True, True]),
        (PROMOTION_CAPTURES, [True, True]),
    ],
    ids=["start", "checks", "promotions", "promotion_captures"],
)
def test_a_promotion_is_available_when_a_legal_move_makes_one(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].promotion_available == available[0]
    assert tactics[esca.THEM].promotion_available == available[1]


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "", ""),
        (CHECKS, "", ""),
        (PROMOTIONS, "bf", "bf"),
        (PROMOTION_CAPTURES, "ab", "fg"),
    ],
    ids=["start", "checks", "promotions", "promotion_captures"],
)
def test_a_promotion_is_filed_under_the_file_it_lands_on(fen: str, us: str, them: str, facts_of: FactsOf) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].promotion_files == us
    assert tactics[esca.THEM].promotion_files == them


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, [False] * 4, [False] * 4),
        (CHECKS, [False] * 4, [False] * 4),
        (PROMOTIONS, [True] * 4, [True] * 4),
        (PROMOTION_CAPTURES, [True] * 4, [True] * 4),
    ],
    ids=["start", "checks", "promotions", "promotion_captures"],
)
def test_every_promotion_piece_is_obtainable_wherever_a_pawn_may_promote(
    fen: str, us: list[bool], them: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].promotion_roles == us
    assert tactics[esca.THEM].promotion_roles == them


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (CHECKS, [False, False]),
        (PROMOTIONS, [True, True]),
        (PROMOTION_CAPTURES, [True, True]),
    ],
    ids=["start", "checks", "promotions", "promotion_captures"],
)
def test_a_safe_promotion_is_one_whose_destination_is_a_safe_destination(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].safe_promotion_available == available[0]
    assert tactics[esca.THEM].safe_promotion_available == available[1]


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "", ""),
        (CHECKS, "", ""),
        (PROMOTIONS, "f", "b"),
        (PROMOTION_CAPTURES, "a", "f"),
    ],
    ids=["start", "checks", "promotions", "promotion_captures"],
)
def test_a_guarded_promotion_square_is_left_out_of_the_safe_files(
    fen: str, us: str, them: str, facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].safe_promotion_files == us
    assert tactics[esca.THEM].safe_promotion_files == them


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (PROMOTIONS, [False, False]),
        (IN_CHECK, [True, False]),
        (CHECKS, [True, True]),
        (FORKS, [False, True]),
        (PINS, [True, False]),
    ],
    ids=["start", "promotions", "in_check", "checks", "forks", "pins"],
)
def test_a_capture_is_available_when_a_legal_move_takes_a_unit(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].capture_available == available[0]
    assert tactics[esca.THEM].capture_available == available[1]


@pytest.mark.parametrize(
    ("fen", "captures"),
    [
        (START, [0, 0]),
        (IN_CHECK, [1, 0]),
        (CHECKS, [2, 2]),
        (FORKS, [0, 1]),
        (PROMOTION_CAPTURES, [4, 4]),
    ],
    ids=["start", "in_check", "checks", "forks", "promotion_captures"],
)
def test_each_capturing_move_counts_for_itself_so_four_promotions_count_four(
    fen: str, captures: list[int], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].capture_count == captures[0]
    assert tactics[esca.THEM].capture_count == captures[1]


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (PINS, [False, False]),
        (CHECKS, [True, True]),
        (MATE, [True, False]),
        (MATE_THEIRS, [False, True]),
    ],
    ids=["start", "pins", "checks", "mate", "mate_theirs"],
)
def test_a_capture_wins_when_the_victim_outvalues_the_capturer_or_stands_undefended(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].winning_capture_available == available[0]
    assert tactics[esca.THEM].winning_capture_available == available[1]


@pytest.mark.parametrize(
    ("fen", "gain"),
    [
        (START, [0, 0]),
        (CHECKS, [0, 0]),
        (MATE, [4, 0]),
        (PROMOTION_CAPTURES, [4, 4]),
        (IN_CHECK, [5, 0]),
    ],
    ids=["start", "checks", "mate", "promotion_captures", "in_check"],
)
def test_the_gain_of_a_capture_is_the_victim_less_the_capturer_and_never_below_zero(
    fen: str, gain: list[int], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].winning_capture_max_gain == gain[0]
    assert tactics[esca.THEM].winning_capture_max_gain == gain[1]


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (FORKS, [False, False]),
        (CHECKS, [True, True]),
        (DISCOVERY, [True, False]),
        (MATE_THEIRS, [False, True]),
    ],
    ids=["start", "forks", "checks", "discovery", "mate_theirs"],
)
def test_a_hanging_victim_is_one_the_owner_leaves_undefended_under_attack(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].captures_hanging == available[0]
    assert tactics[esca.THEM].captures_hanging == available[1]


@pytest.mark.parametrize(
    ("fen", "value"),
    [
        (START, [0, 0]),
        (FORKS, [0, 0]),
        (CHECKS, [5, 3]),
        (CHECKS_THEIRS, [3, 5]),
        (PROMOTION_CAPTURES, [5, 5]),
        (DISCOVERY, [3, 0]),
    ],
    ids=["start", "forks", "checks", "checks_theirs", "promotion_captures", "discovery"],
)
def test_the_hanging_victims_are_ranked_by_value_and_the_largest_is_kept(
    fen: str, value: list[int], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].hanging_victim_max_value == value[0]
    assert tactics[esca.THEM].hanging_victim_max_value == value[1]


@pytest.mark.parametrize(
    ("fen", "captures"),
    [
        (START, [0, 0]),
        (PROMOTION_CAPTURES, [0, 0]),
        (CHECKS, [0, 1]),
        (CHECKS_THEIRS, [1, 0]),
        (DISCOVERY, [0, 1]),
    ],
    ids=["start", "promotion_captures", "checks", "checks_theirs", "discovery"],
)
def test_a_capture_of_a_defended_unit_of_equal_value_is_an_equal_one(
    fen: str, captures: list[int], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].equal_capture_count == captures[0]
    assert tactics[esca.THEM].equal_capture_count == captures[1]


@pytest.mark.parametrize(
    ("fen", "captures"),
    [
        (START, [0, 0]),
        (CHECKS, [0, 0]),
        (MATE, [0, 1]),
        (FORKS, [0, 1]),
        (PINS, [1, 0]),
        (PINS_THEIRS, [0, 1]),
    ],
    ids=["start", "checks", "mate", "forks", "pins", "pins_theirs"],
)
def test_a_capture_of_a_defended_unit_of_lower_value_is_a_losing_one(
    fen: str, captures: list[int], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].losing_capture_count == captures[0]
    assert tactics[esca.THEM].losing_capture_count == captures[1]


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (PINS, [False, False]),
        (CHECKS, [True, False]),
        (CHECKS_THEIRS, [False, True]),
        (FORKS, [True, True]),
    ],
    ids=["start", "pins", "checks", "checks_theirs", "forks"],
)
def test_a_fork_leaves_the_mover_attacking_two_units_it_may_take(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].fork_available == available[0]
    assert tactics[esca.THEM].fork_available == available[1]


@pytest.mark.parametrize(
    ("fen", "forks"),
    [
        (START, [0, 0]),
        (CHECKS, [2, 0]),
        (CHECKS_THEIRS, [0, 2]),
        (PROMOTIONS, [3, 0]),
        (FORKS, [1, 1]),
    ],
    ids=["start", "checks", "checks_theirs", "promotions", "forks"],
)
def test_every_forking_move_is_counted_once_however_many_units_it_forks(
    fen: str, forks: list[int], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].fork_count == forks[0]
    assert tactics[esca.THEM].fork_count == forks[1]


@pytest.mark.parametrize(
    ("fen", "value"),
    [
        (START, [0, 0]),
        (FORKS, [5, 5]),
        (CHECKS, [9, 0]),
        (CHECKS_THEIRS, [0, 9]),
        (PROMOTIONS, [9, 0]),
    ],
    ids=["start", "forks", "checks", "checks_theirs", "promotions"],
)
def test_the_forked_value_is_the_largest_single_target_a_forking_king_counting_nine(
    fen: str, value: list[int], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].fork_max_value == value[0]
    assert tactics[esca.THEM].fork_max_value == value[1]


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (PINS, [False, False]),
        (CHECKS, [True, False]),
        (CHECKS_THEIRS, [False, True]),
        (FORKS, [True, False]),
    ],
    ids=["start", "pins", "checks", "checks_theirs", "forks"],
)
def test_a_knight_fork_is_one_the_knight_itself_makes(fen: str, available: list[bool], facts_of: FactsOf) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].knight_fork_available == available[0]
    assert tactics[esca.THEM].knight_fork_available == available[1]


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (FORKS, [False, False]),
        (CHECKS, [True, False]),
        (CHECKS_THEIRS, [False, True]),
        (PROMOTIONS, [True, False]),
    ],
    ids=["start", "forks", "checks", "checks_theirs", "promotions"],
)
def test_a_royal_fork_is_a_fork_one_of_whose_targets_is_the_king(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].royal_fork_available == available[0]
    assert tactics[esca.THEM].royal_fork_available == available[1]


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (CHECKS, [False, False]),
        (FORKS, [True, False]),
        (MATE, [False, True]),
        (PINS, [True, False]),
        (PINS_THEIRS, [False, True]),
    ],
    ids=["start", "checks", "forks", "mate", "pins", "pins_theirs"],
)
def test_a_pin_is_created_when_the_mover_traps_a_unit_in_front_of_a_dearer_one(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].pin_creation_available == available[0]
    assert tactics[esca.THEM].pin_creation_available == available[1]


@pytest.mark.parametrize(
    ("fen", "pins"),
    [
        (START, [0, 0]),
        (FORKS, [1, 0]),
        (MATE, [0, 1]),
        (PINS, [3, 0]),
        (PINS_THEIRS, [0, 3]),
        (DISCOVERY, [0, 1]),
    ],
    ids=["start", "forks", "mate", "pins", "pins_theirs", "discovery"],
)
def test_every_move_that_pins_is_counted_once_however_many_pins_it_makes(
    fen: str, pins: list[int], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].pin_creation_count == pins[0]
    assert tactics[esca.THEM].pin_creation_count == pins[1]


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (FORKS, [False, False]),
        (MATE, [True, True]),
        (PINS, [True, False]),
        (PINS_THEIRS, [False, True]),
    ],
    ids=["start", "forks", "mate", "pins", "pins_theirs"],
)
def test_a_skewer_puts_the_dearer_unit_in_front_of_the_cheaper_one(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].skewer_creation_available == available[0]
    assert tactics[esca.THEM].skewer_creation_available == available[1]


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (FORKS, [False, False]),
        (PINS, [False, True]),
        (PINS_THEIRS, [True, False]),
        (DISCOVERY, [True, False]),
    ],
    ids=["start", "forks", "pins", "pins_theirs", "discovery"],
)
def test_a_discovered_attack_uncovers_a_slider_onto_a_piece_worth_three_or_more(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].discovered_attack_available == available[0]
    assert tactics[esca.THEM].discovered_attack_available == available[1]


@pytest.mark.parametrize(
    ("fen", "moves"),
    [
        (START, [20, 20]),
        (ONLY_MOVE, [1, 6]),
        (IN_CHECK, [3, 0]),
        (STALEMATE, [13, 0]),
        (CHECKS, [22, 16]),
        (PROMOTION_CAPTURES, [30, 27]),
    ],
    ids=["start", "only_move", "in_check", "stalemate", "checks", "promotion_captures"],
)
def test_the_legal_moves_are_counted_for_us_and_for_them_after_the_null_move(
    fen: str, moves: list[int], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].legal_move_count == moves[0]
    assert tactics[esca.THEM].legal_move_count == moves[1]


@pytest.mark.parametrize(
    ("fen", "only"),
    [
        (START, [False, False]),
        (CHECKS, [False, False]),
        (IN_CHECK, [False, False]),
        (ONLY_MOVE, [True, False]),
        (STALEMATE, [False, True]),
    ],
    ids=["start", "checks", "in_check", "only_move", "stalemate"],
)
def test_a_side_is_down_to_only_moves_with_at_most_two_of_them_to_choose_from(
    fen: str, only: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].only_moves == only[0]
    assert tactics[esca.THEM].only_moves == only[1]


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [True, True]),
        (STALEMATE, [True, True]),
        (DISCOVERY, [True, True]),
        (IN_CHECK, [True, False]),
    ],
    ids=["start", "stalemate", "discovery", "in_check"],
)
def test_their_block_is_unavailable_and_empty_when_the_null_move_does_not_exist(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics[esca.US].available == available[0]
    assert tactics[esca.THEM].available == available[1]
    if not available[1]:
        them = tactics[esca.THEM]
        assert (them.legal_move_count, them.check_count, them.capture_count, them.fork_count) == (0, 0, 0, 0)
        assert not them.only_moves
        assert not them.check_available
        assert not them.promotion_available


def test_the_tactics_of_a_chess960_position_read_a_castling_by_the_kings_landing_square(
    facts_of: FactsOf,
) -> None:
    """No `tactics` fact is among the four `features.md` §4 keeps for classic
    chess only, and the group reads a Chess960 castling by the square its king
    lands on: Black is checked by the rook the castling brings to d1, and the
    mover is a king, so no `check_by_role` bit is set."""
    facts = facts_of(NINE_SIXTY, esca.CHESS960)
    ours = facts.tactics[esca.US]
    theirs = facts.tactics[esca.THEM]

    assert ours.legal_move_count == 12
    assert ours.check_count == 2
    assert ours.check_by_role == roles("r")
    assert ours.safe_check_count == 2
    assert not ours.discovered_check_available
    assert not ours.fork_available
    assert ours.skewer_creation_available
    assert not ours.pin_creation_available

    assert theirs.legal_move_count == 15
    assert theirs.check_count == 1
    assert theirs.check_by_role == roles("r")
    assert theirs.skewer_creation_available
