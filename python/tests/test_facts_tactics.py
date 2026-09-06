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

#: Knight for knight, each defended by a pawn on neither side: an even trade.
TRADE = "4k3/8/2p5/3n4/8/2N5/8/4K3 w - - 0 1"

#: The same board with Black to move, whose knight has nothing behind it.
TRADE_THEIRS = "4k3/8/2p5/3n4/8/2N5/8/4K3 b - - 0 1"

#: The only capture is a rook taking a pawn a pawn defends.
LOSING_TAKE = "4k3/8/2p5/3p4/8/8/3R4/4K3 w - - 0 1"

#: The same board with Black to move, who has no capture at all.
LOSING_TAKE_THEIRS = "4k3/8/2p5/3p4/8/8/3R4/4K3 b - - 0 1"

#: Chess960: castling long lands the king on c1 and the rook on d1, in check.
NINE_SIXTY = "3k3r/8/8/8/8/8/8/RK6 w A - 0 1"

#: Rxe8 takes the rook and checks in one, onto a square nothing covers.
SAFE_CHECK_CAPTURE = "k3r3/8/8/8/8/8/4R3/4R2K w - - 0 1"

#: The same board with Black to move.
SAFE_CHECK_CAPTURE_THEIRS = "k3r3/8/8/8/8/8/4R3/4R2K b - - 0 1"

#: Rxe7 checks and captures, and the king takes the rook straight back.
UNSAFE_CHECK_CAPTURE = "4k3/4r3/8/8/8/8/8/4R2K w - - 0 1"

#: Every knight move uncovers the rook's attack on the queen behind it.
DISCOVERED_QUEEN = "3q2k1/8/8/8/3N4/8/8/3R3K w - - 0 1"

#: The same board with Black to move.
DISCOVERED_QUEEN_THEIRS = "3q2k1/8/8/8/3N4/8/8/3R3K b - - 0 1"

#: The same discovery with the colours exchanged, Black to move.
MIRRORED_QUEEN = "3r3k/8/8/8/3n4/8/8/3Q2K1 b - - 0 1"

#: The same board with White to move.
MIRRORED_QUEEN_THEIRS = "3r3k/8/8/8/3n4/8/8/3Q2K1 w - - 0 1"

#: The same discovery onto a rook: uncovered, but not onto the queen.
DISCOVERED_ROOK = "3r2k1/8/8/8/3N4/8/8/3R3K w - - 0 1"

#: The mating board with the h-pawn a rank on, so the black king has luft.
BACK_RANK_LUFT = "6k1/5pp1/7p/8/8/7r/5PPP/R5K1 w - - 0 1"

#: Ra8 mates; White's own king stands a rank up, on no back rank of its own.
BACK_RANK_ONE_SIDED = "6k1/5ppp/8/8/8/4r2P/5PPK/R7 w - - 0 1"

#: A rook of Black's own seals the eighth rank, so Ra8 arrives without check.
BACK_RANK_BLOCKED = "1r4k1/5ppp/8/8/8/7r/5PPP/R5K1 w - - 0 1"

#: Rd1 attacks the knight the rook reaches nothing from where it stands.
QUIET_THREAT = "6k1/8/8/3n4/8/8/8/R5K1 w - - 0 1"

#: The same board with Black to move.
QUIET_THREAT_THEIRS = "6k1/8/8/3n4/8/8/8/R5K1 b - - 0 1"

#: The rook attacks the knight already, and no move of White's wins more.
THREAT_STANDS = "6k1/8/8/3n4/8/8/8/3R2K1 w - - 0 1"

#: Five legal moves for White, every one of them onto a square a pawn covers.
BOXED_IN = "k7/8/8/8/1p4pp/4p3/5bPP/1N5K w - - 0 1"

#: The same board with Black to move.
BOXED_IN_THEIRS = "k7/8/8/8/1p4pp/4p3/5bPP/1N5K b - - 0 1"

#: The same position with the colours exchanged, so Black is the boxed side.
BOXED_IN_MIRRORED = "1n5k/5Bpp/4P3/1P4PP/8/8/8/K7 b - - 0 1"

#: The knight covers b8, so every promotion there loses the new piece.
GUARDED_PROMOTION = "8/1P6/n6k/8/7K/8/8/8 w - - 0 1"

