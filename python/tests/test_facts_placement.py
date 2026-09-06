"""The `placement` group, role by role.

Every expectation is worked out from the definitions in `docs/features.md`
§2.1 for the named position above it. The cases mirror `tests/facts_placement.rs`.
"""

from __future__ import annotations

from collections.abc import Callable

import esca
import pytest

#: The untouched array, White to move.
START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

#: The same array with Black to move: the same units, read the same way.
START_BLACK = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1"

#: A busy middlegame with a unit of every role a side.
KIWIPETE = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"

#: A bishop endgame, Black to move: most of the sets are empty.
ENDGAME = "8/5pk1/8/8/8/4B3/5PK1/8 b - - 0 1"

#: One unit of each role between the two sides, and no pawn at all.
ONE_EACH = "3qk3/1n6/2b5/8/8/5R2/6N1/4K2Q w - - 0 1"


#: The helpers `conftest.py` hands over.
FactsOf = Callable[..., esca.Facts]
Squares = Callable[[str], set[str]]


@pytest.mark.parametrize(
    ("fen", "white", "black"),
    [
        (START, "a2 b2 c2 d2 e2 f2 g2 h2", "a7 b7 c7 d7 e7 f7 g7 h7"),
        (START_BLACK, "a2 b2 c2 d2 e2 f2 g2 h2", "a7 b7 c7 d7 e7 f7 g7 h7"),
        (KIWIPETE, "a2 b2 c2 d5 e4 f2 g2 h2", "a7 b4 c7 d7 e6 f7 g6 h3"),
        (ENDGAME, "f2", "f7"),
        (ONE_EACH, "", ""),
    ],
    ids=["start", "start_black", "kiwipete", "endgame", "one_each"],
)
def test_the_pawns_of_each_side_are_where_they_stand(
    fen: str, white: str, black: str, facts_of: FactsOf, squares: Squares
) -> None:
    placement = facts_of(fen).placement
    assert set(placement.pawns.white) == squares(white)
    assert set(placement.pawns.black) == squares(black)


@pytest.mark.parametrize(
    ("fen", "white", "black"),
    [
        (START, "b1 g1", "b8 g8"),
        (START_BLACK, "b1 g1", "b8 g8"),
        (KIWIPETE, "c3 e5", "b6 f6"),
        (ENDGAME, "", ""),
        (ONE_EACH, "g2", "b7"),
    ],
    ids=["start", "start_black", "kiwipete", "endgame", "one_each"],
)
def test_the_knights_of_each_side_are_where_they_stand(
    fen: str, white: str, black: str, facts_of: FactsOf, squares: Squares
) -> None:
    placement = facts_of(fen).placement
    assert set(placement.knights.white) == squares(white)
    assert set(placement.knights.black) == squares(black)


@pytest.mark.parametrize(
    ("fen", "white", "black"),
    [
        (START, "c1 f1", "c8 f8"),
        (START_BLACK, "c1 f1", "c8 f8"),
        (KIWIPETE, "d2 e2", "a6 g7"),
        (ENDGAME, "e3", ""),
        (ONE_EACH, "", "c6"),
    ],
    ids=["start", "start_black", "kiwipete", "endgame", "one_each"],
)
def test_the_bishops_of_each_side_are_where_they_stand(
    fen: str, white: str, black: str, facts_of: FactsOf, squares: Squares
) -> None:
    placement = facts_of(fen).placement
    assert set(placement.bishops.white) == squares(white)
    assert set(placement.bishops.black) == squares(black)


@pytest.mark.parametrize(
    ("fen", "white", "black"),
    [
        (START, "a1 h1", "a8 h8"),
        (START_BLACK, "a1 h1", "a8 h8"),
        (KIWIPETE, "a1 h1", "a8 h8"),
        (ENDGAME, "", ""),
        (ONE_EACH, "f3", ""),
    ],
    ids=["start", "start_black", "kiwipete", "endgame", "one_each"],
)
def test_the_rooks_of_each_side_are_where_they_stand(
    fen: str, white: str, black: str, facts_of: FactsOf, squares: Squares
) -> None:
    placement = facts_of(fen).placement
    assert set(placement.rooks.white) == squares(white)
    assert set(placement.rooks.black) == squares(black)


@pytest.mark.parametrize(
    ("fen", "white", "black"),
    [
        (START, "d1", "d8"),
        (START_BLACK, "d1", "d8"),
        (KIWIPETE, "f3", "e7"),
        (ENDGAME, "", ""),
        (ONE_EACH, "h1", "d8"),
    ],
    ids=["start", "start_black", "kiwipete", "endgame", "one_each"],
)
def test_the_queens_of_each_side_are_where_they_stand(
    fen: str, white: str, black: str, facts_of: FactsOf, squares: Squares
) -> None:
    placement = facts_of(fen).placement
    assert set(placement.queens.white) == squares(white)
    assert set(placement.queens.black) == squares(black)


@pytest.mark.parametrize(
    ("fen", "white", "black"),
    [
        (START, "e1", "e8"),
        (START_BLACK, "e1", "e8"),
        (KIWIPETE, "e1", "e8"),
        (ENDGAME, "g2", "g7"),
        (ONE_EACH, "e1", "e8"),
    ],
    ids=["start", "start_black", "kiwipete", "endgame", "one_each"],
)
def test_each_side_has_one_king(fen: str, white: str, black: str, facts_of: FactsOf, squares: Squares) -> None:
    placement = facts_of(fen).placement
    assert set(placement.kings.white) == squares(white)
    assert set(placement.kings.black) == squares(black)


@pytest.mark.parametrize(
    "fen",
    [START, START_BLACK, KIWIPETE, ENDGAME, ONE_EACH],
    ids=["start", "start_black", "kiwipete", "endgame", "one_each"],
)
def test_every_unit_stands_in_one_set(fen: str, facts_of: FactsOf) -> None:
    """Every unit of the position stands in exactly one of the sets, and the
    `units` set holds them all."""
    placement = facts_of(fen).placement
    sets = [
        getattr(placement, name).of(colour)
        for colour in ("white", "black")
        for name in ("pawns", "knights", "bishops", "rooks", "queens", "kings")
    ]
    units = sum(len(one) for one in sets)

    seen: set[str] = set()
    for one in sets:
        assert not seen & set(one), "a unit stands in two sets"
        seen |= set(one)
    assert len(seen) == units
    assert seen == set(placement.units.white) | set(placement.units.black)
