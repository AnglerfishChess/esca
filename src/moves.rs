//! Moves and the fixed-capacity list they are generated into.

use core::fmt;
use core::ops::Deref;

use cozy_chess as cc;

use crate::types::{Role, Square};

/// The most moves any list has to hold. The largest legal move count of a
/// position is 218.
pub const MAX_MOVES: usize = 256;

const FLAG_CAPTURE: u8 = 1;
const FLAG_EN_PASSANT: u8 = 2;
const FLAG_CASTLING: u8 = 4;

/// What kind of move this is, by the first matching case in the order
/// castling, en passant, promotion, capture, quiet.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MoveKind {
    /// Neither a capture nor a promotion.
    Quiet,
    /// The destination holds an enemy unit.
    Capture,
    /// A pawn capture onto the en-passant square.
    EnPassant,
    /// King and rook move together.
    Castling,
    /// A pawn reaches its relative rank 8, capturing or not.
    Promotion,
}

/// One action of one side: origin, destination, promotion role and kind.
///
/// Castling is stored king-to-rook, so the destination is the rook's own
/// square. Moves are produced by move generation and by move-text parsing;
/// two moves are equal when their origins, destinations and promotions are.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Move {
    from: Square,
    to: Square,
    promotion: Option<Role>,
    flags: u8,
}

impl Move {
    pub(crate) fn new(from: Square, to: Square, promotion: Option<Role>, kind: MoveKind) -> Move {
        let flags = match kind {
            MoveKind::Quiet | MoveKind::Promotion => 0,
            MoveKind::Capture => FLAG_CAPTURE,
            MoveKind::EnPassant => FLAG_CAPTURE | FLAG_EN_PASSANT,
            MoveKind::Castling => FLAG_CASTLING,
        };
        Move {
            from,
            to,
            promotion,
            flags,
        }
    }

    pub(crate) fn with_capture(mut self, capture: bool) -> Move {
        if capture {
            self.flags |= FLAG_CAPTURE;
        } else {
            self.flags &= !FLAG_CAPTURE;
        }
        self
    }

    pub(crate) fn to_cozy(self) -> cc::Move {
        cc::Move {
            from: self.from.to_cozy(),
            to: self.to.to_cozy(),
            promotion: self.promotion.map(Role::to_cozy),
        }
    }

    /// The square the moving unit starts on; for castling, the king's.
    #[inline]
    pub fn from(&self) -> Square {
        self.from
    }

    /// The square the moving unit ends on; for castling, the rook's own
    /// square, which is unambiguous in every variant.
    #[inline]
    pub fn to(&self) -> Square {
        self.to
    }

    /// The role a promoting pawn becomes.
    #[inline]
    pub fn promotion(&self) -> Option<Role> {
        self.promotion
    }

    /// The kind, by the precedence [`MoveKind`] documents.
    #[inline]
    pub fn kind(&self) -> MoveKind {
        if self.is_castling() {
            MoveKind::Castling
        } else if self.is_en_passant() {
            MoveKind::EnPassant
        } else if self.promotion.is_some() {
            MoveKind::Promotion
        } else if self.is_capture() {
            MoveKind::Capture
        } else {
            MoveKind::Quiet
        }
    }

    /// Whether an enemy unit is removed. True for capturing promotions and
    /// for en passant.
    #[inline]
    pub fn is_capture(&self) -> bool {
        self.flags & FLAG_CAPTURE != 0
    }

    /// Whether this is a castling move.
    #[inline]
    pub fn is_castling(&self) -> bool {
        self.flags & FLAG_CASTLING != 0
    }

    /// Whether this is an en-passant capture.
    #[inline]
    pub fn is_en_passant(&self) -> bool {
        self.flags & FLAG_EN_PASSANT != 0
    }
}

impl Default for Move {
    /// A placeholder that is not a playable move: a1 to a1, quiet.
    fn default() -> Move {
        Move::new(Square::A1, Square::A1, None, MoveKind::Quiet)
    }
}

impl fmt::Display for Move {
    /// Origin, destination and promotion role, castling king-to-rook. The
    /// spelling a variant asks for comes from `Variant::move_to_uci`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.from, self.to)?;
        if let Some(role) = self.promotion {
            write!(f, "{role}")?;
        }
        Ok(())
    }
}

/// An ordered list of at most [`MAX_MOVES`] items, stored inline.
#[derive(Clone)]
pub struct MoveList<T = Move> {
    items: [T; MAX_MOVES],
    len: usize,
}

impl<T: Copy + Default> MoveList<T> {
    /// An empty list.
    pub fn new() -> MoveList<T> {
        MoveList {
            items: [T::default(); MAX_MOVES],
            len: 0,
        }
    }

    /// Empties the list.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Appends `item`.
    ///
    /// # Panics
    /// If the list already holds [`MAX_MOVES`] items.
    pub fn push(&mut self, item: T) {
        assert!(self.len < MAX_MOVES, "move list overflow");
        self.items[self.len] = item;
        self.len += 1;
    }

    /// The items, in the order they were pushed.
    pub fn as_slice(&self) -> &[T] {
        &self.items[..self.len]
    }

    /// The items, mutably.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.items[..self.len]
    }

    /// How many items the list holds.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the list is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<T: Copy + Default> Default for MoveList<T> {
    fn default() -> MoveList<T> {
        MoveList::new()
    }
}

impl<T: Copy + Default> Deref for MoveList<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T: Copy + Default + fmt::Debug> fmt::Debug for MoveList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}

impl<T: Copy + Default + PartialEq> PartialEq for MoveList<T> {
    fn eq(&self, other: &MoveList<T>) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Copy + Default + Eq> Eq for MoveList<T> {}

impl<'a, T: Copy + Default> IntoIterator for &'a MoveList<T> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> core::slice::Iter<'a, T> {
        self.as_slice().iter()
    }
}

impl<T: Copy + Default> FromIterator<T> for MoveList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> MoveList<T> {
        let mut list = MoveList::new();
        for item in iter {
            list.push(item);
        }
        list
    }
}
