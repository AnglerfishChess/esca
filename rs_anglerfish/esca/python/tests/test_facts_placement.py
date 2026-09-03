"""The `placement` group, plane by plane.

Every expectation is worked out from the definitions in `docs/features.md`
§2.1 for the named position above it. The cases mirror `tests/facts_placement.rs`.
"""

from __future__ import annotations

from collections.abc import Callable

import esca
import pytest

#: The untouched array, White to move.
START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

#: The same array with Black to move: every plane changes hands.
START_BLACK = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1"

#: A busy middlegame with a unit of every role a side.
KIWIPETE = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"

#: A bishop endgame, Black to move: most planes are empty.
ENDGAME = "8/5pk1/8/8/8/4B3/5PK1/8 b - - 0 1"

#: One unit of each role between the two sides, and no pawn at all.
ONE_EACH = "3qk3/1n6/2b5/8/8/5R2/6N1/4K2Q w - - 0 1"

#: The roles, in the order the planes are written.
PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING = range(6)

#: The helpers `conftest.py` hands over.
FactsOf = Callable[..., esca.Facts]
Squares = Callable[[str], set[str]]


def plane_at(side: int, role: int) -> int:
    """Where the plane of `role` for `side` starts in the 768-wide row."""
    return 64 * (6 * side + role)


def placement_row(fen: str) -> list[float]:
    """The encoded `placement` row of `fen`."""
    return list(esca.encode([fen], groups=["placement"])[0])


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "a2 b2 c2 d2 e2 f2 g2 h2", "a7 b7 c7 d7 e7 f7 g7 h7"),
        (START_BLACK, "a7 b7 c7 d7 e7 f7 g7 h7", "a2 b2 c2 d2 e2 f2 g2 h2"),
        (KIWIPETE, "a2 b2 c2 d5 e4 f2 g2 h2", "a7 b4 c7 d7 e6 f7 g6 h3"),
        (ENDGAME, "f7", "f2"),
        (ONE_EACH, "", ""),
    ],
    ids=["start", "start_black", "kiwipete", "endgame", "one_each"],
)
def test_the_pawn_planes_hold_the_pawns_of_their_own_side(
    fen: str, us: str, them: str, facts_of: FactsOf, squares: Squares
) -> None:
    placement = facts_of(fen).placement
    assert set(placement.by_role[esca.US][PAWN]) == squares(us)
    assert set(placement.by_role[esca.THEM][PAWN]) == squares(them)


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "b1 g1", "b8 g8"),
        (START_BLACK, "b8 g8", "b1 g1"),
        (KIWIPETE, "c3 e5", "b6 f6"),
        (ENDGAME, "", ""),
        (ONE_EACH, "g2", "b7"),
    ],
    ids=["start", "start_black", "kiwipete", "endgame", "one_each"],
)
def test_the_knight_planes_hold_the_knights_of_their_own_side(
    fen: str, us: str, them: str, facts_of: FactsOf, squares: Squares
) -> None:
    placement = facts_of(fen).placement
    assert set(placement.by_role[esca.US][KNIGHT]) == squares(us)
    assert set(placement.by_role[esca.THEM][KNIGHT]) == squares(them)


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "c1 f1", "c8 f8"),
        (START_BLACK, "c8 f8", "c1 f1"),
        (KIWIPETE, "d2 e2", "a6 g7"),
        (ENDGAME, "", "e3"),
        (ONE_EACH, "", "c6"),
    ],
    ids=["start", "start_black", "kiwipete", "endgame", "one_each"],
)
def test_the_bishop_planes_hold_the_bishops_of_their_own_side(
    fen: str, us: str, them: str, facts_of: FactsOf, squares: Squares
) -> None:
    placement = facts_of(fen).placement
    assert set(placement.by_role[esca.US][BISHOP]) == squares(us)
    assert set(placement.by_role[esca.THEM][BISHOP]) == squares(them)


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "a1 h1", "a8 h8"),
        (START_BLACK, "a8 h8", "a1 h1"),
        (KIWIPETE, "a1 h1", "a8 h8"),
        (ENDGAME, "", ""),
        (ONE_EACH, "f3", ""),
    ],
    ids=["start", "start_black", "kiwipete", "endgame", "one_each"],
)
def test_the_rook_planes_hold_the_rooks_of_their_own_side(
    fen: str, us: str, them: str, facts_of: FactsOf, squares: Squares
) -> None:
    placement = facts_of(fen).placement
    assert set(placement.by_role[esca.US][ROOK]) == squares(us)
    assert set(placement.by_role[esca.THEM][ROOK]) == squares(them)


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "d1", "d8"),
        (START_BLACK, "d8", "d1"),
        (KIWIPETE, "f3", "e7"),
        (ENDGAME, "", ""),
        (ONE_EACH, "h1", "d8"),
    ],
    ids=["start", "start_black", "kiwipete", "endgame", "one_each"],
)
def test_the_queen_planes_hold_the_queens_of_their_own_side(
    fen: str, us: str, them: str, facts_of: FactsOf, squares: Squares
) -> None:
    placement = facts_of(fen).placement
    assert set(placement.by_role[esca.US][QUEEN]) == squares(us)
    assert set(placement.by_role[esca.THEM][QUEEN]) == squares(them)


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (START, "e1", "e8"),
        (START_BLACK, "e8", "e1"),
        (KIWIPETE, "e1", "e8"),
        (ENDGAME, "g7", "g2"),
        (ONE_EACH, "e1", "e8"),
    ],
    ids=["start", "start_black", "kiwipete", "endgame", "one_each"],
)
def test_the_king_planes_hold_one_king_each(fen: str, us: str, them: str, facts_of: FactsOf, squares: Squares) -> None:
    placement = facts_of(fen).placement
    assert set(placement.by_role[esca.US][KING]) == squares(us)
    assert set(placement.by_role[esca.THEM][KING]) == squares(them)


