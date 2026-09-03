//! Static exchange evaluation: what a capture wins once both sides have taken
//! everything they want to on the square.

use core::cmp::Ordering;

use crate::moves::{Move, MoveList};
use crate::position::Position;
use crate::types::{Colour, Role, Square, SquareSet};

use super::ExchangeFacts;
use super::scan::{attackers, material_value};

/// What a pawn adds by reaching its relative rank 8: it becomes a queen.
const PROMOTION_GAIN: i32 = material_value(Role::Queen) - material_value(Role::Pawn);

/// The placement an exchange is played out over, by colour and role.
///
/// A unit that has left the square set the exchange carries is gone from it,
/// which is what re-admits a slider standing behind a departed attacker.
struct Placement {
    role_units: [[SquareSet; 6]; 2],
}

impl Placement {
    fn new(position: &Position) -> Placement {
        Placement {
            role_units: Colour::ALL.map(|colour| {
                Role::ALL.map(|role| position.by_colour(colour) & position.by_role(role))
            }),
        }
    }

    /// The units of `colour` still standing on `occupied` that attack `square`.
    fn attackers(&self, square: Square, colour: Colour, occupied: SquareSet) -> SquareSet {
        attackers(square, colour, &self.role_units[colour.index()], occupied) & occupied
    }

    /// The cheapest of them, and its role.
    fn least_valuable(
        &self,
        square: Square,
        colour: Colour,
        occupied: SquareSet,
    ) -> Option<(Square, Role)> {
        let set = self.attackers(square, colour, occupied);
        Role::ALL.into_iter().find_map(|role| {
            (set & self.role_units[colour.index()][role.index()])
                .first()
                .map(|from| (from, role))
        })
    }
}

/// What `side` wins by capturing the unit worth `occupant` on `square`, given
/// `occupied`; 0 when it has no attacker left, or when taking would cost more
/// than it wins and it therefore declines.
fn swap(
    placement: &Placement,
    square: Square,
    side: Colour,
    occupant: i32,
    occupied: SquareSet,
) -> i32 {
    let Some((from, role)) = placement.least_valuable(square, side, occupied) else {
        return 0;
    };
    let left = occupied - from.to_set();
    // A king captures only what the other side has stopped defending.
    if role == Role::King && !placement.attackers(square, !side, left).is_empty() {
        return 0;
    }
    let promotes = role == Role::Pawn && square.rank().relative_to(side).index() == 7;
    let landed = if promotes { Role::Queen } else { role };
    let gain = occupant + if promotes { PROMOTION_GAIN } else { 0 }
        - swap(placement, square, !side, material_value(landed), left);
    gain.max(0)
}

impl Position {
    /// The static exchange evaluation of the unit on `square`: what the side
    /// that does not own it wins by starting to capture there.
    ///
    /// Never negative — the opponent may leave the unit alone. 0 for an empty
    /// square, for a king, and wherever the exchange wins nothing.
    pub fn see(&self, square: Square) -> i32 {
        let Some(piece) = self.piece_at(square) else {
            return 0;
        };
        swap(
            &Placement::new(self),
            square,
            !piece.colour,
            material_value(piece.role),
            self.occupied(),
        )
    }

    /// The static exchange evaluation of `mv`: what its side wins if both
    /// sides then keep capturing on its destination.
    ///
    /// Signed, in value units. A quiet move wins nothing and may lose the unit
    /// it moves; castling is 0.
    pub fn see_capture(&self, mv: Move) -> i32 {
        if mv.is_castling() {
            return 0;
        }
        let Some(mover) = self.piece_at(mv.from()) else {
            return 0;
        };
        let to = mv.to();
        let taken = if mv.is_en_passant() {
            Square::new(to.file(), mv.from().rank())
        } else {
            to
        };
        let captured = self
            .piece_at(taken)
            .map_or(0, |piece| material_value(piece.role));
        let landed = mv.promotion().unwrap_or(mover.role);
        let promoted = mv
            .promotion()
            .map_or(0, |role| material_value(role) - material_value(Role::Pawn));
        let occupied = (self.occupied() - mv.from().to_set() - taken.to_set()) | to.to_set();
        captured + promoted
            - swap(
                &Placement::new(self),
                to,
                !mover.colour,
                material_value(landed),
                occupied,
            )
    }
}

/// The `exchange` block of the side to move in `position`, over its `legal`
/// moves.
pub(super) fn exchange_facts(position: &Position, legal: &MoveList) -> ExchangeFacts {
    let mut facts = ExchangeFacts::default();
    let mut best: Option<i32> = None;
    for &mv in legal.as_slice() {
        if !mv.is_capture() {
            continue;
        }
        let see = position.see_capture(mv);
        best = Some(best.map_or(see, |value: i32| value.max(see)));
        match see.cmp(&0) {
            Ordering::Greater => {
                facts.see_positive_capture_count += 1;
                facts.see_positive_total += see;
            }
            Ordering::Equal => facts.see_equal_capture_count += 1,
            Ordering::Less => {}
        }
    }
    facts.see_best_capture = best.unwrap_or(0);
    facts
}
