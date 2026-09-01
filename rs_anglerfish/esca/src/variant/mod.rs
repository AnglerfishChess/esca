//! The variants: one implementation per set of rules.

mod chess960;
mod classic;
mod rules;

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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

/// A draw a player may claim, leaving the position playable until one does.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DrawClaim {
    /// The halfmove clock has reached 100.
    FiftyMoves,
    /// The current position has occurred three times.
    ThreefoldRepetition,
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
