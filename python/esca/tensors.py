"""The facts of many positions as one typed array per fact.

Every fact keeps the type it was declared with: a count is an integer of its
own width, a flag is a `bool`, a set of squares is 64 flags. Nothing is scaled,
normalised or cast to a float, and nothing is written in the mover's view — the
leading axis of a per-colour fact is White then Black, always.

`layout()` says what each array is called, what it holds and how to read its
axes; `facts()` is the arrays themselves, batch axis first. `save()` and
`load()` write and read the safetensors format — a JSON header naming each
array, then the raw bytes — which NumPy and PyTorch both read.

NumPy is not part of the base install: `pip install esca[tensors]`.
"""

from __future__ import annotations

import json
import os
from collections.abc import Iterable, Mapping, Sequence
from typing import Any, Literal

from ._esca import Facts, Game, Position, Variant, facts_layout

try:
    import numpy as np
except ModuleNotFoundError as missing:  # pragma: no cover - exercised by the extra
    raise ModuleNotFoundError(
        "esca.tensors needs NumPy, which the base install leaves out: pip install esca[tensors]"
    ) from missing

__all__ = [
    "SAFETENSORS_DTYPES",
    "expand",
    "facts",
    "layout",
    "load",
    "metadata",
    "pack",
    "save",
]

#: The element type each layout `dtype` names, as NumPy spells it.
_DTYPES: dict[str, Any] = {
    "bool": np.bool_,
    "uint8": np.uint8,
    "int8": np.int8,
    "uint16": np.uint16,
    "int16": np.int16,
    "uint32": np.uint32,
    "int32": np.int32,
    "uint64": np.uint64,
    "float32": np.float32,
}

#: The name the safetensors header spells each element type with.
SAFETENSORS_DTYPES: dict[str, str] = {
    "bool": "BOOL",
    "uint8": "U8",
    "int8": "I8",
    "uint16": "U16",
    "int16": "I16",
    "uint32": "U32",
    "int32": "I32",
    "uint64": "U64",
    "float32": "F32",
}

_SAFETENSORS_NAMES = {name: dtype for dtype, name in SAFETENSORS_DTYPES.items()}

#: Every square, a1 first: the order a square-set plane is written in.
SQUARES: tuple[str, ...] = tuple(f"{file}{rank}" for rank in "12345678" for file in "abcdefgh")

#: Every file, a first: the order a file-set plane is written in.
FILES: tuple[str, ...] = tuple("abcdefgh")

_SQUARE_INDEX = {name: index for index, name in enumerate(SQUARES)}
_FILE_INDEX = {name: index for index, name in enumerate(FILES)}

_LAYOUT: list[dict[str, Any]] | None = None


def layout() -> list[dict[str, Any]]:
    """One entry per fact, in the order the groups declare them.

    Each entry carries the array's `name` (`group.field`), its `dtype` and
    `shape` with the batch axis left off, an `order` note naming every axis,
    the `packed_dtype` and `packed_shape` of the packed form, the fact's
    `meaning`, and the `kind` tree of the type it was declared with.
    """
    global _LAYOUT
    if _LAYOUT is None:
        _LAYOUT = facts_layout()
    return _LAYOUT


def _entries(groups: Sequence[str] | None) -> list[dict[str, Any]]:
    """The layout entries of the named groups, or of every group."""
    if groups is None:
        return layout()
    wanted = set(groups)
    named = {entry["group"] for entry in layout()}
    unknown = sorted(wanted - named)
    if unknown:
        raise ValueError(f"not a facts group: {', '.join(unknown)}")
    return [entry for entry in layout() if entry["group"] in wanted]


def _facts_of(item: str | Position | Game | Facts, variant: Variant | None) -> Facts:
    """The facts of one position, however it was named."""
    if isinstance(item, Facts):
        return item
    if isinstance(item, str):
        item = Position.from_fen(item)
    if isinstance(item, Position):
        return item.facts(variant) if variant is not None else item.facts()
    if isinstance(item, Game):
        return item.facts()
    raise TypeError(f"not a position, game or facts: {item!r}")


def _push(value: Any, kind: Mapping[str, Any], out: list[Any]) -> None:
    """One value of the JSON form appended to `out`, following its type."""
    what = kind["kind"]
    if what == "by_colour":
        _push(value["white"], kind["item"], out)
        _push(value["black"], kind["item"], out)
    elif what == "fixed":
        for item in value:
            _push(item, kind["item"], out)
    elif what == "square_set":
        held = [False] * 64
        for square in value:
            held[_SQUARE_INDEX[square]] = True
        out.extend(held)
    elif what == "file_set":
        held = [False] * 8
        for file in value:
            held[_FILE_INDEX[file]] = True
        out.extend(held)
    elif what == "optional":
        if value is None:
            out.append(-1)
        elif kind["item"]["kind"] == "named":
            out.append(kind["item"]["codes"].index(value))
        else:
            out.append(value)
    elif what == "named":
        out.append(kind["codes"].index(value))
    else:
        out.append(value)


