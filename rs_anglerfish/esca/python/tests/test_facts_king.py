"""The `king` group, fact by fact.

Every expectation is worked out from the definitions in `docs/features.md` §1
for the named position above it. The cases mirror `tests/facts_king.rs`.
"""

from __future__ import annotations

from collections.abc import Callable

import esca
import pytest

#: The untouched array: both kings home, walled in by their own first rank.
START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

#: Both sides castled short with the shelter intact and the pieces still on.
DEVELOPED = "r1bq1rk1/pp3ppp/2n1pn2/2pp4/3P4/2NBPN2/PPP2PPP/R1BQ1RK1 w - - 0 9"

#: Our king still on e1 with the centre pawns gone; theirs already on g8.
UNCASTLED = "r4rk1/ppp2ppp/8/8/8/8/PPP2PPP/R3K2R w KQ - 0 1"

#: The same placement the other way round: ours on g8's mirror, theirs on e8.
UNCASTLED_THEIRS = "r3k2r/ppp2ppp/8/8/8/8/PPP2PPP/R4RK1 w kq - 0 1"

#: A king in each corner: the king files are read off the clamped centre.
CORNERS = "k7/1p1p4/8/8/8/8/4P1P1/7K w - - 0 1"

#: Enemy pawns two, three and four ranks off our king, four and five off theirs.
STORM = "1k6/p7/1p6/2p4p/1P4pP/P4p2/5P2/6K1 w - - 0 1"

#: One open file and one the enemy has left beside each king.
OPEN_FILES = "2k5/3p3p/8/8/2P5/8/6P1/6K1 w - - 0 1"

#: Four pieces bearing on our ring against three on theirs, shelters intact.
SIEGE = "1k6/ppp1R3/8/2b5/5B1q/1Q1n4/5PPP/r4RK1 w - - 0 1"

#: A queen and a rook cover squares next to a king nothing of ours guards.
HOLES = "8/8/1k6/8/7q/8/5P2/R5K1 w - - 0 1"

#: Black to move, its own g-pawn one rank further on than the rest of the shield.
BOXED = "6k1/5p1p/6p1/8/8/8/5PPP/6K1 b - - 0 1"

#: Black to move with nothing but kings: one in the open, one on the back rank.
BARE_KINGS = "8/8/8/3k4/8/8/8/6K1 b - - 0 1"

#: A pawn ending: each king with an enemy pawn one and two ranks ahead of it.
ENDGAME = "8/8/8/3kp3/3P4/4K3/8/8 w - - 0 1"

#: A Chess960 array whose kings start on e1 and e8 without ever having moved.
NINE_SIXTY_HOME = "nnqrkrbb/pppppppp/8/8/8/8/PPPPPPPP/NNQRKRBB w FDfd - 0 1"

#: A Chess960 array whose kings start on g1 and g8, castled zone and all.
NINE_SIXTY_WING = "bbqnnrkr/pppppppp/8/8/8/8/PPPPPPPP/BBQNNRKR w HFhf - 0 1"

#: The helper `conftest.py` hands over.
FactsOf = Callable[..., esca.Facts]


def relative_rank(square: str, colour: str) -> int:
    """The rank of `square` counted from `colour`'s own back rank."""
    rank = int(square[1])
    return rank if colour == "w" else 9 - rank


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "e", "e"),
        (CORNERS, "h", "a"),
        (HOLES, "g", "b"),
        (BOXED, "g", "g"),
        (BARE_KINGS, "d", "g"),
        (ENDGAME, "e", "d"),
    ],
    ids=["start", "corners", "holes", "boxed", "bare_kings", "endgame"],
)
def test_a_kings_file_is_the_file_of_the_square_it_stands_on(fen: str, us: str, them: str, facts_of: FactsOf) -> None:
    square = facts_of(fen).king.square
    assert square[esca.US][0] == us
    assert square[esca.THEM][0] == them


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, 1, 1),
        (UNCASTLED, 1, 1),
        (HOLES, 1, 3),
        (BOXED, 1, 1),
        (BARE_KINGS, 4, 1),
        (ENDGAME, 3, 4),
    ],
    ids=["start", "uncastled", "holes", "boxed", "bare_kings", "endgame"],
)
def test_a_kings_rank_is_counted_from_its_own_back_rank(fen: str, us: int, them: int, facts_of: FactsOf) -> None:
    facts = facts_of(fen)
    ours = facts.side_to_move
    theirs = "b" if ours == "w" else "w"
    assert relative_rank(facts.king.square[esca.US], ours) == us
    assert relative_rank(facts.king.square[esca.THEM], theirs) == them


