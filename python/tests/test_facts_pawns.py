"""The `pawns` group, fact by fact.

Every expectation is worked out from the definitions in `docs/features.md` §1
for the named position above it. The cases mirror `tests/facts_pawns.rs`.
"""

from __future__ import annotations

from collections.abc import Callable

import esca
import pytest

#: The untouched array: one island a side and nothing else true of it.
START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

#: Black to move: doubled f-pawns against doubled c-pawns, over two open files.
WINGS = "4k3/p5pp/5p2/5p2/2P5/2P5/PP5P/4K3 b - - 0 1"

#: Three pawns on one file against two on another, both files free of enemies.
TRIPLED = "4k3/1p6/1p6/8/3P4/3P4/3P4/4K3 w - - 0 1"

#: e3 and e6 each stand still: their stop square is covered by an enemy pawn.
BACKWARD = "4k3/8/4p3/3p1p2/3P1P2/4P3/8/4K3 w - - 0 1"

#: Scattered pawns: two backward ones a side, and a three-island black wing.
SPLIT = "4k3/1p6/8/P6p/5p2/7P/6P1/4K3 w - - 0 1"

#: c4 and f5 each head a majority on a file the enemy has left.
CANDIDATES = "4k3/1p6/8/5pp1/1PP5/8/6P1/4K3 w - - 0 1"

#: A locked centre: four rams, with pawns in contact on both sides of it.
LOCKED = "4k3/8/4p3/1pppPp2/2PP1P2/8/8/4K3 w - - 0 1"

#: Doubled c-pawns against doubled f-pawns; d4 and e6 each head a majority.
MAJORITIES = "4k3/pp3p1p/4pp2/8/3P4/2P5/PPP2PPP/4K3 w - - 0 1"

#: A wedge of three rams, every pawn of it in contact with an enemy pawn.
WEDGE = "4k3/8/2pp4/3Ppp2/4PP2/8/8/4K3 w - - 0 1"

#: One pawn against two that both attack it, and it both of them.
CONTACT = "4k3/8/8/3p1p2/4P3/8/8/4K3 w - - 0 1"

#: Two connected passers a side, the rear pawn of each pair defending the front.
PASSERS = "4k3/5p2/6p1/2P5/1P6/8/8/4K3 w - - 0 1"

#: Two chains of three connected passers; d6 is out of the black king's square.
PHALANX = "k7/8/3P4/2P3p1/1P3p2/4p3/8/7K w - - 0 1"

#: a7 queens: the black king is outside the square and nothing else can help.
RUNAWAY = "8/P6k/8/8/8/8/6K1/8 w - - 0 1"

#: The same runaway one tempo later: it is theirs, and the tempo saves nothing.
RUNAWAY_THEIRS = "8/P6k/8/8/8/8/6K1/8 b - - 0 1"

#: The same locked centre with Black to move: every side-paired value swaps.
LOCKED_THEIRS = "4k3/8/4p3/1pppPp2/2PP1P2/8/8/4K3 b - - 0 1"

#: A knight and a bishop each hold a square the other side's pawns have left.
HOLES = "4k3/pp3ppp/2pB4/8/2PP4/3n4/PP3PPP/6K1 w - - 0 1"

#: Chains of two against a chain of three, both bases under attack.
CHAINS = "4k3/b4p2/4p3/3pP1N1/3P1P2/8/8/4K3 w - - 0 1"

#: Passers blockaded by minor pieces that also stand on holes.
BLOCKADE = "4k3/8/1n6/1P1b1p2/3P1N2/8/8/4K3 w - - 0 1"

#: Both kings castled short, with files left open in front of each.
CASTLED = "6k1/pp3pp1/8/8/4P3/8/PPP4P/6K1 w - - 0 1"

#: Three backward pawns, each on a file its opponent has left.
WEAK = "4k3/8/8/8/1p1p4/8/2P3P1/4K3 w - - 0 1"

#: Two passers on one rank: b5 leads, being the nearer to file a.
TWIN_PASSERS = "6k1/3p4/3K4/1P4P1/8/8/8/8 w - - 0 1"

