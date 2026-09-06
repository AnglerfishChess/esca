//! The shared pass over a position: attack sets, per-side placement, and the
//! square arithmetic every group is written in terms of.

use cozy_chess as cc;

use crate::position::Position;
use crate::types::{Colour, File, Rank, Role, Square, SquareSet};

use super::Side;

/// Conventional value: P=1, N=B=3, R=5, Q=9, a king 0.
#[inline]
pub(crate) const fn material_value(role: Role) -> i32 {
    match role {
        Role::Pawn => 1,
        Role::Knight | Role::Bishop => 3,
        Role::Rook => 5,
        Role::Queen => 9,
        Role::King => 0,
    }
}

/// Value order for comparisons only: P=1, N=B=3, R=5, Q=9, a king above all.
#[inline]
pub(crate) const fn order_value(role: Role) -> i32 {
    match role {
        Role::King => 100,
        other => material_value(other),
    }
}

/// The value a forked or hanging target contributes, the king counting as a
/// queen so that the [0, 9] scale holds.
#[inline]
pub(crate) const fn target_value(role: Role) -> i32 {
    match role {
        Role::King => 9,
        other => material_value(other),
    }
}

/// The squares a unit of `role` and `colour` on `square` attacks, sliders
/// stopping at the first square of `occupied`.
#[inline]
pub(crate) fn attacks_of(
    role: Role,
    square: Square,
    colour: Colour,
    occupied: SquareSet,
) -> SquareSet {
    let sq = square.to_cozy();
    let blockers = cc::BitBoard(occupied.bits());
    let set = match role {
        Role::Pawn => cc::get_pawn_attacks(sq, colour.to_cozy()),
        Role::Knight => cc::get_knight_moves(sq),
        Role::Bishop => cc::get_bishop_moves(sq, blockers),
        Role::Rook => cc::get_rook_moves(sq, blockers),
        Role::Queen => cc::get_bishop_moves(sq, blockers) | cc::get_rook_moves(sq, blockers),
        Role::King => cc::get_king_moves(sq),
    };
    SquareSet::from_bits(set.0)
}

/// The squares strictly between two squares on a common rank, file or
/// diagonal; empty when they share none.
#[inline]
pub(crate) fn between(from: Square, to: Square) -> SquareSet {
    SquareSet::from_bits(cc::get_between_rays(from.to_cozy(), to.to_cozy()).0)
}

/// Every square on the rank, file or diagonal two squares share; empty when
/// they share none.
#[inline]
pub(crate) fn line(from: Square, to: Square) -> SquareSet {
    SquareSet::from_bits(cc::get_line_rays(from.to_cozy(), to.to_cozy()).0)
}

/// The Chebyshev distance, the number of king moves between two squares.
#[inline]
pub(crate) fn distance(a: Square, b: Square) -> u32 {
    let files = a.file().index().abs_diff(b.file().index());
    let ranks = a.rank().index().abs_diff(b.rank().index());
    files.max(ranks) as u32
}

/// The 4×4 block c3–f6, the extended centre.
pub(crate) const EXTENDED_CENTRE: SquareSet = SquareSet::from_bits(0x0000_3C3C_3C3C_0000);
/// The four central squares d4, e4, d5, e5.
pub(crate) const CENTRE: SquareSet = SquareSet::from_bits(0x0000_0018_1800_0000);

/// A position's placement and attack sets, in side-relative terms.
///
/// Every `[_; 2]` is indexed by [`Side`]: index 0 is the side to move.
#[derive(Clone)]
pub(crate) struct Scan {
    /// The side to move.
    pub us: Colour,
    /// Every occupied square.
    pub occupied: SquareSet,
    /// Each side's units.
    pub units: [SquareSet; 2],
    /// Each side's units, by role.
    pub role_units: [[SquareSet; 6]; 2],
    /// Each side's king.
    pub kings: [Square; 2],
    /// The squares the unit on each square attacks; empty for an empty square.
    pub attacks_from: [SquareSet; 64],
    /// Each side's attack map, by role.
    pub by_role: [[SquareSet; 6]; 2],
    /// Each side's whole attack map.
    pub by: [SquareSet; 2],
    /// Whether each side may still castle either way.
    pub castling: [bool; 2],
}

