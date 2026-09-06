"""The `mobility` group, fact by fact.

Every expectation is worked out from the definitions in `docs/features.md` §1
and §2.6 for the named position above it. `mobility_ratio`,
`mobility_diff_by_type` and the control difference are derived at encoding time
and are read off the group's own row. The cases mirror `tests/facts_mobility.rs`.
"""

from __future__ import annotations

from collections.abc import Callable

import esca
import pytest

#: The untouched array: pawn attacks and two knight leaps a side, the rest of
#: the back rank walled in by its own units.
START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

#: One knight out: it frees the rook behind it and reaches into the far half.
ONE_KNIGHT_OUT = "rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQKB1R b KQkq - 0 1"

#: Two queens on touching diagonals: each ray towards the other stops on it.
QUEEN_DUEL = "7k/8/8/3q4/4Q3/8/8/4K3 w - - 0 1"

#: Two rooks against a bare king: all the mobility on the board is ours.
OPEN_ROOKS = "4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1"

#: The same board with the other side to move: every count changes hands.
OPEN_ROOKS_THEIRS = "4k3/8/8/8/8/8/8/R3K2R b KQ - 0 1"

#: A rook on an open board, two enemy pawns covering two squares it reaches.
PAWN_SCREEN = "5k2/8/8/3p1p2/4R3/8/8/4K3 w - - 0 1"

#: The same screen read from the pawns' side.
PAWN_SCREEN_THEIRS = "5k2/8/8/3p1p2/4R3/8/8/4K3 b - - 0 1"

#: Rooks blocking each other on the fourth rank, each side's pawns covering
#: squares the other's rook reaches.
CROSSFIRE = "4k3/8/8/2p1pp2/3R1r2/8/2P1PP2/4K3 w - - 0 1"

#: Rooks nose to nose on the e-file, each side's pawns denying the other one
#: square of it.
TRENCHES = "4k3/8/3p2p1/4r3/4R3/3P1P2/8/4K3 w - - 0 1"

#: A knight with not one legal move, pinned against its king by a rook.
PINNED_KNIGHT = "4r2k/8/8/8/8/8/4N3/4K3 w - - 0 1"

#: A knight and a bishop shut in by their own units, against a shut-in knight.
BOXED_IN = "n3k3/2p5/1p6/8/8/1P6/2P3P1/N3K2B w - - 0 1"

#: Two knights and a bishop covering all four central squares, against a wall
#: of three pawns that covers two of them.
CENTRE_GRIP = "4k3/8/8/3ppp2/8/2N1BN2/8/4K3 w - - 0 1"

#: A pawn phalanx and a knight camped in the enemy half.
SPACE_GRAB = "4k3/8/4N3/3PPP2/8/8/8/4K3 w - - 0 1"

#: Two kings alone: they control squares, but no king's squares are mobility.
BARE_KINGS = "8/8/4k3/8/8/4K3/8/8 w - - 0 1"

#: A Chess960 starting array: the same twelve squares a side as the classic
#: one, reached from other homes.
NINE_SIXTY = "bbqnnrkr/pppppppp/8/8/8/8/PPPPPPPP/BBQNNRKR w HFhf - 0 1"

#: The helpers `conftest.py` hands over.
FactsOf = Callable[..., esca.Facts]


@pytest.mark.parametrize(
    ("fen", "white", "black"),
    [
        (START, [8, 4, 0, 0, 0], [8, 4, 0, 0, 0]),
        (ONE_KNIGHT_OUT, [7, 7, 0, 1, 0], [8, 4, 0, 0, 0]),
        (QUEEN_DUEL, [0, 0, 0, 0, 23], [0, 0, 0, 0, 24]),
        (OPEN_ROOKS, [0, 0, 0, 19, 0], [0, 0, 0, 0, 0]),
        (OPEN_ROOKS_THEIRS, [0, 0, 0, 19, 0], [0, 0, 0, 0, 0]),
        (PAWN_SCREEN, [0, 0, 0, 13, 0], [3, 0, 0, 0, 0]),
        (PAWN_SCREEN_THEIRS, [0, 0, 0, 13, 0], [3, 0, 0, 0, 0]),
        (CROSSFIRE, [5, 0, 0, 12, 0], [4, 0, 0, 6, 0]),
        (BOXED_IN, [5, 0, 0, 0, 0], [3, 0, 0, 0, 0]),
        (CENTRE_GRIP, [0, 15, 11, 0, 0], [5, 0, 0, 0, 0]),
        (SPACE_GRAB, [4, 8, 0, 0, 0], [0, 0, 0, 0, 0]),
        (PINNED_KNIGHT, [0, 6, 0, 0, 0], [0, 0, 0, 12, 0]),
        (BARE_KINGS, [0, 0, 0, 0, 0], [0, 0, 0, 0, 0]),
    ],
    ids=[
        "start",
        "one_knight_out",
        "queen_duel",
        "open_rooks",
        "open_rooks_theirs",
        "pawn_screen",
        "pawn_screen_theirs",
        "crossfire",
        "boxed_in",
        "centre_grip",
        "space_grab",
        "pinned_knight",
        "bare_kings",
    ],
)
def test_a_roles_mobility_is_what_its_attacks_cover_beyond_its_own_units(
    fen: str, white: list[int], black: list[int], facts_of: FactsOf
) -> None:
    mobility = facts_of(fen).mobility
    assert mobility.pawns == esca.ByColour(white[0], black[0])
    assert mobility.knights == esca.ByColour(white[1], black[1])
    assert mobility.bishops == esca.ByColour(white[2], black[2])
    assert mobility.rooks == esca.ByColour(white[3], black[3])
    assert mobility.queens == esca.ByColour(white[4], black[4])


