//! Chess960.

use cozy_chess as cc;

use crate::error::{MoveParseError, PositionError};
use crate::moves::{Move, MoveList};
use crate::position::Position;
use crate::types::Role;
use crate::variant::{CastlingOutput, Outcome, PROMOTION_ROLES, Variant, rules};

/// Fischer Random. [`Variant::start_position`] returns arrangement
/// `seed % 960`, numbered as Scharnagl numbers are.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Chess960;

impl Variant for Chess960 {
    fn name(&self) -> &'static str {
        "chess960"
    }

    fn start_position(&self, seed: u64) -> Position {
        let arrangement = (seed % 960) as u32;
        Position::from_parts(cc::Board::chess960_startpos(arrangement), 0, 1, true)
    }

    fn legal_moves(&self, position: &Position, out: &mut MoveList) {
        rules::legal_moves(position, out);
    }

    fn is_legal(&self, position: &Position, mv: Move) -> bool {
        rules::is_legal(position, mv)
    }

    fn play(&self, position: &Position, mv: Move) -> Position {
        rules::play(position, mv)
    }

    fn outcome(&self, position: &Position) -> Option<Outcome> {
        rules::outcome(position)
    }

    fn promotion_roles(&self) -> &'static [Role] {
        &PROMOTION_ROLES
    }

    /// Every position esca can hold is one Chess960 can play on.
    fn validate(&self, _position: &Position) -> Result<(), PositionError> {
        Ok(())
    }

    /// Always king-to-rook: a king's two-square destination can coincide with
    /// another legal king move, or with its own origin.
    fn move_to_uci(&self, _position: &Position, mv: Move, _style: CastlingOutput) -> String {
        rules::move_to_uci(mv, CastlingOutput::KingToRook, false)
    }

    fn move_from_uci(&self, position: &Position, text: &str) -> Result<Move, MoveParseError> {
        rules::move_from_uci(self, position, text)
    }

    fn move_to_san(&self, position: &Position, mv: Move) -> String {
        rules::move_to_san(self, position, mv)
    }

    fn move_from_san(&self, position: &Position, text: &str) -> Result<Move, MoveParseError> {
        rules::move_from_san(self, position, text)
    }
}
