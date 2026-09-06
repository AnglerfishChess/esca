"""The `threats` group: what each side stands to lose, and the slider geometry
a threat is made of.

Every expectation is worked out from the definitions in `docs/features.md`
§1 and §2.10 for the named position above it. The cases mirror
`tests/facts_threats.rs`.
"""

from __future__ import annotations

from collections.abc import Callable

import esca
import pytest

#: The untouched array: nothing attacked, and a loose rook in each corner.
START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

#: A rook bears on a knight nothing defends.
HANGING_KNIGHT = "4k3/8/8/3n4/8/8/3R4/4K3 w - - 0 1"

#: The same knight, defended by a pawn: taking it costs the rook.
DEFENDED_KNIGHT = "4k3/8/2p5/3n4/8/8/3R4/4K3 w - - 0 1"

#: A pawn attacks our rook, which nothing defends.
PAWN_ATTACKS_ROOK = "7k/8/8/3p4/4R3/8/8/4K3 w - - 0 1"

#: A knight attacks our queen: the costliest thing a lesser unit can attack.
QUEEN_UNDER_KNIGHT = "7k/8/5n2/8/4Q3/8/8/4K3 w - - 0 1"

#: The same with Black to move: the queen under a knight is theirs.
QUEEN_UNDER_KNIGHT_BLACK = "7k/8/5n2/8/4Q3/8/8/4K3 b - - 0 1"

#: Our pawn and their queen attack each other, both undefended.
CROSS_THREATS = "4k3/8/8/3q4/4P3/8/3R4/4K3 w - - 0 1"

#: The same with Black to move: the two blocks read the other way round.
CROSS_THREATS_BLACK = "4k3/8/8/3q4/4P3/8/3R4/4K3 b - - 0 1"

#: The d7 rook alone defends the two pawns two rooks attack, and is safe itself.
OVERLOADED_ROOK = "6k1/p2r4/8/3p4/8/8/8/R2RK3 w - - 0 1"

#: The same with Black to move.
OVERLOADED_ROOK_BLACK = "6k1/p2r4/8/3p4/8/8/8/R2RK3 b - - 0 1"

#: The same overloaded rook, now hanging to a bishop: the defence can be taken.
REMOVABLE_ROOK = "6k1/p2r4/8/3p4/8/7B/8/R2RK3 w - - 0 1"

#: The same with Black to move.
REMOVABLE_ROOK_BLACK = "6k1/p2r4/8/3p4/8/7B/8/R2RK3 b - - 0 1"

#: A knight alone defends the two pawns the rooks attack; nothing attacks it.
OVERLOADED_KNIGHT = "7k/p3p3/2n5/8/8/8/8/R3R1K1 w - - 0 1"

#: The same with Black to move.
OVERLOADED_KNIGHT_BLACK = "7k/p3p3/2n5/8/8/8/8/R3R1K1 b - - 0 1"

#: The same, with a knight that attacks the defender and hangs to it in turn.
REMOVABLE_KNIGHT = "7k/p3p3/2n5/8/1N6/8/8/R3R1K1 w - - 0 1"

#: The same with Black to move.
REMOVABLE_KNIGHT_BLACK = "7k/p3p3/2n5/8/1N6/8/8/R3R1K1 b - - 0 1"

#: The defender is now defended by a pawn: taking it is an even trade, which is
#: enough to remove it.
REMOVABLE_TRADE = "7k/pp2p3/2n5/8/1N6/8/8/R3R1K1 w - - 0 1"

#: The same with Black to move.
REMOVABLE_TRADE_BLACK = "7k/pp2p3/2n5/8/1N6/8/8/R3R1K1 b - - 0 1"

#: The b7 pawn alone defends the two attacked pawns; the queen that attacks it
#: would lose itself to the rook behind.
OVERLOADED_PAWN = "1r5k/1p6/p1p5/8/8/1Q6/8/R1R3K1 w - - 0 1"

#: The same with Black to move.
OVERLOADED_PAWN_BLACK = "1r5k/1p6/p1p5/8/8/1Q6/8/R1R3K1 b - - 0 1"

