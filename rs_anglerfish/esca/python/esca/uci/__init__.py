"""Talking to a UCI engine.

Times are seconds here, scores are `cp`/`mate` pairs, and moves are `Move`
objects. `Engine` blocks and releases the GIL while it waits; `AsyncEngine`
holds the same conversation on asyncio, over `protocol`, with no thread and no
blocking call under it.
"""

from __future__ import annotations

import asyncio
import contextlib
from collections import deque
from collections.abc import AsyncIterator, Sequence
from os import PathLike

from .._esca import (
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
from . import protocol
from .protocol import Command, Message

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
    "protocol",
]

#: How long one wait for a line may take, so that a process that dies without
#: closing its pipes is still noticed.
_POLL = 0.2

#: How many unread lines are kept, as in the Rust client.
_CAPACITY = 4096

#: The states in which the engine still owes a `bestmove`.
_SEARCHING = ("searching", "pondering")

#: The name of the option that puts an engine into Chess960.
_CHESS960 = "UCI_Chess960"


class _Lines:
    """The lines an engine has written that have not been read yet.

    The queue is capped: when it is full, the oldest line that carries no part
    of the conversation goes, and is counted.
    """

    def __init__(self) -> None:
        self._lines: deque[str] = deque()
        self._ready = asyncio.Event()
        self._closed = False
        self.dropped = 0

    def push(self, line: str) -> None:
        """Adds one line, making room for it first when the queue is full."""
        if len(self._lines) >= _CAPACITY:
            for at, kept in enumerate(self._lines):
                if protocol.parse(kept).kind in ("info", "raw"):
                    del self._lines[at]
                    self.dropped += 1
                    break
        self._lines.append(line)
        self._ready.set()

    def close(self) -> None:
        """Records that the engine wrote its last line."""
        self._closed = True
        self._ready.set()

    @property
    def over(self) -> bool:
        """Whether the engine's output is over and what it wrote has been read."""
        return self._closed and not self._lines

    async def next(self, timeout: float) -> str | None:
        """The oldest line, or `None` if none is there within `timeout`."""
        loop = asyncio.get_running_loop()
        deadline = loop.time() + timeout
        while True:
            if self._lines:
                return self._lines.popleft()
            left = deadline - loop.time()
            if self._closed or left <= 0:
                return None
            self._ready.clear()
            if self._lines:  # one arrived while the flag was being cleared
                continue
            try:
                await asyncio.wait_for(self._ready.wait(), left)
            except TimeoutError:
                return None


class AsyncSearch:
    """A search in flight, read as an async iterator over its reports.

    A search that is neither finished nor stopped leaves the engine searching;
    `async with` stops it. Giving up on an awaited report — cancelling the
    task, or letting a timeout do it — asks the engine to stop, and the
    engine's next call waits that search out.
    """

    def __init__(self, engine: AsyncEngine, game: Game, deadline: float, budget: float) -> None:
        self._engine = engine
        self._game = game
        self._deadline = deadline
        self._budget = budget
        self._answer: Answer | None = None

    @property
    def done(self) -> bool:
        """Whether the engine has answered."""
        return self._answer is not None

    async def answer(self) -> Answer:
        """The engine's answer, with the reports still to come dropped."""
        while self._answer is None:
            await self._progress()
        return self._answer

    async def stop(self) -> None:
        """Asks the engine to finish the search now."""
        await self._engine.stop()

    async def ponderhit(self) -> None:
        """Tells the engine the move it is pondering on was played."""
        await self._engine.ponderhit()

    async def _progress(self) -> Info | None:
        """The next report, or `None` when the answer arrived instead."""
        try:
            message = await self._engine._receive(self._deadline, self._budget, "bestmove")
        except asyncio.CancelledError:
            self._engine._abandon()
            raise
        if message.kind == "bestmove":
            self._answer = message.answer
            return None
        return message.info

    def __aiter__(self) -> AsyncIterator[Info]:
        return self

    async def __anext__(self) -> Info:
        while not self.done:
            report = await self._progress()
            if report is not None:
                return report
        raise StopAsyncIteration

    async def __aenter__(self) -> AsyncSearch:
        return self

    async def __aexit__(self, *_exc: object) -> None:
        if not self.done:
            await self.stop()
            await self.answer()

    def __repr__(self) -> str:
        return f"<AsyncSearch {'answered' if self.done else 'running'}>"


