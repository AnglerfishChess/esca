"""Every legal move of a position in SAN, then why one illegal move is illegal.

Run: uv run --no-sync python python/examples/legal_moves.py
"""

from __future__ import annotations

import esca

LISTED = "r3k2r/pP4pp/8/8/8/8/6PP/R3K2R w KQkq - 0 1"

# White to move: cxd6 en passant is pseudo-legal but uncovers the king on a5.
EN_PASSANT = "8/8/8/K1Pp3q/8/8/8/7k w - d6 0 2"
ILLEGAL = "c5d6"


def traits(annotated: esca.AnnotatedMove) -> str:
    """What the move does, beyond moving a unit."""
    facts = annotated.facts
    named = [
        (f"captures {facts.victim}", facts.victim is not None),
        ("castling", facts.is_castling),
        ("en passant", facts.is_en_passant),
        (f"promotes to {facts.promotion}", facts.promotion is not None),
        ("check", facts.gives_check),
    ]
    return ", ".join(name for name, holds in named if holds) or "quiet"


def list_moves(fen: str) -> None:
    """A bare position answers this on its own: no Game is needed."""
    position = esca.Position.from_fen(fen)
    moves = position.annotated_moves()
    print(f"{fen}\n{len(moves)} legal moves:")
    for annotated in moves:
        print(f"  {position.san(annotated.move):8} {traits(annotated)}")


def explain_illegal(fen: str, uci: str) -> None:
    """What the error says, and what the explanations layer says beside it."""
    position = esca.Position.from_fen(fen)
    print(f"\n{fen}\n{uci} is not among the {len(position.legal_moves())} legal moves:")
    try:
        position.play(uci)
    except esca.Error as error:
        print(f"  play({uci!r}) -> {type(error).__name__}: {error}")

    status = position.en_passant_status()
    print(f"  en-passant target: {status.target}")
    for capture in status.captures:
        obstacle = capture.forbidden_by
        reason = obstacle.kind if obstacle else None
        print(f"  {capture.origin}: legal={capture.legal} reason={reason}")
        print(f"    {capture.describe()}")
        if obstacle is not None:
            print(f"    {obstacle.describe()}")
            print(
                f"    attacker={obstacle.attacker} pinner={obstacle.pinner} "
                f"ray={list(obstacle.ray)} checkers={list(obstacle.by)}"
            )


if __name__ == "__main__":
    list_moves(LISTED)
    explain_illegal(EN_PASSANT, ILLEGAL)