#: A bishop attacks the queen and x-rays the rook behind it.
XRAY_BISHOP = "4k1r1/8/8/3q4/8/1B6/8/4K3 w - - 0 1"

#: The same with Black to move.
XRAY_BISHOP_BLACK = "4k1r1/8/8/3q4/8/1B6/8/4K3 b - - 0 1"

#: Our rook x-rays their rook through their pawn; theirs looks through its own.
XRAY_ROOK = "4k3/3r4/8/8/3p4/8/8/3R2K1 w - - 0 1"

#: Both sides double on the d-file, each battery bearing on the other's king.
DOUBLED_ROOKS = "3rk3/3r4/8/8/8/8/3R4/3RK3 w - - 0 1"

#: Two rooks bear on one the enemy defends once: the exchange wins a rook, and
#: no attacker is worth less than its target.
TWO_ON_ONE = "3r3k/3r4/8/8/8/8/3R4/3RK3 w - - 0 1"

#: A queen and a rook on the d-file, with the enemy king's ring on it.
BATTERY_AT_KING = "2k5/8/8/8/8/8/3Q4/3R1K2 w - - 0 1"

#: The same with Black to move.
BATTERY_AT_KING_BLACK = "2k5/8/8/8/8/8/3Q4/3R1K2 b - - 0 1"

#: A queen behind a bishop on the long diagonal, which the enemy king's ring
#: stands on.
BISHOP_BATTERY = "8/7k/8/8/8/2B5/1Q6/6K1 w - - 0 1"

#: The same with Black to move.
BISHOP_BATTERY_BLACK = "8/7k/8/8/8/2B5/1Q6/6K1 b - - 0 1"

#: The helpers `conftest.py` hands over.
FactsOf = Callable[..., esca.Facts]
Squares = Callable[[str], set[str]]


@pytest.mark.parametrize(
    ("fen", "white", "black"),
    [
        (START, "", ""),
        (HANGING_KNIGHT, "", "d5"),
        (DEFENDED_KNIGHT, "", ""),
        (PAWN_ATTACKS_ROOK, "e4", ""),
        (QUEEN_UNDER_KNIGHT, "e4", ""),
        (CROSS_THREATS, "e4", "d5"),
        (CROSS_THREATS_BLACK, "e4", "d5"),
        (OVERLOADED_ROOK, "", ""),
        (REMOVABLE_ROOK, "", "d7"),
        (REMOVABLE_ROOK_BLACK, "", "d7"),
        (REMOVABLE_KNIGHT, "b4", "c6"),
        (REMOVABLE_TRADE, "b4", ""),
        (XRAY_BISHOP, "b3", "d5"),
        (XRAY_BISHOP_BLACK, "b3", "d5"),
        (DOUBLED_ROOKS, "", ""),
        (TWO_ON_ONE, "", "d7"),
    ],
    ids=[
        "start",
        "hanging_knight",
        "defended_knight",
        "pawn_attacks_rook",
        "queen_under_knight",
        "cross_threats",
        "cross_threats_black",
        "overloaded_rook",
        "removable_rook",
        "removable_rook_black",
        "removable_knight",
        "removable_trade",
        "xray_bishop",
        "xray_bishop_black",
        "doubled_rooks",
        "two_on_one",
    ],
)
def test_a_unit_is_threatened_when_the_exchange_on_its_square_wins_material(
    fen: str, white: str, black: str, facts_of: FactsOf, squares: Squares
) -> None:
    threats = facts_of(fen).threats
    assert set(threats.threatened.white) == squares(white)
    assert set(threats.threatened.black) == squares(black)