class AsyncEngine:
    """A UCI engine process, addressed with coroutines.

    Every wait is bounded by `timeout`, or by the `timeout` of the call that
    takes one; an engine that has exited raises `EngineDied` on every call
    after. The process is started by the first call and killed by `kill`; use
    the engine as an async context manager, or call `quit`.

    `name`, `author` and `options` are read once the handshake is over and need
    no call after.
    """

    def __init__(
        self,
        command: str | PathLike[str],
        args: Sequence[str] = (),
        *,
        cwd: str | PathLike[str] | None = None,
        timeout: float = 10.0,
    ) -> None:
        self._command = command
        self._args = [str(argument) for argument in args]
        self._cwd = cwd
        self._timeout = timeout
        self._process: asyncio.subprocess.Process | None = None
        self._readers: list[asyncio.Task[None]] = []
        self._lines = _Lines()
        self._session = protocol.Session()
        self._name: str | None = None
        self._author: str | None = None
        self._options: dict[str, Option] = {}
        self._identified = False
        self._game: Game | None = None
        self._chess960 = False
        self._settling = False
        self._dead = False

    @property
    def timeout(self) -> float:
        """How long a wait that is not given its own limit may take, in seconds."""
        return self._timeout

    @timeout.setter
    def timeout(self, timeout: float) -> None:
        self._timeout = timeout

    # -- What the engine said about itself ----------------------------------

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
        """What the engine is doing."""
        return self._session.state

    @property
    def dropped_lines(self) -> int:
        """How many lines the engine wrote that were never read, because it
        wrote them faster than they were read."""
        return self._lines.dropped

    # -- The protocol -------------------------------------------------------

    async def handshake(self) -> None:
        """Sends `uci` and collects what the engine says about itself."""
        await self._issue(Command.uci())
        self._name = None
        self._author = None
        self._options = {}
        deadline = self._deadline(None)
        while True:
            message = await self._receive(deadline, self._timeout, "uciok")
            if message.kind == "uciok":
                break
            if message.kind == "id":
                if message.key == "name":
                    self._name = message.value
                elif message.key == "author":
                    self._author = message.value
            elif message.kind == "option" and message.option is not None:
                self._options[message.option.name] = message.option
        self._identified = True

    async def set_option(self, name: str, value: object = None) -> None:
        """Sets one option to a value of the option's own type."""
        await self._started()
        if not self._identified:
            raise ProtocolError("the engine has not answered uci")
        option = self._declared(name)
        if option is None:
            raise ValueError(f"the engine offers no option {name!r}")
        await self._issue(Command.setoption(option.name, option.value_text(value)))

    async def debug(self, on: bool) -> None:
        """Turns the engine's `info string` diagnostics on or off."""
        await self._issue(Command.debug(on))

    async def is_ready(self) -> None:
        """Sends `isready` and waits for `readyok`."""
        await self._issue(Command.isready())
        deadline = self._deadline(None)
        while (await self._receive(deadline, self._timeout, "readyok")).kind != "readyok":
            pass

    async def new_game(self) -> None:
        """Announces a new game and waits for the engine to be ready again."""
        await self._issue(Command.ucinewgame())
        self._game = None
        await self.is_ready()

    async def set_position(self, game: Game) -> None:
        """Sets the position to `game`, putting the engine into its variant.

        A Chess960 game needs the engine to offer `UCI_Chess960`; one that does
        not raises `ValueError` rather than playing by the wrong rules.
        """
        await self._started()
        chess960 = game.variant.name == "chess960"
        if chess960 != self._chess960:
            if not self._identified:
                raise ProtocolError("the engine has not answered uci")
            if chess960 or self._declared(_CHESS960) is not None:
                await self.set_option(_CHESS960, chess960)
                await self.is_ready()
            self._chess960 = chess960
        castling = "king_to_rook" if chess960 else "king_two_squares"
        await self._issue(Command.position(game, castling))
        self._game = game

    async def go(self, limits: Limits | None = None, *, timeout: float | None = None) -> AsyncSearch:
        """Starts a search on the position last set."""
        await self._issue(Command.go(limits))
        budget = timeout if timeout is not None else self._timeout
        return AsyncSearch(self, self._searched, self._deadline(timeout), budget)

    async def play(self, game: Game, limits: Limits | None = None, *, timeout: float | None = None) -> Answer:
        """Sets the position, searches it, and answers."""
        await self.set_position(game)
        search = await self.go(limits, timeout=timeout)
        return await search.answer()

    async def analyse(
        self,
        game: Game,
        limits: Limits | None = None,
        *,
        multipv: int | None = None,
        timeout: float | None = None,
    ) -> list[Info]:
        """The deepest report of each variation, ranked as the engine ranked them."""
        if multipv is not None:
            await self.set_option("MultiPV", multipv)
        await self.set_position(game)
        search = await self.go(limits, timeout=timeout)
        deepest: dict[int, Info] = {}
        async for report in search:
            if report.cp is None and report.mate is None:
                continue
            deepest[report.multipv or 1] = report
        return [deepest[line] for line in sorted(deepest)]

    async def stop(self) -> None:
        """Asks the search in flight to finish now."""
        await self._issue(Command.stop())

    async def ponderhit(self) -> None:
        """Tells the engine the move it is pondering on was played."""
        await self._issue(Command.ponderhit())

    # -- The raw line interface ---------------------------------------------

    async def send_line(self, text: str) -> None:
        """Writes one line, bypassing the order of the conversation."""
        await self._started()
        self._write(text)

    async def next_line(self, timeout: float | None = None) -> str | None:
        """The next line the engine wrote, or `None` if it wrote none in time."""
        await self._started()
        line = await self._lines.next(timeout if timeout is not None else self._timeout)
        if line is None:
            if self._lines.over:
                raise self._died()
            self._check_alive()
        return line

    # -- Ending it ----------------------------------------------------------

    async def quit(self) -> int | None:
        """Asks the engine to exit, killing it if it will not, and answers with
        its exit code."""
        process = self._process
        if process is None:
            return None
        if process.returncode is None:
            with contextlib.suppress(UciError, ValueError):
                self._send(Command.quit())
            if process.stdin is not None:
                process.stdin.close()
            try:
                await asyncio.wait_for(process.wait(), self._timeout)
            except TimeoutError:
                process.kill()
                await process.wait()
        self._dead = True
        await self._let_readers_go()
        return process.returncode

    def kill(self) -> None:
        """Kills the process."""
        self._dead = True
        for reader in self._readers:
            reader.cancel()
        self._readers = []
        if self._process is not None and self._process.returncode is None:
            self._process.kill()

    async def __aenter__(self) -> AsyncEngine:
        await self._started()
        return self

    async def __aexit__(self, *_exc: object) -> None:
        await self.quit()

    def __repr__(self) -> str:
        return f"<AsyncEngine {self._name or '?'} {self.state}>"

    # -- Machinery ----------------------------------------------------------

    async def _started(self) -> None:
        """Starts the process, once."""
        if self._process is not None:
            return
        self._process = await asyncio.create_subprocess_exec(
            self._command,
            *self._args,
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
            cwd=self._cwd,
            limit=1 << 20,
        )
        stdout, stderr = self._process.stdout, self._process.stderr
        self._readers = [
            asyncio.create_task(self._read_output(stdout)),
            asyncio.create_task(self._read_diagnostics(stderr)),
        ]

    async def _read_output(self, stdout: asyncio.StreamReader | None) -> None:
        """Fills the line queue until the engine writes no more."""
        while stdout is not None:
            raw = await stdout.readline()
            if not raw:
                break
            self._lines.push(raw.decode("utf-8", "replace").rstrip("\r\n"))
        self._lines.close()

    async def _read_diagnostics(self, stderr: asyncio.StreamReader | None) -> None:
        """Reads what the engine writes to stderr, so it never blocks on it."""
        while stderr is not None and await stderr.readline():
            pass

    async def _let_readers_go(self) -> None:
        """Waits for the readers to reach the end of the engine's output."""
        readers, self._readers = self._readers, []
        for reader in readers:
            reader.cancel()
        await asyncio.gather(*readers, return_exceptions=True)

    @property
    def _searched(self) -> Game:
        """The game a search runs on: the one last set, or the standard array,
        which is where an engine that was told nothing stands."""
        return self._game if self._game is not None else Game()

    def _declared(self, name: str) -> Option | None:
        """The option of that name, matched without regard to case."""
        for option in self._options.values():
            if option.name.lower() == name.lower():
                return option
        return None

    def _deadline(self, timeout: float | None) -> float:
        """When a wait of `timeout` seconds, or of the engine's own, runs out."""
        return asyncio.get_running_loop().time() + (timeout if timeout is not None else self._timeout)

    async def _issue(self, command: Command) -> None:
        """Sends one command, once a search that was given up on is over."""
        await self._started()
        await self._settle()
        self._send(command)

    def _send(self, command: Command) -> None:
        """Sends one command, after the state machine has allowed it. Writing
        never waits, so no cancelled task can leave half a command on the wire."""
        self._session.sent(command)
        self._write(command.to_line())

    def _write(self, text: str) -> None:
        """Writes one line."""
        self._check_alive()
        if self._process is None or self._process.stdin is None:
            raise EngineDied("the engine closed its output")
        self._process.stdin.write(text.encode("utf-8") + b"\n")

    async def _receive(self, deadline: float, budget: float, awaited: str) -> Message:
        """The next message the state machine accepts, awaiting `awaited` until
        `deadline`. A message it has no room for is dropped, so that one stray
        line cannot derail the conversation."""
        loop = asyncio.get_running_loop()
        while True:
            left = deadline - loop.time()
            if left <= 0:
                raise EngineTimeout(f"no {awaited} within {budget:.3f} s")
            line = await self._lines.next(min(left, _POLL))
            if line is None:
                if self._lines.over:
                    raise self._died()
                self._check_alive()
                continue
            message = protocol.parse(line, self._searched)
            try:
                self._session.received(message)
            except ProtocolError:
                continue
            return message

    async def _settle(self) -> None:
        """Waits out a search that was given up on, so that the engine is idle
        before anything else is asked of it."""
        if not self._settling:
            return
        self._settling = False
        deadline = self._deadline(None)
        while self._session.state in _SEARCHING:
            await self._receive(deadline, self._timeout, "bestmove")

    def _abandon(self) -> None:
        """Asks a search nobody is waiting for any more to finish."""
        if self._session.state not in _SEARCHING:
            return
        try:
            self._send(Command.stop())
        except (UciError, ValueError):
            return
        self._settling = True

    def _check_alive(self) -> None:
        """Raises if the process has exited."""
        if self._dead or (self._process is not None and self._process.returncode is not None):
            raise self._died()

    def _died(self) -> EngineDied:
        """The engine is gone, and says so from here on."""
        self._dead = True
        code = self._process.returncode if self._process is not None else None
        if code is None:
            return EngineDied("the engine closed its output")
        return EngineDied(f"the engine exited with code {code}")
