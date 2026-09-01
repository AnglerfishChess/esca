//! The errors the public API returns.

use core::fmt;

/// Why a FEN could not be read.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FenError {
    /// Not four fields (EPD) and not six.
    FieldCount,
    /// The placement field does not describe 8 ranks of 8 squares.
    Placement,
    /// The side-to-move field is neither `w` nor `b`.
    SideToMove,
    /// The castling field names a right the placement cannot support.
    Castling,
    /// The en-passant field is neither `-` nor a square on the right rank.
    EnPassant,
    /// The halfmove clock is not a number.
    HalfmoveClock,
    /// The full-move number is not a number, or is zero.
    FullmoveNumber,
    /// The placement is not a legal chess position: a missing or duplicated
    /// king, a pawn on a back rank, or the side not to move left in check.
    Position,
}

impl fmt::Display for FenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            FenError::FieldCount => "a FEN has four or six fields",
            FenError::Placement => "invalid placement field",
            FenError::SideToMove => "invalid side-to-move field",
            FenError::Castling => "invalid castling field",
            FenError::EnPassant => "invalid en-passant field",
            FenError::HalfmoveClock => "invalid halfmove clock",
            FenError::FullmoveNumber => "invalid full-move number",
            FenError::Position => "not a legal position",
        })
    }
}

impl std::error::Error for FenError {}

/// Why a position is not one a variant can play on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PositionError {
    /// Castling rights naming rook files this variant never starts from.
    CastlingRights,
}

impl fmt::Display for PositionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            PositionError::CastlingRights => "castling rights this variant cannot have",
        })
    }
}

impl std::error::Error for PositionError {}

/// Why move text did not name one legal move.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoveParseError {
    /// The text is not shaped like a move at all.
    Syntax,
    /// The text is well formed but names no legal move in the position.
    Illegal,
    /// The text names more than one legal move.
    Ambiguous,
}

impl fmt::Display for MoveParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            MoveParseError::Syntax => "malformed move text",
            MoveParseError::Illegal => "no such legal move",
            MoveParseError::Ambiguous => "ambiguous move text",
        })
    }
}

impl std::error::Error for MoveParseError {}

/// The move is not legal in the position it was played in.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct IllegalMove;

impl fmt::Display for IllegalMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("illegal move")
    }
}

impl std::error::Error for IllegalMove {}
