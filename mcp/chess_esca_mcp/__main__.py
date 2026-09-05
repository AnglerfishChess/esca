"""The command line: `chess-esca-mcp` serves the tools over stdio."""

import argparse
import logging
import sys

from chess_esca_mcp import __version__
from chess_esca_mcp.server import run


def main() -> None:
    """Parses the arguments and serves until the client closes stdin."""
    parser = argparse.ArgumentParser(
        prog="chess-esca-mcp",
        description="An MCP server over the esca chess library: rules, facts and explanations.",
    )
    parser.add_argument("--version", action="version", version=f"chess-esca-mcp {__version__}")
    parser.add_argument("--debug", action="store_true", help="Log at debug level, to stderr.")
    arguments = parser.parse_args()

    # stdout carries the protocol; every line of ours goes to stderr.
    logging.basicConfig(
        level=logging.DEBUG if arguments.debug else logging.INFO,
        format="%(asctime)s %(name)s %(levelname)s %(message)s",
        stream=sys.stderr,
    )
    run()


if __name__ == "__main__":
    main()
