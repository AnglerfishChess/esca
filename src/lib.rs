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
//!
//! # Features
//!
//! Everything above is always there. The rest is behind a feature, and each
//! feature-gated item carries its own badge here:
//!
//! | feature | what it adds |
//! | --- | --- |
//! | `pgn` | [`pgn`]: reading and writing PGN. |
//! | `uci` | [`uci`]: the protocol as values, and engines as subprocesses. |
//! | `tokio` | [`uci::tokio`]: the same client on a tokio runtime. |
//! | `polyglot` | [`polyglot`]: opening books in the Polyglot format. |
//! | `openings` | [`openings`]: the bundled ECO catalogue. |
//! | `lichess` | [`lichess`]: the Lichess evaluation dump reader. |
//! | `serde` | `serde::Serialize` for the facts and the board vocabulary. |
//! | `tensors` | [`facts::tensors`]: a run of positions as one typed array per fact. |
//! | `python` | the PyO3 module the `esca` Python package is built from. |
//!
//! # The three pages to read first
//!
//! The published crate carries none of them, so they are linked to the
//! repository:
//!
//! - [the API guide](https://github.com/AnglerfishChess/esca/blob/main/docs/esca-api.md),
//!   variant, position, move, game and facts first;
//! - [the vocabulary](https://github.com/AnglerfishChess/esca/blob/main/docs/esca-vocabulary.md),
//!   which fixes what every term here means;
//! - [the facts](https://github.com/AnglerfishChess/esca/blob/main/docs/features.md),
//!   every fact with its type, its tensor array name and its meaning.

#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod endings;
mod error;
pub mod explain;
pub mod facts;
mod fen;
mod game;
#[cfg(feature = "lichess")]
#[cfg_attr(docsrs, doc(cfg(feature = "lichess")))]
pub mod lichess;
mod moves;
#[cfg(feature = "openings")]
#[cfg_attr(docsrs, doc(cfg(feature = "openings")))]
pub mod openings;
#[cfg(feature = "pgn")]
#[cfg_attr(docsrs, doc(cfg(feature = "pgn")))]
pub mod pgn;
#[cfg(feature = "polyglot")]
#[cfg_attr(docsrs, doc(cfg(feature = "polyglot")))]
pub mod polyglot;
mod position;
#[cfg(feature = "python")]
mod python;
mod types;
#[cfg(feature = "uci")]
#[cfg_attr(docsrs, doc(cfg(feature = "uci")))]
pub mod uci;
mod variant;
mod zobrist;

pub use error::{FenError, IllegalMove, MoveParseError, PositionError};
pub use facts::catalogue::{FactsGroup, Field, FieldValue, Group};
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
    ByColour, Colour, File, FileSet, FileSetIter, ParseError, PerFile, PerRank, PerRole, Piece,
    Rank, Role, Square, SquareSet, SquareSetIter, Wing,
};
pub use variant::{
    CHESS960, CLASSIC, CastlingOutput, Chess960, Classic, DrawClaim, Outcome, Variant, chess960,
    classic,
};
