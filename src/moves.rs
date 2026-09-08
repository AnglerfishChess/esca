//! Moves and the fixed-capacity list they are generated into.

use core::fmt;
use core::mem::MaybeUninit;
use core::ops::Deref;
use core::str::FromStr;

use cozy_chess as cc;

use crate::types::{ParseError, Role, Square};
use crate::variant::CastlingOutput;

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
/// two moves are equal when their origins, destinations and promotions are,
/// whether or not both were told what they capture.
#[derive(Clone, Copy)]
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

    /// The UCI text, with castling spelled as `output` asks.
    ///
    /// [`CastlingOutput::KingTwoSquares`] assumes classic geometry, so a
    /// Chess960 game asks its variant instead, through
    /// [`Variant::move_to_uci`](crate::Variant::move_to_uci) or
    /// [`Game::move_to_uci`](crate::Game::move_to_uci), which never spells a
    /// castling a Chess960 engine would misread.
    pub fn uci(&self, output: CastlingOutput) -> String {
        let to = match output {
            CastlingOutput::KingTwoSquares if self.is_castling() => {
                let file = if self.to.file() > self.from.file() {
                    crate::types::File::G
                } else {
                    crate::types::File::C
                };
                Square::new(file, self.from.rank())
            }
            _ => self.to,
        };
        let mut text = format!("{}{to}", self.from);
        if let Some(role) = self.promotion {
            text.push(role.to_char());
        }
        text
    }
}

/// Origin, destination and promotion role only: the flags say what the move
/// did in one position, and the same move is the same move in another.
impl PartialEq for Move {
    fn eq(&self, other: &Move) -> bool {
        self.from == other.from && self.to == other.to && self.promotion == other.promotion
    }
}

impl Eq for Move {}

impl core::hash::Hash for Move {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.from.hash(state);
        self.to.hash(state);
        self.promotion.hash(state);
    }
}

impl fmt::Debug for Move {
    /// The UCI text, castling king-to-rook.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.uci(CastlingOutput::KingToRook))
    }
}

impl FromStr for Move {
    type Err = ParseError;

    /// Origin, destination and promotion letter, four or five characters.
    ///
    /// A move read this way carries no kind: only a position says what it
    /// captures and whether it castles, so a move to play is parsed with
    /// [`Variant::move_from_uci`](crate::Variant::move_from_uci) or played
    /// with [`Game::play_uci`](crate::Game::play_uci).
    fn from_str(s: &str) -> Result<Move, ParseError> {
        let fail = || ParseError::new("move", s);
        let bytes = s.as_bytes();
        if bytes.len() != 4 && bytes.len() != 5 {
            return Err(fail());
        }
        let from = Square::from_name(&s[0..2]).ok_or_else(fail)?;
        let to = Square::from_name(&s[2..4]).ok_or_else(fail)?;
        let promotion = match bytes.get(4) {
            None => None,
            Some(&c) => match Role::from_char(c as char) {
                Some(role) if role != Role::Pawn && role != Role::King => Some(role),
                _ => return Err(fail()),
            },
        };
        Ok(Move {
            from,
            to,
            promotion,
            flags: 0,
        })
    }
}

/// An ordered list of at most [`MAX_MOVES`] items, stored inline.
///
/// Derefs to `[T]`, so the slice methods — `iter`, `first`, `contains`,
/// `sort_by_key` — are all there.
pub struct MoveList<T = Move> {
    /// The first `len` are initialised; the rest are never read.
    items: [MaybeUninit<T>; MAX_MOVES],
    len: usize,
}

impl<T: Copy> MoveList<T> {
    /// An empty list.
    pub fn new() -> MoveList<T> {
        MoveList {
            items: [const { MaybeUninit::uninit() }; MAX_MOVES],
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
        self.items[self.len] = MaybeUninit::new(item);
        self.len += 1;
    }

    /// The items, in the order they were pushed.
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: `push` initialised every item below `len`, and `T: Copy`
        // leaves nothing to drop, so no item below `len` is ever stale.
        unsafe { core::slice::from_raw_parts(self.items.as_ptr().cast::<T>(), self.len) }
    }

    /// The items, mutably.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: as `as_slice`.
        unsafe { core::slice::from_raw_parts_mut(self.items.as_mut_ptr().cast::<T>(), self.len) }
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

impl<T: Copy> Clone for MoveList<T> {
    fn clone(&self) -> MoveList<T> {
        MoveList {
            items: self.items,
            len: self.len,
        }
    }
}

impl<T: Copy> Default for MoveList<T> {
    fn default() -> MoveList<T> {
        MoveList::new()
    }
}

impl<T: Copy> Deref for MoveList<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T: Copy + fmt::Debug> fmt::Debug for MoveList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}

impl<T: Copy + PartialEq> PartialEq for MoveList<T> {
    fn eq(&self, other: &MoveList<T>) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Copy + Eq> Eq for MoveList<T> {}

impl<'a, T: Copy> IntoIterator for &'a MoveList<T> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> core::slice::Iter<'a, T> {
        self.as_slice().iter()
    }
}

impl<T: Copy> FromIterator<T> for MoveList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> MoveList<T> {
        let mut list = MoveList::new();
        for item in iter {
            list.push(item);
        }
        list
    }
}
