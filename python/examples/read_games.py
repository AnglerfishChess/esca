"""Summarise every game of a PGN file: opening, final position, how it ended.

Run: uv run --no-sync python python/examples/read_games.py examples/games.pgn
"""

from __future__ import annotations

import sys
from pathlib import Path

import esca


def ending(game: esca.Game) -> str:
    """How the game stands after its last move.

    `outcome()` is `None` while a draw is only claimable, so both calls are
    needed to answer "how did this game end".
    """
    outcome = game.outcome()
    if outcome is not None:
        return outcome
    claims = game.claims()
    if claims:
        return "draw claimable (" + ", ".join(claims) + ")"
    return "ongoing"


def passed_pawn_files(game: esca.Game) -> dict[str, list[str]]:
    """The files carrying a passed pawn, per colour."""
    passed = game.facts().pawns.passed
    return {colour: list(squares.files) for colour, squares in passed.items()}


def main(path: Path) -> None:
    """One block per game of the file, in the order they are stored."""
    for pgn_game in esca.pgn.read(path):
        game = pgn_game.game()
        opening = game.opening()
        passed = passed_pawn_files(game)
        print(pgn_game.headers.get("Event", "?"))
        print(f"  opening: {opening.eco} {opening.name}" if opening else "  opening: -")
        print(f"  final:   {game.position.fen}")
        print(f"  ended:   {ending(game)} (PGN says {pgn_game.result})")
        for colour, files in passed.items():
            print(f"  passed {colour}: {', '.join(files) or '-'}")


if __name__ == "__main__":
    main(Path(sys.argv[1] if len(sys.argv) > 1 else "examples/games.pgn"))
