//! Constructors that let a case state its expectation the way the definition
//! is written.

#![allow(dead_code)]

use esca::{CLASSIC, Facts, FileSet, MoveFacts, Position, Square, SquareSet, Variant};

/// The facts of `fen` under classic chess.
pub fn facts_of(fen: &str) -> Facts {
    facts_under(&CLASSIC, fen)
}

/// The facts of `fen` under `variant`.
pub fn facts_under(variant: &dyn Variant, fen: &str) -> Facts {
    Position::from_fen(fen)
        .expect("a test FEN is a legal position")
        .facts(variant)
}

/// The squares named by a space-separated list: `squares("e4 d5")`.
pub fn squares(names: &str) -> SquareSet {
    names
        .split_whitespace()
        .map(|name| name.parse::<Square>().expect("a square name"))
        .collect()
}

/// The files named by their letters: `files("bcg")`.
pub fn files(letters: &str) -> FileSet {
    letters
        .chars()
        .map(|letter| esca::File::from_char(letter).expect("a file letter"))
        .collect()
}

/// The facts of the move `uci` in `fen`, under classic chess.
pub fn move_facts(fen: &str, uci: &str) -> MoveFacts {
    move_facts_under(&CLASSIC, fen, uci)
}

/// The facts of the move `uci` in `fen`, under `variant`. Castling is written
/// king-to-rook, so `"e1h1"` is the classic short castling.
pub fn move_facts_under(variant: &dyn Variant, fen: &str, uci: &str) -> MoveFacts {
    facts_under(variant, fen)
        .moves
        .iter()
        .find(|annotated| annotated.mv.to_string() == uci)
        .unwrap_or_else(|| panic!("{uci} is a legal move of the position"))
        .facts
}
