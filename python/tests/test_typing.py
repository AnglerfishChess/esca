"""What a type checker sees when it reads the facts.

Every assertion here is checked twice: `pyrefly` reads it statically, and
pytest runs it so that a stub which has drifted from the extension shows up as
a failure rather than only as a warning.
"""

from __future__ import annotations

from typing import assert_type

import esca

START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

FACTS = esca.Position.from_fen(START).facts()


def test_a_square_set_fact_is_a_square_set_per_colour() -> None:
    assert_type(FACTS.pawns.passed, esca.ByColour[esca.SquareSet])
    assert_type(FACTS.pawns.passed.white, esca.SquareSet)
    assert_type(FACTS.pawns.passed.of("black"), esca.SquareSet)
    assert isinstance(FACTS.pawns.passed.white, esca.SquareSet)


def test_a_per_file_fact_is_eight_counts_in_file_order() -> None:
    assert_type(FACTS.pawns.count_by_file.white[0], int)
    assert_type(FACTS.pawns.count_by_rank.white[7], int)
    assert len(FACTS.pawns.count_by_file.white) == 8


def test_a_file_set_fact_is_a_file_set() -> None:
    assert_type(FACTS.pawns.open_files, esca.FileSet)
    assert isinstance(FACTS.pawns.open_files, esca.FileSet)


def test_a_count_is_an_int_and_a_flag_is_a_bool() -> None:
    assert_type(FACTS.material.phase_points, int)
    assert_type(FACTS.state.in_check, bool)
    assert_type(FACTS.material.value.black, int)
    assert_type(FACTS.king.tropism.white, float)


def test_an_absent_value_is_none_or_the_value() -> None:
    assert_type(FACTS.pawns.passer_lead_rank.white, int | None)
    assert FACTS.state.en_passant is None
    assert FACTS.history.last_move_victim is None


def test_a_run_of_three_is_a_tuple_of_three() -> None:
    assert_type(FACTS.king.shield.white[1], int | None)
    assert len(FACTS.king.shield_files.white) == 3


def test_the_plain_data_is_typed_group_by_group_and_field_by_field() -> None:
    written = FACTS.to_dict()
    assert_type(written["pawns"]["passed"]["white"], list[str])
    assert_type(written["pawns"]["count_by_file"]["black"], list[int])
    assert_type(written["material"]["phase_points"], int)
    assert_type(written["state"]["in_check"], bool)
    assert written["pawns"]["passed"]["white"] == []

    pawns = FACTS.pawns.to_dict()
    assert_type(pawns["open_files"], list[str])
    assert_type(pawns["rams"], int)


def test_a_move_carries_its_own_typed_facts() -> None:
    move = FACTS.moves[0].facts
    assert_type(move.see, int)
    assert_type(move.gives_check, bool)
    assert_type(move.to_dict()["gives_check"], bool)
