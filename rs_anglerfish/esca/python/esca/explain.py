"""The evidence behind a rules answer: what forbids a castling or an en-passant
capture, what pins or skewers a unit, and what makes a position a draw.

An enum that carries nothing is its name in `snake_case`; an enum that carries
something is one class with a `kind` naming the case and the payload of every
case as attributes, empty where the case does not carry it.
"""

from ._esca import (
    AutomaticDraw,
    Castling,
    ClaimableDraw,
    DrawStatus,
    EnPassant,
    EpCapture,
    EpObstacle,
    FiftyMove,
    NearMiss,
    Pin,
    Repetition,
    Reset,
    Skewer,
    StalemateDetail,
    Stuck,
)

__all__ = [
    "AutomaticDraw",
    "Castling",
    "ClaimableDraw",
    "DrawStatus",
    "EnPassant",
    "EpCapture",
    "EpObstacle",
    "FiftyMove",
    "NearMiss",
    "Pin",
    "Repetition",
    "Reset",
    "Skewer",
    "StalemateDetail",
    "Stuck",
]