@pytest.mark.parametrize(
    ("fen", "value"),
    [
        (START, (0, 0)),
        (HANGING_KNIGHT, (0, 3)),
        (DEFENDED_KNIGHT, (0, 0)),
        (PAWN_ATTACKS_ROOK, (5, 0)),
        (QUEEN_UNDER_KNIGHT, (9, 0)),
        (CROSS_THREATS, (1, 9)),
        (CROSS_THREATS_BLACK, (1, 9)),
        (REMOVABLE_ROOK, (0, 5)),
        (REMOVABLE_ROOK_BLACK, (0, 5)),
        (REMOVABLE_KNIGHT, (3, 3)),
        (REMOVABLE_TRADE, (3, 0)),
        (XRAY_BISHOP, (3, 9)),
        (XRAY_BISHOP_BLACK, (3, 9)),
    ],
    ids=[
        "start",
        "hanging_knight",
        "defended_knight",
        "pawn_attacks_rook",
        "queen_under_knight",
        "cross_threats",
        "cross_threats_black",
        "removable_rook",
        "removable_rook_black",
        "removable_knight",
        "removable_trade",
        "xray_bishop",
        "xray_bishop_black",
    ],
)
def test_the_threatened_value_adds_up_what_is_about_to_be_lost(
    fen: str, value: tuple[int, int], facts_of: FactsOf
) -> None:
    threats = facts_of(fen).threats
    assert threats.threatened_value == esca.ByColour(*value)


@pytest.mark.parametrize(
    ("fen", "gain"),
    [
        (START, (0, 0)),
        (HANGING_KNIGHT, (0, 3)),
        (DEFENDED_KNIGHT, (0, 0)),
        (PAWN_ATTACKS_ROOK, (5, 0)),
        (QUEEN_UNDER_KNIGHT, (9, 0)),
        (CROSS_THREATS, (1, 9)),
        (CROSS_THREATS_BLACK, (1, 9)),
        (REMOVABLE_ROOK, (0, 5)),
        (REMOVABLE_ROOK_BLACK, (0, 5)),
        (REMOVABLE_KNIGHT, (3, 3)),
        (REMOVABLE_TRADE, (3, 0)),
        (XRAY_BISHOP, (3, 9)),
        (DOUBLED_ROOKS, (0, 0)),
        (TWO_ON_ONE, (0, 5)),
    ],
    ids=[
        "start",
        "hanging_knight",
        "defended_knight",
        "pawn_attacks_rook",
        "queen_under_knight",
        "cross_threats",
        "cross_threats_black",
        "removable_rook",
        "removable_rook_black",
        "removable_knight",
        "removable_trade",
        "xray_bishop",
        "doubled_rooks",
        "two_on_one",
    ],
)
def test_the_max_gain_is_the_largest_exchange_the_opponent_can_start(
    fen: str, gain: tuple[int, int], facts_of: FactsOf
) -> None:
    threats = facts_of(fen).threats
    assert threats.threat_max_gain == esca.ByColour(*gain)


@pytest.mark.parametrize(
    ("fen", "white", "black"),
    [
        (START, "", ""),
        (HANGING_KNIGHT, "", ""),
        (PAWN_ATTACKS_ROOK, "e4", ""),
        (QUEEN_UNDER_KNIGHT, "e4", ""),
        (CROSS_THREATS, "", "d5"),
        (CROSS_THREATS_BLACK, "", "d5"),
        (REMOVABLE_ROOK, "", "d7"),
        (REMOVABLE_ROOK_BLACK, "", "d7"),
        (REMOVABLE_KNIGHT, "", ""),
        (XRAY_BISHOP, "", "d5"),
        (XRAY_BISHOP_BLACK, "", "d5"),
        (DOUBLED_ROOKS, "", ""),
    ],
    ids=[
        "start",
        "hanging_knight",
        "pawn_attacks_rook",
        "queen_under_knight",
        "cross_threats",
        "cross_threats_black",
        "removable_rook",
        "removable_rook_black",
        "removable_knight",
        "xray_bishop",
        "xray_bishop_black",
        "doubled_rooks",
    ],
)
def test_a_lesser_attacker_is_one_the_defender_would_be_glad_to_trade_with(
    fen: str, white: str, black: str, facts_of: FactsOf, squares: Squares
) -> None:
    threats = facts_of(fen).threats
    assert set(threats.attacked_by_lesser.white) == squares(white)
    assert set(threats.attacked_by_lesser.black) == squares(black)


