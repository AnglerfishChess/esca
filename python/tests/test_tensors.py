"""The typed tensor form: one array per fact, batch first, nothing scaled."""

from __future__ import annotations

import json
from collections.abc import Callable, Mapping, Sequence
from pathlib import Path
from typing import Any

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

#: White may take on f6 en passant.
EN_PASSANT = "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3"
#: Both sides have a pawn one step from promotion.
PROMOTIONS = "8/1P4k1/8/8/8/8/6p1/K7 w - - 0 1"
#: Black to move with all four castlings open.
CASTLINGS = "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1"
#: Bishop and knight against a bare king.
BISHOP_AND_KNIGHT = "8/8/8/4k3/8/8/8/4KBN1 w - - 0 1"
#: One bishop each, on opposite square colours.
OPPOSITE_BISHOPS = "8/5k2/8/3b4/8/8/2B5/4K3 w - - 0 1"
#: White is mated.
FOOLS_MATE = "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3"
#: Kings in direct opposition.
OPPOSITION = "8/8/4k3/8/4K3/8/8/8 w - - 0 1"
#: A Chess960 start, where the back rank is not the classic one.
CHESS960_START = "bqnbnrkr/pppppppp/8/8/8/8/PPPPPPPP/BQNBNRKR w HFhf - 0 1"

#: The classic positions the agreement tests read.
VARIED = [
    START,
    PASSER,
    KIWIPETE,
    EN_PASSANT,
    PROMOTIONS,
    CASTLINGS,
    BISHOP_AND_KNIGHT,
    OPPOSITE_BISHOPS,
    FOOLS_MATE,
    OPPOSITION,
]

#: The classic positions the golden batch was written for, in that order.
GOLDEN_FENS = [
    entry["fen"] for entry in json.loads((GOLDEN / "positions.json").read_text()) if entry["variant"] == "chess"
]


def moves_of(fens: list[str]) -> list[esca.AnnotatedMove]:
    return [annotated for fen in fens for annotated in esca.Position.from_fen(fen).annotated_moves()]


def test_every_fact_of_every_group_has_a_layout_entry() -> None:
    positions = [f"{group['name']}.{field['name']}" for group in esca.facts_catalogue() for field in group["fields"]]
    moves = [f"move.{field['name']}" for field in esca.move_facts_catalogue()]
    assert tensors.names() == positions
    assert tensors.names(["move"]) == moves
    # The position groups come first, then the move group.
    assert [entry["name"] for entry in tensors.layout()] == positions + moves


def test_every_array_holds_what_the_layout_says() -> None:
    arrays = tensors.facts([START, PASSER, KIWIPETE])
    assert list(arrays) == tensors.names()
    for entry in tensors.layout()[: len(arrays)]:
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
    with pytest.raises(ValueError, match=r"tensors\.moves\(\)"):
        tensors.facts([START], groups=["move"])
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


def test_an_array_is_writable() -> None:
    arrays = tensors.facts([START])
    # 8 pawns, 4 minors at 3, 2 rooks at 5 and a queen at 9 each: 39.
    arrays["material.value"][0, 0] = 0
    assert arrays["material.value"].tolist() == [[0, 39]]


def test_no_positions_give_empty_arrays() -> None:
    arrays = tensors.facts([])
    assert arrays["pawns.passed"].shape == (0, 2, 64)
    packed = tensors.facts([], layout="packed")
    assert packed["pawns.passed"].shape == (0, 2)
    assert packed["material.pawns_only"].shape == (0,)


def test_every_move_array_holds_what_the_layout_says() -> None:
    moves = moves_of([START, KIWIPETE])
    # 20 legal moves at the start, 48 in Kiwipete.
    assert len(moves) == 68
    arrays = tensors.moves(moves)
    assert list(arrays) == tensors.names(["move"])
    for entry in tensors.layout()[len(tensors.names()) :]:
        array = arrays[entry["name"]]
        assert array.dtype == np.dtype(entry["dtype"]), entry["name"]
        assert array.shape == (68,), entry["name"]


@pytest.mark.parametrize(
    ("name", "dtype"),
    [
        ("move.gives_check", "bool"),
        ("move.see", "int32"),
        ("move.mover", "uint8"),
        ("move.victim", "int16"),
    ],
    ids=["a_flag", "a_value", "a_named_value", "an_absent_value"],
)
def test_a_move_fact_keeps_the_type_it_was_declared_with(name: str, dtype: str) -> None:
    [entry] = [item for item in tensors.layout() if item["name"] == name]
    assert entry["dtype"] == dtype
    assert tuple(entry["shape"]) == ()


