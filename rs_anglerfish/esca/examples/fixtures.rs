//! Regenerates the golden fixtures under `tests/data/`.
//!
//! Run it after a deliberate change to the schema or to a feature, and read
//! the diff before committing: every difference is a change of the trained
//! contract.
//!
//! ```text
//! cargo run --release --example fixtures
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use esca::{CHESS960, CLASSIC, Game, Position, Schema, Variant, chess960, encode_fens};

/// A xorshift, so the played-out Chess960 games are the same everywhere.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data")
}

/// The FEN lines of a corpus file, `#` comments and blanks dropped.
fn read_fens(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .expect("a corpus file")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect()
}

fn write_vectors(path: &Path, variant: &dyn Variant, fens: &[String]) {
    let schema = Schema::v0();
    let borrowed: Vec<&str> = fens.iter().map(String::as_str).collect();
    let mut values = vec![0.0f32; borrowed.len() * schema.width()];
    encode_fens(variant, &borrowed, schema, schema.all(), &mut values)
        .unwrap_or_else(|error| panic!("{} in {}", error, path.display()));
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for value in &values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    fs::write(path, bytes).expect("the fixture is writable");
}

/// Twenty Chess960 start positions and two played-out lines from each.
fn chess960_fens() -> Vec<String> {
    let mut fens = Vec::new();
    let mut rng = Rng(0x9E3779B97F4A7C15);
    for seed in 0..20u64 {
        let mut game = Game::with_seed(chess960(), seed * 47);
        fens.push(game.position().fen());
        for ply in 0..24 {
            let moves = game.legal_moves();
            if moves.is_empty() {
                break;
            }
            let choice = rng.next() as usize % moves.len();
            game.play(moves[choice]).expect("a generated move is legal");
            if ply == 7 || ply == 17 {
                fens.push(game.position().fen());
            }
        }
    }
    fens
}

fn main() {
    let dir = data_dir();
    let schema = Schema::v0();

    fs::write(dir.join("schema_v0.txt"), schema.canonical()).expect("the canonical text");
    fs::write(dir.join("schema_v0_id.txt"), format!("{}\n", schema.id())).expect("the schema id");

    let classic = read_fens(&dir.join("fens_classic.txt"));
    for fen in &classic {
        Position::from_fen(fen).unwrap_or_else(|error| panic!("{error} in {fen}"));
    }
    write_vectors(&dir.join("vectors_classic.bin"), &CLASSIC, &classic);

    let fens = chess960_fens();
    let header = "# Chess960 start positions and played-out lines, from\n\
                  # `cargo run --release --example fixtures`.\n";
    fs::write(
        dir.join("fens_chess960.txt"),
        format!("{header}{}\n", fens.join("\n")),
    )
    .expect("the Chess960 corpus");
    write_vectors(&dir.join("vectors_chess960.bin"), &CHESS960, &fens);

    println!(
        "{} classic rows, {} Chess960 rows, schema {}",
        classic.len(),
        fens.len(),
        schema.id()
    );
}
