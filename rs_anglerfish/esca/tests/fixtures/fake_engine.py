#!/usr/bin/env python3
"""
A scripted UCI engine double, for the transport tests of both languages.

It knows no chess: it answers from the start position and nothing else. With no
flags it behaves conformingly; each flag makes it break one rule or take one
unhappy path:

``--no-uciok``      never end the identification
``--no-readyok``    never answer ``isready``
``--no-chess960``   do not offer ``UCI_Chess960``
``--no-move``       answer ``bestmove (none)``
``--die-on-go``     exit while searching
``--slow``          take 0.5 s to answer
``--flood``         write more reports than a client keeps, before answering
``--garbage``       write malformed and out-of-turn lines before the answer
``--twice``         write the answer twice
``--zombie``        ignore ``quit``
``--log=PATH``      append every command line received to PATH
"""

import sys
import time

#: What the engine offers, less the one Chess960 flag `--no-chess960` drops.
OPTIONS = [
    "option name Hash type spin default 16 min 1 max 1024",
    "option name Ponder type check default false",
    "option name MultiPV type spin default 1 min 1 max 8",
    "option name Style type combo default Solid var Solid var Wild",
    "option name Clear Hash type button",
    "option name Debug Log File type string default <empty>",
]

#: Lines an engine has no business writing, that a client has to survive.
GARBAGE = [
    "",
    "   ",
    "Fake engine 1.0, not a real one",
    "info depth",
    "info string weird: colons: and   spacing",
    "bestmove",
    "readyok",
]

#: How many reports ``--flood`` writes: more than either client's line buffer
#: holds, so that a client reading later has to drop some of them.
FLOOD = 6000

#: The reports of a search, in the order a real engine would write them.
REPORTS = [
    "info depth 1 seldepth 1 multipv 1 score cp 20 nodes 20 nps 2000 time 10 pv e2e4",
    "info depth 2 seldepth 3 multipv 1 score cp 25 lowerbound hashfull 3 nodes 90 pv e2e4 e7e5",
    "info string thinking about it: hard",
]


def emit(text: str) -> None:
    print(text, flush=True)


def main(argv: list[str]) -> int:
    flags = {argument for argument in argv if not argument.startswith("--log=")}
    paths = [argument[len("--log=") :] for argument in argv if argument.startswith("--log=")]
    log = open(paths[0], "a", encoding="utf-8") if paths else None  # noqa: SIM115
    waiting = False  # a search that must not answer until stop or ponderhit

    for raw in sys.stdin:
        command = raw.strip()
        if log is not None:
            log.write(command + "\n")
            log.flush()
        word = command.split(maxsplit=1)[0] if command else ""

        if word == "uci":
            emit("id name Fake Engine 1.0")
            emit("id author The esca test suite")
            for option in OPTIONS:
                emit(option)
            if "--no-chess960" not in flags:
                emit("option name UCI_Chess960 type check default false")
            if "--no-uciok" not in flags:
                emit("uciok")
        elif word == "isready":
            if "--no-readyok" not in flags:
                emit("readyok")
        elif word == "go":
            if "--die-on-go" in flags:
                return 3
            if "infinite" in command.split() or "ponder" in command.split():
                waiting = True
            else:
                answer(flags)
        elif word in ("stop", "ponderhit"):
            if waiting:
                waiting = False
                answer(flags)
        elif word == "quit":
            if "--zombie" in flags:
                continue
            return 0
    return 0


def answer(flags: set[str]) -> None:
    """Report a search and its result."""
    if "--slow" in flags:
        time.sleep(0.5)
    if "--flood" in flags:
        for number in range(FLOOD):
            emit(f"info depth 1 nodes {number} score cp 20 pv e2e4")
    if "--garbage" in flags:
        for line in GARBAGE:
            emit(line)
    for report in REPORTS:
        emit(report)
    emit("bestmove (none)" if "--no-move" in flags else "bestmove e2e4 ponder e7e5")
    if "--twice" in flags:
        emit("bestmove e2e4")


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