def test_a_move_batch_writes_each_move_in_turn(move_facts: Callable[..., esca.MoveFacts]) -> None:
    # Black to move, the g2 pawn one step from promotion.
    promotions = "8/1P4k1/8/8/8/8/6p1/K7 b - - 0 1"
    arrays = tensors.moves([move_facts(promotions, uci) for uci in ["g2g1q", "g2g1n", "g7f6"]])
    # p = 0, k = 5.
    assert arrays["move.mover"].tolist() == [0, 0, 5]
    # q = 4, n = 1, and the king move promotes nothing.
    assert arrays["move.promotion"].tolist() == [4, 1, -1]
    assert arrays["move.victim"].tolist() == [-1, -1, -1]


def test_an_annotated_move_or_its_facts_both_answer() -> None:
    moves = moves_of([KIWIPETE])
    by_move = tensors.moves(moves)
    by_facts = tensors.moves([annotated.facts for annotated in moves])
    for name, array in by_move.items():
        assert np.array_equal(by_facts[name], array), name
    with pytest.raises(TypeError, match="not an annotated move or move facts"):
        tensors.moves([START])  # type: ignore[list-item]


def test_the_packed_move_form_holds_the_same_facts() -> None:
    moves = moves_of([START, KIWIPETE, PASSER])
    expanded = tensors.moves(moves)
    revived = tensors.expand(tensors.moves(moves, layout="packed"), len(moves))
    for name, array in expanded.items():
        assert np.array_equal(revived[name], array), name


@pytest.mark.parametrize("form", ["expanded", "packed"], ids=["expanded", "packed"])
def test_the_golden_move_batch_the_crate_wrote_is_what_python_computes(form: str) -> None:
    name = "moves.safetensors" if form == "expanded" else "moves.packed.safetensors"
    written = tensors.load(GOLDEN / name)
    computed = tensors.moves(moves_of(GOLDEN_FENS), layout=form)  # type: ignore[arg-type]
    assert list(written) == list(computed)
    for key, array in computed.items():
        assert np.array_equal(written[key], array), key


def walked(value: Any, kind: Mapping[str, Any], out: list[Any]) -> None:
    """One value of the JSON form, as the values its tensor holds: the reading
    of `docs/features.md` the native filler has to agree with."""
    what = kind["kind"]
    if what == "by_colour":
        walked(value["white"], kind["item"], out)
        walked(value["black"], kind["item"], out)
    elif what == "fixed":
        for item in value:
            walked(item, kind["item"], out)
    elif what == "square_set":
        out.extend(square in value for square in tensors.SQUARES)
    elif what == "file_set":
        out.extend(file in value for file in tensors.FILES)
    elif what == "optional" and value is None:
        out.append(-1)
    elif what == "optional" and kind["item"]["kind"] == "named":
        out.append(kind["item"]["codes"].index(value))
    elif what == "named":
        out.append(kind["codes"].index(value))
    else:
        out.append(value)


def walked_arrays(items: Sequence[Mapping[str, Any]], entries: list[dict[str, Any]]) -> dict[str, np.ndarray]:
    """The arrays the JSON form of `items` gives, one per entry."""
    arrays = {}
    for entry in entries:
        values: list[Any] = []
        for item in items:
            walked(item[entry["group"]][entry["field"]], entry["kind"], values)
        arrays[entry["name"]] = np.array(values, dtype=entry["dtype"]).reshape((len(items), *entry["shape"]))
    return arrays


def assert_byte_identical(native: Mapping[str, np.ndarray], walked: Mapping[str, np.ndarray]) -> None:
    assert list(native) == list(walked)
    for name, array in walked.items():
        assert native[name].dtype == array.dtype, name
        assert native[name].shape == array.shape, name
        assert native[name].tobytes() == array.tobytes(), name


@pytest.mark.parametrize("form", ["expanded", "packed"], ids=["expanded", "packed"])
@pytest.mark.parametrize(
    ("variant", "fens"),
    [(esca.CLASSIC, VARIED), (esca.CHESS960, [CHESS960_START, START])],
    ids=["classic", "chess960"],
)
def test_every_position_array_agrees_with_the_json_form(variant: esca.Variant, fens: list[str], form: str) -> None:
    entries = [entry for entry in tensors.layout() if entry["group"] != "move"]
    items = [esca.Position.from_fen(fen).facts(variant=variant).to_dict() for fen in fens]
    walked = walked_arrays(items, entries)
    if form == "packed":
        walked = tensors.pack(walked)
    assert_byte_identical(tensors.facts(fens, variant=variant, layout=form), walked)  # type: ignore[arg-type]


@pytest.mark.parametrize("form", ["expanded", "packed"], ids=["expanded", "packed"])
def test_every_move_array_agrees_with_the_json_form(form: str) -> None:
    entries = [entry for entry in tensors.layout() if entry["group"] == "move"]
    moves = moves_of(VARIED)
    walked = walked_arrays([{"move": annotated.facts.to_dict()} for annotated in moves], entries)
    if form == "packed":
        walked = tensors.pack(walked)
    assert_byte_identical(tensors.moves(moves, layout=form), walked)  # type: ignore[arg-type]
