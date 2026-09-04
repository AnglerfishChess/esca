"""The ECO catalogue, case by case.

Every code and name is the one the bundled data set gives for the line above
it. The cases mirror `tests/openings.rs`.
"""

from __future__ import annotations

import esca
import pytest
from esca import openings

#: The Queen's Gambit Declined as the data set writes it.
QGD = "d4 d5 c4 e6 Nc3 Nf6 Bg5 Be7 Nf3"

#: The same nine moves in another order: the knights and the bishop come out
#: when it suits them, and the ninth move stands on the same board.
QGD_TRANSPOSED = "Nf3 d5 d4 Nf6 c4 e6 Nc3 Be7 Bg5"

#: A Najdorf, then a move nobody has named.
NAJDORF_THEN_OFF_BOOK = "e4 c5 Nf3 d6 d4 cxd4 Nxd4 Nf6 Nc3 a6 h4"

#: The classic array is Chess960 arrangement 518.
CLASSIC_ARRANGEMENT = 518


def played(moves: str) -> esca.Game:
    """The classic game the space-separated SAN `moves` reach."""
    game = esca.Game()
    for text in moves.split():
        game.play_san(text)
    return game


@pytest.mark.parametrize(
    ("moves", "eco", "name"),
    [
        pytest.param("e4", "B00", "King's Pawn Game", id="kings_pawn"),
        pytest.param("d4", "A40", "Queen's Pawn Game", id="queens_pawn"),
        pytest.param("e4 c5", "B20", "Sicilian Defense", id="sicilian"),
        pytest.param("e4 e5 Nf3 Nc6 Bb5", "C60", "Ruy Lopez", id="ruy_lopez"),
        pytest.param("e4 e5 Nf3 Nc6 Bc4", "C50", "Italian Game", id="italian"),
        pytest.param("d4 Nf6 c4 g6 Nc3", "E61", "King's Indian Defense", id="kings_indian"),
        pytest.param(QGD, "D53", "Queen's Gambit Declined", id="queens_gambit_declined"),
        pytest.param(
            "e4 c5 Nf3 d6 d4 cxd4 Nxd4 Nf6 Nc3 a6",
            "B90",
            "Sicilian Defense: Najdorf Variation",
            id="najdorf",
        ),
    ],
)
def test_a_named_position_answers_with_its_code_and_its_name(moves: str, eco: str, name: str) -> None:
    opening = openings.lookup(played(moves).position)
    assert opening is not None
    assert opening.eco == eco
    assert opening.name == name


def test_a_line_that_transposes_into_a_named_position_is_named() -> None:
    direct = played(QGD)
    transposed = played(QGD_TRANSPOSED)
    assert direct.moves != transposed.moves
    assert openings.lookup(direct.position) == openings.lookup(transposed.position)
    named = openings.lookup(transposed.position)
    assert named is not None
    assert named.eco == "D53"


def test_the_starting_array_has_no_name() -> None:
    assert openings.lookup(esca.Game().position) is None


def test_a_position_nobody_has_named_has_no_name() -> None:
    assert openings.lookup(esca.Position.from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1")) is None


def test_a_game_keeps_the_deepest_name_it_reached() -> None:
    game = played(NAJDORF_THEN_OFF_BOOK)
    assert openings.lookup(game.position) is None
    opening = game.opening()
    assert opening is not None
    assert opening.eco == "B90"
    assert opening.name == "Sicilian Defense: Najdorf Variation"


def test_a_game_that_has_reached_no_named_position_has_no_opening() -> None:
    assert esca.Game().opening() is None


def test_the_catalogue_is_keyed_by_position_and_not_by_the_rules_in_force() -> None:
    game = esca.Game(variant=esca.CHESS960, seed=CLASSIC_ARRANGEMENT)
    assert game.position.fen == esca.Game().position.fen
    game.play_san("e4")
    opening = game.opening()
    assert opening is not None
    assert opening.name == "King's Pawn Game"


def test_every_row_of_the_data_set_names_a_position_of_its_own() -> None:
    # The bundled volumes hold 3,810 rows between them.
    assert openings.count() == 3810


def test_an_opening_reads_as_its_code_and_then_its_name() -> None:
    opening = openings.lookup(played("e4 e5 Nf3 Nc6 Bb5").position)
    assert str(opening) == "C60 Ruy Lopez"
