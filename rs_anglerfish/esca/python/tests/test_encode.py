"""Batch encoding, against the golden fixtures the Rust side is pinned to."""

from __future__ import annotations

import struct
from pathlib import Path

import esca
import numpy as np
import pytest

DATA = Path(__file__).resolve().parents[2] / "tests" / "data"


def corpus(name: str) -> list[str]:
    lines = (DATA / name).read_text().splitlines()
    return [line.strip() for line in lines if line.strip() and not line.startswith("#")]


def vectors(name: str, rows: int) -> np.ndarray:
    raw = (DATA / name).read_bytes()
    values = struct.unpack(f"<{rows * esca.WIDTH}f", raw)
    return np.asarray(values, dtype=np.float32).reshape(rows, esca.WIDTH)


def test_the_schema_is_the_v1_one() -> None:
    assert esca.SCHEMA == esca.SCHEMA_V1
    assert esca.SCHEMA.id == esca.SCHEMA_ID
    assert (DATA / "schema_v1_id.txt").read_text().strip() == esca.SCHEMA_ID
    assert esca.SCHEMA.width == esca.WIDTH == 1070
    assert esca.SCHEMA.canonical() == (DATA / "schema_v1.txt").read_text()
    assert [group["name"] for group in esca.schema()] == esca.SCHEMA.group_names
    assert esca.schema()[0] == {"name": "placement", "version": 1, "width": 0, "offset": 0}
    assert esca.schema()[1] == {"name": "state", "version": 2, "width": 16, "offset": 0}
    assert esca.SCHEMA.width_of(["state", "pawns"]) == 16 + 165
    assert esca.SCHEMA.width_of(["exchange", "threats", "endgame"]) == 0


def test_encode_returns_a_contiguous_float32_matrix() -> None:
    fens = corpus("fens_classic.txt")[:16]
    encoded = esca.encode(fens)
    assert encoded.dtype == np.float32
    assert encoded.shape == (16, esca.WIDTH)
    assert encoded.flags["C_CONTIGUOUS"]
    assert encoded.flags["WRITEABLE"]
    assert np.isfinite(encoded).all()


def test_encode_matches_the_classic_fixture() -> None:
    fens = corpus("fens_classic.txt")
    expected = vectors("vectors_classic.bin", len(fens))
    assert np.array_equal(esca.encode(fens), expected)


def test_encode_matches_the_chess960_fixture() -> None:
    fens = corpus("fens_chess960.txt")
    expected = vectors("vectors_chess960.bin", len(fens))
    assert np.array_equal(esca.encode(fens, variant=esca.CHESS960), expected)


def test_a_group_subset_is_the_matching_slice() -> None:
    fens = corpus("fens_classic.txt")[:8]
    whole = esca.encode(fens)
    state = esca.encode(fens, groups=["state"])
    assert state.shape == (8, 16)
    assert np.array_equal(state, whole[:, :16])
    with pytest.raises(ValueError, match="groups of the schema"):
        esca.encode(fens, groups=["nonsense"])


def test_encode_into_writes_the_callers_array() -> None:
    fens = corpus("fens_classic.txt")[:8]
    out = np.zeros((8, esca.WIDTH), dtype=np.float32)
    esca.encode_into(fens, out)
    assert np.array_equal(out, esca.encode(fens))
    with pytest.raises(ValueError, match="the output is"):
        esca.encode_into(fens, np.zeros((7, esca.WIDTH), dtype=np.float32))


def test_a_malformed_fen_names_its_row() -> None:
    with pytest.raises(ValueError, match="row 2"):
        esca.encode(["8/8/8/8/8/8/8/K6k w - - 0 1", "8/8/8/8/8/8/8/K6k b - - 0 1", "nonsense"])


def test_encode_moves_gives_a_row_per_legal_move() -> None:
    moves, encoded = esca.encode_moves("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    assert len(moves) == 20
    assert encoded.dtype == np.float32
    assert encoded.shape == (20, esca.MOVE_WIDTH)
    assert encoded.flags["C_CONTIGUOUS"]
    assert {move.uci for move in moves} >= {"e2e4", "g1f3"}


def test_encode_moves_stacks_a_sequence_and_cuts_it_by_offsets() -> None:
    fens = corpus("fens_classic.txt")[:16]
    moves, encoded, offsets = esca.encode_moves(fens)
    assert len(moves) == 16
    assert encoded.dtype == np.float32
    assert encoded.flags["C_CONTIGUOUS"]
    assert offsets.dtype == np.int64
    assert offsets.shape == (17,)
    assert offsets[0] == 0
    assert offsets[-1] == encoded.shape[0] == sum(len(row) for row in moves)
    for row, fen in enumerate(fens):
        one, rows = esca.encode_moves(fen)
        assert [move.uci for move in moves[row]] == [move.uci for move in one]
        assert np.array_equal(encoded[offsets[row] : offsets[row + 1]], rows)


def test_encode_moves_takes_an_empty_sequence() -> None:
    moves, encoded, offsets = esca.encode_moves([])
    assert moves == []
    assert encoded.shape == (0, esca.MOVE_WIDTH)
    assert np.array_equal(offsets, [0])


def test_encode_moves_follows_its_variant() -> None:
    fens = corpus("fens_chess960.txt")[:8]
    moves, encoded, offsets = esca.encode_moves(fens, variant=esca.CHESS960)
    for row, fen in enumerate(fens):
        one, rows = esca.encode_moves(fen, variant=esca.CHESS960)
        assert len(moves[row]) == len(one)
        assert np.array_equal(encoded[offsets[row] : offsets[row + 1]], rows)


def test_a_malformed_fen_in_a_move_batch_names_its_row() -> None:
    with pytest.raises(ValueError, match="row 1"):
        esca.encode_moves(["8/8/8/8/8/8/8/K6k w - - 0 1", "nonsense"])


def test_features_for_drops_what_a_variant_does_not_define() -> None:
    classic = esca.features_for(esca.CLASSIC)
    chess960 = esca.features_for(esca.CHESS960)
    assert ("pieces", "minors_undeveloped") in classic
    assert ("pieces", "minors_undeveloped") not in chess960
    assert ("king", "king_on_home_square") not in chess960
    assert set(chess960) < set(classic)
    assert esca.SCHEMA.features_for(esca.CHESS960) == chess960
