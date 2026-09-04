"""Positions, games and moves through the Python classes."""

from __future__ import annotations

import pickle

import esca
import pytest

START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"


def test_the_variants_are_the_two_named_ones() -> None:
    assert esca.CLASSIC.name == "chess"
    assert esca.CHESS960.name == "chess960"
    assert esca.Variant.named("chess") == esca.CLASSIC
    assert esca.CLASSIC != esca.CHESS960
    assert esca.CLASSIC.promotion_roles == ["q", "r", "b", "n"]
    with pytest.raises(ValueError, match="not a variant"):
        esca.Variant.named("horde")


def test_a_position_round_trips_through_its_fen() -> None:
    position = esca.Position.from_fen(START)
    assert position.fen == START
    assert position.epd == "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -"
    assert position.side_to_move == "w"
    assert position.castling_rights == "KQkq"
    assert position.en_passant is None
    assert position.clocks_known
    assert not position.in_check
    assert position.piece_at("e1") == "K"
    assert position.piece_at("e4") is None
    assert position.king_of("b") == "e8"
    assert len(position.occupied) == 32
    assert position == esca.Position.from_fen(START)
    assert hash(position) == hash(esca.Position.from_fen(START))
    assert pickle.loads(pickle.dumps(position)) == position


def test_a_four_field_fen_leaves_the_clocks_unknown() -> None:
    position = esca.Position.from_fen("4k3/8/8/8/8/8/8/4K3 w - -")
    assert not position.clocks_known
    assert position.halfmove_clock == 0
    assert position.fullmove_number == 1
    assert pickle.loads(pickle.dumps(position)).epd == position.epd


def test_a_malformed_fen_is_a_value_error() -> None:
    with pytest.raises(ValueError, match="four or six fields"):
        esca.Position.from_fen("not a fen")


def test_square_sets_behave_like_sets() -> None:
    position = esca.Position.from_fen(START)
    pawns = position.by_piece("p", "w")
    assert len(pawns) == 8
    assert "e2" in pawns
    assert "e4" not in pawns
    assert sorted(pawns) == [f"{file}2" for file in "abcdefgh"]
    assert (pawns & position.occupied) == pawns
    assert len(pawns | position.by_piece("p", "b")) == 16
    assert (pawns - pawns).bits == 0
    assert not (pawns - pawns)
    assert pawns.is_subset(position.by_role("p"))
    assert pickle.loads(pickle.dumps(pawns)) == pawns
    assert esca.SquareSet(["a1", "h8"]).squares == ["a1", "h8"]


def test_a_game_plays_uci_and_san() -> None:
    game = esca.Game()
    game.play("e2e4")
    game.play_san("e5")
    game.play(game.legal_moves()[0])
    assert game.ply == 3
    assert len(game.moves) == 3
    assert len(game.positions) == 4
    assert game.position != game.start_position
    played = game.undo()
    assert played is not None
    assert game.ply == 2
    assert game.position.fen == "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2"


def test_a_move_carries_its_own_facts() -> None:
    game = esca.Game.from_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2")
    capture = next(mv for mv in game.legal_moves() if mv.destination == "d5")
    assert capture.origin == "e4"
    assert capture.is_capture
    assert capture.kind == "capture"
    assert capture.uci == "e4d5"
    assert game.move_to_san(capture) == "exd5"
    assert pickle.loads(pickle.dumps(capture)) == capture


def test_castling_is_spelled_as_the_game_asks() -> None:
    game = esca.Game.from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1")
    castling = next(mv for mv in game.legal_moves() if mv.is_castling and mv.destination == "h1")
    assert game.castling_output == esca.KING_TO_ROOK
    assert game.move_to_uci(castling) == "e1h1"
    game.castling_output = esca.KING_TWO_SQUARES
    assert game.move_to_uci(castling) == "e1g1"
    assert game.move_to_san(castling) == "O-O"
    with pytest.raises(ValueError, match="castling output"):
        game.castling_output = "nonsense"


def test_a_chess960_game_starts_from_its_seed() -> None:
    game = esca.Game(variant=esca.CHESS960, seed=518)
    assert game.variant == esca.CHESS960
    assert game.position.fen == esca.CHESS960.start_position(518).fen
    assert len(game.legal_moves()) == 20


def test_outcomes_and_claims() -> None:
    mated = esca.Game.from_fen("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3")
    assert mated.outcome() == "checkmate"
    assert mated.claims() == []
    fifty = esca.Game.from_fen("8/8/4k3/8/8/4K3/8/6R1 w - - 100 80")
    assert fifty.claims() == ["fifty_moves"]
    assert esca.Game().outcome() is None


def test_repetition_needs_the_game() -> None:
    game = esca.Game()
    for move in ["g1f3", "g8f6", "f3g1", "f6g8"] * 2:
        game.play(move)
    assert game.repetitions() == 3
    assert "threefold_repetition" in game.claims()
    assert game.facts().history.repetition_seen
    assert game.facts().history.known
    assert not game.position.facts().history.known
