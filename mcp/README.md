# chess-esca-mcp

An MCP server over [esca](https://github.com/AnglerfishChess/esca): the rules of chess as JSON an
agent can read.

<!-- mcp-name: io.github.AnglerfishChess/chess-esca-mcp -->

FEN and moves go in; enums and the squares behind them come out. Whether a move is legal and why
not, what a position's status is and what either side may claim, the named facts of the position,
the opening's ECO code, the moves an opening book holds, and PGN read and written.

**There is no engine here and no search.** Nothing this server answers says how *good* a move is.
For a number and a line, use [chess-uci-mcp](https://github.com/AnglerfishChess/chess-uci-mcp),
which drives Stockfish or Leela; the two answer different questions and sit well side by side.

## Dependencies

Python 3.12 or newer, and `uv`/`uvx`.

## Usage

```json
"mcpServers": {
  "chess-esca-mcp": {
    "command": "uvx",
    "args": ["chess-esca-mcp@latest"]
  }
}
```

For Claude Desktop that is `claude_desktop_config.json`, under **Settings** → **Developer** →
**Edit Config**:

* macOS: `~/Library/Application\ Support/Claude/claude_desktop_config.json`
* Windows: `%APPDATA%/Claude/claude_desktop_config.json`
* Linux: `~/.config/Claude/claude_desktop_config.json`

For Claude Code, the plugin does the wiring:

```
/plugin marketplace add AnglerfishChess/plugins
/plugin install chess-esca-mcp@anglerfish-chess
```

The server takes no arguments. It reads nothing but the position you hand it and, for
`book_moves`, the book file you name.

## Tools

Every tool takes the position the same way: `fen` (the start position when omitted), `moves`
played from it — each written as SAN (`Nf3`) or as UCI (`g1f3`) — or a whole game as `pgn`
instead. `variant` is `classic` or `chess960`. Repetition and fifty-move claims are only visible
when the moves that led to the position are given, since a FEN does not carry them.

| Tool | Answers |
|---|---|
| `position` | The whole state: side to move, check and its checkers, game status, automatic and claimable draws with their evidence, legal-move count, opening, material, both castlings of both colours with every obstacle, the en-passant offer, pins and skewers. |
| `legal_moves` | Every legal move in SAN and UCI, each with its role, victim, promotion, check and static exchange, and grouped as captures, checks, castling, promotions, en passant and quiet. |
| `explain_move` | Whether one move is legal. If not, every reason at once, each with the squares it was read off. If so, what it changes, the position after it, and the draws it would open. |
| `facts` | The named facts of the position, group by group, every value labelled by name and by side. `groups` selects; `placement` and `planes` are left out unless asked for. |
| `opening` | The ECO code and name of the position, and the deepest named position the line reached. |
| `book_moves` | The moves a Polyglot opening book holds for the position, with their weights. |
| `pgn` | A PGN game read into headers, moves, comments, final position, opening and result. |
| `to_pgn` | A list of moves written out as a PGN game. |

An input that names no position, no move or no game comes back as
`{"error": {"kind": ..., "message": ..., "hint": ...}}` — never a traceback.

One resource, `esca://facts-schema`, lists the fact groups and the feature names of each, with
the schema id. One prompt, `analyse-position`, walks an agent through reading a position with
these tools.

### Opening books

No book is bundled. `book_moves` takes the path to a Polyglot `.bin` file of your own; the format
is the common one, so a book written for any engine that reads Polyglot works. `esca.polyglot`
has `download(url, path)` for fetching one, and a `Builder` that writes a book from PGN.

## Development

The server is a second distribution inside the esca repository: the library at the root, this
directory beside it. It takes esca from the crate next door while you work on it, and from PyPI
once installed.

```bash
git clone https://github.com/AnglerfishChess/esca.git
cd esca/mcp

uv sync --all-groups     # builds the esca extension from ../ and installs it here
uv run --no-sync pytest
uvx ruff check . && uvx ruff format --check . && uvx pyrefly check
```

Run it from the checkout with `uv run chess-esca-mcp`.

The version is esca's own: one tag releases the library and this server together, and the
`esca==` pin names that same version.

### The MCP registry

[registry.modelcontextprotocol.io](https://registry.modelcontextprotocol.io) is the index of
public MCP servers. The listing is described by `server.json`, under the name
`io.github.AnglerfishChess/chess-esca-mcp`; ownership of the PyPI package is proven by the
`mcp-name:` marker near the top of this file, which becomes the package description PyPI serves.
The registry reads it from the *published* artifact, so it counts only once a release carrying it
reaches PyPI.

## Related projects

* [esca](https://github.com/AnglerfishChess/esca) — the library this serves.
* [chess-uci-mcp](https://github.com/AnglerfishChess/chess-uci-mcp) — the engine side: analysis
  and best moves from Stockfish or Leela.

## License

MIT — see [LICENSE](https://github.com/AnglerfishChess/esca/blob/main/LICENSE).
