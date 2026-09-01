# esca

A chess model that answers what is true about a position.

*The esca is the anglerfish's lure: the small thing that lights up what is in front of it.*

`Position` is placement and state and nothing else. Rules live in `Variant` implementations —
`Classic` and `Chess960` — so a position can be asked the same question under different rules, and
a new variant is a new implementation and nothing else. A `Game` pairs a variant with the moves
played, which is what repetition and claimable draws need.

## Use

```toml
[dependencies]
esca = "0.1"
```

```rust
use esca::{Game, classic};

let mut game = Game::new(classic());
game.play_san("e4").unwrap();
game.play_uci("e7e5").unwrap();

println!("{}", game.position().fen());
for mv in game.legal_moves().as_slice() {
    println!("{}", game.move_to_san(*mv));
}
assert_eq!(game.outcome(), None);
```

## What it covers

- Classic chess and Chess960, behind one `Variant` trait.
- FEN and EPD, reading `KQkq` and the `AHah` of X-FEN and Shredder-FEN alike, and writing `KQkq`
  whenever the rook files allow it.
- Legal move generation into a `MoveList` that never allocates.
- UCI move text in either castling spelling, and SAN with the disambiguation it needs.
- Checkmate, stalemate, insufficient material, the fifty- and seventy-five-move rules, and
  threefold and fivefold repetition.

## Develop

```sh
cargo test
cargo test -- --ignored     # the deep perft counts
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

## License

MIT — see [LICENSE](../../LICENSE).