#: Nothing covers f8: the push alone wins a queen.
FREE_PROMOTION = "8/5P2/7k/8/7K/8/8/8 w - - 0 1"

#: The same board with Black to move.
FREE_PROMOTION_THEIRS = "8/5P2/7k/8/7K/8/8/8 b - - 0 1"

#: The helper `conftest.py` hands over.
FactsOf = Callable[..., esca.Facts]
Files = Callable[[str], esca.FileSet]


def roles(letters: str) -> list[bool]:
    """The five counted roles, named by their letters in the order the group
    writes them: `roles("nr")` is a knight and a rook."""
    return [letter in letters for letter in "pnbrq"]


def check_roles(tactics: esca.TacticsFacts, colour: str) -> list[bool]:
    """The five checking flags of `colour`, in the order `roles` writes them."""
    return [getattr(tactics, f"check_by_{role}").of(colour) for role in ("pawn", "knight", "bishop", "rook", "queen")]


def safe_check_roles(tactics: esca.TacticsFacts, colour: str) -> list[bool]:
    """The same for the checks whose destination is safe."""
    return [
        getattr(tactics, f"safe_check_by_{role}").of(colour) for role in ("pawn", "knight", "bishop", "rook", "queen")
    ]


def promotion_roles(tactics: esca.TacticsFacts, colour: str) -> list[bool]:
    """The four promotion flags of `colour`, queen first."""
    return [getattr(tactics, f"promotes_to_{role}").of(colour) for role in ("queen", "rook", "bishop", "knight")]


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (MATE, [True, False]),
        (MATE_THEIRS, [True, False]),
        (CHECKS, [True, True]),
        (DISCOVERY, [True, False]),
    ],
    ids=["start", "mate", "mate_theirs", "checks", "discovery"],
)
def test_a_check_is_available_when_a_legal_move_leaves_the_enemy_king_attacked(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.check_available == esca.ByColour(*available)


@pytest.mark.parametrize(
    ("fen", "checks"),
    [
        (START, [0, 0]),
        (MATE, [1, 0]),
        (CHECKS, [3, 2]),
        (CHECKS_THEIRS, [3, 2]),
        (PROMOTIONS, [4, 0]),
        (DISCOVERY, [7, 0]),
    ],
    ids=["start", "mate", "checks", "checks_theirs", "promotions", "discovery"],
)
def test_every_checking_move_is_counted_once(fen: str, checks: list[int], facts_of: FactsOf) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.check_count == esca.ByColour(*checks)


@pytest.mark.parametrize(
    ("fen", "white", "black"),
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
    fen: str, white: str, black: str, facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert check_roles(tactics, "white") == roles(white)
    assert check_roles(tactics, "black") == roles(black)


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
    assert tactics.safe_check_available == esca.ByColour(*available)


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
    assert tactics.safe_check_count == esca.ByColour(*checks)


@pytest.mark.parametrize(
    ("fen", "white", "black"),
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
    fen: str, white: str, black: str, facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert safe_check_roles(tactics, "white") == roles(white)
    assert safe_check_roles(tactics, "black") == roles(black)


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (CHECKS, [False, False]),
        (DISCOVERY, [True, False]),
        (DISCOVERY_THEIRS, [True, False]),
    ],
    ids=["start", "checks", "discovery", "discovery_theirs"],
)
def test_a_double_check_is_a_move_that_leaves_two_units_giving_check(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.double_check_available == esca.ByColour(*available)


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (CHECKS, [False, False]),
        (FORKS, [False, False]),
        (DISCOVERY, [True, False]),
        (DISCOVERY_THEIRS, [True, False]),
    ],
    ids=["start", "checks", "forks", "discovery", "discovery_theirs"],
)
def test_a_discovered_check_comes_from_a_unit_that_did_not_move(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.discovered_check_available == esca.ByColour(*available)


@pytest.mark.parametrize(
    ("fen", "mate"),
    [
        (START, [False, False]),
        (STALEMATE, [False, False]),
        (CHECKS, [False, False]),
        (MATE, [True, False]),
        (MATE_THEIRS, [True, False]),
    ],
    ids=["start", "stalemate", "checks", "mate", "mate_theirs"],
)
def test_a_mate_in_1_is_a_legal_move_that_leaves_the_opponent_checkmated(
    fen: str, mate: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.mate_in_1 == esca.ByColour(*mate)


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
    assert tactics.stalemate_in_1 == esca.ByColour(*stalemate)


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
    assert tactics.promotion_available == esca.ByColour(*available)


@pytest.mark.parametrize(
    ("fen", "white", "black"),
    [
        (START, "", ""),
        (CHECKS, "", ""),
        (PROMOTIONS, "bf", "bf"),
        (PROMOTION_CAPTURES, "ab", "fg"),
    ],
    ids=["start", "checks", "promotions", "promotion_captures"],
)
def test_a_promotion_is_filed_under_the_file_it_lands_on(
    fen: str, white: str, black: str, facts_of: FactsOf, files: Files
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.promotion_files.white == files(white)
    assert tactics.promotion_files.black == files(black)


@pytest.mark.parametrize(
    ("fen", "white", "black"),
    [
        (START, [False] * 4, [False] * 4),
        (CHECKS, [False] * 4, [False] * 4),
        (PROMOTIONS, [True] * 4, [True] * 4),
        (PROMOTION_CAPTURES, [True] * 4, [True] * 4),
    ],
    ids=["start", "checks", "promotions", "promotion_captures"],
)
def test_every_promotion_piece_is_obtainable_wherever_a_pawn_may_promote(
    fen: str, white: list[bool], black: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert promotion_roles(tactics, "white") == white
    assert promotion_roles(tactics, "black") == black


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
    assert tactics.safe_promotion_available == esca.ByColour(*available)


@pytest.mark.parametrize(
    ("fen", "white", "black"),
    [
        (START, "", ""),
        (CHECKS, "", ""),
        (PROMOTIONS, "f", "b"),
        (PROMOTION_CAPTURES, "a", "f"),
    ],
    ids=["start", "checks", "promotions", "promotion_captures"],
)
def test_a_guarded_promotion_square_is_left_out_of_the_safe_files(
    fen: str, white: str, black: str, facts_of: FactsOf, files: Files
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.safe_promotion_files.white == files(white)
    assert tactics.safe_promotion_files.black == files(black)


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
    assert tactics.capture_available == esca.ByColour(*available)


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
    assert tactics.capture_count == esca.ByColour(*captures)


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (PINS, [False, False]),
        (FORKS, [False, False]),
        (CHECKS, [True, True]),
        (MATE, [True, False]),
        (MATE_THEIRS, [True, False]),
        (PROMOTION_CAPTURES, [True, True]),
        (DISCOVERY, [True, False]),
    ],
    ids=["start", "pins", "forks", "checks", "mate", "mate_theirs", "promotion_captures", "discovery"],
)
def test_a_capture_wins_when_the_exchange_it_starts_wins_material(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.winning_capture_available == esca.ByColour(*available)


@pytest.mark.parametrize(
    ("fen", "gain"),
    [
        (START, [0, 0]),
        (PINS, [0, 0]),
        (CHECKS, [5, 3]),
        (MATE, [5, 0]),
        (MATE_THEIRS, [5, 0]),
        (PROMOTION_CAPTURES, [13, 13]),
        (IN_CHECK, [5, 0]),
        (DISCOVERY, [3, 0]),
    ],
    ids=["start", "pins", "checks", "mate", "mate_theirs", "promotion_captures", "in_check", "discovery"],
)
def test_the_max_gain_is_the_best_see_over_the_captures_and_never_below_zero(
    fen: str, gain: list[int], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.winning_capture_max_gain == esca.ByColour(*gain)


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (FORKS, [False, False]),
        (CHECKS, [True, True]),
        (DISCOVERY, [True, False]),
        (MATE_THEIRS, [True, False]),
    ],
    ids=["start", "forks", "checks", "discovery", "mate_theirs"],
)
def test_a_hanging_victim_is_one_the_owner_leaves_undefended_under_attack(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.captures_hanging == esca.ByColour(*available)


@pytest.mark.parametrize(
    ("fen", "value"),
    [
        (START, [0, 0]),
        (FORKS, [0, 0]),
        (CHECKS, [5, 3]),
        (CHECKS_THEIRS, [5, 3]),
        (PROMOTION_CAPTURES, [5, 5]),
        (DISCOVERY, [3, 0]),
    ],
    ids=["start", "forks", "checks", "checks_theirs", "promotion_captures", "discovery"],
)
def test_the_hanging_victims_are_ranked_by_value_and_the_largest_is_kept(
    fen: str, value: list[int], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.hanging_victim_max_value == esca.ByColour(*value)


@pytest.mark.parametrize(
    ("fen", "captures"),
    [
        (START, [0, 0]),
        (PROMOTION_CAPTURES, [0, 0]),
        (CHECKS, [0, 1]),
        (CHECKS_THEIRS, [0, 1]),
        (DISCOVERY, [0, 1]),
        (DISCOVERY_THEIRS, [0, 1]),
        (TRADE, [1, 0]),
        (TRADE_THEIRS, [1, 0]),
    ],
    ids=[
        "start",
        "promotion_captures",
        "checks",
        "checks_theirs",
        "discovery",
        "discovery_theirs",
        "trade",
        "trade_theirs",
    ],
)
def test_an_equal_capture_is_one_whose_exchange_comes_out_level(
    fen: str, captures: list[int], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.equal_capture_count == esca.ByColour(*captures)


@pytest.mark.parametrize(
    ("fen", "captures"),
    [
        (START, [0, 0]),
        (CHECKS, [0, 0]),
        (MATE, [0, 1]),
        (MATE_THEIRS, [0, 1]),
        (FORKS, [0, 1]),
        (PINS, [1, 0]),
        (PINS_THEIRS, [1, 0]),
        (LOSING_TAKE, [1, 0]),
        (LOSING_TAKE_THEIRS, [1, 0]),
    ],
    ids=[
        "start",
        "checks",
        "mate",
        "mate_theirs",
        "forks",
        "pins",
        "pins_theirs",
        "losing_take",
        "losing_take_theirs",
    ],
)
def test_a_losing_capture_is_one_whose_exchange_costs_material(
    fen: str, captures: list[int], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.losing_capture_count == esca.ByColour(*captures)


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (PINS, [False, False]),
        (CHECKS, [True, False]),
        (CHECKS_THEIRS, [True, False]),
        (FORKS, [True, True]),
    ],
    ids=["start", "pins", "checks", "checks_theirs", "forks"],
)
def test_a_fork_leaves_the_mover_attacking_two_units_it_may_take(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.fork_available == esca.ByColour(*available)


@pytest.mark.parametrize(
    ("fen", "forks"),
    [
        (START, [0, 0]),
        (CHECKS, [2, 0]),
        (CHECKS_THEIRS, [2, 0]),
        (PROMOTIONS, [3, 0]),
        (FORKS, [1, 1]),
    ],
    ids=["start", "checks", "checks_theirs", "promotions", "forks"],
)
def test_every_forking_move_is_counted_once_however_many_units_it_forks(
    fen: str, forks: list[int], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.fork_count == esca.ByColour(*forks)


@pytest.mark.parametrize(
    ("fen", "value"),
    [
        (START, [0, 0]),
        (FORKS, [5, 5]),
        (CHECKS, [9, 0]),
        (CHECKS_THEIRS, [9, 0]),
        (PROMOTIONS, [9, 0]),
    ],
    ids=["start", "forks", "checks", "checks_theirs", "promotions"],
)
def test_the_forked_value_is_the_largest_single_target_a_forking_king_counting_nine(
    fen: str, value: list[int], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.fork_max_value == esca.ByColour(*value)


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (PINS, [False, False]),
        (CHECKS, [True, False]),
        (CHECKS_THEIRS, [True, False]),
        (FORKS, [True, False]),
    ],
    ids=["start", "pins", "checks", "checks_theirs", "forks"],
)
def test_a_knight_fork_is_one_the_knight_itself_makes(fen: str, available: list[bool], facts_of: FactsOf) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.knight_fork_available == esca.ByColour(*available)


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (FORKS, [False, False]),
        (CHECKS, [True, False]),
        (CHECKS_THEIRS, [True, False]),
        (PROMOTIONS, [True, False]),
    ],
    ids=["start", "forks", "checks", "checks_theirs", "promotions"],
)
def test_a_royal_fork_is_a_fork_one_of_whose_targets_is_the_king(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.royal_fork_available == esca.ByColour(*available)


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (CHECKS, [False, False]),
        (FORKS, [True, False]),
        (MATE, [False, True]),
        (PINS, [True, False]),
        (PINS_THEIRS, [True, False]),
    ],
    ids=["start", "checks", "forks", "mate", "pins", "pins_theirs"],
)
def test_a_pin_is_created_when_the_mover_traps_a_unit_in_front_of_a_dearer_one(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.pin_creation_available == esca.ByColour(*available)


@pytest.mark.parametrize(
    ("fen", "pins"),
    [
        (START, [0, 0]),
        (FORKS, [1, 0]),
        (MATE, [0, 1]),
        (PINS, [3, 0]),
        (PINS_THEIRS, [3, 0]),
        (DISCOVERY, [0, 1]),
    ],
    ids=["start", "forks", "mate", "pins", "pins_theirs", "discovery"],
)
def test_every_move_that_pins_is_counted_once_however_many_pins_it_makes(
    fen: str, pins: list[int], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.pin_creation_count == esca.ByColour(*pins)


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (FORKS, [False, False]),
        (MATE, [True, True]),
        (PINS, [True, False]),
        (PINS_THEIRS, [True, False]),
    ],
    ids=["start", "forks", "mate", "pins", "pins_theirs"],
)
def test_a_skewer_puts_the_dearer_unit_in_front_of_the_cheaper_one(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.skewer_creation_available == esca.ByColour(*available)


@pytest.mark.parametrize(
    ("fen", "available"),
    [
        (START, [False, False]),
        (FORKS, [False, False]),
        (PINS, [False, True]),
        (PINS_THEIRS, [False, True]),
        (DISCOVERY, [True, False]),
    ],
    ids=["start", "forks", "pins", "pins_theirs", "discovery"],
)
def test_a_discovered_attack_uncovers_a_slider_onto_a_piece_worth_three_or_more(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.discovered_attack_available == esca.ByColour(*available)


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
    assert tactics.legal_move_count == esca.ByColour(*moves)


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
    assert tactics.only_moves == esca.ByColour(*only)


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
def test_the_side_not_to_move_is_unread_when_the_null_move_does_not_exist(
    fen: str, available: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.available == esca.ByColour(*available)
    if not available[1]:
        assert tactics.legal_move_count.black == 0
        assert tactics.check_count.black == 0
        assert tactics.capture_count.black == 0
        assert not tactics.only_moves.black
        assert not tactics.check_available.black
        assert not tactics.promotion_available.black


@pytest.mark.parametrize(
    ("fen", "capturing"),
    [
        (START, [False, False]),
        (MATE, [False, False]),
        (SAFE_CHECK_CAPTURE, [True, False]),
        (SAFE_CHECK_CAPTURE_THEIRS, [True, False]),
        (UNSAFE_CHECK_CAPTURE, [False, True]),
        (DISCOVERY, [True, False]),
    ],
    ids=[
        "start",
        "mate",
        "safe_check_capture",
        "safe_check_capture_theirs",
        "unsafe_check_capture",
        "discovery",
    ],
)
def test_a_capturing_check_is_reported_only_when_its_destination_is_safe(
    fen: str, capturing: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.safe_check_capturing == esca.ByColour(*capturing)


@pytest.mark.parametrize(
    ("fen", "uncovered"),
    [
        (START, [False, False]),
        (DISCOVERED_QUEEN, [True, False]),
        (DISCOVERED_QUEEN_THEIRS, [True, False]),
        (MIRRORED_QUEEN, [False, True]),
        (MIRRORED_QUEEN_THEIRS, [False, True]),
        (DISCOVERED_ROOK, [False, False]),
    ],
    ids=[
        "start",
        "discovered_queen",
        "discovered_queen_theirs",
        "mirrored_queen",
        "mirrored_queen_theirs",
        "discovered_rook",
    ],
)
def test_a_discovered_attack_on_the_queen_wants_the_queen_and_no_lesser_unit(
    fen: str, uncovered: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.discovered_attack_on_queen == esca.ByColour(*uncovered)
    for colour in ("white", "black"):
        assert tactics.discovered_attack_available.of(colour) or not (tactics.discovered_attack_on_queen.of(colour))


@pytest.mark.parametrize(
    ("fen", "threat"),
    [
        (START, [False, False]),
        (MATE, [True, False]),
        (MATE_THEIRS, [True, False]),
        (BACK_RANK_LUFT, [False, False]),
        (BACK_RANK_ONE_SIDED, [True, False]),
        (BACK_RANK_BLOCKED, [False, True]),
    ],
    ids=[
        "start",
        "mate",
        "mate_theirs",
        "back_rank_luft",
        "back_rank_one_sided",
        "back_rank_blocked",
    ],
)
def test_a_back_rank_mate_threat_is_a_rook_or_queen_check_on_a_sealed_back_rank(
    fen: str, threat: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.back_rank_mate_threat == esca.ByColour(*threat)


@pytest.mark.parametrize(
    ("fen", "threat"),
    [
        (START, [False, False]),
        (QUIET_THREAT, [True, False]),
        (QUIET_THREAT_THEIRS, [True, False]),
        (THREAT_STANDS, [False, True]),
        (PROMOTION_CAPTURES, [False, False]),
        (MATE, [False, True]),
        (DISCOVERED_QUEEN, [True, False]),
    ],
    ids=[
        "start",
        "quiet_threat",
        "quiet_threat_theirs",
        "threat_stands",
        "promotion_captures",
        "mate",
        "discovered_queen",
    ],
)
def test_a_quiet_threat_leaves_more_to_be_won_than_stands_to_be_won_now(
    fen: str, threat: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.quiet_threat_available == esca.ByColour(*threat)


@pytest.mark.parametrize(
    ("fen", "boxed"),
    [
        (START, [False, False]),
        (MATE, [False, False]),
        (BOXED_IN, [True, False]),
        (BOXED_IN_THEIRS, [True, False]),
        (BOXED_IN_MIRRORED, [False, True]),
        (STALEMATE, [False, True]),
    ],
    ids=[
        "start",
        "mate",
        "boxed_in",
        "boxed_in_theirs",
        "boxed_in_mirrored",
        "stalemate",
    ],
)
def test_a_side_has_no_safe_moves_when_every_legal_move_lands_where_it_can_be_taken(
    fen: str, boxed: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.no_safe_moves == esca.ByColour(*boxed)


@pytest.mark.parametrize(
    ("fen", "positive"),
    [
        (START, [False, False]),
        (PROMOTIONS, [True, True]),
        (GUARDED_PROMOTION, [False, False]),
        (PROMOTION_CAPTURES, [True, True]),
        (FREE_PROMOTION, [True, False]),
        (FREE_PROMOTION_THEIRS, [True, False]),
    ],
    ids=[
        "start",
        "promotions",
        "guarded_promotion",
        "promotion_captures",
        "free_promotion",
        "free_promotion_theirs",
    ],
)
def test_a_promotion_is_see_positive_when_the_exchange_on_its_square_wins_material(
    fen: str, positive: list[bool], facts_of: FactsOf
) -> None:
    tactics = facts_of(fen).tactics
    assert tactics.promotion_see_positive == esca.ByColour(*positive)


def test_a_side_the_null_move_does_not_allow_states_no_tactic_of_its_own(facts_of: FactsOf) -> None:
    """A side with no move to make states no tactic, `no_safe_moves` included:
    false there says the values were not read, not that a safe move exists."""
    theirs = facts_of(IN_CHECK).tactics
    assert not theirs.available.black
    assert not theirs.safe_check_capturing.black
    assert not theirs.discovered_attack_on_queen.black
    assert not theirs.back_rank_mate_threat.black
    assert not theirs.quiet_threat_available.black
    assert not theirs.no_safe_moves.black
    assert not theirs.promotion_see_positive.black


def test_the_tactics_of_a_chess960_position_read_a_castling_by_the_kings_landing_square(
    facts_of: FactsOf,
) -> None:
    """The group reads a Chess960 castling by the square its king lands on:
    Black is checked by the rook the castling brings to d1, and the mover is a
    king, so no checking-role flag is set for it."""
    tactics = facts_of(NINE_SIXTY, esca.CHESS960).tactics

    assert tactics.legal_move_count.white == 12
    assert tactics.check_count.white == 2
    assert check_roles(tactics, "white") == roles("r")
    assert tactics.safe_check_count.white == 2
    assert not tactics.discovered_check_available.white
    assert not tactics.fork_available.white
    assert tactics.skewer_creation_available.white
    assert not tactics.pin_creation_available.white

    assert tactics.legal_move_count.black == 15
    assert tactics.check_count.black == 1
    assert check_roles(tactics, "black") == roles("r")
    assert tactics.skewer_creation_available.black
