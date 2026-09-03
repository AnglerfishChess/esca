//! What the plies before a position say: how forcing they were, and which way
//! the material has been going.

use crate::moves::Move;
use crate::position::Position;
use crate::types::{Colour, Role, Square};

use super::HistoryFacts;
use super::scan::material_value;

/// How far back the recent-play facts look.
const WINDOW: usize = 8;

/// The square the unit `mv` captures stands on before it is played.
fn victim_square(mv: Move) -> Square {
    if mv.is_en_passant() {
        Square::new(mv.to().file(), mv.from().rank())
    } else {
        mv.to()
    }
}

/// The value sum of `colour`'s units less the other side's.
fn balance(position: &Position, colour: Colour) -> i32 {
    let mut value = 0;
    for role in [
        Role::Pawn,
        Role::Knight,
        Role::Bishop,
        Role::Rook,
        Role::Queen,
    ] {
        let ours = (position.by_colour(colour) & position.by_role(role)).len() as i32;
        let theirs = (position.by_colour(!colour) & position.by_role(role)).len() as i32;
        value += material_value(role) * (ours - theirs);
    }
    value
}

/// What the plies of a game say about the position it has reached.
///
/// `positions` holds every position from the start, `moves` the moves between
/// them, so `positions.len()` is `moves.len() + 1`.
pub(crate) fn from_history(positions: &[Position], moves: &[Move]) -> HistoryFacts {
    let now = positions.last().expect("a history holds its start");
    let us = now.side_to_move();
    let played = moves.len();
    let window = played.min(WINDOW);

    let mut facts = HistoryFacts {
        known: true,
        halfmove_clock: now.halfmove_clock(),
        halfmove_known: now.clocks_known(),
        ..HistoryFacts::default()
    };

    for ply in played - window..played {
        if moves[ply].is_capture() {
            facts.captures_in_last_8 += 1;
        }
        if positions[ply + 1].in_check() {
            facts.checks_in_last_8 += 1;
        }
    }

    facts.quiet_plies = (0..played)
        .rev()
        .find(|&ply| moves[ply].is_capture() || positions[ply + 1].in_check())
        .map_or(played, |ply| played - 1 - ply) as u32;

    facts.material_trend = balance(now, us) - balance(&positions[played - window], us);

    if let Some(&mv) = moves.last() {
        let before = &positions[played - 1];
        facts.last_move_mover = before.piece_at(mv.from()).map(|piece| piece.role);
        if mv.is_capture() {
            facts.last_move_victim = before.piece_at(victim_square(mv)).map(|piece| piece.role);
        }
    }
    facts
}
