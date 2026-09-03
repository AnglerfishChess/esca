//! A chess model that answers what is true about a position.
//!
//! [`Position`] is placement and state, with no rules attached. Rules live in
//! [`Variant`] implementations — [`Classic`] and [`Chess960`] — and a
//! [`Game`] pairs a variant with the moves played, which is what repetition
//! and claimable draws need.
//!
//! [`Facts`] answers what is true about a position — material, pawn structure,
//! king safety, one-ply tactics — for a reader and, through [`Schema`] and the
//! encoders, as the flat `f32` row a net consumes.
//!
//! ```
//! use esca::{Game, Schema, Side, classic};
//!
//! let mut game = Game::new(classic());
//! game.play_san("e4").unwrap();
//! game.play_uci("e7e5").unwrap();
//! assert_eq!(game.position().fen(),
//!            "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2");
//!
//! let facts = game.facts();
//! assert_eq!(facts.material.phase, 1.0);
//! assert!(facts.pawns.open_files.is_empty());
//! assert_eq!(facts.tactics[Side::Us.index()].legal_move_count, 29);
//!
//! let schema = Schema::v1();
//! assert_eq!(facts.encode(schema, schema.all()).len(), schema.width());
//! ```

#![deny(missing_docs)]

mod error;
mod facts;
mod fen;
mod game;
#[cfg(feature = "lichess")]
pub mod lichess;
mod moves;
#[cfg(feature = "pgn")]
pub mod pgn;
mod position;
#[cfg(feature = "python")]
mod python;
mod schema;
mod types;
#[cfg(feature = "uci")]
pub mod uci;
mod variant;

pub use error::{FenError, IllegalMove, MoveParseError, PositionError};
pub use facts::{
    AnnotatedMove, AttackFacts, Facts, HistoryFacts, KingFacts, MaterialFacts, MobilityFacts,
    MoveFacts, PawnFacts, PieceFacts, PlacementFacts, PlaneFacts, RowError, Scratch, Side,
    StateFacts, TacticsFacts, encode_fens, encode_positions,
};
pub use game::Game;
pub use moves::{MAX_MOVES, Move, MoveKind, MoveList};
pub use position::{CastlingRights, Key, Position, Score};
pub use schema::{FeatureSet, FeatureSpec, GroupSet, GroupSpec, Schema, SchemaId};
pub use types::{
    Colour, File, FileSet, FileSetIter, Piece, Rank, Role, Square, SquareParseError, SquareSet,
    SquareSetIter,
};
pub use variant::{
    CHESS960, CLASSIC, CastlingOutput, Chess960, Classic, DrawClaim, Outcome, Variant, chess960,
    classic,
};