#: A Chess960 middlegame; no pawn fact reads the back rank, so nothing moves.
NINE_SIXTY = "nnqrkr1b/1p1pp3/3p4/p4ppp/PP5P/1b2PP2/2PPK1P1/N1QR1RBB w fd - 0 10"

#: The helpers `conftest.py` hands over.
FactsOf = Callable[..., esca.Facts]
Squares = Callable[[str], set[str]]


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "a2 b2 c2 d2 e2 f2 g2 h2", "a7 b7 c7 d7 e7 f7 g7 h7"),
        (WINGS, "a7 f5 f6 g7 h7", "a2 b2 c3 c4 h2"),
        (TRIPLED, "d2 d3 d4", "b6 b7"),
        (LOCKED, "c4 d4 e5 f4", "b5 c5 d5 e6 f5"),
        (MAJORITIES, "a2 b2 c2 c3 d4 f2 g2 h2", "a7 b7 e6 f6 f7 h7"),
        (RUNAWAY, "a7", ""),
        (RUNAWAY_THEIRS, "", "a7"),
    ],
    ids=["start", "wings", "tripled", "locked", "majorities", "runaway", "runaway_theirs"],
)
def test_the_pawns_of_each_side_are_listed_us_first(
    fen: str, us: str, them: str, facts_of: FactsOf, squares: Squares
) -> None:
    pawns = facts_of(fen).pawns
    assert set(pawns.pawns[esca.US]) == squares(us)
    assert set(pawns.pawns[esca.THEM]) == squares(them)


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "", ""),
        (WINGS, "f5 f6", "c3 c4"),
        (TRIPLED, "d2 d3 d4", "b6 b7"),
        (LOCKED, "", ""),
        (PASSERS, "b4 c5", "f7 g6"),
        (PHALANX, "b4 c5 d6", "e3 f4 g5"),
        (RUNAWAY, "a7", ""),
        (RUNAWAY_THEIRS, "", "a7"),
    ],
    ids=["start", "wings", "tripled", "locked", "passers", "phalanx", "runaway", "runaway_theirs"],
)
def test_a_passer_has_no_enemy_pawn_ahead_on_its_own_or_a_neighbouring_file(
    fen: str, us: str, them: str, facts_of: FactsOf, squares: Squares
) -> None:
    pawns = facts_of(fen).pawns
    assert set(pawns.passed[esca.US]) == squares(us)
    assert set(pawns.passed[esca.THEM]) == squares(them)


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "", ""),
        (WINGS, "g7", "b2"),
        (CANDIDATES, "c4", "f5"),
        (MAJORITIES, "c2 c3 d4", "e6"),
        (WEDGE, "", "c6"),
        (LOCKED, "", "b5"),
        (PASSERS, "", ""),
        (SPLIT, "", ""),
    ],
    ids=["start", "wings", "candidates", "majorities", "wedge", "locked", "passers", "split"],
)
def test_a_candidate_has_a_free_file_ahead_and_support_enough_to_use_it(
    fen: str, us: str, them: str, facts_of: FactsOf, squares: Squares
) -> None:
    pawns = facts_of(fen).pawns
    assert set(pawns.candidates[esca.US]) == squares(us)
    assert set(pawns.candidates[esca.THEM]) == squares(them)


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "", ""),
        (WINGS, "f5 f6", "c3 c4"),
        (TRIPLED, "d2 d3 d4", "b6 b7"),
        (LOCKED, "", ""),
        (SPLIT, "", ""),
        (MAJORITIES, "c2 c3", "f6 f7"),
    ],
    ids=["start", "wings", "tripled", "locked", "split", "majorities"],
)
def test_every_pawn_of_a_shared_file_is_doubled(
    fen: str, us: str, them: str, facts_of: FactsOf, squares: Squares
) -> None:
    pawns = facts_of(fen).pawns
    assert set(pawns.doubled[esca.US]) == squares(us)
    assert set(pawns.doubled[esca.THEM]) == squares(them)


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "", ""),
        (WINGS, "a7", "h2"),
        (TRIPLED, "d2 d3 d4", "b6 b7"),
        (SPLIT, "a5", "b7 f4 h5"),
        (CONTACT, "e4", "d5 f5"),
        (RUNAWAY, "a7", ""),
    ],
    ids=["start", "wings", "tripled", "split", "contact", "runaway"],
)
def test_an_isolated_pawn_has_no_friendly_pawn_on_either_neighbouring_file(
    fen: str, us: str, them: str, facts_of: FactsOf, squares: Squares
) -> None:
    pawns = facts_of(fen).pawns
    assert set(pawns.isolated[esca.US]) == squares(us)
    assert set(pawns.isolated[esca.THEM]) == squares(them)


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "", ""),
        (BACKWARD, "e3", "e6"),
        (SPLIT, "a5 g2", "b7 f4"),
        (LOCKED, "f4", "e6"),
        (PASSERS, "", ""),
    ],
    ids=["start", "backward", "split", "locked", "passers"],
)
def test_a_backward_pawn_is_unsupported_and_its_stop_square_is_held(
    fen: str, us: str, them: str, facts_of: FactsOf, squares: Squares
) -> None:
    pawns = facts_of(fen).pawns
    assert set(pawns.backward[esca.US]) == squares(us)
    assert set(pawns.backward[esca.THEM]) == squares(them)


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "", ""),
        (WINGS, "f6", "c3"),
        (BACKWARD, "d4 f4", "d5 f5"),
        (SPLIT, "h3", ""),
        (PHALANX, "c5 d6", "e3 f4"),
        (MAJORITIES, "c3 d4", "e6"),
        (WEDGE, "d5", "e5"),
    ],
    ids=["start", "wings", "backward", "split", "phalanx", "majorities", "wedge"],
)
def test_a_defended_pawn_stands_where_its_own_pawns_attack(
    fen: str, us: str, them: str, facts_of: FactsOf, squares: Squares
) -> None:
    pawns = facts_of(fen).pawns
    assert set(pawns.defended[esca.US]) == squares(us)
    assert set(pawns.defended[esca.THEM]) == squares(them)


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, [1] * 8, [1] * 8),
        (WINGS, [1, 0, 0, 0, 0, 2, 1, 1], [1, 1, 2, 0, 0, 0, 0, 1]),
        (TRIPLED, [0, 0, 0, 3, 0, 0, 0, 0], [0, 2, 0, 0, 0, 0, 0, 0]),
        (LOCKED, [0, 0, 1, 1, 1, 1, 0, 0], [0, 1, 1, 1, 1, 1, 0, 0]),
        (MAJORITIES, [1, 1, 2, 1, 0, 1, 1, 1], [1, 1, 0, 0, 1, 2, 0, 1]),
        (RUNAWAY, [1, 0, 0, 0, 0, 0, 0, 0], [0] * 8),
    ],
    ids=["start", "wings", "tripled", "locked", "majorities", "runaway"],
)
def test_pawns_are_counted_by_file_from_a(fen: str, us: list[int], them: list[int], facts_of: FactsOf) -> None:
    pawns = facts_of(fen).pawns
    assert pawns.count_by_file[esca.US] == us
    assert pawns.count_by_file[esca.THEM] == them


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, [0, 8, 0, 0, 0, 0, 0, 0], [0, 8, 0, 0, 0, 0, 0, 0]),
        (SPLIT, [0, 1, 1, 0, 1, 0, 0, 0], [0, 1, 0, 1, 1, 0, 0, 0]),
        (LOCKED, [0, 0, 0, 3, 1, 0, 0, 0], [0, 0, 1, 4, 0, 0, 0, 0]),
        (PASSERS, [0, 0, 0, 1, 1, 0, 0, 0], [0, 1, 1, 0, 0, 0, 0, 0]),
        (MAJORITIES, [0, 6, 1, 1, 0, 0, 0, 0], [0, 4, 2, 0, 0, 0, 0, 0]),
        (RUNAWAY, [0, 0, 0, 0, 0, 0, 1, 0], [0] * 8),
    ],
    ids=["start", "split", "locked", "passers", "majorities", "runaway"],
)
def test_pawns_are_counted_by_the_rank_their_owner_reads(
    fen: str, us: list[int], them: list[int], facts_of: FactsOf
) -> None:
    pawns = facts_of(fen).pawns
    assert pawns.count_by_rank[esca.US] == us
    assert pawns.count_by_rank[esca.THEM] == them


