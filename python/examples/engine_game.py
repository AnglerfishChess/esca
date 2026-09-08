"""A UCI engine against itself for ten plies, then the facts of the final position.

Without a usable engine it says so and stops, so it is safe to run anywhere.

Run: uv run --no-sync python python/examples/engine_game.py /usr/local/bin/stockfish
"""

from __future__ import annotations

import asyncio
import json
import sys
import tempfile
from pathlib import Path

import esca
import esca.tensors as tensors
import numpy as np
from esca import uci

PLIES = 10
MOVETIME = 0.1


async def selfplay(engine_path: str, plies: int = PLIES, movetime: float = MOVETIME) -> esca.Game:
    """A game the engine plays against itself, `movetime` seconds a move."""
    game = esca.Game()
    async with uci.AsyncEngine(engine_path) as engine:
        await engine.handshake()
        print(f"{engine.name} by {engine.author}")
        await engine.new_game()
        for _ in range(plies):
            # `go` is not a coroutine: this is the whole call.
            answer: uci.Answer = await engine.go(uci.Limits(movetime=movetime)).answer()
            if answer.best is None:
                break
            print(game.move_to_san(answer.best), end=" ", flush=True)
            game.play(answer.best)
            await engine.set_position(game)
    print()
    return game


async def deepest(engine_path: str, game: esca.Game) -> None:
    """The reports of one search, read as they arrive."""
    async with uci.AsyncEngine(engine_path) as engine:
        await engine.handshake()
        await engine.set_position(game)
        search: uci.AsyncSearch = engine.go(uci.Limits(depth=8))
        last: uci.Info | None = None
        async for report in search:
            last = report
        answer: uci.Answer = await search.answer()
        print(f"depth {last.depth if last else '-'}, best {answer.best and answer.best.uci()}")


def report(game: esca.Game, out: Path) -> None:
    """The final position as English, as the one JSON form, and as arrays."""
    print(f"\nfinal: {game.position.fen}")
    print(f"outcome: {game.outcome() or 'ongoing'}; claims: {game.claims() or '-'}")
    ending = game.ending()
    print(f"ending: {ending.class_} / {ending.verdict} / {ending.technique}")
    print(f"        {ending.describe()}")

    facts = json.dumps(game.facts().to_dict(), indent=None)
    print(f"\nfacts as JSON: {len(facts)} bytes, e.g. {json.dumps(game.facts().to_dict()['material'])[:100]}…")

    print(f"array names: {tensors.names(['material'])[:3]} …")
    arrays = tensors.facts([game], groups=["material", "pawns"])
    tensors.save(arrays, out, metadata={"fen": game.position.fen})
    reloaded = tensors.load(out)
    same = all(np.array_equal(arrays[name], reloaded[name]) for name in arrays)
    print(f"\n{len(arrays)} arrays -> {out.name} ({out.stat().st_size} bytes), reload equal: {same}")
    print(f"metadata: {tensors.metadata(out)}")
    for name in ("material.value", "pawns.passed"):
        array = reloaded[name]
        print(f"  {name}: shape={array.shape} dtype={array.dtype}")


def main(argv: list[str]) -> int:
    if not argv:
        print("no engine given: pass the path to a UCI engine, e.g. /usr/local/bin/stockfish")
        return 0
    engine_path = argv[0]
    try:
        # Missing, or there and not runnable, reads the same from here.
        played = asyncio.run(selfplay(engine_path))
    except (OSError, uci.UciError) as error:
        print(f"{engine_path} does not run as a UCI engine: {error}")
        return 0
    with tempfile.TemporaryDirectory() as directory:
        report(played, Path(directory) / "final_position.safetensors")
    asyncio.run(deepest(engine_path, played))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
