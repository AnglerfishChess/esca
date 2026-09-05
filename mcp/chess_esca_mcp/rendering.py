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

#: The letter esca spells each role name with.
ROLE_LETTERS = {name: letter for letter, name in ROLES.items()}

#: The ending class of a position that holds too much material to be one.
NOT_AN_ENDING = "not_an_ending"


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

    Empty where esca has no `describe()`; the field is present in every answer
    so that the shape does not change when a value grows sentences. A sentence
    sits beside the value it was read off, and an aggregate that already says
    what its parts say carries it instead of them.
    """
    describe = getattr(subject, "describe", None)
    if not callable(describe):
        return []
    said = describe()
    if isinstance(said, str):
        return [said]
    return [str(line) for line in said] if isinstance(said, Iterable) else []


def gathered(answer: object) -> list[str]:
    """Every sentence `answer` carries anywhere in it, in reading order.

    One index over the `prose` of the fields below it, each sentence once, so
    that the whole answer can be read as English without walking the object.
    """
    found: list[str] = []

    def walk(node: object) -> None:
        if isinstance(node, dict):
            for key, value in node.items():
                if key == "prose" and isinstance(value, list):
                    found.extend(line for line in value if line not in found)
                else:
                    walk(value)
        elif isinstance(node, list):
            for item in node:
                walk(item)

    walk(answer)
    return found


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
    answer["prose"] = prose(obstacle)
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
    return {
        "pinned": item.pinned,
        "pinner": item.pinner,
        "king": item.king,
        "ray": squares(item.ray),
        "prose": prose(item),
    }


def skewer(item: esca.explain.Skewer) -> dict[str, Any]:
    """One skewer, the more valuable unit in front."""
    return {
        "attacker": item.attacker,
        "front": item.front,
        "behind": item.behind,
        "ray": squares(item.ray),
        "prose": prose(item),
    }


def repetition(item: esca.explain.Repetition) -> dict[str, Any]:
    """How often this position has stood, and what nearly counted."""
    return {
        "count": item.count,
        "plies": list(item.plies),
        "near_misses": [
            {"ply": miss.ply, "differs": list(miss.differs), "prose": prose(miss)} for miss in item.near_misses
        ],
        "prose": prose(item),
    }


def fifty_move(item: esca.explain.FiftyMove) -> dict[str, Any]:
    """The halfmove clock and how far it is from ending the game."""
    reset = item.last_reset
    return {
        "clock": item.clock,
        "plies_to_claim": item.plies_to_claim,
        "plies_to_automatic": item.plies_to_automatic,
        "last_reset": (None if reset is None else {"ply": reset.ply, "kind": reset.kind, "prose": prose(reset)}),
        "prose": prose(item),
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
                "prose": prose(held),
            }
            for square, held in item.stuck_units
        ],
        "prose": prose(item),
    }


def draw(item: esca.explain.AutomaticDraw | esca.explain.ClaimableDraw, *, sentence: bool = True) -> dict[str, Any]:
    """One draw condition and the evidence behind it.

    `sentence` is false where the answer around it already says what this one
    would say, which is the whole draw status.
    """
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
    if sentence:
        answer["prose"] = prose(item)
    return answer


def draw_status(game: esca.Game) -> dict[str, Any]:
    """Every draw condition that holds, automatic and claimable alike.

    The `prose` here is the whole status in one go, the sentence of every
    condition listed included, so the conditions carry none of their own.
    """
    status = game.draw_status()
    return {
        "automatic": [draw(item, sentence=False) for item in status.automatic],
        "claimable": [draw(item, sentence=False) for item in status.claimable],
        "prose": prose(status),
    }


# --- the named ending ------------------------------------------------------


def signature(item: esca.endings.MaterialSignature) -> dict[str, Any]:
    """The material both sides hold, written the way endings are named."""
    return {
        "text": item.text,
        "stronger": colour(item.stronger),
        "pawns": item.pawns,
        "count": {
            name: {role_name: item.count(letter, ROLE_LETTERS[role_name]) for role_name in ROLE_ORDER}
            for letter, name in COLOURS.items()
        },
        "pieces": {name: item.pieces(letter) for letter, name in COLOURS.items()},
        "value": {name: item.value(letter) for letter, name in COLOURS.items()},
    }


def pawn_race(item: esca.endings.PawnRace) -> dict[str, Any]:
    """The race of the only pawn on the board."""
    return {
        "pawn": item.pawn,
        "colour": colour(item.colour),
        "promotion": item.promotion,
        "rook_pawn": item.rook_pawn,
        "steps": item.steps,
        "defender_inside_square": item.defender_inside_square,
        "attacker_in_front": item.attacker_in_front,
        "defender_in_front": item.defender_in_front,
        "defender_holds_the_corner": item.defender_holds_the_corner,
    }


def bishops(item: esca.endings.Bishops) -> dict[str, Any]:
    """The bishops on the board, when at least one stands on it."""
    return {
        "opposite_colours": item.opposite_colours,
        "same_colour": item.same_colour,
        "wrong_bishop": item.wrong_bishop,
    }


def ending_evidence(item: esca.endings.EndingEvidence) -> dict[str, Any]:
    """The position-specific facts behind an ending's verdict and technique.

    Each group is filled in only where the material puts it in question, and
    the `prose` covers every group at once, so the groups carry none of it.
    """
    return {
        "pawn": None if item.pawn is None else pawn_race(item.pawn),
        "bishops": None if item.bishops is None else bishops(item.bishops),
        "opposition": item.opposition,
        "prose": prose(item),
    }


def verdict(item: esca.endings.EndingVerdict) -> dict[str, Any]:
    """What theory says the result of an ending is, and for whom."""
    winner = item.winner
    return {"kind": item.name, "colour": None if winner is None else colour(winner)}


def ending(item: esca.endings.Ending) -> dict[str, Any]:
    """The ending on the board: its name, its result and the facts behind it."""
    return {
        "signature": signature(item.signature),
        "class": item.class_.name,
        "verdict": verdict(item.verdict),
        "technique": item.technique.name,
        "evidence": ending_evidence(item.evidence),
        "prose": prose(item),
    }


def ending_summary(item: esca.endings.Ending) -> dict[str, Any]:
    """The ending as a whole-position answer carries it: the naming only."""
    return {
        "class": item.class_.name,
        "verdict": verdict(item.verdict),
        "technique": item.technique.name,
        "prose": prose(item),
    }
