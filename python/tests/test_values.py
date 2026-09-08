"""The pair by colour, the file set, and the colour names both read."""

from __future__ import annotations

import pickle
from typing import TYPE_CHECKING

import esca
import pytest

if TYPE_CHECKING:
    from esca._esca import ColourName

START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"


def test_a_pair_is_read_by_name_and_by_colour() -> None:
    pair = esca.ByColour(3, 5)
    assert pair.white == 3
    assert pair.black == 5
    assert pair.of("white") == 3
    assert pair["black"] == 5
    assert pair == esca.ByColour(3, 5)
    assert pair != esca.ByColour(5, 3)
    assert repr(pair) == "ByColour(white=3, black=5)"


@pytest.mark.parametrize("name", ["white", "WHITE", "White", "w", "W"])
def test_a_colour_is_read_from_any_of_its_names(name: ColourName) -> None:
    assert esca.ByColour(1, 2).of(name) == 1


def test_a_name_that_is_no_colour_says_what_it_was_given() -> None:
    with pytest.raises(esca.ParseError, match='not a colour: "grey"'):
        esca.ByColour(1, 2).of("grey")  # type: ignore[bad-argument-type]
    with pytest.raises(esca.ParseError, match='not a colour: "grey"'):
        esca.Position.from_fen(START).by_colour("grey")  # type: ignore[bad-argument-type]


def test_a_pair_survives_a_round_trip() -> None:
    pair = esca.ByColour(["a"], None)
    assert pickle.loads(pickle.dumps(pair)) == pair


def test_a_file_set_holds_files_and_nothing_else() -> None:
    files = esca.FileSet(["b", "g"])
    assert files.files == ["b", "g"]
    assert list(files) == ["b", "g"]
    assert len(files) == 2
    assert "b" in files
    assert "c" not in files
    assert files
    assert not esca.FileSet()
    assert files == esca.FileSet(["g", "b"])
    assert repr(files) == 'FileSet(["b", "g"])'


def test_a_file_set_combines_with_another() -> None:
    left = esca.FileSet(["a", "b"])
    right = esca.FileSet(["b", "c"])
    assert (left & right).files == ["b"]
    assert (left | right).files == ["a", "b", "c"]
    assert (left - right).files == ["a"]
    assert (~left).files == ["c", "d", "e", "f", "g", "h"]
    assert esca.FileSet(["b"]).is_subset(left)


def test_a_letter_that_is_no_file_is_refused() -> None:
    with pytest.raises(ValueError, match="not a file"):
        esca.FileSet(["j"])  # type: ignore[bad-argument-type]


def test_a_file_set_survives_a_round_trip() -> None:
    files = esca.FileSet(["d", "h"])
    assert pickle.loads(pickle.dumps(files)) == files


def test_the_files_of_a_square_set_are_the_files_its_members_stand_on() -> None:
    squares = esca.SquareSet(["d4", "d5", "h1"])
    assert squares.files == esca.FileSet(["d", "h"])
    assert list(squares.files) == ["d", "h"]
    assert not esca.SquareSet().files
