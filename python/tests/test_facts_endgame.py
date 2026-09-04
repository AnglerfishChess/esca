"""The `endgame` group, fact by fact.

Every expectation is worked out from the definitions in `docs/features.md`
§1 and §2.12 for the named position above it. The cases mirror
`tests/facts_endgame.rs`.
"""

from __future__ import annotations

from collections.abc import Callable

import esca
import pytest

#: The untouched array: no passer, no opposition, no ending.
START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

#: Kings on d3 and d5 with d4 between them: the direct opposition.
BARE_KINGS = "8/8/8/3k4/8/3K4/8/8 w - - 0 1"

#: The same with Black to move, so the opposition changes hands and the
#: one-hot, which says only which kind stands, does not.
BARE_KINGS_BLACK = "8/8/8/3k4/8/3K4/8/8 b - - 0 1"

#: Kings on e2 and e8: five empty squares between them.
DISTANT = "4k3/8/8/8/8/8/4K3/8 w - - 0 1"

#: Kings on e2 and e7: four, so neither side has the opposition.
NO_OPPOSITION = "8/4k3/8/8/8/8/4K3/8 w - - 0 1"

#: A pawn on e5 stands between the kings, so the file is no corridor.
BLOCKED_FILE = "8/8/4k3/4p3/4K3/8/8/8 w - - 0 1"

#: Kings on c3 and e5: the opposition holds on a diagonal too.
DIAGONAL = "8/8/8/4k3/8/2K5/8/8 w - - 0 1"

#: A pawn each on opposite wings, the black one a move from queening with its
#: king already on the squares it promotes through.
PAWN_RACE = "8/8/8/P7/8/8/6p1/K6k w - - 0 1"

#: The same with Black to move: the tempo takes a ply off Black's race.
PAWN_RACE_BLACK = "8/8/8/P7/8/8/6p1/K6k b - - 0 1"

#: Pawns that block each other head on: neither side has a passer.
BLOCKED_PAWNS = "8/8/8/3p4/3P4/8/8/K6k w - - 0 1"

#: The white king two ranks ahead of its passer on e4, on a key square.
KEY_SQUARE = "8/8/4K3/8/4P3/8/8/4k3 w - - 0 1"

#: The same with Black to move, so the key square is theirs.
KEY_SQUARE_BLACK = "8/8/4K3/8/4P3/8/8/4k3 b - - 0 1"

#: A passer on the fifth: its key squares are the three squares in front.
KEY_SQUARE_HIGH = "8/8/3K4/4P3/8/8/8/4k3 w - - 0 1"

#: The king on b6 escorts an a-pawn, which has no key squares at all.
ROOK_PAWN_KING = "8/8/1K6/P7/8/8/8/4k3 w - - 0 1"

#: The king right in front of its passer, a rank short of a key square.
SHORT_OF_KEY = "8/8/8/4K3/4P3/8/8/4k3 w - - 0 1"

#: The king on e6 with the e4 pawn no longer passed: the d6 pawn stops it.
KEY_SQUARE_NOT_PASSED = "8/8/3pK3/8/4P3/8/8/k7 w - - 0 1"

#: A light-squared bishop with an h-pawn, which promotes on a dark square.
WRONG_BISHOP = "7k/8/8/8/8/7P/6B1/6K1 w - - 0 1"

#: The same with Black to move: the wrong bishop is theirs.
WRONG_BISHOP_BLACK = "7k/8/8/8/8/7P/6B1/6K1 b - - 0 1"

#: The same bishop on h2, the colour its own h-pawn promotes on.
RIGHT_BISHOP = "7k/8/8/8/8/7P/7B/6K1 w - - 0 1"

#: Rook pawns on both wings: the a-pawn promotes on the bishop's colour.
BOTH_ROOK_PAWNS = "7k/8/8/8/8/P6P/6B1/6K1 w - - 0 1"

#: A bishop whose only pawn stands on e3: not a rook pawn.
CENTRE_PAWN_BISHOP = "7k/8/8/8/8/4P3/6B1/6K1 w - - 0 1"

#: Black's dark-squared bishop against its own h-pawn, promoting on h1.
WRONG_BISHOP_THEM = "6k1/6b1/7p/8/8/8/8/7K w - - 0 1"

#: Two knights against a bare king: no forced mate.
TWO_KNIGHTS = "8/8/8/3k4/8/8/1NN5/3K4 w - - 0 1"

#: The same two knights on the other side.
TWO_KNIGHTS_THEM = "3k4/8/1nn5/8/8/8/8/3K4 w - - 0 1"

#: Two knights and a pawn: the pawn takes the material out of the drawn set.
TWO_KNIGHTS_AND_PAWN = "8/8/8/3k4/8/8/1NN2P2/3K4 w - - 0 1"

