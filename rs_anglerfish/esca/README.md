# esca

A chess model that answers what is true about a position — a Rust crate and a Python package built
from it.

*The esca is the anglerfish's lure: the small thing that lights up what is in front of it.*

`Position` is placement and state and nothing else. Rules live in `Variant` implementations —
`Classic` and `Chess960` — so a position can be asked the same question under different rules, and
a new variant is a new implementation and nothing else. A `Game` pairs a variant with the moves
played, which is what repetition and claimable draws need. `Facts` answers what is true about a
position, both as prose a reader can check and as the flat `f32` row a net consumes.

## Rust

```toml
[dependencies]
esca = "0.1"
```

```rust
use esca::{Game, Schema, Side, classic};

let mut game = Game::new(classic());   // Chess960 rules: `esca::chess960()`
game.play_san("e4").unwrap();
game.play_uci("e7e5").unwrap();
println!("{}", game.position().fen());

let facts = game.facts();              // side-relative, in the mover's view
println!("{}", facts.tactics[Side::Us.index()].legal_move_count);
println!("{}", facts.summary());

let schema = Schema::v1();             // the row a net eats: 1906 f32
println!("{}", facts.encode(schema, schema.all()).len());
```

Cargo features, none on by default: `lichess` (streaming reader for the Lichess evaluation
dump), `pgn` (reading and writing games as PGN) and `python` (the PyO3 module the wheel is
built from).

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

facts = game.facts()  # side-relative: index with esca.US / esca.THEM
print(facts.tactics[esca.US].legal_move_count)
print(facts.summary())

rows = esca.encode([game.position.fen])  # (1, 1906) float32, ready for a net
print(rows.shape, esca.SCHEMA_ID)
```

Wheels are abi3 for Python 3.12 and up. `esca.lichess.batches()` streams the evaluation dump as
encoded batches with their targets.

## What it covers

- Classic chess and Chess960, behind one `Variant` trait.
- FEN and EPD, reading `KQkq` and the `AHah` of X-FEN and Shredder-FEN alike, and writing `KQkq`
  whenever the rook files allow it.
- Legal move generation into a `MoveList` that never allocates.
- UCI move text in either castling spelling, and SAN with the disambiguation it needs.
- Checkmate, stalemate, insufficient material, the fifty- and seventy-five-move rules, and
  threefold and fivefold repetition.
- `Facts`: nine groups of cheap position facts — state, material, pawns, pieces, king, mobility,
  attacks, one-ply tactics and attack planes — plus `MoveFacts` for every legal move, all
  side-relative and in the mover's view.
- `Schema`, a versioned manifest with a `schema_id`, and batch encoders that write `f32` rows
  without allocating.

## Documentation

- [`docs/esca-api.md`](https://github.com/AnglerfishChess/anglerfish/blob/main/docs/esca-api.md) —
  the API in both languages.
- [`docs/esca-vocabulary.md`](https://github.com/AnglerfishChess/anglerfish/blob/main/docs/esca-vocabulary.md) —
  the terms the API and the facts are named after.

## License

MIT — see [LICENSE](https://github.com/AnglerfishChess/anglerfish/blob/main/LICENSE).
