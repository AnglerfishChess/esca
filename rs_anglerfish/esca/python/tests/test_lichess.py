"""Batches over the synthetic dump sample."""

from __future__ import annotations

from pathlib import Path

import esca
import numpy as np
import pytest
from esca import lichess

SAMPLE = Path(__file__).resolve().parents[2] / "tests" / "data" / "lichess_sample.jsonl.zst"


def test_a_batch_carries_features_and_targets() -> None:
    (batch,) = list(lichess.batches(SAMPLE, batch_size=64))
    assert len(batch) == 12
    assert batch.features.shape == (12, esca.WIDTH)
    assert batch.features.dtype == np.float32
    assert batch.features.flags["C_CONTIGUOUS"]
    assert batch.cp.shape == (12,)
    assert batch.mate.shape == (12,)
    assert len(batch.best_moves) == 12
    assert np.array_equal(batch.features, esca.encode(batch.fens))


def test_scores_reach_python_side_relative() -> None:
    (batch,) = list(lichess.batches(SAMPLE, batch_size=64))
    row = batch.fens.index("7r/1p3k2/p1bPR3/5p2/2B2P1p/8/PP4P1/3K4 b - -")
    assert batch.cp[row] == -69.0
    assert batch.mate[row] == 0.0
    mating = batch.fens.index("1r3rk1/2p2ppp/3p4/3Bp3/4P3/KP1P4/8/4q3 b - -")
    assert batch.mate[mating] == 1.0
    assert batch.cp[mating] == 0.0
    assert batch.best_moves[mating].uci == "e1b4"


def test_batches_are_cut_to_size() -> None:
    sizes = [len(batch) for batch in lichess.batches(SAMPLE, batch_size=5)]
    assert sizes == [5, 5, 2]


def test_min_depth_drops_the_shallow_records() -> None:
    deep = list(lichess.batches(SAMPLE, batch_size=64, min_depth=40))
    assert [len(batch) for batch in deep] == [5]
    assert not list(lichess.batches(SAMPLE, batch_size=64, min_depth=200))


def test_a_group_subset_narrows_the_features() -> None:
    (batch,) = list(lichess.batches(SAMPLE, batch_size=64, groups=["state", "material"]))
    assert batch.features.shape == (12, 29 + 26)


def test_a_missing_dump_is_an_error() -> None:
    with pytest.raises(OSError):
        lichess.batches(SAMPLE.parent / "no_such_dump.jsonl.zst")