#: One bishop each on opposite colours, with a pawn each and no other piece.
OPPOSITE_BISHOPS = "8/3k4/4p3/4b3/3P4/3B4/8/3K4 w - - 0 1"

#: Both bishops on light squares.
SAME_COLOUR_BISHOPS = "8/3k4/2b1p3/8/8/3B4/8/3K4 w - - 0 1"

#: The opposite bishops with a knight still on: a piece too many.
OPPOSITE_BISHOPS_AND_KNIGHT = "8/3k4/4p3/4b3/3P4/3B4/5N2/3K4 w - - 0 1"

#: The helper `conftest.py` hands over.
FactsOf = Callable[..., esca.Facts]


def endgame_row(fen: str) -> list[float]:
    """The 15 values of the `endgame` row of `fen`."""
    return list(esca.encode([fen], groups=["endgame"])[0])


@pytest.mark.parametrize(
    ("fen", "distance"),
    [
        (START, (3, 3)),
        (BARE_KINGS, (1, 0)),
        (BARE_KINGS_BLACK, (0, 1)),
        (DISTANT, (2, 3)),
        (NO_OPPOSITION, (2, 2)),
        (BLOCKED_FILE, (0, 1)),
        (KEY_SQUARE, (1, 3)),
        (KEY_SQUARE_BLACK, (3, 1)),
        (TWO_KNIGHTS, (3, 0)),
        (OPPOSITE_BISHOPS, (3, 2)),
    ],
    ids=[
        "start",
        "bare_kings",
        "bare_kings_black",
        "distant",
        "no_opposition",
        "blocked_file",
        "key_square",
        "key_square_black",
        "two_knights",
        "opposite_bishops",
    ],
)
def test_the_king_centralisation_is_the_distance_to_the_nearest_central_square(
    fen: str, distance: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).endgame.king_centralisation == distance


@pytest.mark.parametrize(
    ("fen", "plies"),
    [
        (START, (8, 8)),
        (BLOCKED_PAWNS, (8, 8)),
        (PAWN_RACE, (2, 1)),
        (PAWN_RACE_BLACK, (0, 3)),
        (BLOCKED_FILE, (8, 4)),
        (KEY_SQUARE, (3, 8)),
        (KEY_SQUARE_BLACK, (8, 4)),
        (KEY_SQUARE_HIGH, (2, 8)),
        (WRONG_BISHOP, (4, 8)),
        (WRONG_BISHOP_THEM, (8, 5)),
    ],
    ids=[
        "start",
        "blocked_pawns",
        "pawn_race",
        "pawn_race_black",
        "blocked_file",
        "key_square",
        "key_square_black",
        "key_square_high",
        "wrong_bishop",
        "wrong_bishop_them",
    ],
)
def test_race_plies_are_what_the_leading_passer_still_needs(
    fen: str, plies: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).endgame.race_plies == plies


@pytest.mark.parametrize(
    ("fen", "difference"),
    [
        (START, 0),
        (PAWN_RACE, 1),
        (PAWN_RACE_BLACK, -3),
        (BLOCKED_FILE, 4),
        (KEY_SQUARE, -5),
        (KEY_SQUARE_HIGH, -6),
        (WRONG_BISHOP_THEM, 3),
    ],
    ids=[
        "start",
        "pawn_race",
        "pawn_race_black",
        "blocked_file",
        "key_square",
        "key_square_high",
        "wrong_bishop_them",
    ],
)
def test_the_race_difference_is_ours_less_theirs(fen: str, difference: int, facts_of: FactsOf) -> None:
    assert facts_of(fen).endgame.race_plies_diff == difference


@pytest.mark.parametrize(
    ("fen", "opposition"),
    [
        (START, None),
        (BARE_KINGS, "direct"),
        (BARE_KINGS_BLACK, "direct"),
        (DIAGONAL, "direct"),
        (DISTANT, "distant"),
        (TWO_KNIGHTS, "distant"),
        (NO_OPPOSITION, None),
        (BLOCKED_FILE, None),
        (PAWN_RACE, None),
    ],
    ids=[
        "start",
        "bare_kings",
        "bare_kings_black",
        "diagonal",
        "distant",
        "two_knights",
        "no_opposition",
        "blocked_file",
        "pawn_race",
    ],
)
def test_the_opposition_needs_an_odd_number_of_empty_squares_between_the_kings(
    fen: str, opposition: str | None, facts_of: FactsOf
) -> None:
    assert facts_of(fen).endgame.opposition == opposition