@pytest.mark.parametrize(
    ("side", "role", "offset"),
    [
        (esca.US, PAWN, 0),
        (esca.US, KNIGHT, 64),
        (esca.US, BISHOP, 128),
        (esca.US, ROOK, 192),
        (esca.US, QUEEN, 256),
        (esca.US, KING, 320),
        (esca.THEM, PAWN, 384),
        (esca.THEM, KNIGHT, 448),
        (esca.THEM, BISHOP, 512),
        (esca.THEM, ROOK, 576),
        (esca.THEM, QUEEN, 640),
        (esca.THEM, KING, 704),
    ],
    ids=[
        "our_pawns",
        "our_knights",
        "our_bishops",
        "our_rooks",
        "our_queens",
        "our_king",
        "their_pawns",
        "their_knights",
        "their_bishops",
        "their_rooks",
        "their_queens",
        "their_king",
    ],
)
def test_each_plane_sits_where_the_schema_names_it(side: int, role: int, offset: int) -> None:
    """The plane order the row is written in: ours before theirs, and P, N, B,
    R, Q, K within a side."""
    assert plane_at(side, role) == offset


def test_the_movers_view_makes_the_two_starting_rows_the_same() -> None:
    """A plane is read in the mover's view, so the untouched array writes the
    same row whichever side is to move."""
    white = placement_row(START)
    black = placement_row(START_BLACK)
    assert white == black
    assert len(white) == 768

    # Our pawns stand on relative rank 2, which is plane index 8 to 15.
    start = plane_at(esca.US, PAWN)
    pawns = white[start : start + 64]
    assert [index for index, value in enumerate(pawns) if value == 1.0] == list(range(8, 16))


@pytest.mark.parametrize(
    "fen",
    [START, START_BLACK, KIWIPETE, ENDGAME, ONE_EACH],
    ids=["start", "start_black", "kiwipete", "endgame", "one_each"],
)
def test_the_planes_hold_every_unit_once(fen: str, facts_of: FactsOf) -> None:
    """Every unit of the position stands in exactly one plane, and nothing else
    is set."""
    placement = facts_of(fen).placement
    row = placement_row(fen)
    planes = [placement.by_role[side][role] for side in (esca.US, esca.THEM) for role in range(6)]
    units = sum(len(plane) for plane in planes)

    assert sum(1 for value in row if value == 1.0) == units
    assert all(value in (0.0, 1.0) for value in row)

    seen: set[str] = set()
    for plane in planes:
        assert not seen & set(plane), "a unit stands in two planes"
        seen |= set(plane)
    assert len(seen) == units
