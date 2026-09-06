"""The typed tensor form: one array per fact, batch first, nothing scaled."""

from __future__ import annotations

import json
from pathlib import Path

import esca
import numpy as np
import pytest
from esca import tensors

ROOT = Path(__file__).resolve().parents[2]
GOLDEN = ROOT / "tests" / "data" / "facts"

#: One position with a white passer on d5 and nothing but kings and pawns.
PASSER = "8/pp3ppk/8/3P4/8/8/PP3PPK/8 w - - 0 1"
#: The opening array.
START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
#: A middlegame busy with attacks, pins and castling rights.
KIWIPETE = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"

#: The classic positions the golden batch was written for, in that order.
GOLDEN_FENS = [
    entry["fen"] for entry in json.loads((GOLDEN / "positions.json").read_text()) if entry["variant"] == "chess"
]


def test_the_layout_names_every_fact_of_every_group() -> None:
    named = [f"{group['name']}.{field['name']}" for group in esca.facts_catalogue() for field in group["fields"]]
    assert [entry["name"] for entry in tensors.layout()] == named


def test_every_array_holds_what_the_layout_says() -> None:
    arrays = tensors.facts([START, PASSER, KIWIPETE])
    assert set(arrays) == {entry["name"] for entry in tensors.layout()}
    for entry in tensors.layout():
        array = arrays[entry["name"]]
        assert array.dtype == np.dtype(entry["dtype"]), entry["name"]
        assert array.shape == (3, *entry["shape"]), entry["name"]


@pytest.mark.parametrize(
    ("name", "dtype", "shape"),
    [
        ("state.in_check", "bool", ()),
        ("state.can_castle_short", "bool", (2,)),
        ("pawns.count_by_file", "uint8", (2, 8)),
        ("material.value", "int32", (2,)),
        ("king.tropism", "float32", (2,)),
        ("pawns.passer_lead_rank", "int16", (2,)),
        ("pawns.passed", "bool", (2, 64)),
        ("pawns.open_files", "bool", (8,)),
        ("king.square", "uint8", (2,)),
        ("king.shield", "int16", (2, 3)),
    ],
    ids=[
        "a_flag",
        "a_flag_per_side",
        "a_count",
        "a_value",
        "a_fraction",
        "an_absent_value",
        "a_square_set",
        "a_file_set",
        "a_named_value",
        "a_run_of_three",
    ],
)
def test_a_fact_keeps_the_type_it_was_declared_with(name: str, dtype: str, shape: tuple[int, ...]) -> None:
    entry = next(item for item in tensors.layout() if item["name"] == name)
    assert entry["dtype"] == dtype
    assert tuple(entry["shape"]) == shape


def test_a_square_set_is_one_flag_per_square_a1_first() -> None:
    plane = tensors.facts([PASSER], groups=["pawns"])["pawns.passed"]
    assert plane.shape == (1, 2, 64)
    # d5 is White's only passer, and Black has none.
    assert [tensors.SQUARES[index] for index in plane[0, 0].nonzero()[0]] == ["d5"]
    assert not plane[0, 1].any()


def test_a_count_stands_where_its_file_says() -> None:
    counts = tensors.facts([PASSER], groups=["pawns"])["pawns.count_by_file"]
    # White's pawns stand on a2 b2 d5 f2 g2, Black's on a7 b7 f7 g7.
    assert counts[0, 0].tolist() == [1, 1, 0, 1, 0, 1, 1, 0]
    assert counts[0, 1].tolist() == [1, 1, 0, 0, 0, 1, 1, 0]


def test_an_absent_value_is_minus_one() -> None:
    ranks = tensors.facts([PASSER], groups=["pawns"])["pawns.passer_lead_rank"]
    # White's passer stands on its rank 5; Black has no passer at all.
    assert ranks[0].tolist() == [5, -1]


def test_nothing_is_a_float_but_the_one_fact_that_is_fractional() -> None:
    assert [entry["name"] for entry in tensors.layout() if entry["dtype"] == "float32"] == ["king.tropism"]


def test_the_packed_form_holds_the_same_facts() -> None:
    fens = [START, PASSER, KIWIPETE]
    expanded = tensors.facts(fens)
    packed = tensors.facts(fens, layout="packed")
    revived = tensors.expand(packed, len(fens))
    for name, array in expanded.items():
        assert np.array_equal(revived[name], array), name


def test_a_packed_square_set_is_one_bitboard() -> None:
    packed = tensors.facts([PASSER], groups=["pawns"], layout="packed")
    boards = packed["pawns.passed"]
    assert boards.dtype == np.uint64
    assert boards.shape == (1, 2)
    assert boards[0].tolist() == [1 << tensors.SQUARES.index("d5"), 0]
    # Files c, e and h carry no pawn of either colour.
    assert packed["pawns.open_files"].tolist() == [0b1001_0100]


def test_a_group_can_be_asked_for_on_its_own() -> None:
    arrays = tensors.facts([START], groups=["pawns", "king"])
    assert {name.split(".")[0] for name in arrays} == {"pawns", "king"}
    with pytest.raises(ValueError, match="not a facts group: knights"):
        tensors.facts([START], groups=["knights"])
    with pytest.raises(ValueError, match="expanded"):
        tensors.facts([START], layout="sparse")  # type: ignore[arg-type]


def test_a_position_a_game_or_its_facts_all_answer() -> None:
    position = esca.Position.from_fen(START)
    game = esca.Game.from_fen(START)
    arrays = tensors.facts([START, position, game, position.facts()])
    assert arrays["material.value"].shape == (4, 2)
    assert arrays["material.value"][0].tolist() == arrays["material.value"][3].tolist()
    with pytest.raises(TypeError, match="not a position, game or facts"):
        tensors.facts([17])  # type: ignore[list-item]


def test_a_batch_survives_a_round_trip_through_safetensors(tmp_path: Path) -> None:
    arrays = tensors.facts([START, PASSER])
    path = tmp_path / "batch.safetensors"
    tensors.save(arrays, path, metadata={"count": "2", "form": "expanded"})
    revived = tensors.load(path)
    assert tensors.metadata(path) == {"count": "2", "form": "expanded"}
    assert set(revived) == set(arrays)
    for name, array in arrays.items():
        assert np.array_equal(revived[name], array), name


@pytest.mark.parametrize("form", ["expanded", "packed"], ids=["expanded", "packed"])
def test_the_golden_batch_the_crate_wrote_is_what_python_computes(form: str) -> None:
    name = "batch.safetensors" if form == "expanded" else "batch.packed.safetensors"
    written = tensors.load(GOLDEN / name)
    computed = tensors.facts(GOLDEN_FENS, layout=form)  # type: ignore[arg-type]
    assert tensors.metadata(GOLDEN / name)["form"] == form
    assert set(written) == set(computed)
    for key, array in computed.items():
        assert np.array_equal(written[key], array), key