@pytest.mark.parametrize(
    ("fen", "open_files"),
    [
        (START, ""),
        (WINGS, "de"),
        (TRIPLED, "acefgh"),
        (LOCKED, "agh"),
        (MAJORITIES, ""),
        (WEDGE, "abgh"),
        (RUNAWAY, "bcdefgh"),
    ],
    ids=["start", "wings", "tripled", "locked", "majorities", "wedge", "runaway"],
)
def test_an_open_file_carries_no_pawn_of_either_colour(fen: str, open_files: str, facts_of: FactsOf) -> None:
    assert facts_of(fen).pawns.open_files == open_files


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "", ""),
        (WINGS, "bc", "fg"),
        (SPLIT, "bf", "ag"),
        (CONTACT, "df", "e"),
        (MAJORITIES, "e", "cdg"),
        (WEDGE, "c", ""),
        (RUNAWAY, "", "a"),
        (RUNAWAY_THEIRS, "a", ""),
    ],
    ids=["start", "wings", "split", "contact", "majorities", "wedge", "runaway", "runaway_theirs"],
)
def test_a_file_is_semi_open_for_the_side_that_has_left_it(fen: str, us: str, them: str, facts_of: FactsOf) -> None:
    pawns = facts_of(fen).pawns
    assert pawns.semi_open_files[esca.US] == us
    assert pawns.semi_open_files[esca.THEM] == them


