"""The facts groups, read through their attributes."""

from __future__ import annotations

import pickle

import esca

START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"


def test_the_groups_answer_by_colour() -> None:
    facts = esca.Position.from_fen(START).facts(esca.CLASSIC)
    assert facts.variant == esca.CLASSIC
    assert facts.side_to_move == "white"
    assert facts.material.phase_points == 24
    assert facts.material.pawns == esca.ByColour(8, 8)
    assert facts.pawns.open_files == esca.FileSet()
    assert facts.pawns.pawns.white.squares == [f"{file}2" for file in "abcdefgh"]
    assert facts.king.square == esca.ByColour("e1", "e8")
    assert facts.tactics.legal_move_count == esca.ByColour(20, 20)
    assert not facts.tactics.check_available.white
    assert len(facts.moves) == 20
    assert facts.moves[0].move in [annotated.move for annotated in facts.moves]
    assert facts.moves[0].facts.mover in {"p", "n"}
    assert "material" in facts.summary()


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
    fields = facts.pawns.fields()
    assert list(fields)[:2] == ["pawns", "passed"]
    assert fields["rams"] == 0


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
        assert list(value.fields()) == [field["name"] for field in group["fields"]]
        for field in group["fields"]:
            assert field["type"]
            assert field["meaning"]
            assert hasattr(value, field["name"])


def test_the_move_catalogue_names_every_field_of_a_move() -> None:
    facts = esca.Position.from_fen(START).facts()
    names = [field["name"] for field in esca.move_facts_catalogue()]
    move = facts.moves[0].facts
    assert list(move.fields()) == names
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
    facts = position.facts(esca.CHESS960)
    assert facts.variant == esca.CHESS960
    assert facts.tactics.legal_move_count.white == 20