@pytest.mark.parametrize(
    ("fen", "home"),
    [
        (START, (True, True)),
        (UNCASTLED, (True, False)),
        (UNCASTLED_THEIRS, (False, True)),
        (DEVELOPED, (False, False)),
        (CORNERS, (False, False)),
        (ENDGAME, (False, False)),
    ],
    ids=["start", "uncastled", "uncastled_theirs", "developed", "corners", "endgame"],
)
def test_a_king_is_home_on_the_e_file_of_its_own_first_rank(
    fen: str, home: tuple[bool, bool], facts_of: FactsOf
) -> None:
    assert facts_of(fen).king.on_home_square == home


@pytest.mark.parametrize(
    ("fen", "queenside", "kingside"),
    [
        (START, (False, False), (False, False)),
        (DEVELOPED, (False, False), (True, True)),
        (CORNERS, (False, True), (True, False)),
        (STORM, (False, True), (True, False)),
        (UNCASTLED, (False, False), (False, True)),
        (UNCASTLED_THEIRS, (False, False), (True, False)),
    ],
    ids=["start", "developed", "corners", "storm", "uncastled", "uncastled_theirs"],
)
def test_a_castled_zone_is_the_wing_of_the_board_the_king_stands_on(
    fen: str, queenside: tuple[bool, bool], kingside: tuple[bool, bool], facts_of: FactsOf
) -> None:
    king = facts_of(fen).king
    assert king.castled_queenside == queenside
    assert king.castled_kingside == kingside


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "def", "def"),
        (CORNERS, "fgh", "abc"),
        (STORM, "fgh", "abc"),
        (OPEN_FILES, "fgh", "bcd"),
        (BARE_KINGS, "cde", "fgh"),
        (ENDGAME, "def", "cde"),
    ],
    ids=["start", "corners", "storm", "open_files", "bare_kings", "endgame"],
)
def test_the_king_files_are_the_kings_own_clamped_to_b_to_g_and_its_neighbours(
    fen: str, us: str, them: str, facts_of: FactsOf
) -> None:
    king_files = facts_of(fen).king.shield_files
    assert king_files[esca.US] == us
    assert king_files[esca.THEM] == them


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, [1, 1, 1], [1, 1, 1]),
        (STORM, [1, None, 3], [1, 2, 3]),
        (BOXED, [1, 2, 1], [1, 1, 1]),
        (CORNERS, [None, 1, None], [None, 1, None]),
        (UNCASTLED, [None, None, 1], [1, 1, 1]),
        (ENDGAME, [1, None, None], [None, None, None]),
        (BARE_KINGS, [None, None, None], [None, None, None]),
    ],
    ids=["start", "storm", "boxed", "corners", "uncastled", "endgame", "bare_kings"],
)
def test_a_pawn_shield_is_how_far_ahead_the_nearest_friendly_pawn_of_a_king_file_is(
    fen: str, us: list[int | None], them: list[int | None], facts_of: FactsOf
) -> None:
    shield = facts_of(fen).king.shield
    assert shield[esca.US] == us
    assert shield[esca.THEM] == them