def facts(
    positions: Iterable[str | Position | Game | Facts],
    *,
    groups: Sequence[str] | None = None,
    layout: Literal["expanded", "packed"] = "expanded",
    variant: Variant | None = None,
) -> dict[str, np.ndarray]:
    """One array per fact over `positions`, batch axis first.

    `positions` holds FEN strings, `Position`s, `Game`s or `Facts`, in any
    mixture; a `Game` brings the history facts its plies give it. `groups`
    narrows the answer to the named groups, `variant` names the rules a FEN or
    a position is read under, and `layout` chooses the form: `"expanded"` is
    one value per element, `"packed"` the same facts as bits.
    """
    if layout not in ("expanded", "packed"):
        raise ValueError(f'a layout is "expanded" or "packed", not {layout!r}')
    entries = _entries(groups)
    columns: dict[str, list[Any]] = {entry["name"]: [] for entry in entries}
    count = 0
    for item in positions:
        written: dict[str, Any] = dict(_facts_of(item, variant).to_dict())
        for entry in entries:
            _push(written[entry["group"]][entry["field"]], entry["kind"], columns[entry["name"]])
        count += 1

    arrays = {
        entry["name"]: np.array(columns[entry["name"]], dtype=_DTYPES[entry["dtype"]]).reshape((count, *entry["shape"]))
        for entry in entries
    }
    return pack(arrays) if layout == "packed" else arrays


def _packing(entry: Mapping[str, Any]) -> str:
    """How the fact is packed: as a bitboard, a file mask, bits, or not at all."""
    kind = entry["kind"]
    while kind["kind"] in ("by_colour", "fixed"):
        kind = kind["item"]
    if kind["kind"] == "square_set":
        return "bitboard"
    if kind["kind"] == "file_set":
        return "file_mask"
    return "bits" if kind["kind"] == "bool" else "plain"


def pack(arrays: Mapping[str, np.ndarray]) -> dict[str, np.ndarray]:
    """The same facts as bits.

    A square set becomes one `uint64` bitboard with a1 as bit 0, a file set one
    `uint8` mask with file a as bit 0, and every other flag one bit, eight to a
    byte over the whole array flattened in C order.
    """
    known = {entry["name"]: entry for entry in layout()}
    out: dict[str, np.ndarray] = {}
    for name, array in arrays.items():
        packing = _packing(known[name])
        if packing == "bitboard":
            bits = np.packbits(array, axis=-1, bitorder="little")
            out[name] = np.ascontiguousarray(bits).view(np.uint64).reshape(array.shape[:-1])
        elif packing == "file_mask":
            out[name] = np.packbits(array, axis=-1, bitorder="little").reshape(array.shape[:-1])
        elif packing == "bits":
            out[name] = np.packbits(np.ascontiguousarray(array).reshape(-1))
        else:
            out[name] = array
    return out


def expand(arrays: Mapping[str, np.ndarray], count: int) -> dict[str, np.ndarray]:
    """The packed arrays of `count` positions back to one value per element."""
    known = {entry["name"]: entry for entry in layout()}
    out: dict[str, np.ndarray] = {}
    for name, array in arrays.items():
        entry = known[name]
        shape = (count, *entry["shape"])
        packing = _packing(entry)
        if packing == "bitboard":
            bytes_ = np.ascontiguousarray(array).view(np.uint8).reshape((*array.shape, 8))
            out[name] = np.unpackbits(bytes_, axis=-1, bitorder="little").astype(bool).reshape(shape)
        elif packing == "file_mask":
            bytes_ = np.ascontiguousarray(array).reshape((*array.shape, 1))
            out[name] = np.unpackbits(bytes_, axis=-1, bitorder="little").astype(bool).reshape(shape)
        elif packing == "bits":
            wanted = int(np.prod(shape)) if shape else 0
            out[name] = np.unpackbits(array)[:wanted].astype(bool).reshape(shape)
        else:
            out[name] = array
    return out


def save(
    arrays: Mapping[str, np.ndarray],
    path: str | os.PathLike[str],
    *,
    metadata: Mapping[str, str] | None = None,
) -> None:
    """The arrays written to `path` in the safetensors format."""
    header: dict[str, Any] = {}
    if metadata:
        header["__metadata__"] = dict(metadata)
    body = bytearray()
    for name, array in arrays.items():
        contiguous = np.ascontiguousarray(array)
        start = len(body)
        body.extend(contiguous.tobytes())
        header[name] = {
            "dtype": SAFETENSORS_DTYPES[str(contiguous.dtype)],
            "shape": list(contiguous.shape),
            "data_offsets": [start, len(body)],
        }
    text = json.dumps(header, separators=(",", ":")).encode()
    text += b" " * (-len(text) % 8)
    with open(path, "wb") as out:
        out.write(len(text).to_bytes(8, "little"))
        out.write(text)
        out.write(body)


def load(path: str | os.PathLike[str]) -> dict[str, np.ndarray]:
    """The arrays a safetensors file holds, its metadata left out."""
    with open(path, "rb") as source:
        raw = source.read()
    length = int.from_bytes(raw[:8], "little")
    header = json.loads(raw[8 : 8 + length])
    body = raw[8 + length :]
    out: dict[str, np.ndarray] = {}
    for name, entry in header.items():
        if name == "__metadata__":
            continue
        start, end = entry["data_offsets"]
        dtype = _DTYPES[_SAFETENSORS_NAMES[entry["dtype"]]]
        out[name] = np.frombuffer(body[start:end], dtype=dtype).reshape(entry["shape"])
    return out


def metadata(path: str | os.PathLike[str]) -> dict[str, str]:
    """The `__metadata__` entry of a safetensors file, empty where it has none."""
    with open(path, "rb") as source:
        length = int.from_bytes(source.read(8), "little")
        header = json.loads(source.read(length))
    return header.get("__metadata__", {})
