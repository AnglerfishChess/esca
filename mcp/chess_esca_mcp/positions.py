"""Reading a position out of what a caller supplied, and saying what is wrong
with it when nothing can be read.

Every failure is a `ChessInputError`, which carries a `kind`, a sentence and
the evidence a caller needs to fix the call — never a traceback.
"""

from typing import Any

import esca

#: The variants a caller may name. esca calls classic chess `chess`; both
#: spellings are accepted and `classic` is the one the answers use.
VARIANTS = {"classic": esca.CLASSIC, "chess": esca.CLASSIC, "chess960": esca.CHESS960}


class ChessInputError(Exception):
    """An input that names no position, no move or no game."""

    def __init__(self, kind: str, message: str, hint: str | None = None, **evidence: Any) -> None:
        super().__init__(message)
        self.kind = kind
        self.message = message
        self.hint = hint
        self.evidence = evidence

    def content(self) -> dict[str, Any]:
        """The error as a tool answers it."""
        error: dict[str, Any] = {"kind": self.kind, "message": self.message}
        if self.hint is not None:
            error["hint"] = self.hint
        error.update(self.evidence)
        return {"error": error}


def variant_named(name: str) -> esca.Variant:
    """The rules `name` selects."""
    if name not in VARIANTS:
        raise ChessInputError(
            "unknown_variant",
            f"{name!r} is not a variant this server knows",
            hint=f"use one of: {', '.join(sorted(VARIANTS))}",
        )
    return VARIANTS[name]


def variant_label(variant: esca.Variant) -> str:
    """The name the answers give `variant`."""
    return "chess960" if variant == esca.CHESS960 else "classic"


def play(game: esca.Game, text: str, index: int) -> None:
    """Plays one move written as SAN or as UCI.

    `index` is the move's place in the sequence, for the error to name.
    """
    for attempt in (game.play_san, game.play):
        try:
            attempt(text)
        except ValueError:
            continue
        return
    raise ChessInputError(
        "illegal_move",
        f"{text!r} names no legal move in {game.position.fen}",
        hint="moves are SAN ('Nf3', 'exd6') or UCI ('g1f3', 'e5d6'); see legal_moves for what is playable",
        move=text,
        move_index=index,
        fen=game.position.fen,
        legal_moves=sorted(game.move_to_san(mv) for mv in game.legal_moves()),
    )


def read_game(
    fen: str | None = None,
    moves: list[str] | None = None,
    pgn: str | None = None,
    variant: str = "classic",
) -> esca.Game:
    """The game `fen`, `moves` and `pgn` describe between them.

    `pgn` stands alone and brings its own rules and start position. Otherwise
    the game starts at `fen`, or at the variant's own start position, and
    `moves` are played from there in order, each written as SAN or as UCI.
    """
    if pgn is not None:
        if fen is not None or moves:
            raise ChessInputError(
                "conflicting_inputs",
                "pgn carries its own start position and moves",
                hint="pass pgn on its own, or pass fen and moves instead",
            )
        try:
            return read_pgn(pgn).game()
        except ValueError as error:
            raise ChessInputError(
                "unplayable_pgn",
                f"the movetext does not play out: {error}",
                hint="every move must be legal in the position it is written for",
            ) from None

    rules = variant_named(variant)
    if fen is None:
        game = esca.Game(variant=rules)
    else:
        try:
            game = esca.Game.from_fen(fen, variant=rules)
        except ValueError as error:
            raise ChessInputError(
                "invalid_fen",
                f"{fen!r} is not a position under {variant} rules: {error}",
                hint=(
                    "a FEN is six fields: placement, side to move, castling rights, en-passant "
                    "target, halfmove clock, fullmove number"
                ),
                fen=fen,
            ) from None
    for index, text in enumerate(moves or []):
        play(game, text, index)
    return game


def pgn_games(text: str) -> list[esca.pgn.Game]:
    """Every game a PGN source holds."""
    try:
        return list(esca.pgn.read_string(text))
    except ValueError as error:
        raise ChessInputError(
            "invalid_pgn",
            f"the PGN could not be read: {error}",
            hint="tag pairs in brackets, a blank line, then the movetext",
        ) from None


def read_pgn(text: str, index: int = 0) -> esca.pgn.Game:
    """The game at `index` of a PGN source."""
    games = pgn_games(text)
    if not games:
        raise ChessInputError(
            "empty_pgn",
            "the PGN holds no game",
            hint="a game is a tag section and a movetext section",
        )
    if not 0 <= index < len(games):
        raise ChessInputError(
            "no_such_game",
            f"the PGN holds {len(games)} game(s), so there is no game {index}",
            hint=f"index is zero-based: 0 to {len(games) - 1}",
            games_in_source=len(games),
        )
    return games[index]


def parse_move(game: esca.Game, text: str) -> esca.Move | None:
    """The legal move `text` names, or `None` when it names none."""
    position = game.position
    for legal in game.legal_moves():
        if text == game.move_to_san(legal) or text == game.move_to_uci(legal):
            return legal
    # SAN written without its check or mate suffix, and UCI in either castling
    # spelling, both go through the variant's own reader.
    scratch = esca.Game.from_position(position, variant=game.variant)
    for attempt in (scratch.play_san, scratch.play):
        try:
            attempt(text)
        except ValueError:
            continue
        return scratch.moves[-1]
    return None
