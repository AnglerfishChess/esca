"""The facts groups, read through their attributes."""

from __future__ import annotations

import pickle

import esca

START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"


def test_the_groups_answer_in_the_movers_view() -> None:
    facts = esca.Position.from_fen(START).facts(esca.CLASSIC)
    assert facts.variant == esca.CLASSIC
    assert facts.side_to_move == "w"
    assert facts.material.phase == 1.0
    assert facts.material.count[esca.US] == [8, 2, 2, 2, 1]
    assert facts.pawns.open_files == ""
    assert facts.pawns.pawns[esca.US].squares == [f"{file}2" for file in "abcdefgh"]
    assert facts.king.square == ("e1", "e8")
    assert facts.tactics[esca.US].legal_move_count == 20
    assert not facts.tactics[esca.US].check_available
    assert len(facts.moves) == 20
    assert facts.moves[0].move in [annotated.move for annotated in facts.moves]
    assert facts.moves[0].facts.mover in {"p", "n"}
    assert "material" in facts.summary()


def test_black_to_move_flips_the_view() -> None:
    facts = esca.Position.from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1").facts()
    assert facts.side_to_move == "b"
    assert facts.king.square == ("e8", "e1")
    assert facts.pawns.pawns[esca.US].squares == [f"{file}7" for file in "abcdefgh"]


def test_attack_facts_answer_about_single_squares() -> None:
    facts = esca.Position.from_fen("4k3/8/8/3q4/8/8/4P3/4K3 w - - 0 1").facts()
    attackers = facts.attacks.attackers_of("d5", esca.US)
    assert attackers.squares == []
    assert facts.attacks.units(esca.THEM).squares == ["d5", "e8"]
    assert facts.attacks.is_hanging("d5") is False
    assert len(facts.attacks.by_role[esca.THEM]) == 6


def test_a_group_is_picklable_through_its_facts() -> None:
    facts = esca.Position.from_fen(START).facts()
    revived = pickle.loads(pickle.dumps(facts.pawns))
    assert revived.open_files == facts.pawns.open_files
    assert revived.pawns[esca.US] == facts.pawns.pawns[esca.US]
    tactics = pickle.loads(pickle.dumps(facts.tactics[esca.THEM]))
    assert tactics.legal_move_count == facts.tactics[esca.THEM].legal_move_count
    assert pickle.loads(pickle.dumps(facts)).position == facts.position


def test_chess960_facts_use_the_chess960_rules() -> None:
    position = esca.CHESS960.start_position(518)
    facts = position.facts(esca.CHESS960)
    assert facts.variant == esca.CHESS960
    assert facts.tactics[esca.US].legal_move_count == 20
