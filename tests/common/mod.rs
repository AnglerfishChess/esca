//! Constructors that let a case state its expectation the way the definition
//! is written.

#![allow(dead_code)]

use esca::{
    ByColour, CLASSIC, CastlingOutput, Facts, FileSet, MoveFacts, Position, Square, SquareSet,
    Variant,
};

/// The facts of `fen` under classic chess.
pub fn facts_of(fen: &str) -> Facts {
    facts_under(&CLASSIC, fen)
}

/// The facts of `fen` under `variant`.
pub fn facts_under(variant: &dyn Variant, fen: &str) -> Facts {
    position_of(fen).facts(variant)
}

/// The position `fen` describes.
pub fn position_of(fen: &str) -> Position {
    Position::from_fen(fen).expect("a test FEN is a legal position")
}

/// The pair `[white, black]`, the order every case writes a per-side fact in.
pub fn by_colour<T>(pair: [T; 2]) -> ByColour<T> {
    let [white, black] = pair;
    ByColour::new(white, black)
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
    position_of(fen)
        .annotated_moves(variant)
        .iter()
        .find(|annotated| annotated.mv.uci(CastlingOutput::KingToRook) == uci)
        .unwrap_or_else(|| panic!("{uci} is a legal move of the position"))
        .facts
}
