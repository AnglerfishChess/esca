//! Classic chess.

use cozy_chess as cc;

use crate::error::{MoveParseError, PositionError};
use crate::moves::{Move, MoveList};
use crate::position::Position;
use crate::types::Role;
use crate::variant::{CastlingOutput, Outcome, PROMOTION_ROLES, Variant, rules};

/// The standard game. The seed of [`Variant::start_position`] is ignored.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Classic;

impl Variant for Classic {
    fn name(&self) -> &'static str {
        "chess"
    }

    fn start_position(&self, _seed: u64) -> Position {
        Position::from_parts(cc::Board::startpos(), 0, 1, true)
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

    /// Rejects castling rights that only a shuffled back rank can produce.
    fn validate(&self, position: &Position) -> Result<(), PositionError> {
        if rules::has_classic_castling(position) {
            Ok(())
        } else {
            Err(PositionError::CastlingRights)
        }
    }

    fn move_to_uci(&self, _position: &Position, mv: Move, style: CastlingOutput) -> String {
        rules::move_to_uci(mv, style, true)
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
