"""The JSON shapes every tool answers in.

Squares are text, square sets are sorted lists of squares, colours and roles
are their English names, and a categorical answer is the enum name esca gives
it. A reason never travels alone: each one carries the squares it was read
off, and every reason that applies is present, not the first of them.
"""

from collections.abc import Iterable
from typing import Any

import esca

#: A colour letter as the answers spell it.
COLOURS = {"w": "white", "b": "black"}

#: A role letter as the answers spell it.
ROLES = {"p": "pawn", "n": "knight", "b": "bishop", "r": "rook", "q": "queen", "k": "king"}

#: The role order of a side's six placement and attack planes.
ROLE_ORDER = ("pawn", "knight", "bishop", "rook", "queen", "king")


def colour(letter: str) -> str:
    """The name of the colour `letter` spells."""
    return COLOURS[letter.lower()]


def other(name: str) -> str:
    """The colour facing `name`."""
    return "black" if name == "white" else "white"


def role(letter: str) -> str:
    """The name of the role `letter` spells."""
    return ROLES[letter.lower()]


def squares(square_set: esca.SquareSet) -> list[str]:
    """The squares of `square_set`, in board order."""
    return sorted(square_set)


def unit(position: esca.Position, square: str) -> dict[str, str] | None:
    """The unit standing on `square`, or `None` when it is empty."""
    letter = position.piece_at(square)
    if letter is None:
        return None
    return {"square": square, "colour": colour("w" if letter.isupper() else "b"), "role": role(letter)}


def units(position: esca.Position, square_set: esca.SquareSet) -> list[dict[str, str]]:
    """Every unit of `square_set`, in board order."""
    return [u for square in squares(square_set) if (u := unit(position, square)) is not None]


def prose(subject: object) -> list[str]:
    """The sentences esca tells about `subject`.

    Empty while esca has no `describe()`; the field is present in every answer
    so that the shape does not change when it grows sentences.
    """
    describe = getattr(subject, "describe", None)
    if not callable(describe):
        return []
    said = describe()
    if isinstance(said, str):
        return [said]
    return [str(line) for line in said] if isinstance(said, Iterable) else []


# --- the explanations -----------------------------------------------------


def castling_wing(position: esca.Position, side: str, wing: str) -> dict[str, Any]:
    """What `side` needs for `wing` castling and what stands in the way.

    `obstacles` holds every reason the castling is unavailable, each with its
    own evidence; it is empty exactly when `allowed` is true.
    """
    letter = "w" if side == "white" else "b"
    answer = position.castling(letter, wing)
    rank = "1" if side == "white" else "8"
    obstacles: list[dict[str, Any]] = []
    if not answer.right:
        obstacles.append({"reason": "no_castling_right", "castling_rights": position.castling_rights})
    elif not answer.rook_present:
        obstacles.append({"reason": "rook_missing"})
    if answer.king_in_check_by:
        obstacles.append(
            {
                "reason": "king_in_check",
                "king": position.king_of(letter),
                "attackers": units(position, answer.king_in_check_by),
            }
        )
    if answer.path_attacked:
        obstacles.append(
            {
                "reason": "path_attacked",
                "squares": [
                    {"square": square, "attackers": units(position, by)} for square, by in answer.path_attacked
                ],
            }
        )
    if answer.path_blocked:
        obstacles.append({"reason": "path_blocked", "occupants": units(position, answer.path_blocked)})
    return {
        "wing": wing,
        "king_lands_on": ("g" if wing == "short" else "c") + rank,
        "right": answer.right,
        "rook_present": answer.rook_present,
        "allowed": answer.allowed,
        "obstacles": obstacles,
        "prose": prose(answer),
    }


def castling(position: esca.Position) -> dict[str, Any]:
    """Both castlings of both colours, obstacles and all.

    `allowed` ignores whose turn it is, so for the side to move it is exactly
    legality and for the other side it is what would hold if the turn passed.
    """
    return {
        side: {wing: castling_wing(position, side, wing) for wing in ("short", "long")} for side in ("white", "black")
    }


