"""The one JSON form: what `to_dict()` answers, against the golden files.

`tests/data/facts/<name>.json` is written by the crate, checked by the Rust
suite, and compared here character for character: the two languages answer in
one format or one of them is wrong.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, cast

import esca
import jsonschema
import pytest

ROOT = Path(__file__).resolve().parents[2]
GOLDEN = ROOT / "tests" / "data" / "facts"

#: The positions the golden files were written for.
NAMED: list[dict[str, str]] = json.loads((GOLDEN / "positions.json").read_text())

#: The JSON form as a JSON Schema, generated from the same catalogue.
SCHEMA: dict[str, Any] = json.loads((ROOT / "docs" / "facts.schema.json").read_text())

IDS = [entry["name"] for entry in NAMED]


def facts_of(entry: dict[str, str]) -> esca.Facts:
    """The facts of one named position, under the variant it names."""
    variant = esca.CHESS960 if entry["variant"] == "chess960" else esca.CLASSIC
    return esca.Position.from_fen(entry["fen"]).facts(variant=variant)


@pytest.mark.parametrize("entry", NAMED, ids=IDS)
def test_to_dict_writes_the_golden_file_the_crate_wrote(entry: dict[str, str]) -> None:
    written = json.dumps(facts_of(entry).to_dict(), indent=2, sort_keys=False) + "\n"
    assert written == (GOLDEN / f"{entry['name']}.json").read_text()


@pytest.mark.parametrize("entry", NAMED, ids=IDS)
def test_the_golden_files_answer_the_schema(entry: dict[str, str]) -> None:
    jsonschema.validate(json.loads((GOLDEN / f"{entry['name']}.json").read_text()), SCHEMA)


@pytest.mark.parametrize("entry", NAMED, ids=IDS)
def test_what_to_dict_answers_answers_the_schema(entry: dict[str, str]) -> None:
    jsonschema.validate(facts_of(entry).to_dict(), SCHEMA)


def test_a_group_writes_the_same_data_as_the_whole() -> None:
    facts = facts_of(NAMED[0])
    whole: dict[str, Any] = dict(facts.to_dict())
    assert list(whole) == facts.groups()
    for name in facts.groups():
        assert facts.group(name).to_dict() == whole[name]


@pytest.mark.parametrize(
    ("group", "field", "expected"),
    [
        ("pawns", "passed", {"white": ["d5"], "black": []}),
        ("pawns", "count_by_file", {"white": [1, 1, 0, 1, 0, 1, 1, 0], "black": [1, 1, 0, 0, 0, 1, 1, 0]}),
        ("king", "shield_files", {"white": ["f", "g", "h"], "black": ["f", "g", "h"]}),
        ("king", "shield", {"white": [None, None, None], "black": [None, None, None]}),
    ],
    ids=["a_square_set", "a_run_per_file", "a_run_of_three", "an_absent_value"],
)
def test_every_shape_the_form_carries(group: str, field: str, expected: object) -> None:
    facts = esca.Position.from_fen("8/pp3ppk/8/3P4/8/8/PP3PPK/8 w - - 0 1").facts()
    written = cast("dict[str, object]", facts.group(group).to_dict())
    assert written[field] == expected


def test_an_open_file_set_is_the_list_of_its_letters() -> None:
    facts = esca.Position.from_fen("8/pp3ppk/8/3P4/8/8/PP3PPK/8 w - - 0 1").facts()
    assert facts.pawns.to_dict()["open_files"] == ["c", "e", "h"]


def test_the_schema_closes_every_group_it_names() -> None:
    assert list(SCHEMA["properties"]) == sorted(esca.Facts.groups())
    assert SCHEMA["required"] == esca.Facts.groups()
    for group in esca.facts_catalogue():
        node = SCHEMA["properties"][group["name"]]
        assert node["additionalProperties"] is False
        assert node["required"] == [field["name"] for field in group["fields"]]
        for field in group["fields"]:
            assert node["properties"][field["name"]]["description"] == field["meaning"]
