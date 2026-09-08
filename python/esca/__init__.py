"""A chess model that answers what is true about a position.

Squares, roles, colours and files are text here; every fact that differs
between the two sides is an `esca.ByColour`, read as `.white`, `.black` or
`.of(colour)`. `Facts.to_dict()` writes any of it as plain data, in the one
JSON form `docs/facts.schema.json` describes.

`esca.tensors` turns a run of positions into one typed array per fact; it needs
NumPy, which the base install leaves out: `pip install esca[tensors]`.

Every error raised here is an `esca.Error` and a `ValueError`: `FenError`,
`IllegalMove`, `MoveParseError`, `PositionError` and `ParseError`.

The three pages to read first are the README's Python block,
<https://github.com/AnglerfishChess/esca/blob/main/docs/esca-api.md> §11, and
<https://github.com/AnglerfishChess/esca/blob/main/docs/features.md>.
"""

from . import endings, explain, openings, pgn, polyglot, uci
from ._esca import (
    CHESS960,
    CLASSIC,
    KING_TO_ROOK,
    KING_TWO_SQUARES,
    AnnotatedMove,
    AttackFacts,
    ByColour,
    EndgameFacts,
    Error,
    ExchangeFacts,
    Facts,
    FenError,
    FileSet,
    Game,
    HistoryFacts,
    IllegalMove,
    KingFacts,
    MapFacts,
    MaterialFacts,
    MobilityFacts,
    Move,
    MoveFacts,
    MoveParseError,
    ParseError,
    PawnFacts,
    PieceFacts,
    PlacementFacts,
    Position,
    PositionError,
    SquareSet,
    StateFacts,
    TacticsFacts,
    ThreatFacts,
    Variant,
    __version__,
    facts_catalogue,
    move_facts_catalogue,
)

__all__ = [
    "CHESS960",
    "CLASSIC",
    "KING_TO_ROOK",
    "KING_TWO_SQUARES",
    "AnnotatedMove",
    "AttackFacts",
    "ByColour",
    "EndgameFacts",
    "Error",
    "ExchangeFacts",
    "Facts",
    "FenError",
    "FileSet",
    "Game",
    "HistoryFacts",
    "IllegalMove",
    "KingFacts",
    "MapFacts",
    "MaterialFacts",
    "MobilityFacts",
    "Move",
    "MoveFacts",
    "MoveParseError",
    "ParseError",
    "PawnFacts",
    "PieceFacts",
    "PlacementFacts",
    "Position",
    "PositionError",
    "SquareSet",
    "StateFacts",
    "TacticsFacts",
    "ThreatFacts",
    "Variant",
    "__version__",
    "endings",
    "explain",
    "facts_catalogue",
    "move_facts_catalogue",
    "openings",
    "pgn",
    "polyglot",
    "uci",
]
