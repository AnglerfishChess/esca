"""The facts groups, read through their attributes."""

from __future__ import annotations

import pickle

import esca

START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"


def test_the_groups_answer_by_colour() -> None:
    facts = esca.Position.from_fen(START).facts(variant=esca.CLASSIC)
    assert facts.variant == esca.CLASSIC
    assert facts.side_to_move == "white"
    assert facts.material.phase_points == 24
    assert facts.material.pawns == esca.ByColour(8, 8)
    assert facts.pawns.open_files == esca.FileSet()
    assert facts.pawns.pawns.white.squares == [f"{file}2" for file in "abcdefgh"]
    assert facts.king.square == esca.ByColour("e1", "e8")
    assert facts.tactics.legal_move_count == esca.ByColour(20, 20)
    assert not facts.tactics.check_available.white
    assert "material" in facts.summary()

    annotated = esca.Position.from_fen(START).annotated_moves()
    assert len(annotated) == 20
    assert annotated[0].move in [one.move for one in annotated]
    assert annotated[0].facts.mover in {"p", "n"}


def test_the_same_placement_reads_the_same_whoever_is_to_move() -> None:
    facts = esca.Position.from_fen(START.replace(" w ", " b ")).facts()
    assert facts.side_to_move == "black"
    assert facts.king.square == esca.ByColour("e1", "e8")
    assert facts.pawns.pawns.white.squares == [f"{file}2" for file in "abcdefgh"]
    assert facts.pawns.pawns.black.squares == [f"{file}7" for file in "abcdefgh"]


def test_a_colour_is_read_by_any_of_its_names() -> None:
    facts = esca.Position.from_fen(START).facts()
    black = facts.pawns.pawns.black
    for name in ("black", "Black", "b", "B"):
        assert facts.pawns.pawns.of(name) == black
        assert facts.pawns.pawns[name] == black


def test_the_groups_are_walked_by_name() -> None:
    facts = esca.Position.from_fen(START).facts()
    assert facts.groups()[:3] == ["placement", "state", "history"]
    assert facts.group("pawns").pawns == facts.pawns.pawns
    written = facts.pawns.to_dict()
    assert list(written)[:2] == ["pawns", "passed"]
    assert written["rams"] == 0


def test_a_group_becomes_plain_data() -> None:
    facts = esca.Position.from_fen("8/pp3ppk/8/3P4/8/8/PP3PPK/8 w - - 0 1").facts()
    pawns = facts.pawns.to_dict()
    assert pawns["passed"] == {"white": ["d5"], "black": []}
    assert pawns["open_files"] == ["c", "e", "h"]
    assert pawns["rams"] == 0

    whole = facts.to_dict()
    assert list(whole) == facts.groups()
    assert whole["pawns"] == pawns
    assert whole["state"]["in_check"] is False


def test_the_catalogue_names_every_field_of_every_group() -> None:
    facts = esca.Position.from_fen(START).facts()
    catalogue = esca.facts_catalogue()
    assert [group["name"] for group in catalogue] == facts.groups()
    for group in catalogue:
        assert group["meaning"]
        value = facts.group(group["name"])
        assert list(value.to_dict()) == [field["name"] for field in group["fields"]]
        for field in group["fields"]:
            assert field["type"]
            assert field["meaning"]
            assert hasattr(value, field["name"])


def test_the_move_catalogue_names_every_field_of_a_move() -> None:
    names = [field["name"] for field in esca.move_facts_catalogue()]
    move = esca.Position.from_fen(START).annotated_moves()[0].facts
    assert list(move.to_dict()) == names
    assert move.to_dict()["mover"] in {"p", "n"}


def test_the_files_a_passer_stands_on_are_read_off_its_squares() -> None:
    facts = esca.Position.from_fen("8/pp3ppk/8/3P4/8/8/PP3PPK/8 w - - 0 1").facts()
    assert list(facts.pawns.passed.white.files) == ["d"]
    assert facts.pawns.passed.white.files == esca.FileSet(["d"])
    assert not facts.pawns.passed.black.files


def test_a_group_is_picklable_through_its_facts() -> None:
    facts = esca.Position.from_fen(START).facts()
    revived = pickle.loads(pickle.dumps(facts.pawns))
    assert revived.open_files == facts.pawns.open_files
    assert revived.pawns.white == facts.pawns.pawns.white
    tactics = pickle.loads(pickle.dumps(facts.tactics))
    assert tactics.legal_move_count == facts.tactics.legal_move_count
    assert pickle.loads(pickle.dumps(facts)).position == facts.position


def test_chess960_facts_use_the_chess960_rules() -> None:
    position = esca.CHESS960.start_position(518)
    facts = position.facts(variant=esca.CHESS960)
    assert facts.variant == esca.CHESS960
    assert facts.tactics.legal_move_count.white == 20


def test_a_group_reads_as_its_values_and_compares_by_them() -> None:
    facts = esca.Position.from_fen(START).facts()
    same = esca.Position.from_fen(START).facts()
    assert facts.pawns == same.pawns
    assert hash(facts.pawns) == hash(same.pawns)
    assert facts.pawns != facts.pieces
    assert facts == same
    assert repr(facts.material).startswith("MaterialFacts(pawns=ByColour(white=8, black=8)")
    assert "passed=ByColour(white=SquareSet([]), black=SquareSet([]))" in repr(facts.pawns)


def test_a_pair_by_colour_is_read_positionally_too() -> None:
    facts = esca.Position.from_fen(START).facts()
    match facts.material.pawns:
        case esca.ByColour(white, black):
            assert (white, black) == (8, 8)
        case _:  # pragma: no cover - the pattern above always matches
            raise AssertionError("a ByColour matches on white and black")


def test_a_run_of_values_is_a_tuple_in_the_order_its_type_states() -> None:
    facts = esca.Position.from_fen("8/pp3ppk/8/3P4/8/8/PP3PPK/8 w - - 0 1").facts()
    assert facts.pawns.count_by_file.white == (1, 1, 0, 1, 0, 1, 1, 0)
    assert facts.pawns.count_by_rank.white == (0, 4, 0, 0, 1, 0, 0, 0)
    assert facts.king.shield_files.white == ("f", "g", "h")
    assert facts.king.shield.white == (None, None, None)