@pytest.mark.parametrize(
    ("fen", "us_open", "us_semi_open", "them_open", "them_semi_open"),
    [
        (START, [False] * 3, [False] * 3, [False] * 3, [False] * 3),
        (
            CORNERS,
            [True, False, True],
            [False, True, False],
            [True, False, True],
            [False, True, False],
        ),
        (
            OPEN_FILES,
            [True, False, False],
            [False, True, False],
            [True, False, False],
            [False, False, True],
        ),
        (UNCASTLED, [True, True, False], [False] * 3, [False] * 3, [False] * 3),
        (HOLES, [False, True, True], [True, False, False], [True] * 3, [False] * 3),
        (
            ENDGAME,
            [False, False, True],
            [True, False, False],
            [True, False, False],
            [False, False, True],
        ),
        (SIEGE, [False] * 3, [True] * 3, [False] * 3, [True] * 3),
    ],
    ids=["start", "corners", "open_files", "uncastled", "holes", "endgame", "siege"],
)
def test_a_king_file_is_open_when_bare_and_semi_open_when_only_the_enemy_has_left_it(
    fen: str,
    us_open: list[bool],
    us_semi_open: list[bool],
    them_open: list[bool],
    them_semi_open: list[bool],
    facts_of: FactsOf,
) -> None:
    king = facts_of(fen).king
    assert king.file_open[esca.US] == us_open
    assert king.file_open[esca.THEM] == them_open
    assert king.file_semi_open_for_enemy[esca.US] == us_semi_open
    assert king.file_semi_open_for_enemy[esca.THEM] == them_semi_open


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, [6, 6, 6], [6, 6, 6]),
        (STORM, [2, 3, 4], [5, 4, None]),
        (OPEN_FILES, [None, None, 6], [None, 4, None]),
        (ENDGAME, [None, 2, None], [None, 1, None]),
        (BOXED, [6, 6, 6], [6, 5, 6]),
        (UNCASTLED, [None, None, 6], [6, 6, 6]),
        (CORNERS, [None, None, None], [None, None, None]),
    ],
    ids=["start", "storm", "open_files", "endgame", "boxed", "uncastled", "corners"],
)
def test_a_pawn_storm_is_how_far_ahead_the_nearest_enemy_pawn_of_a_king_file_is(
    fen: str, us: list[int | None], them: list[int | None], facts_of: FactsOf
) -> None:
    storm = facts_of(fen).king.storm
    assert storm[esca.US] == us
    assert storm[esca.THEM] == them


@pytest.mark.parametrize(
    ("fen", "attackers"),
    [
        (START, (0, 0)),
        (SIEGE, (4, 3)),
        (HOLES, (1, 1)),
        (DEVELOPED, (0, 1)),
        (BOXED, (0, 0)),
        (ENDGAME, (0, 0)),
    ],
    ids=["start", "siege", "holes", "developed", "boxed", "endgame"],
)
def test_a_ring_attacker_is_an_enemy_piece_bearing_on_a_square_next_to_the_king(
    fen: str, attackers: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).king.ring_attackers == attackers


@pytest.mark.parametrize(
    ("fen", "weight"),
    [
        (START, (0, 0)),
        (SIEGE, (8, 7)),
        (HOLES, (4, 2)),
        (DEVELOPED, (0, 1)),
        (UNCASTLED, (0, 0)),
        (ENDGAME, (0, 0)),
    ],
    ids=["start", "siege", "holes", "developed", "uncastled", "endgame"],
)
def test_ring_attack_weight_counts_a_queen_four_a_rook_two_and_a_minor_one(
    fen: str, weight: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).king.ring_attack_weight == weight


@pytest.mark.parametrize(
    ("fen", "defended"),
    [
        (START, (2, 2)),
        (DEVELOPED, (3, 3)),
        (UNCASTLED, (2, 2)),
        (SIEGE, (2, 1)),
        (HOLES, (1, 0)),
        (ENDGAME, (0, 1)),
        (BARE_KINGS, (0, 0)),
    ],
    ids=["start", "developed", "uncastled", "siege", "holes", "endgame", "bare_kings"],
)
def test_a_ring_square_is_defended_by_the_kings_own_side_but_never_by_the_king(
    fen: str, defended: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).king.ring_defended == defended


@pytest.mark.parametrize(
    ("fen", "holes"),
    [
        (START, (0, 0)),
        (SIEGE, (1, 2)),
        (HOLES, (3, 3)),
        (ENDGAME, (3, 3)),
        (STORM, (1, 0)),
        (DEVELOPED, (0, 0)),
        (BOXED, (0, 0)),
    ],
    ids=["start", "siege", "holes", "endgame", "storm", "developed", "boxed"],
)
def test_a_ring_hole_is_a_ring_square_the_enemy_attacks_and_nothing_of_ours_covers(
    fen: str, holes: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).king.ring_holes == holes


