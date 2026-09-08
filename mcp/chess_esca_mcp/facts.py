"""The named facts of a position, group by group.

A group is served in the one JSON form esca writes everywhere — `{field:
value}`, a per-side value `{"white": …, "black": …}`, a set of squares the list
of its square names in board order, a set of files the list of its letters —
so what this tool answers and what `docs/facts.schema.json` describes are the
same thing.

Where a value is a run of positions, `field_notes` carries that field's
catalogue meaning once, since the meaning is what says which end is which.
"""

from typing import Any, cast

import esca

from chess_esca_mcp.rendering import prose

#: Every group `docs/features.md` defines, in the order `Facts` declares them.
GROUPS: tuple[str, ...] = tuple(esca.Facts.groups())

#: What `groups` answers when a caller names none: `placement` restates the
#: board the FEN already gives, and `maps` restates the attacks and threats.
DEFAULT_GROUPS: tuple[str, ...] = tuple(name for name in GROUPS if name not in ("placement", "maps"))


def _is_run(declared: str) -> bool:
    """Whether the declared type is a run of positions rather than one value."""
    return "PerFile<" in declared or "PerRank<" in declared or "[" in declared


#: The catalogue meaning of every fact whose value is a run of positions,
#: keyed `group.field`. A reader of `[1, 1, 1, 1, 1, 1, 1, 1]` needs it.
RUN_NOTES: dict[str, str] = {
    f"{group['name']}.{field['name']}": field["meaning"]
    for group in esca.facts_catalogue()
    for field in group["fields"]
    if _is_run(field["type"])
}


def group_facts(facts: esca.Facts, name: str) -> dict[str, Any]:
    """Every fact of one group, by name, as plain data."""
    return cast("dict[str, Any]", facts.group(name).to_dict())


def unknown_groups(names: list[str]) -> list[str]:
    """The names among `names` that no group answers to."""
    return [name for name in names if name not in GROUPS]


def run_notes(names: list[str]) -> dict[str, str]:
    """What the positions of every run the named groups carry mean."""
    wanted = set(names)
    return {key: note for key, note in RUN_NOTES.items() if key.split(".")[0] in wanted}


def facts_content(facts: esca.Facts, names: list[str] | None) -> dict[str, Any]:
    """The named groups of `facts`, and what else could have been asked for."""
    wanted = list(DEFAULT_GROUPS) if not names else [name for name in GROUPS if name in set(names)]
    return {
        "side_to_move": facts.side_to_move,
        "groups": {name: group_facts(facts, name) for name in wanted},
        "field_notes": run_notes(wanted),
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
            "SquareSet": (
                "a set of squares, written as a list of square names in board order: a1, b1 through h1, then a2"
            ),
            "FileSet": "a set of files, written as a list of file letters, file a first",
            "PerFile": "eight values, file a first, through file h",
            "PerRank": "eight values, rank 1 first, through rank 8",
            "[T; n]": "n values, in the order the fact's own meaning states",
            "Option": "a value that may be absent, written null when it is",
        },
    }