@pytest.mark.parametrize(
    ("fen", "under"),
    [
        (START, (False, False)),
        (PAWN_ATTACKS_ROOK, (False, False)),
        (QUEEN_UNDER_KNIGHT, (True, False)),
        (QUEEN_UNDER_KNIGHT_BLACK, (True, False)),
        (CROSS_THREATS, (False, True)),
        (CROSS_THREATS_BLACK, (False, True)),
        (XRAY_BISHOP, (False, True)),
        (XRAY_BISHOP_BLACK, (False, True)),
        (DOUBLED_ROOKS, (False, False)),
    ],
    ids=[
        "start",
        "pawn_attacks_rook",
        "queen_under_knight",
        "queen_under_knight_black",
        "cross_threats",
        "cross_threats_black",
        "xray_bishop",
        "xray_bishop_black",
        "doubled_rooks",
    ],
)
def test_a_queen_under_a_lesser_unit_is_its_own_fact(fen: str, under: tuple[bool, bool], facts_of: FactsOf) -> None:
    threats = facts_of(fen).threats
    assert threats.queen_attacked_by_lesser.white is under[0]
    assert threats.queen_attacked_by_lesser.black is under[1]


@pytest.mark.parametrize(
    ("fen", "white", "black"),
    [
        (START, "", ""),
        (DEFENDED_KNIGHT, "", ""),
        (OVERLOADED_ROOK, "", "d7"),
        (OVERLOADED_ROOK_BLACK, "", "d7"),
        (REMOVABLE_ROOK, "", "d7"),
        (OVERLOADED_KNIGHT, "", "c6"),
        (OVERLOADED_KNIGHT_BLACK, "", "c6"),
        (REMOVABLE_KNIGHT, "", "c6"),
        (REMOVABLE_TRADE, "", "c6"),
        (REMOVABLE_TRADE_BLACK, "", "c6"),
        (OVERLOADED_PAWN, "", "b7"),
        (OVERLOADED_PAWN_BLACK, "", "b7"),
        (DOUBLED_ROOKS, "", ""),
    ],
    ids=[
        "start",
        "defended_knight",
        "overloaded_rook",
        "overloaded_rook_black",
        "removable_rook",
        "overloaded_knight",
        "overloaded_knight_black",
        "removable_knight",
        "removable_trade",
        "removable_trade_black",
        "overloaded_pawn",
        "overloaded_pawn_black",
        "doubled_rooks",
    ],
)
def test_a_defender_of_two_attacked_units_is_overloaded(
    fen: str, white: str, black: str, facts_of: FactsOf, squares: Squares
) -> None:
    threats = facts_of(fen).threats
    assert set(threats.overloaded_defenders.white) == squares(white)
    assert set(threats.overloaded_defenders.black) == squares(black)


@pytest.mark.parametrize(
    ("fen", "white", "black"),
    [
        (START, "", ""),
        (OVERLOADED_ROOK, "", ""),
        (REMOVABLE_ROOK, "", "d7"),
        (REMOVABLE_ROOK_BLACK, "", "d7"),
        (OVERLOADED_KNIGHT, "", ""),
        (REMOVABLE_KNIGHT, "", "c6"),
        (REMOVABLE_KNIGHT_BLACK, "", "c6"),
        (REMOVABLE_TRADE, "", "c6"),
        (REMOVABLE_TRADE_BLACK, "", "c6"),
        (OVERLOADED_PAWN, "", ""),
        (OVERLOADED_PAWN_BLACK, "", ""),
    ],
    ids=[
        "start",
        "overloaded_rook",
        "removable_rook",
        "removable_rook_black",
        "overloaded_knight",
        "removable_knight",
        "removable_knight_black",
        "removable_trade",
        "removable_trade_black",
        "overloaded_pawn",
        "overloaded_pawn_black",
    ],
)
def test_a_defender_the_enemy_can_take_for_free_is_removable(
    fen: str, white: str, black: str, facts_of: FactsOf, squares: Squares
) -> None:
    threats = facts_of(fen).threats
    assert set(threats.removable_defenders.white) == squares(white)
    assert set(threats.removable_defenders.black) == squares(black)


