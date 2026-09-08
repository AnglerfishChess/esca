# esca

[![crates.io](https://img.shields.io/crates/v/esca)](https://crates.io/crates/esca)
[![docs.rs](https://img.shields.io/docsrs/esca)](https://docs.rs/esca)
[![PyPI](https://img.shields.io/pypi/v/esca)](https://pypi.org/project/esca/)
[![Python](https://img.shields.io/pypi/pyversions/esca)](https://pypi.org/project/esca/)
[![CI](https://github.com/AnglerfishChess/esca/actions/workflows/ci.yml/badge.svg)](https://github.com/AnglerfishChess/esca/actions/workflows/ci.yml)
[![MIT](https://img.shields.io/badge/license-MIT-blue)](https://github.com/AnglerfishChess/esca/blob/main/LICENSE)

*Esca is the anglerfish's lure — the light that shows what is really on the board.*

Rust/Python MIT chess library: rules, facts, explanations, PGN, opening books and names, UCI client, and an MCP server over it; one API, Chess960 throughout.

`Position` is placement and state and nothing else. Rules live in `Variant` implementations —
`Classic` and `Chess960` — so a position answers a rules question by taking the variant that
defines it, and a new variant is a new implementation and nothing else. A `Game` pairs a variant
with the moves played, which is what repetition and claimable draws need. `Facts` answers what is
true about one position — 221 named facts in 14 groups — and `annotated_moves()` answers what each
of its legal moves does, with 27 more. Every fact is typed, named after what a player would call
it, and told about White and Black by name.

## Rust

```toml
[dependencies]
esca = "0.3"
```

```rust
use esca::{Colour, Game, classic};

let mut game = Game::new(classic());   // Chess960 rules: `esca::chess960()`
game.play_san("e4").unwrap();
game.play_uci("e7e5").unwrap();
println!("{}", game.position().fen());

let facts = game.facts();
println!("{}", facts.tactics.legal_move_count.white);
println!("{:?}", facts.pawns.passed.of(Colour::Black).files());
println!("{}", facts.summary());
```

Cargo features, none on by default: `lichess` (streaming reader for the Lichess evaluation
dump), `pgn` (reading and writing games as PGN), `polyglot` (opening books), `openings` (the
bundled ECO catalogue), `serde` (the one JSON form of the facts, and the JSON Schema for it),
`tensors` (a run of positions as one typed array per fact) and `python` (the PyO3 module the
wheel is built from). `Position::polyglot_key` needs no feature.

## Python

```sh
pip install esca
```

```python
import esca

game = esca.Game()  # Chess960 rules: esca.Game(variant=esca.CHESS960)
game.play_san("e4")
game.play("e7e5")
print(game.position.fen)

facts = game.facts()
print(facts.tactics.legal_move_count.white)
print(list(facts.pawns.passed.black.files))
print(facts.to_dict()["material"])  # every group in the one JSON form
```

Wheels are abi3 for Python 3.12 and up. `pip install esca[tensors]` adds NumPy and
`esca.tensors`, which turns a run of positions into one typed array per fact.

## What it covers

- Classic chess and Chess960, behind one `Variant` trait.
- FEN and EPD, reading `KQkq` and the `AHah` of X-FEN and Shredder-FEN alike, and writing `KQkq`
  whenever the rook files allow it.
- Legal move generation into a `MoveList` that never allocates.
- UCI move text in either castling spelling, and SAN with the disambiguation it needs.
- Checkmate, stalemate, insufficient material, the fifty- and seventy-five-move rules, and
  threefold and fivefold repetition.
- `Facts`: fourteen groups of cheap facts about one position — the board itself, game state,
  history, material, pawns, pieces, king, mobility, attacks, exchanges, threats, one-ply tactics,
  endgame and the attack maps side by side — and `MoveFacts` for every legal move, from
  `annotated_moves()`. Every value that differs between the two sides is a `ByColour`, read as
  `.white`, `.black` or `.of(colour)`.
- A catalogue of those facts as data — name, type, dtype, shape and meaning — which
  `docs/features.md`, `docs/facts.schema.json`, the Python type stubs and the tensor layout are
  all generated from, and which the MCP server serves.
- One JSON form for the facts, written by Rust's `serde::Serialize` and by Python's `to_dict()`,
  byte for byte the same and described by `docs/facts.schema.json`.
- A typed tensor export: one array per fact, batch first, each keeping the width and sign it was
  declared with — nothing scaled, normalised or cast to a float — expanded or bit-packed, and
  written as safetensors.
- Polyglot opening books: the format's own key on every `Position`, books read, drawn from and
  built, and an ECO code and name for some 3,800 named positions.
- Named endings with theory verdicts and technique names, and a one-line English `describe()`
  beside every value the explanations layer answers with.

## MCP server

`mcp/` is a second distribution from this repository: `chess-esca-mcp`, an MCP server that hands
esca's answers to an LLM as JSON — the whole state of a position, whether a move is legal and
every reason it is not, the named facts, the ECO name, opening-book moves, and PGN read and
written. It carries no engine and does no search. It runs as `uvx chess-esca-mcp`, is versioned
with the library and pins the matching `esca`, and is documented in
[`mcp/README.md`](https://github.com/AnglerfishChess/esca/blob/main/mcp/README.md).

## Documentation

- [`docs/esca-api.md`](https://github.com/AnglerfishChess/esca/blob/main/docs/esca-api.md) —
  the API in both languages; §11 is the whole Python surface.
- [`docs/features.md`](https://github.com/AnglerfishChess/esca/blob/main/docs/features.md) —
  every fact, its type and its meaning, group by group.
- [`docs/esca-vocabulary.md`](https://github.com/AnglerfishChess/esca/blob/main/docs/esca-vocabulary.md) —
  the terms the API and the facts are named after.

## Related projects

- [AnglerfishChess/anglerfish](https://github.com/AnglerfishChess/anglerfish) — the chess engine
  that plays from a learned evaluation, and the Python trainer that produces it. Both are built on
  esca; the trainer turns its facts into the rows a net eats.
- [AnglerfishChess/uci-test-suite](https://github.com/AnglerfishChess/uci-test-suite) — a
  conformance suite that checks a program is a valid UCI engine, whatever its strength. It talks to
  the engine under test through esca's UCI client.
- [AnglerfishChess/chess-uci-mcp](https://github.com/AnglerfishChess/chess-uci-mcp) — an MCP server
  that drives UCI engines from an LLM, so an esca position can be handed to Stockfish for a number
  and a line to go with the facts esca reads off it.
- [AnglerfishChess/plugins](https://github.com/AnglerfishChess/plugins) — the agent-plugin
  marketplace, where `chess-esca-mcp` ships with a skill that teaches an agent which of its tools
  answers which question.

## License

MIT — see [LICENSE](https://github.com/AnglerfishChess/esca/blob/main/LICENSE).

## Acknowledgements

- [cozy-chess](https://github.com/analog-hors/cozy-chess) (MIT) — the move generator esca
  stands on.
- [Lichess](https://lichess.org) — the evaluation dump the `lichess` reader streams, the game
  database, and [lichess-org/chess-openings](https://github.com/lichess-org/chess-openings),
  whose opening names the `openings` feature bundles (CC0 1.0 Universal Public Domain
  Dedication).
- The Polyglot opening-book format and its key scheme, by Fabien Letouzey; the key constants
  are those published in [polyglot-book-rs](https://crates.io/crates/polyglot-book-rs)
  (MIT OR Apache-2.0).
- [Stockfish](https://stockfishchess.org) and [Leela Chess Zero](https://lczero.org), the
  engines the UCI client is tested against.