@pytest.mark.parametrize(
    ("fen", "escapes"),
    [
        (START, (0, 0)),
        (SIEGE, (1, 2)),
        (CORNERS, (2, 2)),
        (BOXED, (3, 2)),
        (UNCASTLED, (4, 1)),
        (ENDGAME, (5, 4)),
        (BARE_KINGS, (8, 5)),
    ],
    ids=["start", "siege", "corners", "boxed", "uncastled", "endgame", "bare_kings"],
)
def test_an_escape_square_is_next_to_the_king_free_of_our_own_and_unattacked(
    fen: str, escapes: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).king.escape_squares == escapes


@pytest.mark.parametrize(
    ("fen", "risk"),
    [
        (START, (True, True)),
        (SIEGE, (True, True)),
        (UNCASTLED, (False, True)),
        (UNCASTLED_THEIRS, (True, False)),
        (BOXED, (False, True)),
        (ENDGAME, (False, False)),
    ],
    ids=["start", "siege", "uncastled", "uncastled_theirs", "boxed", "endgame"],
)
def test_back_rank_risk_is_a_first_rank_king_with_its_own_units_on_every_square_ahead(
    fen: str, risk: tuple[bool, bool], facts_of: FactsOf
) -> None:
    assert facts_of(fen).king.back_rank_risk == risk


@pytest.mark.parametrize(
    ("fen", "distance"),
    [(START, 7), (SIEGE, 7), (CORNERS, 7), (HOLES, 5), (BARE_KINGS, 4), (ENDGAME, 2)],
    ids=["start", "siege", "corners", "holes", "bare_kings", "endgame"],
)
def test_the_kings_stand_a_chebyshev_distance_apart(fen: str, distance: int, facts_of: FactsOf) -> None:
    assert facts_of(fen).king.distance == distance


@pytest.mark.parametrize(
    ("fen", "tropism"),
    [
        (START, (7.0, 7.0)),
        (UNCASTLED, (7.0, 7.0)),
        (SIEGE, (4.0, 4.75)),
        (HOLES, (3.0, 5.0)),
        (CORNERS, (0.0, 0.0)),
        (BARE_KINGS, (0.0, 0.0)),
    ],
    ids=["start", "uncastled", "siege", "holes", "corners", "bare_kings"],
)
def test_tropism_is_the_mean_distance_of_the_enemy_pieces_to_the_king(
    fen: str, tropism: tuple[float, float], facts_of: FactsOf
) -> None:
    assert facts_of(fen).king.tropism == tropism


@pytest.mark.parametrize(
    ("fen", "mobility"),
    [
        (START, (5, 5)),
        (SIEGE, (5, 10)),
        (STORM, (12, 16)),
        (HOLES, (16, 22)),
        (ENDGAME, (19, 21)),
        (BARE_KINGS, (27, 21)),
    ],
    ids=["start", "siege", "storm", "holes", "endgame", "bare_kings"],
)
def test_virtual_mobility_is_what_a_queen_on_the_kings_square_would_attack(
    fen: str, mobility: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).king.virtual_mobility == mobility


def test_the_home_square_bit_is_left_out_of_a_chess960_vector(facts_of: FactsOf) -> None:
    """`features.md` §4 keeps `king_on_home_square` to classic chess: a Chess960
    array can start a king on e1 that has never moved, so the vector drops the
    bit the facts still read off the geometry."""
    facts = facts_of(NINE_SIXTY_HOME, esca.CHESS960)
    assert facts.king.on_home_square == (True, True)
    assert ("king", "king_on_home_square") not in esca.features_for(esca.CHESS960)


def test_the_castled_zone_bits_are_left_out_of_a_chess960_vector(facts_of: FactsOf) -> None:
    """The same for `king_castled_zone`: a Chess960 array can start both kings in
    the kingside zone with no castling having happened."""
    facts = facts_of(NINE_SIXTY_WING, esca.CHESS960)
    assert facts.king.castled_kingside == (True, True)
    assert facts.king.castled_queenside == (False, False)
    assert ("king", "king_castled_zone") not in esca.features_for(esca.CHESS960)
