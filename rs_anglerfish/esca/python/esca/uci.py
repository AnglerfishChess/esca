"""Talking to a UCI engine.

Times are seconds here, scores are `cp`/`mate` pairs, and moves are `Move`
objects. `Engine` blocks and releases the GIL while it waits; `AsyncEngine` is
the same surface as coroutines, driven from one worker thread.
"""

from __future__ import annotations

import asyncio
import functools
from collections.abc import Callable, Sequence
from concurrent.futures import ThreadPoolExecutor
from os import PathLike
from typing import Any, TypeVar

from ._esca import (
    Answer,
    Engine,
    EngineDied,
    EngineTimeout,
    Game,
    Info,
    Limits,
    Option,
    ProtocolError,
    Search,
    UciError,
)

__all__ = [
    "Answer",
    "AsyncEngine",
    "AsyncSearch",
    "Engine",
    "EngineDied",
    "EngineTimeout",
    "Info",
    "Limits",
    "Option",
    "ProtocolError",
    "Search",
    "UciError",
]

_T = TypeVar("_T")


def _next_report(search: Search) -> Info | None:
    """The next report of a search, or `None` once the engine has answered."""
    return next(search, None)


class AsyncSearch:
    """A search in flight, read as an async iterator over its reports.

    A search that is neither finished nor stopped leaves the engine searching;
    `async with` stops it.
    """

    def __init__(self, engine: AsyncEngine, search: Search) -> None:
        self._engine = engine
        self._search = search

    @property
    def done(self) -> bool:
        """Whether the engine has answered."""
        return self._search.done

    async def answer(self) -> Answer:
        """The engine's answer, with the reports still to come dropped."""
        return await self._engine.run(self._search.answer)

    async def stop(self) -> None:
        """Asks the engine to finish the search now."""
        await self._engine.run(self._search.stop)

    async def ponderhit(self) -> None:
        """Tells the engine the move it is pondering on was played."""
        await self._engine.run(self._search.ponderhit)

    def __aiter__(self) -> AsyncSearch:
        return self

    async def __anext__(self) -> Info:
        report = await self._engine.run(_next_report, self._search)
        if report is None:
            raise StopAsyncIteration
        return report

    async def __aenter__(self) -> AsyncSearch:
        return self

    async def __aexit__(self, *_exc: object) -> None:
        if not self._search.done:
            await self.stop()
            await self.answer()

    def __repr__(self) -> str:
        return f"<AsyncSearch {'answered' if self.done else 'running'}>"


class AsyncEngine:
    """A UCI engine as coroutines, for use inside an asyncio server.

    Every call runs on one worker thread, so the event loop is never blocked
    and the engine is spoken to by one caller at a time. `name`, `author` and
    `options` are read once the handshake is over and need no call after.
    """

    def __init__(
        self,
        command: str | PathLike[str],
        args: Sequence[str] = (),
        *,
        cwd: str | PathLike[str] | None = None,
        timeout: float = 10.0,
    ) -> None:
        self._pool = ThreadPoolExecutor(max_workers=1, thread_name_prefix="esca-uci")
        self._engine = Engine(command, list(args), cwd=cwd, timeout=timeout)
        self._name: str | None = None
        self._author: str | None = None
        self._options: dict[str, Option] = {}

    async def run(self, work: Callable[..., _T], *args: Any) -> _T:
        """Runs one call on this engine's worker thread."""
        loop = asyncio.get_running_loop()
        return await loop.run_in_executor(self._pool, functools.partial(work, *args))

    @property
    def name(self) -> str | None:
        """The engine's `id name`."""
        return self._name

    @property
    def author(self) -> str | None:
        """The engine's `id author`."""
        return self._author

    @property
    def options(self) -> dict[str, Option]:
        """Every option the engine offers, by name."""
        return self._options

    @property
    def state(self) -> str:
        """What the engine is doing; waits for the call in flight, if any."""
        return self._engine.state

    async def handshake(self) -> None:
        """Sends `uci` and collects what the engine says about itself."""
        await self.run(self._engine.handshake)
        self._name = self._engine.name
        self._author = self._engine.author
        self._options = dict(self._engine.options)

    async def set_option(self, name: str, value: object = None) -> None:
        """Sets one option to a value of the option's own type."""
        await self.run(self._engine.set_option, name, value)

    async def debug(self, on: bool) -> None:
        """Turns the engine's `info string` diagnostics on or off."""
        await self.run(self._engine.debug, on)

    async def new_game(self) -> None:
        """Announces a new game and waits for the engine to be ready again."""
        await self.run(self._engine.new_game)

    async def is_ready(self) -> None:
        """Sends `isready` and waits for `readyok`."""
        await self.run(self._engine.is_ready)

    async def set_position(self, game: Game) -> None:
        """Sets the position to `game`, putting the engine into its variant."""
        await self.run(self._engine.set_position, game)

    async def go(self, limits: Limits | None = None, *, timeout: float | None = None) -> AsyncSearch:
        """Starts a search on the position last set."""
        search = await self.run(functools.partial(self._engine.go, limits, timeout=timeout))
        return AsyncSearch(self, search)

    async def play(self, game: Game, limits: Limits | None = None, *, timeout: float | None = None) -> Answer:
        """Sets the position, searches it, and answers."""
        return await self.run(functools.partial(self._engine.play, game, limits, timeout=timeout))

    async def analyse(
        self,
        game: Game,
        limits: Limits | None = None,
        *,
        multipv: int | None = None,
        timeout: float | None = None,
    ) -> list[Info]:
        """The deepest report of each variation, ranked as the engine ranked them."""
        return await self.run(functools.partial(self._engine.analyse, game, limits, multipv=multipv, timeout=timeout))

    async def stop(self) -> None:
        """Asks the search in flight to finish now."""
        await self.run(self._engine.stop)

    async def ponderhit(self) -> None:
        """Tells the engine the move it is pondering on was played."""
        await self.run(self._engine.ponderhit)

    async def send_line(self, text: str) -> None:
        """Writes one line, bypassing the order of the conversation."""
        await self.run(self._engine.send_line, text)

    async def next_line(self, timeout: float | None = None) -> str | None:
        """The next line the engine wrote, or `None` if it wrote none in time."""
        return await self.run(functools.partial(self._engine.next_line, timeout))

    async def quit(self) -> int | None:
        """Asks the engine to exit, and answers with its exit code."""
        try:
            return await self.run(self._engine.quit)
        finally:
            self._pool.shutdown(wait=False)

    def kill(self) -> None:
        """Kills the process and lets the worker thread go."""
        self._engine.kill()
        self._pool.shutdown(wait=False)

    async def __aenter__(self) -> AsyncEngine:
        return self

    async def __aexit__(self, *_exc: object) -> None:
        await self.quit()

    def __repr__(self) -> str:
        return f"<AsyncEngine {self._name or '?'} {self.state}>"
