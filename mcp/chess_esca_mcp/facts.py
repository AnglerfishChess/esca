"""The named facts of a position, group by group.

esca computes facts in the mover's view and pairs every side-relative value
`(us, them)`; here each pair is labelled `white` and `black` instead, so that
no answer needs a second lookup to be read. A value that is one signed number
is still from the side to move's view, which the `perspective` field says.
"""

from typing import Any

import esca

from chess_esca_mcp.rendering import ROLE_ORDER, prose

#: Every group `docs/features.md` defines, in schema order.
GROUPS: tuple[str, ...] = tuple(esca.SCHEMA.group_names)

#: What `groups` answers when a caller names none: `placement` restates the
#: board the FEN already gives, and `planes` restates the attack maps.
DEFAULT_GROUPS: tuple[str, ...] = tuple(name for name in GROUPS if name not in ("placement", "planes"))

#: The groups whose value is one block per side rather than paired fields.
PAIRED_GROUPS = frozenset({"exchange", "tactics"})

_FILES = tuple("abcdefgh")
_RANKS = tuple("12345678")

#: What the positions of a list or of an inner pair mean, where they mean
#: something other than a side.
LABELS: dict[str, tuple[str, ...]] = {
    "placement.by_role": ROLE_ORDER,
    "attacks.by_role": ROLE_ORDER,
    "material.count": ROLE_ORDER[:5],
    "mobility.by_role": ROLE_ORDER[:5],
    "mobility.safe_by_role": ROLE_ORDER[:5],
    "tactics.check_by_role": ROLE_ORDER[:5],
    "tactics.safe_check_by_role": ROLE_ORDER[:5],
    "tactics.promotion_roles": ("queen", "rook", "bishop", "knight"),
    "pawns.count_by_file": _FILES,
    "pawns.count_by_rank": _RANKS,
    "pawns.majority_by_wing": ("queen_side", "king_side"),
    "pawns.passer_king_distance": ("own_king", "enemy_king"),
}

#: The `king` fields written one value per shield file, which the side's own
#: `shield_files` names.
SHIELD_FIELDS = ("shield", "storm", "file_open", "file_semi_open_for_enemy")


def fields(block: object) -> list[str]:
    """The named facts `block` holds, in alphabetical order."""
    return sorted(
        name for name in dir(type(block)) if not name.startswith("_") and not callable(getattr(block, name, None))
    )


def _render(value: Any, key: str, white: int, depth: int) -> Any:
    """One fact's value as JSON.

    At the top of a field a pair is the two sides; below it, a list or a pair
    is labelled by `LABELS` where its positions carry a meaning of their own.
    """
    if isinstance(value, esca.SquareSet):
        return sorted(value)
    if depth == 0 and isinstance(value, tuple) and len(value) == 2:
        return {
            "white": _render(value[white], key, white, 1),
            "black": _render(value[1 - white], key, white, 1),
        }
    if isinstance(value, tuple | list):
        items = [_render(item, key, white, depth + 1) for item in value]
        labels = LABELS.get(key)
        if labels is not None and len(labels) == len(items):
            return dict(zip(labels, items, strict=True))
        return items
    return value


def _relabel_shield(group: dict[str, Any], shield_files: dict[str, str]) -> None:
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
    """Every fact of one group, labelled by name."""
    white = facts.side("w")
    block = getattr(facts, name)
    if name in PAIRED_GROUPS:
        return {
            side: {
                field: _render(getattr(block[index], field), f"{name}.{field}", white, 0)
                for field in fields(block[index])
            }
            for side, index in (("white", white), ("black", 1 - white))
        }
    rendered = {field: _render(getattr(block, field), f"{name}.{field}", white, 0) for field in fields(block)}
    if name == "king":
        files = rendered.get("shield_files")
        if isinstance(files, dict):
            _relabel_shield(rendered, {side: str(value) for side, value in files.items()})
    return rendered


def unknown_groups(names: list[str]) -> list[str]:
    """The names among `names` that no group answers to."""
    return [name for name in names if name not in GROUPS]


def facts_content(facts: esca.Facts, names: list[str] | None) -> dict[str, Any]:
    """The named groups of `facts`, and what else could have been asked for."""
    wanted = list(DEFAULT_GROUPS) if not names else [name for name in GROUPS if name in set(names)]
    return {
        "schema_id": esca.SCHEMA_ID,
        "schema_version": esca.SCHEMA.semver,
        "perspective": {
            "us": "white" if facts.side_to_move == "w" else "black",
            "them": "black" if facts.side_to_move == "w" else "white",
            "note": (
                "docs/features.md names the two sides us and them; a lone signed number is from the side to move's view"
            ),
        },
        "groups": {name: group_facts(facts, name) for name in wanted},
        "groups_returned": wanted,
        "groups_available": list(GROUPS),
        "prose": prose(facts),
    }
