//! A chess model that answers what is true about a position.
//!
//! [`Position`] is placement and state, with no rules attached. Rules live in
//! [`Variant`] implementations — [`Classic`] and [`Chess960`] — and a
//! [`Game`] pairs a variant with the moves played, which is what repetition
//! and claimable draws need.
//!
//! ```
//! use esca::{Game, classic};
//!
//! let mut game = Game::new(classic());
//! game.play_san("e4").unwrap();
//! game.play_uci("e7e5").unwrap();
//! assert_eq!(game.position().fen(),
//!            "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2");
//! ```

#![deny(missing_docs)]

mod error;
mod fen;
mod game;
mod moves;
mod position;
mod types;
mod variant;

pub use error::{FenError, IllegalMove, MoveParseError, PositionError};
pub use game::Game;
pub use moves::{MAX_MOVES, Move, MoveKind, MoveList};
pub use position::{CastlingRights, Key, Position, Score};
pub use types::{
    Colour, File, Piece, Rank, Role, Square, SquareParseError, SquareSet, SquareSetIter,
};
pub use variant::{
    CHESS960, CLASSIC, CastlingOutput, Chess960, Classic, DrawClaim, Outcome, Variant, chess960,
    classic,
};