impl Scan {
    /// The scan of `position`.
    pub fn new(position: &Position) -> Scan {
        let us = position.side_to_move();
        let occupied = position.occupied();
        let mut scan = Scan {
            us,
            occupied,
            units: [SquareSet::EMPTY; 2],
            role_units: [[SquareSet::EMPTY; 6]; 2],
            kings: [position.king_of(us), position.king_of(!us)],
            attacks_from: [SquareSet::EMPTY; 64],
            by_role: [[SquareSet::EMPTY; 6]; 2],
            by: [SquareSet::EMPTY; 2],
            castling: [
                position.castling_rights().any(us),
                position.castling_rights().any(!us),
            ],
        };
        for side in Side::ALL {
            let colour = scan.colour(side);
            scan.units[side.index()] = position.by_colour(colour);
            for role in Role::ALL {
                let set = position.by_colour(colour) & position.by_role(role);
                scan.role_units[side.index()][role.index()] = set;
                let mut attacks = SquareSet::EMPTY;
                for square in set {
                    let from = attacks_of(role, square, colour, occupied);
                    scan.attacks_from[square.index()] = from;
                    attacks |= from;
                }
                scan.by_role[side.index()][role.index()] = attacks;
                scan.by[side.index()] |= attacks;
            }
        }
        scan
    }

    /// The colour playing as `side`.
    #[inline]
    pub fn colour(&self, side: Side) -> Colour {
        match side {
            Side::Us => self.us,
            Side::Them => !self.us,
        }
    }

    /// The units of `side` that attack `square`.
    #[inline]
    pub fn attackers_of(&self, square: Square, side: Side) -> SquareSet {
        attackers(
            square,
            self.colour(side),
            &self.role_units[side.index()],
            self.occupied,
        )
    }

    /// The rank of `square` counted from `side`'s own back rank, from 1.
    #[inline]
    pub fn relative_rank(&self, square: Square, side: Side) -> u32 {
        square.rank().relative_to(self.colour(side)).index() as u32 + 1
    }

    /// The square on `file` at `side`'s relative rank `rank`, from 1.
    #[inline]
    pub fn relative_square(&self, file: File, rank: u32, side: Side) -> Square {
        let rank = Rank::from_index(rank as usize - 1).expect("a relative rank is 1 to 8");
        Square::new(file, rank.relative_to(self.colour(side)))
    }

    /// The 32 squares that are light in the mover's view.
    #[inline]
    pub fn view_light(&self) -> SquareSet {
        match self.us {
            Colour::White => SquareSet::LIGHT,
            Colour::Black => SquareSet::DARK,
        }
    }

    /// The 32 squares that are dark in the mover's view.
    #[inline]
    pub fn view_dark(&self) -> SquareSet {
        !self.view_light()
    }

    /// The half of the board `side` starts on.
    pub fn own_half(&self, side: Side) -> SquareSet {
        let mut set = SquareSet::EMPTY;
        for square in Square::ALL {
            if self.relative_rank(square, side) <= 4 {
                set.insert(square);
            }
        }
        set
    }
}

/// The units of `colour` that attack `square`, given that colour's placement
/// by role and the occupancy the sliders see.
pub(crate) fn attackers(
    square: Square,
    colour: Colour,
    role_units: &[SquareSet; 6],
    occupied: SquareSet,
) -> SquareSet {
    let bishops = role_units[Role::Bishop.index()] | role_units[Role::Queen.index()];
    let rooks = role_units[Role::Rook.index()] | role_units[Role::Queen.index()];
    // A pawn of `colour` attacks `square` from exactly the squares a pawn of
    // the other colour standing on `square` would attack.
    (attacks_of(Role::Pawn, square, !colour, occupied) & role_units[Role::Pawn.index()])
        | (attacks_of(Role::Knight, square, colour, occupied) & role_units[Role::Knight.index()])
        | (attacks_of(Role::King, square, colour, occupied) & role_units[Role::King.index()])
        | (attacks_of(Role::Bishop, square, colour, occupied) & bishops)
        | (attacks_of(Role::Rook, square, colour, occupied) & rooks)
}