@pytest.mark.parametrize(
    ("fen", "white", "black"),
    [
        (START, [8, 4, 0, 0, 0], [8, 4, 0, 0, 0]),
        (ONE_KNIGHT_OUT, [7, 7, 0, 1, 0], [8, 4, 0, 0, 0]),
        (PAWN_SCREEN, [0, 0, 0, 11, 0], [3, 0, 0, 0, 0]),
        (PAWN_SCREEN_THEIRS, [0, 0, 0, 11, 0], [3, 0, 0, 0, 0]),
        (CROSSFIRE, [5, 0, 0, 9, 0], [4, 0, 0, 5, 0]),
        (TRENCHES, [2, 0, 0, 9, 0], [3, 0, 0, 9, 0]),
        (BOXED_IN, [5, 0, 0, 0, 0], [3, 0, 0, 0, 0]),
        (CENTRE_GRIP, [0, 13, 9, 0, 0], [5, 0, 0, 0, 0]),
    ],
    ids=[
        "start",
        "one_knight_out",
        "pawn_screen",
        "pawn_screen_theirs",
        "crossfire",
        "trenches",
        "boxed_in",
        "centre_grip",
    ],
)
def test_safe_mobility_drops_the_squares_an_enemy_pawn_attacks(
    fen: str, white: list[int], black: list[int], facts_of: FactsOf
) -> None:
    mobility = facts_of(fen).mobility
    assert mobility.safe_pawns == esca.ByColour(white[0], black[0])
    assert mobility.safe_knights == esca.ByColour(white[1], black[1])
    assert mobility.safe_bishops == esca.ByColour(white[2], black[2])
    assert mobility.safe_rooks == esca.ByColour(white[3], black[3])
    assert mobility.safe_queens == esca.ByColour(white[4], black[4])


@pytest.mark.parametrize(
    ("fen", "space"),
    [
        (START, (0, 0)),
        (ONE_KNIGHT_OUT, (2, 0)),
        (QUEEN_DUEL, (8, 8)),
        (OPEN_ROOKS, (8, 0)),
        (OPEN_ROOKS_THEIRS, (8, 0)),
        (PAWN_SCREEN, (4, 3)),
        (CROSSFIRE, (4, 8)),
        (TRENCHES, (1, 1)),
        (CENTRE_GRIP, (8, 5)),
        (SPACE_GRAB, (11, 0)),
    ],
    ids=[
        "start",
        "one_knight_out",
        "queen_duel",
        "open_rooks",
        "open_rooks_theirs",
        "pawn_screen",
        "crossfire",
        "trenches",
        "centre_grip",
        "space_grab",
    ],
)
def test_space_is_what_a_side_attacks_in_the_half_the_other_starts_on(
    fen: str, space: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).mobility.space == esca.ByColour(*space)


@pytest.mark.parametrize(
    ("fen", "controlled"),
    [
        (START, (22, 22)),
        (ONE_KNIGHT_OUT, (26, 22)),
        (QUEEN_DUEL, (28, 26)),
        (OPEN_ROOKS, (23, 5)),
        (OPEN_ROOKS_THEIRS, (23, 5)),
        (PAWN_SCREEN, (18, 8)),
        (CROSSFIRE, (19, 14)),
        (BOXED_IN, (13, 10)),
        (BARE_KINGS, (8, 8)),
    ],
    ids=[
        "start",
        "one_knight_out",
        "queen_duel",
        "open_rooks",
        "open_rooks_theirs",
        "pawn_screen",
        "crossfire",
        "boxed_in",
        "bare_kings",
    ],
)
def test_the_controlled_squares_are_a_sides_whole_attack_map_kings_included(
    fen: str, controlled: tuple[int, int], facts_of: FactsOf
) -> None:
    attacks = facts_of(fen).attacks
    assert len(attacks.controlled.white) == controlled[0]
    assert len(attacks.controlled.black) == controlled[1]


