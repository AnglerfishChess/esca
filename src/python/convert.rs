//! The spellings the Python surface uses for the crate's small value types.
//!
//! Squares, roles, colours and files are text there; a colour is read from
//! `"white"`, `"black"`, `"w"` or `"b"`, in any case.

use std::sync::Arc;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::explain::Wing;
use crate::moves::MoveKind;
use crate::types::{Colour, File, Role, Square};
use crate::variant::{CastlingOutput, DrawClaim, Outcome, Variant, chess960, classic};

/// A `ValueError` carrying `error`'s message.
pub(crate) fn value_error<E: std::fmt::Display>(error: E) -> PyErr {
    PyValueError::new_err(error.to_string())
}

pub(crate) fn square_name(square: Square) -> String {
    square.to_string()
}

pub(crate) fn square_from(name: &str) -> PyResult<Square> {
    name.parse()
        .map_err(|_| PyValueError::new_err(format!("not a square name: {name:?}")))
}

pub(crate) fn role_name(role: Role) -> String {
    role.to_char().to_string()
}

pub(crate) fn role_from(name: &str) -> PyResult<Role> {
    let mut chars = name.chars();
    match (chars.next().and_then(Role::from_char), chars.next()) {
        (Some(role), None) => Ok(role),
        _ => Err(PyValueError::new_err(format!("not a role: {name:?}"))),
    }
}

/// The name a colour carries in text: `"white"` or `"black"`.
pub(crate) fn colour_name(colour: Colour) -> String {
    match colour {
        Colour::White => "white",
        Colour::Black => "black",
    }
    .to_string()
}

/// The colour a name means: `"white"`, `"black"`, `"w"` or `"b"`, in any case.
pub(crate) fn colour_from(name: &str) -> PyResult<Colour> {
    match name.to_ascii_lowercase().as_str() {
        "white" | "w" => Ok(Colour::White),
        "black" | "b" => Ok(Colour::Black),
        _ => Err(PyValueError::new_err(format!(
            "a colour is \"white\" or \"black\", not {name:?}"
        ))),
    }
}

pub(crate) fn file_name(file: File) -> String {
    file.to_char().to_string()
}

pub(crate) fn file_from(name: &str) -> PyResult<File> {
    let mut chars = name.chars();
    match (chars.next().and_then(File::from_char), chars.next()) {
        (Some(file), None) => Ok(file),
        _ => Err(PyValueError::new_err(format!("not a file: {name:?}"))),
    }
}

pub(crate) fn wing_name(wing: Wing) -> String {
    match wing {
        Wing::Short => "short",
        Wing::Long => "long",
    }
    .to_string()
}

pub(crate) fn wing_from(name: &str) -> PyResult<Wing> {
    match name {
        "short" => Ok(Wing::Short),
        "long" => Ok(Wing::Long),
        other => Err(PyValueError::new_err(format!(
            "a castling wing is \"short\" or \"long\", not {other:?}"
        ))),
    }
}

pub(crate) fn move_kind_name(kind: MoveKind) -> &'static str {
    match kind {
        MoveKind::Quiet => "quiet",
        MoveKind::Capture => "capture",
        MoveKind::EnPassant => "en_passant",
        MoveKind::Castling => "castling",
        MoveKind::Promotion => "promotion",
    }
}

pub(crate) fn move_kind_from(name: &str) -> PyResult<MoveKind> {
    match name {
        "quiet" => Ok(MoveKind::Quiet),
        "capture" => Ok(MoveKind::Capture),
        "en_passant" => Ok(MoveKind::EnPassant),
        "castling" => Ok(MoveKind::Castling),
        "promotion" => Ok(MoveKind::Promotion),
        other => Err(PyValueError::new_err(format!("not a move kind: {other:?}"))),
    }
}

pub(crate) fn outcome_name(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Checkmate { .. } => "checkmate",
        Outcome::Stalemate => "stalemate",
        Outcome::InsufficientMaterial => "insufficient_material",
        Outcome::SeventyFiveMoves => "seventy_five_moves",
        Outcome::FivefoldRepetition => "fivefold_repetition",
    }
}

pub(crate) fn claim_name(claim: DrawClaim) -> &'static str {
    match claim {
        DrawClaim::FiftyMoves => "fifty_moves",
        DrawClaim::ThreefoldRepetition => "threefold_repetition",
    }
}

pub(crate) fn castling_output_name(style: CastlingOutput) -> &'static str {
    match style {
        CastlingOutput::KingToRook => "king_to_rook",
        CastlingOutput::KingTwoSquares => "king_two_squares",
    }
}

pub(crate) fn castling_output_from(name: &str) -> PyResult<CastlingOutput> {
    match name {
        "king_to_rook" => Ok(CastlingOutput::KingToRook),
        "king_two_squares" => Ok(CastlingOutput::KingTwoSquares),
        other => Err(PyValueError::new_err(format!(
            "not a castling output style: {other:?}"
        ))),
    }
}

/// The shared variant of that name.
pub(crate) fn variant_by_name(name: &str) -> PyResult<Arc<dyn Variant>> {
    match name {
        "chess" => Ok(classic()),
        "chess960" => Ok(chess960()),
        other => Err(PyValueError::new_err(format!("not a variant: {other:?}"))),
    }
}
