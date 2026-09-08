//! The variants: one implementation per set of rules.

mod chess960;
mod classic;
mod rules;

use core::fmt;
use std::sync::{Arc, OnceLock};

pub use chess960::Chess960;
pub use classic::Classic;

use crate::error::{MoveParseError, PositionError};
use crate::moves::{Move, MoveList};
use crate::position::Position;
use crate::types::{Colour, Role};

/// How castling is spelled in UCI text.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CastlingOutput {
    /// `e1h1`. Correct in every variant.
    #[default]
    KingToRook,
    /// `e1g1`. Classic geometry only.
    KingTwoSquares,
}

/// How a game ended, without a player having to claim it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Outcome {
    /// The side to move is in check and has no legal move.
    Checkmate {
        /// The side that gave mate.
        winner: Colour,
    },
    /// The side to move is not in check and has no legal move.
    Stalemate,
    /// Neither side has material that could ever deliver mate.
    InsufficientMaterial,
    /// The halfmove clock reached 150.
    SeventyFiveMoves,
    /// The current position has occurred five times.
    FivefoldRepetition,
}

impl Outcome {
    /// The condition's name, the one spelling every surface uses:
    /// `checkmate`, `stalemate`, `insufficient_material`,
    /// `seventy_five_moves`, `fivefold_repetition`.
    pub const fn name(self) -> &'static str {
        match self {
            Outcome::Checkmate { .. } => "checkmate",
            Outcome::Stalemate => "stalemate",
            Outcome::InsufficientMaterial => "insufficient_material",
            Outcome::SeventyFiveMoves => "seventy_five_moves",
            Outcome::FivefoldRepetition => "fivefold_repetition",
        }
    }

    /// The colour the result favours; `None` for the draws.
    pub const fn winner(self) -> Option<Colour> {
        match self {
            Outcome::Checkmate { winner } => Some(winner),
            _ => None,
        }
    }

    /// One plain sentence: how the game ended.
    pub fn describe(self) -> String {
        match self {
            Outcome::Checkmate { winner } => format!(
                "{} is checkmated, so {} wins.",
                spelled(!winner),
                spelled(winner)
            ),
            Outcome::Stalemate => {
                "The side to move is not in check and has no legal move, which is stalemate: a \
                 draw."
                    .to_string()
            }
            Outcome::InsufficientMaterial => {
                "Neither side has material that could ever deliver mate, so the game is drawn."
                    .to_string()
            }
            Outcome::SeventyFiveMoves => {
                "Seventy-five moves have passed with no capture and no pawn move, which draws the \
                 game with no claim."
                    .to_string()
            }
            Outcome::FivefoldRepetition => {
                "The position has stood five times, which draws the game with no claim.".to_string()
            }
        }
    }
}

impl fmt::Display for Outcome {
    /// The condition's name.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A draw a player may claim, leaving the position playable until one does.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DrawClaim {
    /// The halfmove clock has reached 100.
    FiftyMoves,
    /// The current position has occurred three times.
    ThreefoldRepetition,
}

impl DrawClaim {
    /// The condition's name, the one spelling every surface uses:
    /// `fifty_moves`, `threefold_repetition`.
    pub const fn name(self) -> &'static str {
        match self {
            DrawClaim::FiftyMoves => "fifty_moves",
            DrawClaim::ThreefoldRepetition => "threefold_repetition",
        }
    }

    /// One plain sentence: what a player may claim, and on what ground.
    pub fn describe(self) -> &'static str {
        match self {
            DrawClaim::FiftyMoves => {
                "Fifty moves have passed with no capture and no pawn move, so either player may \
                 claim a draw."
            }
            DrawClaim::ThreefoldRepetition => {
                "The position has stood three times, so either player may claim a draw."
            }
        }
    }
}

impl fmt::Display for DrawClaim {
    /// The condition's name.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A colour written out, for a sentence.
fn spelled(colour: Colour) -> &'static str {
    match colour {
        Colour::White => "White",
        Colour::Black => "Black",
    }
}

/// A complete set of chess rules. Adding rules means adding an
/// implementation.
pub trait Variant: Send + Sync + 'static {
    /// Stable identifier, lower-case, as used in PGN and UCI: `chess`,
    /// `chess960`.
    fn name(&self) -> &'static str;

    /// A position to start a game from.
    fn start_position(&self, seed: u64) -> Position;

    /// Appends every legal move in `position` to `out`.
    fn legal_moves(&self, position: &Position, out: &mut MoveList);

    /// Whether `mv` may be played in `position`.
    fn is_legal(&self, position: &Position, mv: Move) -> bool;

    /// The position after `mv`.
    ///
    /// # Panics
    /// If `mv` is not legal in `position`.
    fn play(&self, position: &Position, mv: Move) -> Position;

    /// The terminal state of `position` judged from the position alone:
    /// checkmate, stalemate, insufficient material, and the automatic
    /// move-count draw. Repetition needs history and belongs to a `Game`.
    fn outcome(&self, position: &Position) -> Option<Outcome>;

    /// The roles a pawn may promote to.
    fn promotion_roles(&self) -> &'static [Role];

    /// Whether `position` is one this variant can reach and play on.
    fn validate(&self, position: &Position) -> Result<(), PositionError>;

    /// Castling is written as `style` asks; every other move is unaffected.
    fn move_to_uci(&self, position: &Position, mv: Move, style: CastlingOutput) -> String;

    /// Accepts both castling spellings whatever the output style.
    fn move_from_uci(&self, position: &Position, text: &str) -> Result<Move, MoveParseError>;

    /// The move in Standard Algebraic Notation, with check and mate marks.
    fn move_to_san(&self, position: &Position, mv: Move) -> String;

    /// Accepts the check, mate and annotation marks, and `0-0` for `O-O`.
    fn move_from_san(&self, position: &Position, text: &str) -> Result<Move, MoveParseError>;
}

/// Classic chess, shared.
pub fn classic() -> Arc<dyn Variant> {
    static SHARED: OnceLock<Arc<dyn Variant>> = OnceLock::new();
    SHARED.get_or_init(|| Arc::new(Classic)).clone()
}

/// Chess960, shared.
pub fn chess960() -> Arc<dyn Variant> {
    static SHARED: OnceLock<Arc<dyn Variant>> = OnceLock::new();
    SHARED.get_or_init(|| Arc::new(Chess960)).clone()
}

/// Classic chess, as a value.
pub static CLASSIC: Classic = Classic;
/// Chess960, as a value.
pub static CHESS960: Chess960 = Chess960;

pub(crate) const PROMOTION_ROLES: [Role; 4] = [Role::Queen, Role::Rook, Role::Bishop, Role::Knight];