@pytest.mark.parametrize(
    ("fen", "centre"),
    [
        (START, (0, 0)),
        (ONE_KNIGHT_OUT, (2, 0)),
        (QUEEN_DUEL, (3, 3)),
        (PAWN_SCREEN, (2, 1)),
        (PAWN_SCREEN_THEIRS, (2, 1)),
        (CROSSFIRE, (2, 2)),
        (TRENCHES, (3, 3)),
        (CENTRE_GRIP, (4, 2)),
        (SPACE_GRAB, (1, 0)),
        (BARE_KINGS, (2, 2)),
    ],
    ids=[
        "start",
        "one_knight_out",
        "queen_duel",
        "pawn_screen",
        "pawn_screen_theirs",
        "crossfire",
        "trenches",
        "centre_grip",
        "space_grab",
        "bare_kings",
    ],
)
def test_centre_control_counts_the_attacks_on_d4_e4_d5_and_e5(
    fen: str, centre: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).mobility.centre_control == esca.ByColour(*centre)


@pytest.mark.parametrize(
    ("fen", "extended"),
    [
        (START, (4, 4)),
        (ONE_KNIGHT_OUT, (6, 4)),
        (QUEEN_DUEL, (10, 10)),
        (OPEN_ROOKS, (0, 0)),
        (PAWN_SCREEN, (6, 2)),
        (PAWN_SCREEN_THEIRS, (6, 2)),
        (CROSSFIRE, (8, 5)),
        (BOXED_IN, (3, 2)),
        (CENTRE_GRIP, (6, 4)),
        (SPACE_GRAB, (7, 0)),
        (BARE_KINGS, (5, 5)),
    ],
    ids=[
        "start",
        "one_knight_out",
        "queen_duel",
        "open_rooks",
        "pawn_screen",
        "pawn_screen_theirs",
        "crossfire",
        "boxed_in",
        "centre_grip",
        "space_grab",
        "bare_kings",
    ],
)
def test_the_extended_centre_is_the_sixteen_squares_from_c3_to_f6(
    fen: str, extended: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).mobility.extended_centre_control == esca.ByColour(*extended)


@pytest.mark.parametrize(
    ("fen", "immobile"),
    [
        (START, (5, 5)),
        (ONE_KNIGHT_OUT, (4, 5)),
        (BOXED_IN, (2, 1)),
        (QUEEN_DUEL, (0, 0)),
        (OPEN_ROOKS, (0, 0)),
        (CROSSFIRE, (0, 0)),
        (CENTRE_GRIP, (0, 0)),
        (PINNED_KNIGHT, (0, 0)),
    ],
    ids=[
        "start",
        "one_knight_out",
        "boxed_in",
        "queen_duel",
        "open_rooks",
        "crossfire",
        "centre_grip",
        "pinned_knight",
    ],
)
def test_an_immobile_piece_reaches_nothing_its_own_side_has_left_free(
    fen: str, immobile: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).mobility.immobile_pieces == esca.ByColour(*immobile)


@pytest.mark.parametrize(
    ("fen", "total"),
    [
        (START, (12, 12)),
        (ONE_KNIGHT_OUT, (15, 12)),
        (QUEEN_DUEL, (23, 24)),
        (OPEN_ROOKS, (19, 0)),
        (OPEN_ROOKS_THEIRS, (19, 0)),
        (PAWN_SCREEN, (13, 3)),
        (CROSSFIRE, (17, 10)),
        (TRENCHES, (12, 13)),
        (CENTRE_GRIP, (26, 5)),
        (BARE_KINGS, (0, 0)),
    ],
    ids=[
        "start",
        "one_knight_out",
        "queen_duel",
        "open_rooks",
        "open_rooks_theirs",
        "pawn_screen",
        "crossfire",
        "trenches",
        "centre_grip",
        "bare_kings",
    ],
)
def test_the_total_mobility_adds_the_five_types_up(fen: str, total: tuple[int, int], facts_of: FactsOf) -> None:
    assert facts_of(fen).mobility.total == esca.ByColour(*total)


def test_the_mobility_facts_of_a_chess960_position_are_the_classic_ones(facts_of: FactsOf) -> None:
    """Every `mobility` fact reads attack maps alone, so a Chess960 starting
    array answers as the same placement would."""
    facts = facts_of(NINE_SIXTY, esca.CHESS960)
    mobility = facts.mobility
    assert mobility.pawns == esca.ByColour(8, 8)
    assert mobility.knights == esca.ByColour(4, 4)
    assert mobility.safe_pawns == esca.ByColour(8, 8)
    assert mobility.total == esca.ByColour(12, 12)
    assert mobility.space == esca.ByColour(0, 0)
    # The two knights leave e2 and e7 uncovered, so a square fewer than the
    # classic array's 22.
    assert len(facts.attacks.controlled.white) == 21
    assert len(facts.attacks.controlled.black) == 21
    assert mobility.centre_control == esca.ByColour(0, 0)
    assert mobility.extended_centre_control == esca.ByColour(4, 4)
    assert mobility.immobile_pieces == esca.ByColour(5, 5)

    classic = facts_of("bbqnnrkr/pppppppp/8/8/8/8/PPPPPPPP/BBQNNRKR w - - 0 1").mobility
    assert classic.pawns == mobility.pawns
    assert classic.total == mobility.total
    assert classic.immobile_pieces == mobility.immobile_pieces
