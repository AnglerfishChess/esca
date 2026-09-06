//! A chess model that answers what is true about a position.
//!
//! [`Position`] is placement and state, with no rules attached. Rules live in
//! [`Variant`] implementations — [`Classic`] and [`Chess960`] — and a
//! [`Game`] pairs a variant with the moves played, which is what repetition
//! and claimable draws need.
//!
//! [`explain`] answers why a rules answer is what it is, and [`endings`]
//! names the ending a position has reached, what theory says its result is
//! and the method it is played by.
//!
//! [`Facts`] answers what is true about a position — material, pawn structure,
//! king safety, one-ply tactics. Every fact names the colour it is about.
//!
//! ```
//! use esca::{Colour, Game, classic};
//!
//! let mut game = Game::new(classic());
//! game.play_san("e4").unwrap();
//! game.play_uci("e7e5").unwrap();
//! assert_eq!(game.position().fen(),
//!            "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2");
//!
//! let facts = game.facts();
//! assert_eq!(facts.material.phase_points, 24);
//! assert!(facts.pawns.open_files.is_empty());
//! assert_eq!(facts.tactics.legal_move_count.white, 29);
//! assert!(facts.pawns.passed.white.files().is_empty());
//! ```

#![deny(missing_docs)]

pub mod endings;
mod error;
pub mod explain;
pub mod facts;
mod fen;
mod game;
#[cfg(feature = "lichess")]
pub mod lichess;
mod moves;
#[cfg(feature = "openings")]
pub mod openings;
#[cfg(feature = "pgn")]
pub mod pgn;
#[cfg(feature = "polyglot")]
pub mod polyglot;
mod position;
#[cfg(feature = "python")]
mod python;
mod types;
#[cfg(feature = "uci")]
pub mod uci;
mod variant;
mod zobrist;

pub use error::{FenError, IllegalMove, MoveParseError, PositionError};
pub use facts::catalogue::{Field, Group};
pub use facts::{
    AnnotatedMove, AttackFacts, CastledSide, DrawishMaterial, EndgameFacts, ExchangeFacts, Facts,
    HistoryFacts, KingFacts, MapFacts, MaterialFacts, MobilityFacts, MoveFacts, Opposition,
    PawnFacts, PieceFacts, PlacementFacts, Scratch, StateFacts, TacticsFacts, ThreatFacts,
    catalogue,
};
pub use game::Game;
pub use moves::{MAX_MOVES, Move, MoveKind, MoveList};
pub use position::{CastlingRights, Key, Position, Score};
pub use types::{
    ByColour, Colour, File, FileSet, FileSetIter, PerFile, PerRank, Piece, Rank, Role, Square,
    SquareParseError, SquareSet, SquareSetIter,
};
pub use variant::{
    CHESS960, CLASSIC, CastlingOutput, Chess960, Classic, DrawClaim, Outcome, Variant, chess960,
    classic,
};