@pytest.mark.parametrize(
    ("fen", "islands"),
    [
        (START, (1, 1)),
        (WINGS, (2, 2)),
        (SPLIT, (2, 3)),
        (CONTACT, (1, 2)),
        (MAJORITIES, (2, 3)),
        (RUNAWAY, (1, 0)),
        (RUNAWAY_THEIRS, (0, 1)),
    ],
    ids=["start", "wings", "split", "contact", "majorities", "runaway", "runaway_theirs"],
)
def test_an_island_is_a_maximal_run_of_files_carrying_a_pawn(
    fen: str, islands: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pawns.islands == islands


@pytest.mark.parametrize(
    ("fen", "levers"),
    [
        (START, (0, 0)),
        (SPLIT, (0, 0)),
        (CONTACT, (1, 2)),
        (LOCKED, (2, 3)),
        (WEDGE, (3, 3)),
        (WINGS, (0, 0)),
    ],
    ids=["start", "split", "contact", "locked", "wedge", "wings"],
)
def test_a_lever_is_a_pawn_whose_attacks_reach_an_enemy_pawn(
    fen: str, levers: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pawns.levers == levers


@pytest.mark.parametrize(
    ("fen", "rams"),
    [(START, 0), (CONTACT, 0), (MAJORITIES, 0), (BACKWARD, 2), (WEDGE, 3), (LOCKED, 4)],
    ids=["start", "contact", "majorities", "backward", "wedge", "locked"],
)
def test_a_ram_is_a_pawn_pair_blocking_each_other_head_on(fen: str, rams: int, facts_of: FactsOf) -> None:
    assert facts_of(fen).pawns.rams == rams


@pytest.mark.parametrize(
    ("fen", "lead"),
    [
        (START, (None, None)),
        (WINGS, (4, 4)),
        (TRIPLED, (4, 3)),
        (PASSERS, (5, 3)),
        (PHALANX, (6, 6)),
        (RUNAWAY, (7, None)),
        (RUNAWAY_THEIRS, (None, 7)),
    ],
    ids=["start", "wings", "tripled", "passers", "phalanx", "runaway", "runaway_theirs"],
)
def test_the_lead_rank_is_how_far_the_furthest_passer_has_come(
    fen: str, lead: tuple[int | None, int | None], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pawns.passer_lead_rank == lead


@pytest.mark.parametrize(
    ("fen", "protected"),
    [
        (START, (0, 0)),
        (TRIPLED, (0, 0)),
        (WINGS, (1, 1)),
        (PASSERS, (1, 1)),
        (PHALANX, (2, 2)),
    ],
    ids=["start", "tripled", "wings", "passers", "phalanx"],
)
def test_a_protected_passer_stands_on_a_square_a_friendly_pawn_attacks(
    fen: str, protected: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pawns.passer_protected == protected


@pytest.mark.parametrize(
    ("fen", "connected"),
    [
        (START, (False, False)),
        (TRIPLED, (False, False)),
        (WINGS, (False, False)),
        (PASSERS, (True, True)),
        (PHALANX, (True, True)),
    ],
    ids=["start", "tripled", "wings", "passers", "phalanx"],
)
def test_passers_are_connected_when_two_of_them_stand_on_neighbouring_files(
    fen: str, connected: tuple[bool, bool], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pawns.passers_connected == connected


@pytest.mark.parametrize(
    ("fen", "unstoppable"),
    [
        (START, (False, False)),
        (TRIPLED, (False, False)),
        (PASSERS, (False, False)),
        (PHALANX, (True, False)),
        (RUNAWAY, (True, False)),
        (RUNAWAY_THEIRS, (False, True)),
    ],
    ids=["start", "tripled", "passers", "phalanx", "runaway", "runaway_theirs"],
)
def test_an_unstoppable_passer_beats_the_defending_king_to_its_promotion_square(
    fen: str, unstoppable: tuple[bool, bool], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pawns.passer_unstoppable == unstoppable


@pytest.mark.parametrize(
    ("fen", "length"),
    [
        (START, (1, 1)),
        (RUNAWAY, (1, 0)),
        (HOLES, (1, 2)),
        (CHAINS, (2, 3)),
        (MAJORITIES, (3, 2)),
        (PHALANX, (3, 3)),
    ],
    ids=["start", "runaway", "holes", "chains", "majorities", "phalanx"],
)
def test_the_longest_chain_is_the_longest_run_of_pawns_each_defending_the_next(
    fen: str, length: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pawns.chain_max_length == length


@pytest.mark.parametrize(
    ("fen", "attacked"),
    [
        (START, (False, False)),
        (PHALANX, (False, False)),
        (LOCKED, (True, False)),
        (LOCKED_THEIRS, (False, True)),
        (WEDGE, (True, False)),
        (CHAINS, (True, True)),
    ],
    ids=["start", "phalanx", "locked", "locked_theirs", "wedge", "chains"],
)
def test_a_chain_base_is_attacked_when_an_enemy_unit_bears_on_its_rearmost_pawn(
    fen: str, attacked: tuple[bool, bool], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pawns.chain_base_attacked == attacked


@pytest.mark.parametrize(
    ("fen", "majority"),
    [
        (START, ((False, False), (False, False))),
        (TRIPLED, ((True, False), (False, False))),
        (CONTACT, ((False, False), (True, False))),
        (MAJORITIES, ((True, False), (False, True))),
        (WINGS, ((False, True), (True, False))),
        (WEAK, ((False, True), (True, False))),
    ],
    ids=["start", "tripled", "contact", "majorities", "wings", "weak"],
)
def test_a_majority_is_more_own_pawns_than_enemy_pawns_on_a_wing(
    fen: str, majority: tuple[tuple[bool, bool], tuple[bool, bool]], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pawns.majority_by_wing == majority


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "", ""),
        (MAJORITIES, "", "d6 f6 h3 h4 h5 h6"),
        (HOLES, "d3 d4", "d6"),
        (CASTLED, "e3 e4 e5 e6 f3 f4 h3 h4 h5 h6", "d3 d4 d5 d6"),
    ],
    ids=["start", "majorities", "holes", "castled"],
)
def test_a_hole_is_a_square_no_pawn_of_the_side_can_ever_attack(
    fen: str, us: str, them: str, facts_of: FactsOf, squares: Squares
) -> None:
    pawns = facts_of(fen).pawns
    assert set(pawns.holes[esca.US]) == squares(us)
    assert set(pawns.holes[esca.THEM]) == squares(them)


@pytest.mark.parametrize(
    ("fen", "holes"),
    [
        (START, (0, 0)),
        (MAJORITIES, (0, 6)),
        (CASTLED, (10, 4)),
        (WINGS, (13, 13)),
        (BLOCKADE, (27, 28)),
    ],
    ids=["start", "majorities", "castled", "wings", "blockade"],
)
def test_holes_are_counted_over_the_four_ranks_the_definition_names(
    fen: str, holes: tuple[int, int], facts_of: FactsOf
) -> None:
    """The encoding counts what the sets hold."""
    pawns = facts_of(fen).pawns
    assert (len(pawns.holes[esca.US]), len(pawns.holes[esca.THEM])) == holes


@pytest.mark.parametrize(
    ("fen", "occupied"),
    [
        (START, (0, 0)),
        (CASTLED, (0, 0)),
        (MAJORITIES, (0, 0)),
        (HOLES, (1, 1)),
        (BLOCKADE, (2, 1)),
    ],
    ids=["start", "castled", "majorities", "holes", "blockade"],
)
def test_a_hole_is_occupied_when_an_enemy_knight_or_bishop_stands_on_it(
    fen: str, occupied: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pawns.holes_occupied == occupied


@pytest.mark.parametrize(
    ("fen", "fixed"),
    [
        (START, (0, 0)),
        (MAJORITIES, (1, 1)),
        (BLOCKADE, (2, 1)),
        (TRIPLED, (2, 1)),
        (WEDGE, (3, 3)),
        (LOCKED, (4, 4)),
    ],
    ids=["start", "majorities", "blockade", "tripled", "wedge", "locked"],
)
def test_a_fixed_pawn_has_a_unit_of_either_colour_on_its_stop_square(
    fen: str, fixed: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pawns.fixed_pawns == fixed


@pytest.mark.parametrize(
    ("fen", "blocked"),
    [
        (START, (0, 0)),
        (PASSERS, (0, 0)),
        (PHALANX, (0, 0)),
        (TWIN_PASSERS, (0, 1)),
        (BLOCKADE, (2, 1)),
    ],
    ids=["start", "passers", "phalanx", "twin_passers", "blockade"],
)
def test_a_blocked_passer_has_an_enemy_unit_on_its_stop_square(
    fen: str, blocked: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pawns.blocked_passers == blocked


@pytest.mark.parametrize(
    ("fen", "distance"),
    [
        (START, (None, None)),
        (RUNAWAY, (1, None)),
        (PHALANX, (2, 2)),
        (PASSERS, (3, 5)),
        (TWIN_PASSERS, (3, 6)),
        (TRIPLED, (4, 5)),
    ],
    ids=["start", "runaway", "phalanx", "passers", "twin_passers", "tripled"],
)
def test_the_passer_distance_is_what_the_lead_passer_still_has_to_push(
    fen: str, distance: tuple[int | None, int | None], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pawns.passer_distance == distance


@pytest.mark.parametrize(
    ("fen", "distance"),
    [
        (START, ((None, None), (None, None))),
        (RUNAWAY, ((6, 7), (None, None))),
        (PASSERS, ((7, 2), (7, 2))),
        (PHALANX, ((7, 3), (7, 3))),
        (BLOCKADE, ((7, 3), (7, 1))),
        (TWIN_PASSERS, ((2, 5), (7, 5))),
    ],
    ids=["start", "runaway", "passers", "phalanx", "blockade", "twin_passers"],
)
def test_both_kings_are_measured_to_the_lead_passers_promotion_square(
    fen: str,
    distance: tuple[tuple[int | None, int | None], tuple[int | None, int | None]],
    facts_of: FactsOf,
) -> None:
    assert facts_of(fen).pawns.passer_king_distance == distance


@pytest.mark.parametrize(
    ("fen", "caught"),
    [
        (START, (False, False)),
        (RUNAWAY, (False, False)),
        (RUNAWAY_THEIRS, (False, False)),
        (PHALANX, (False, True)),
        (TWIN_PASSERS, (False, True)),
        (PASSERS, (True, True)),
        (BLOCKADE, (True, True)),
    ],
    ids=["start", "runaway", "runaway_theirs", "phalanx", "twin_passers", "passers", "blockade"],
)
def test_a_defending_king_in_the_square_catches_the_lead_passer(
    fen: str, caught: tuple[bool, bool], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pawns.passer_in_square == caught


@pytest.mark.parametrize(
    ("fen", "free"),
    [
        (START, (0, 0)),
        (BLOCKADE, (0, 0)),
        (RUNAWAY, (1, 0)),
        (TWIN_PASSERS, (1, 0)),
        (TRIPLED, (1, 1)),
        (PASSERS, (2, 2)),
        (PHALANX, (3, 3)),
    ],
    ids=["start", "blockade", "runaway", "twin_passers", "tripled", "passers", "phalanx"],
)
def test_a_free_path_is_a_passer_with_nothing_at_all_ahead_of_it(
    fen: str, free: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pawns.passer_free_path == free


@pytest.mark.parametrize(
    ("fen", "aimed"),
    [
        (START, (0, 0)),
        (WINGS, (0, 1)),
        (SPLIT, (1, 0)),
        (MAJORITIES, (1, 1)),
        (CONTACT, (2, 1)),
        (CASTLED, (2, 1)),
    ],
    ids=["start", "wings", "split", "majorities", "contact", "castled"],
)
def test_a_file_aimed_at_the_enemy_king_is_one_semi_open_for_us_among_its_three(
    fen: str, aimed: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pawns.half_open_at_enemy_king == aimed


@pytest.mark.parametrize(
    ("fen", "weak"),
    [
        (START, (0, 0)),
        (BACKWARD, (0, 0)),
        (LOCKED, (0, 0)),
        (WEAK, (1, 2)),
        (SPLIT, (2, 2)),
    ],
    ids=["start", "backward", "locked", "weak", "split"],
)
def test_a_backward_pawn_counts_again_on_a_file_the_enemy_has_left(
    fen: str, weak: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pawns.backward_on_semi_open == weak


def test_the_pawn_facts_of_a_chess960_position_are_the_classic_ones(facts_of: FactsOf, squares: Squares) -> None:
    """No `pawns` fact is one of the four `features.md` §4 defines for classic
    chess only, so a Chess960 position answers as the same placement would."""
    pawns = facts_of(NINE_SIXTY, esca.CHESS960).pawns
    assert set(pawns.pawns[esca.US]) == squares("a4 b4 c2 d2 e3 f3 g2 h4")
    assert set(pawns.pawns[esca.THEM]) == squares("a5 b7 d6 d7 e7 f5 g5 h5")
    assert set(pawns.doubled[esca.THEM]) == squares("d6 d7")
    assert set(pawns.defended[esca.US]) == squares("e3 f3")
    assert set(pawns.defended[esca.THEM]) == squares("d6")
    assert pawns.count_by_file[esca.THEM] == [1, 1, 0, 2, 1, 1, 1, 1]
    assert pawns.count_by_rank[esca.US] == [0, 3, 2, 3, 0, 0, 0, 0]
    assert pawns.islands == (1, 2)
    assert pawns.levers == (2, 2)
    assert pawns.rams == 2
    assert pawns.semi_open_files[esca.THEM] == "c"
    assert pawns.open_files == ""
    assert pawns.chain_max_length == (2, 2)
    assert set(pawns.holes[esca.US]) == squares("a3 a4 g3")
    assert set(pawns.holes[esca.THEM]) == squares("b5 b6 g5 g6 h5 h6")
    assert pawns.fixed_pawns == (2, 3)
    assert pawns.majority_by_wing == ((False, False), (False, False))

    classic = facts_of("nnqrkr1b/1p1pp3/3p4/p4ppp/PP5P/1b2PP2/2PPK1P1/N1QR1RBB w - - 0 10").pawns
    assert set(classic.passed[esca.US]) == set(pawns.passed[esca.US])
    assert classic.count_by_rank[esca.THEM] == pawns.count_by_rank[esca.THEM]