@pytest.mark.parametrize(
    ("fen", "occupied"),
    [
        (START, (False, False)),
        (KEY_SQUARE, (True, False)),
        (KEY_SQUARE_BLACK, (False, True)),
        (KEY_SQUARE_HIGH, (True, False)),
        (PAWN_RACE, (False, True)),
        (PAWN_RACE_BLACK, (True, False)),
        (ROOK_PAWN_KING, (False, False)),
        (SHORT_OF_KEY, (False, False)),
        (KEY_SQUARE_NOT_PASSED, (False, False)),
    ],
    ids=[
        "start",
        "key_square",
        "key_square_black",
        "key_square_high",
        "pawn_race",
        "pawn_race_black",
        "rook_pawn_king",
        "short_of_key",
        "not_passed",
    ],
)
def test_the_king_stands_on_a_key_square_of_a_passer_of_its_own(
    fen: str, occupied: tuple[bool, bool], facts_of: FactsOf
) -> None:
    assert facts_of(fen).endgame.key_square_occupied == occupied


@pytest.mark.parametrize(
    ("fen", "wrong"),
    [
        (START, (False, False)),
        (WRONG_BISHOP, (True, False)),
        (WRONG_BISHOP_BLACK, (False, True)),
        (WRONG_BISHOP_THEM, (False, True)),
        (RIGHT_BISHOP, (False, False)),
        (BOTH_ROOK_PAWNS, (False, False)),
        (CENTRE_PAWN_BISHOP, (False, False)),
        (OPPOSITE_BISHOPS, (False, False)),
    ],
    ids=[
        "start",
        "wrong_bishop",
        "wrong_bishop_black",
        "wrong_bishop_them",
        "right_bishop",
        "both_rook_pawns",
        "centre_pawn",
        "opposite_bishops",
    ],
)
def test_a_bishop_is_the_wrong_colour_for_rook_pawns_promoting_on_the_other(
    fen: str, wrong: tuple[bool, bool], facts_of: FactsOf
) -> None:
    assert facts_of(fen).endgame.wrong_colour_bishop == wrong


@pytest.mark.parametrize(
    ("fen", "drawn"),
    [
        (START, None),
        (BARE_KINGS, None),
        (TWO_KNIGHTS, "two_knights"),
        (TWO_KNIGHTS_THEM, "two_knights"),
        (TWO_KNIGHTS_AND_PAWN, None),
        (WRONG_BISHOP, "wrong_bishop"),
        (WRONG_BISHOP_THEM, "wrong_bishop"),
        (RIGHT_BISHOP, None),
        (OPPOSITE_BISHOPS, "opposite_bishops"),
        (SAME_COLOUR_BISHOPS, None),
        (OPPOSITE_BISHOPS_AND_KNIGHT, None),
    ],
    ids=[
        "start",
        "bare_kings",
        "two_knights",
        "two_knights_them",
        "two_knights_and_pawn",
        "wrong_bishop",
        "wrong_bishop_them",
        "right_bishop",
        "opposite_bishops",
        "same_colour_bishops",
        "opposite_bishops_and_knight",
    ],
)
def test_drawish_material_names_the_three_configurations_that_still_draw(
    fen: str, drawn: str | None, facts_of: FactsOf
) -> None:
    assert facts_of(fen).endgame.drawish_material == drawn


# The seven features in schema order: two centralisations, two race counts,
# their difference, the opposition one-hot with its third slot for none, the
# two bit pairs, and the drawn-material one-hot.
# fmt: off
BARE_KINGS_ROW = [
    1.0 / 3.0, 0.0,
    1.0, 1.0,
    0.0,
    1.0, 0.0, 0.0,
    0.0, 0.0,
    0.0, 0.0,
    0.0, 0.0, 0.0,
]

PAWN_RACE_ROW = [
    1.0, 1.0,
    0.25, 0.125,
    0.125,
    0.0, 0.0, 1.0,
    0.0, 1.0,
    0.0, 0.0,
    0.0, 0.0, 0.0,
]

WRONG_BISHOP_ROW = [
    1.0, 1.0,
    0.5, 1.0,
    -0.5,
    0.0, 0.0, 1.0,
    0.0, 0.0,
    1.0, 0.0,
    0.0, 1.0, 0.0,
]
# fmt: on


@pytest.mark.parametrize(
    ("fen", "row"),
    [
        (BARE_KINGS, BARE_KINGS_ROW),
        (PAWN_RACE, PAWN_RACE_ROW),
        (WRONG_BISHOP, WRONG_BISHOP_ROW),
    ],
    ids=["bare_kings", "pawn_race", "wrong_bishop"],
)
def test_the_encoded_row_carries_the_group_in_the_schemas_order(fen: str, row: list[float]) -> None:
    assert endgame_row(fen) == pytest.approx(row)