def ep_obstacle(obstacle: esca.explain.EpObstacle) -> dict[str, Any]:
    """Why one en-passant capture is off the board."""
    answer: dict[str, Any] = {"reason": obstacle.kind}
    if obstacle.pinner is not None:
        answer["pinner"] = obstacle.pinner
    if obstacle.attacker is not None:
        answer["attacker"] = obstacle.attacker
    if obstacle.ray:
        answer["ray"] = squares(obstacle.ray)
    if obstacle.by:
        answer["checked_by"] = squares(obstacle.by)
    return answer


def en_passant(position: esca.Position) -> dict[str, Any]:
    """The en-passant capture the position offers the side to move.

    `target` is the skipped square the FEN names; `captures` is every pawn
    standing beside it, legal or not, each with what forbids it.
    """
    answer = position.en_passant_status()
    captures = [
        {
            "origin": capture.origin,
            "legal": capture.legal,
            "forbidden_by": None if capture.forbidden_by is None else ep_obstacle(capture.forbidden_by),
        }
        for capture in answer.captures
    ]
    return {
        "target": answer.target,
        "fen_field": position.en_passant,
        "available": any(capture["legal"] for capture in captures),
        "captures": captures,
        "prose": prose(answer),
    }


def pin(item: esca.explain.Pin) -> dict[str, Any]:
    """One absolute pin, and the line it holds."""
    return {"pinned": item.pinned, "pinner": item.pinner, "king": item.king, "ray": squares(item.ray)}


def skewer(item: esca.explain.Skewer) -> dict[str, Any]:
    """One skewer, the more valuable unit in front."""
    return {
        "attacker": item.attacker,
        "front": item.front,
        "behind": item.behind,
        "ray": squares(item.ray),
    }


def repetition(item: esca.explain.Repetition) -> dict[str, Any]:
    """How often this position has stood, and what nearly counted."""
    return {
        "count": item.count,
        "plies": list(item.plies),
        "near_misses": [{"ply": miss.ply, "differs": list(miss.differs)} for miss in item.near_misses],
    }


def fifty_move(item: esca.explain.FiftyMove) -> dict[str, Any]:
    """The halfmove clock and how far it is from ending the game."""
    return {
        "clock": item.clock,
        "plies_to_claim": item.plies_to_claim,
        "plies_to_automatic": item.plies_to_automatic,
        "last_reset": (None if item.last_reset is None else {"ply": item.last_reset.ply, "kind": item.last_reset.kind}),
    }


def stalemate(item: esca.explain.StalemateDetail) -> dict[str, Any]:
    """Why the side to move has no move at all."""
    return {
        "king": item.king,
        "escape_squares": [{"square": square, "attackers": squares(by)} for square, by in item.escape_squares],
        "stuck_units": [
            {
                "square": square,
                "reason": held.kind,
                "pinner": held.pinner,
                "ray": squares(held.ray),
            }
            for square, held in item.stuck_units
        ],
    }


def draw(item: esca.explain.AutomaticDraw | esca.explain.ClaimableDraw) -> dict[str, Any]:
    """One draw condition and the evidence behind it."""
    answer: dict[str, Any] = {"kind": item.kind}
    material = getattr(item, "material", None)
    if material is not None:
        answer["material"] = material
    detail = getattr(item, "stalemate", None)
    if detail is not None:
        answer["stalemate"] = stalemate(detail)
    if item.repetition is not None:
        answer["repetition"] = repetition(item.repetition)
    if item.fifty_move is not None:
        answer["fifty_move"] = fifty_move(item.fifty_move)
    answer["prose"] = prose(item)
    return answer


def draw_status(game: esca.Game) -> dict[str, Any]:
    """Every draw condition that holds, automatic and claimable alike."""
    status = game.draw_status()
    return {
        "automatic": [draw(item) for item in status.automatic],
        "claimable": [draw(item) for item in status.claimable],
    }