@pytest.mark.parametrize(
    ("fen", "white", "black"),
    [
        (START, "a1 h1", "a8 h8"),
        (HANGING_KNIGHT, "", "d5"),
        (DEFENDED_KNIGHT, "", "c6"),
        (PAWN_ATTACKS_ROOK, "e4", "d5"),
        (QUEEN_UNDER_KNIGHT, "e4", "f6"),
        (CROSS_THREATS, "e4", "d5"),
        (OVERLOADED_ROOK, "", "d7"),
        (REMOVABLE_ROOK, "h3", "d7"),
        (REMOVABLE_TRADE, "b4", "b7"),
        (OVERLOADED_PAWN, "b3", "b8"),
        (XRAY_BISHOP, "b3", "d5"),
        (XRAY_ROOK, "d1", ""),
        (DOUBLED_ROOKS, "", ""),
    ],
    ids=[
        "start",
        "hanging_knight",
        "defended_knight",
        "pawn_attacks_rook",
        "queen_under_knight",
        "cross_threats",
        "overloaded_rook",
        "removable_rook",
        "removable_trade",
        "overloaded_pawn",
        "xray_bishop",
        "xray_rook",
        "doubled_rooks",
    ],
)
def test_a_loose_unit_is_one_its_own_side_does_not_defend(
    fen: str, white: str, black: str, facts_of: FactsOf, squares: Squares
) -> None:
    threats = facts_of(fen).threats
    assert set(threats.loose.white) == squares(white)
    assert set(threats.loose.black) == squares(black)


@pytest.mark.parametrize(
    ("fen", "white", "black"),
    [
        (START, "", ""),
        (DEFENDED_KNIGHT, "", ""),
        (PAWN_ATTACKS_ROOK, "e4", ""),
        (QUEEN_UNDER_KNIGHT, "e4", ""),
        (CROSS_THREATS, "", "d5"),
        (CROSS_THREATS_BLACK, "", "d5"),
        (REMOVABLE_ROOK, "", "d7"),
        (REMOVABLE_KNIGHT, "b4", "c6"),
        (REMOVABLE_TRADE, "b4", ""),
        (XRAY_BISHOP, "", "d5"),
        (OVERLOADED_PAWN, "", ""),
        (DOUBLED_ROOKS, "", ""),
    ],
    ids=[
        "start",
        "defended_knight",
        "pawn_attacks_rook",
        "queen_under_knight",
        "cross_threats",
        "cross_threats_black",
        "removable_rook",
        "removable_knight",
        "removable_trade",
        "xray_bishop",
        "overloaded_pawn",
        "doubled_rooks",
    ],
)
def test_a_surplus_counts_only_the_attackers_and_defenders_worth_at_most_the_unit(
    fen: str, white: str, black: str, facts_of: FactsOf, squares: Squares
) -> None:
    threats = facts_of(fen).threats
    assert set(threats.attacker_surplus.white) == squares(white)
    assert set(threats.attacker_surplus.black) == squares(black)


@pytest.mark.parametrize(
    ("fen", "count"),
    [
        (START, (0, 0)),
        (DEFENDED_KNIGHT, (0, 0)),
        (OVERLOADED_ROOK, (1, 0)),
        (OVERLOADED_ROOK_BLACK, (1, 0)),
        (REMOVABLE_ROOK, (1, 0)),
        (XRAY_BISHOP, (1, 0)),
        (XRAY_BISHOP_BLACK, (1, 0)),
        (XRAY_ROOK, (1, 0)),
        (DOUBLED_ROOKS, (1, 1)),
        (OVERLOADED_PAWN, (1, 0)),
        (BISHOP_BATTERY, (0, 0)),
    ],
    ids=[
        "start",
        "defended_knight",
        "overloaded_rook",
        "overloaded_rook_black",
        "removable_rook",
        "xray_bishop",
        "xray_bishop_black",
        "xray_rook",
        "doubled_rooks",
        "overloaded_pawn",
        "bishop_battery",
    ],
)
def test_an_x_ray_looks_through_one_enemy_unit_at_another(fen: str, count: tuple[int, int], facts_of: FactsOf) -> None:
    threats = facts_of(fen).threats
    assert threats.xray_through_enemy == esca.ByColour(*count)


