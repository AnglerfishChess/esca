"""The named facts of a position, group by group.

Every value comes from esca already labelled by colour, so a group is rendered
by walking it: `{field: value}`, with a per-side value written
`{"white": …, "black": …}`. The catalogue says what each field means.
"""

from typing import Any

import esca

from chess_esca_mcp.rendering import prose

#: Every group `docs/features.md` defines, in the order `Facts` declares them.
GROUPS: tuple[str, ...] = tuple(esca.Facts.groups())

#: What `groups` answers when a caller names none: `placement` restates the
#: board the FEN already gives, and `maps` restates the attacks and threats.
DEFAULT_GROUPS: tuple[str, ...] = tuple(name for name in GROUPS if name not in ("placement", "maps"))

_FILES = tuple("abcdefgh")
_RANKS = tuple("12345678")

#: What the positions of a list mean, where they mean something of their own.
LABELS: dict[str, tuple[str, ...]] = {
    "pawns.count_by_file": _FILES,
    "pawns.count_by_rank": _RANKS,
}

#: The `king` fields written one value per shield file, which the side's own
#: `shield_files` names.
SHIELD_FIELDS = ("shield", "storm", "shield_file_open", "shield_file_semi_open_for_enemy")


def _label(value: Any, key: str) -> Any:
    """A list whose positions carry a meaning of their own, as an object."""
    labels = LABELS.get(key)
    if labels is None or not isinstance(value, list) or len(value) != len(labels):
        return value
    return dict(zip(labels, value, strict=True))


def _label_by_colour(value: Any, key: str) -> Any:
    """The same, inside a per-side value."""
    if not isinstance(value, dict) or set(value) != {"white", "black"}:
        return _label(value, key)
    return {side: _label(item, key) for side, item in value.items()}


def _relabel_shield(group: dict[str, Any], shield_files: dict[str, list[str]]) -> None:
    """Names the king's three shield values by the files they stand on."""
    for name in SHIELD_FIELDS:
        paired = group.get(name)
        if not isinstance(paired, dict):
            continue
        for side, files in shield_files.items():
            values = paired.get(side)
            if isinstance(values, list) and len(values) == len(files):
                paired[side] = dict(zip(files, values, strict=True))


def group_facts(facts: esca.Facts, name: str) -> dict[str, Any]:
    """Every fact of one group, by name, as plain data."""
    rendered = {
        field: _label_by_colour(value, f"{name}.{field}") for field, value in facts.group(name).to_dict().items()
    }
    if name == "king":
        files = rendered.get("shield_files")
        if isinstance(files, dict):
            _relabel_shield(rendered, {side: list(value) for side, value in files.items()})
    return rendered


def unknown_groups(names: list[str]) -> list[str]:
    """The names among `names` that no group answers to."""
    return [name for name in names if name not in GROUPS]


def facts_content(facts: esca.Facts, names: list[str] | None) -> dict[str, Any]:
    """The named groups of `facts`, and what else could have been asked for."""
    wanted = list(DEFAULT_GROUPS) if not names else [name for name in GROUPS if name in set(names)]
    return {
        "side_to_move": facts.side_to_move,
        "groups": {name: group_facts(facts, name) for name in wanted},
        "groups_available": list(GROUPS),
        "prose": prose(facts),
    }


def catalogue() -> dict[str, Any]:
    """Every group, field, type and meaning, as the resource serves it."""
    return {
        "groups": [
            {
                "name": group["name"],
                "meaning": group["meaning"],
                "fields": [
                    {"name": field["name"], "type": field["type"], "meaning": field["meaning"]}
                    for field in group["fields"]
                ],
            }
            for group in esca.facts_catalogue()
        ],
        "move": [
            {"name": field["name"], "type": field["type"], "meaning": field["meaning"]}
            for field in esca.move_facts_catalogue()
        ],
        "types": {
            "ByColour": 'one value per colour, written {"white": …, "black": …}',
            "SquareSet": "a set of squares, written as a list of square names",
            "FileSet": "a set of files, written as a list of file letters",
            "Option": "a value that may be absent, written null when it is",
        },
    }