@pytest.mark.parametrize(
    ("fen", "count"),
    [
        (START, (0, 0)),
        (XRAY_ROOK, (0, 0)),
        (OVERLOADED_ROOK, (1, 0)),
        (OVERLOADED_ROOK_BLACK, (1, 0)),
        (OVERLOADED_KNIGHT, (1, 0)),
        (OVERLOADED_PAWN, (1, 0)),
        (DOUBLED_ROOKS, (1, 1)),
        (BATTERY_AT_KING, (1, 0)),
        (BATTERY_AT_KING_BLACK, (1, 0)),
        (BISHOP_BATTERY, (1, 0)),
        (BISHOP_BATTERY_BLACK, (1, 0)),
    ],
    ids=[
        "start",
        "xray_rook",
        "overloaded_rook",
        "overloaded_rook_black",
        "overloaded_knight",
        "overloaded_pawn",
        "doubled_rooks",
        "battery_at_king",
        "battery_at_king_black",
        "bishop_battery",
        "bishop_battery_black",
    ],
)
def test_a_battery_is_two_sliders_on_one_line_they_both_move_along(
    fen: str, count: tuple[int, int], facts_of: FactsOf
) -> None:
    threats = facts_of(fen).threats
    assert threats.battery_count == esca.ByColour(*count)


@pytest.mark.parametrize(
    ("fen", "at_king"),
    [
        (START, (False, False)),
        (OVERLOADED_ROOK, (False, False)),
        (OVERLOADED_KNIGHT, (False, False)),
        (DOUBLED_ROOKS, (True, True)),
        (BATTERY_AT_KING, (True, False)),
        (BATTERY_AT_KING_BLACK, (True, False)),
        (BISHOP_BATTERY, (True, False)),
        (BISHOP_BATTERY_BLACK, (True, False)),
    ],
    ids=[
        "start",
        "overloaded_rook",
        "overloaded_knight",
        "doubled_rooks",
        "battery_at_king",
        "battery_at_king_black",
        "bishop_battery",
        "bishop_battery_black",
    ],
)
def test_a_battery_at_the_king_is_one_whose_line_meets_the_enemy_ring(
    fen: str, at_king: tuple[bool, bool], facts_of: FactsOf
) -> None:
    threats = facts_of(fen).threats
    assert threats.battery_at_king.white is at_king[0]
    assert threats.battery_at_king.black is at_king[1]


@pytest.mark.parametrize(
    ("white_to_move", "black_to_move"),
    [
        (CROSS_THREATS, CROSS_THREATS_BLACK),
        (REMOVABLE_ROOK, REMOVABLE_ROOK_BLACK),
        (OVERLOADED_PAWN, OVERLOADED_PAWN_BLACK),
        (BISHOP_BATTERY, BISHOP_BATTERY_BLACK),
    ],
    ids=["cross_threats", "removable_rook", "overloaded_pawn", "bishop_battery"],
)
def test_the_same_placement_reads_the_same_whoever_is_to_move(
    white_to_move: str, black_to_move: str, facts_of: FactsOf
) -> None:
    """Nothing in the group depends on whose turn it is."""
    first = facts_of(white_to_move).threats
    second = facts_of(black_to_move).threats
    assert set(first.threatened.white) == set(second.threatened.white)
    assert first.threatened_value == second.threatened_value
    assert first.threat_max_gain == second.threat_max_gain
    assert set(first.overloaded_defenders.black) == set(second.overloaded_defenders.black)
    assert set(first.loose.white) == set(second.loose.white)
    assert first.battery_count == second.battery_count


def test_threatened_is_what_en_prise_approximates(facts_of: FactsOf, squares: Squares) -> None:
    """`en_prise` reads one attacker against one defender, `threatened` plays the
    exchange out: two rooks against one win the defended rook that no cheaper
    unit attacks."""
    facts = facts_of(TWO_ON_ONE)
    assert set(facts.attacks.en_prise.black) == squares("")
    assert set(facts.threats.threatened.black) == squares("d7")

    # The other way round on the same board: a knight a pawn defends is en
    # prise to nothing cheaper, and the rook that attacks it wins nothing.
    facts = facts_of(DEFENDED_KNIGHT)
    assert set(facts.attacks.en_prise.black) == squares("")
    assert set(facts.threats.threatened.black) == squares("")
