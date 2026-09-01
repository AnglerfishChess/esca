//! Board vocabulary: colours, roles, pieces, squares and sets of squares.

use core::fmt;
use core::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Sub, SubAssign,
};
use core::str::FromStr;

use cozy_chess as cc;

/// A player: White or Black.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Colour {
    /// The side that moves first in the starting array.
    White,
    /// The side that moves second in the starting array.
    Black,
}

impl Colour {
    /// Both colours, White first.
    pub const ALL: [Colour; 2] = [Colour::White, Colour::Black];

    /// 0 for White, 1 for Black.
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// The FEN side-to-move letter: `w` or `b`.
    #[inline]
    pub const fn to_char(self) -> char {
        match self {
            Colour::White => 'w',
            Colour::Black => 'b',
        }
    }

    /// The colour written `w` or `b`.
    #[inline]
    pub const fn from_char(c: char) -> Option<Colour> {
        match c {
            'w' => Some(Colour::White),
            'b' => Some(Colour::Black),
            _ => None,
        }
    }

    #[inline]
    pub(crate) const fn to_cozy(self) -> cc::Color {
        match self {
            Colour::White => cc::Color::White,
            Colour::Black => cc::Color::Black,
        }
    }

    #[inline]
    pub(crate) const fn from_cozy(c: cc::Color) -> Colour {
        match c {
            cc::Color::White => Colour::White,
            cc::Color::Black => Colour::Black,
        }
    }
}

impl Not for Colour {
    type Output = Colour;

    #[inline]
    fn not(self) -> Colour {
        match self {
            Colour::White => Colour::Black,
            Colour::Black => Colour::White,
        }
    }
}

impl fmt::Display for Colour {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Colour::White => "w",
            Colour::Black => "b",
        })
    }
}

/// What a piece is, without its colour.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Role {
    /// Pawn.
    Pawn,
    /// Knight.
    Knight,
    /// Bishop.
    Bishop,
    /// Rook.
    Rook,
    /// Queen.
    Queen,
    /// King.
    King,
}

impl Role {
    /// Every role, in ascending conventional value.
    pub const ALL: [Role; 6] = [
        Role::Pawn,
        Role::Knight,
        Role::Bishop,
        Role::Rook,
        Role::Queen,
        Role::King,
    ];

    /// 0 for a pawn through 5 for a king.
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// The lower-case FEN letter.
    #[inline]
    pub const fn to_char(self) -> char {
        match self {
            Role::Pawn => 'p',
            Role::Knight => 'n',
            Role::Bishop => 'b',
            Role::Rook => 'r',
            Role::Queen => 'q',
            Role::King => 'k',
        }
    }

    /// The upper-case letter SAN names the role by. A pawn has none.
    #[inline]
    pub const fn to_san_char(self) -> Option<char> {
        match self {
            Role::Pawn => None,
            Role::Knight => Some('N'),
            Role::Bishop => Some('B'),
            Role::Rook => Some('R'),
            Role::Queen => Some('Q'),
            Role::King => Some('K'),
        }
    }

    /// The role a FEN letter names, of either case.
    #[inline]
    pub const fn from_char(c: char) -> Option<Role> {
        match c.to_ascii_lowercase() {
            'p' => Some(Role::Pawn),
            'n' => Some(Role::Knight),
            'b' => Some(Role::Bishop),
            'r' => Some(Role::Rook),
            'q' => Some(Role::Queen),
            'k' => Some(Role::King),
            _ => None,
        }
    }

    #[inline]
    pub(crate) const fn to_cozy(self) -> cc::Piece {
        match self {
            Role::Pawn => cc::Piece::Pawn,
            Role::Knight => cc::Piece::Knight,
            Role::Bishop => cc::Piece::Bishop,
            Role::Rook => cc::Piece::Rook,
            Role::Queen => cc::Piece::Queen,
            Role::King => cc::Piece::King,
        }
    }

    #[inline]
    pub(crate) const fn from_cozy(p: cc::Piece) -> Role {
        match p {
            cc::Piece::Pawn => Role::Pawn,
            cc::Piece::Knight => Role::Knight,
            cc::Piece::Bishop => Role::Bishop,
            cc::Piece::Rook => Role::Rook,
            cc::Piece::Queen => Role::Queen,
            cc::Piece::King => Role::King,
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Role::Pawn => "p",
            Role::Knight => "n",
            Role::Bishop => "b",
            Role::Rook => "r",
            Role::Queen => "q",
            Role::King => "k",
        })
    }
}

/// A role plus a colour.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Piece {
    /// What the piece is.
    pub role: Role,
    /// Whose it is.
    pub colour: Colour,
}

impl Piece {
    /// A piece of `role` belonging to `colour`.
    #[inline]
    pub const fn new(role: Role, colour: Colour) -> Piece {
        Piece { role, colour }
    }

    /// The FEN letter: upper case for White, lower case for Black.
    #[inline]
    pub const fn to_char(self) -> char {
        match self.colour {
            Colour::White => self.role.to_char().to_ascii_uppercase(),
            Colour::Black => self.role.to_char(),
        }
    }

    /// The piece a FEN letter names.
    #[inline]
    pub const fn from_char(c: char) -> Option<Piece> {
        let colour = if c.is_ascii_uppercase() {
            Colour::White
        } else {
            Colour::Black
        };
        match Role::from_char(c) {
            Some(role) => Some(Piece { role, colour }),
            None => None,
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

/// A column of the board, a to h.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum File {
    /// File a.
    A,
    /// File b.
    B,
    /// File c.
    C,
    /// File d.
    D,
    /// File e.
    E,
    /// File f.
    F,
    /// File g.
    G,
    /// File h.
    H,
}

impl File {
    /// Every file, a to h.
    pub const ALL: [File; 8] = [
        File::A,
        File::B,
        File::C,
        File::D,
        File::E,
        File::F,
        File::G,
        File::H,
    ];

    /// 0 for a through 7 for h.
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// The file at `index`, counting a as 0.
    #[inline]
    pub const fn from_index(index: usize) -> Option<File> {
        if index < 8 {
            Some(File::ALL[index])
        } else {
            None
        }
    }

    /// The lower-case letter.
    #[inline]
    pub const fn to_char(self) -> char {
        (b'a' + self as u8) as char
    }

    /// The file a letter of either case names.
    #[inline]
    pub const fn from_char(c: char) -> Option<File> {
        File::from_index((c.to_ascii_lowercase() as u8).wrapping_sub(b'a') as usize)
    }

    #[inline]
    pub(crate) const fn to_cozy(self) -> cc::File {
        cc::File::index_const(self as usize)
    }

    #[inline]
    pub(crate) const fn from_cozy(f: cc::File) -> File {
        File::ALL[f as usize]
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

/// A row of the board, 1 to 8.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Rank {
    /// Rank 1.
    First,
    /// Rank 2.
    Second,
    /// Rank 3.
    Third,
    /// Rank 4.
    Fourth,
    /// Rank 5.
    Fifth,
    /// Rank 6.
    Sixth,
    /// Rank 7.
    Seventh,
    /// Rank 8.
    Eighth,
}

impl Rank {
    /// Every rank, 1 to 8.
    pub const ALL: [Rank; 8] = [
        Rank::First,
        Rank::Second,
        Rank::Third,
        Rank::Fourth,
        Rank::Fifth,
        Rank::Sixth,
        Rank::Seventh,
        Rank::Eighth,
    ];

    /// 0 for rank 1 through 7 for rank 8.
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// The rank at `index`, counting rank 1 as 0.
    #[inline]
    pub const fn from_index(index: usize) -> Option<Rank> {
        if index < 8 {
            Some(Rank::ALL[index])
        } else {
            None
        }
    }

    /// The digit.
    #[inline]
    pub const fn to_char(self) -> char {
        (b'1' + self as u8) as char
    }

    /// The rank a digit names.
    #[inline]
    pub const fn from_char(c: char) -> Option<Rank> {
        Rank::from_index((c as u8).wrapping_sub(b'1') as usize)
    }

    /// The same rank counted from `colour`'s own back rank.
    #[inline]
    pub const fn relative_to(self, colour: Colour) -> Rank {
        match colour {
            Colour::White => self,
            Colour::Black => Rank::ALL[7 - self as usize],
        }
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

/// One of the 64 cells, indexed 0 to 63 with a1 = 0 and h8 = 63.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Square(u8);

macro_rules! square_consts {
    ($($name:ident = $index:expr,)*) => {
        impl Square {
            $(
                #[doc = concat!("The ", stringify!($name), " square.")]
                pub const $name: Square = Square($index);
            )*

            /// Every square, a1 to h8.
            pub const ALL: [Square; 64] = [$(Square::$name),*];
        }
    };
}

square_consts! {
    A1 = 0, B1 = 1, C1 = 2, D1 = 3, E1 = 4, F1 = 5, G1 = 6, H1 = 7,
    A2 = 8, B2 = 9, C2 = 10, D2 = 11, E2 = 12, F2 = 13, G2 = 14, H2 = 15,
    A3 = 16, B3 = 17, C3 = 18, D3 = 19, E3 = 20, F3 = 21, G3 = 22, H3 = 23,
    A4 = 24, B4 = 25, C4 = 26, D4 = 27, E4 = 28, F4 = 29, G4 = 30, H4 = 31,
    A5 = 32, B5 = 33, C5 = 34, D5 = 35, E5 = 36, F5 = 37, G5 = 38, H5 = 39,
    A6 = 40, B6 = 41, C6 = 42, D6 = 43, E6 = 44, F6 = 45, G6 = 46, H6 = 47,
    A7 = 48, B7 = 49, C7 = 50, D7 = 51, E7 = 52, F7 = 53, G7 = 54, H7 = 55,
    A8 = 56, B8 = 57, C8 = 58, D8 = 59, E8 = 60, F8 = 61, G8 = 62, H8 = 63,
}

impl Square {
    /// The square where `file` and `rank` cross.
    #[inline]
    pub const fn new(file: File, rank: Rank) -> Square {
        Square(((rank as u8) << 3) | file as u8)
    }

    /// The square at `index`, a1 = 0 and h8 = 63.
    #[inline]
    pub const fn from_index(index: usize) -> Option<Square> {
        if index < 64 {
            Some(Square(index as u8))
        } else {
            None
        }
    }

    /// The index, a1 = 0 and h8 = 63.
    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    /// The file this square is on.
    #[inline]
    pub const fn file(self) -> File {
        File::ALL[(self.0 & 7) as usize]
    }

    /// The rank this square is on.
    #[inline]
    pub const fn rank(self) -> Rank {
        Rank::ALL[(self.0 >> 3) as usize]
    }

    /// The same file, on the rank mirrored across the middle of the board.
    #[inline]
    pub const fn flip_rank(self) -> Square {
        Square::new(self.file(), Rank::ALL[7 - (self.0 >> 3) as usize])
    }

    /// Whether the square is dark. a1 is dark.
    #[inline]
    pub const fn is_dark(self) -> bool {
        (self.0 & 7) % 2 == (self.0 >> 3) % 2
    }

    /// This square alone.
    #[inline]
    pub const fn to_set(self) -> SquareSet {
        SquareSet(1u64 << self.0)
    }

    #[inline]
    pub(crate) const fn to_cozy(self) -> cc::Square {
        cc::Square::index_const(self.0 as usize)
    }

    #[inline]
    pub(crate) const fn from_cozy(s: cc::Square) -> Square {
        Square(s as u8)
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

/// The value was not a square name such as `e4`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SquareParseError;

impl fmt::Display for SquareParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("not a square name")
    }
}

impl std::error::Error for SquareParseError {}

impl FromStr for Square {
    type Err = SquareParseError;

    fn from_str(s: &str) -> Result<Square, SquareParseError> {
        let bytes = s.as_bytes();
        if bytes.len() != 2 {
            return Err(SquareParseError);
        }
        let file = File::from_char(bytes[0] as char).ok_or(SquareParseError)?;
        let rank = Rank::from_char(bytes[1] as char).ok_or(SquareParseError)?;
        Ok(Square::new(file, rank))
    }
}

/// A set of squares, one bit per square.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SquareSet(u64);

impl SquareSet {
    /// No squares.
    pub const EMPTY: SquareSet = SquareSet(0);
    /// All 64 squares.
    pub const FULL: SquareSet = SquareSet(!0);
    /// The 32 dark squares; a1 is dark.
    pub const DARK: SquareSet = SquareSet(0xAA55_AA55_AA55_AA55);
    /// The 32 light squares.
    pub const LIGHT: SquareSet = SquareSet(0x55AA_55AA_55AA_55AA);

    /// The set whose bit *i* is set iff square *i* is a member.
    #[inline]
    pub const fn from_bits(bits: u64) -> SquareSet {
        SquareSet(bits)
    }

    /// The membership bits, bit *i* for square *i*.
    #[inline]
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// Whether `square` is a member.
    #[inline]
    pub const fn contains(self, square: Square) -> bool {
        self.0 & square.to_set().0 != 0
    }

    /// Adds `square`.
    #[inline]
    pub fn insert(&mut self, square: Square) {
        self.0 |= square.to_set().0;
    }

    /// Removes `square`.
    #[inline]
    pub fn remove(&mut self, square: Square) {
        self.0 &= !square.to_set().0;
    }

    /// How many squares are members.
    #[inline]
    pub const fn len(self) -> u32 {
        self.0.count_ones()
    }

    /// Whether there are no members.
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Whether every member of `self` is a member of `other`.
    #[inline]
    pub const fn is_subset(self, other: SquareSet) -> bool {
        self.0 & !other.0 == 0
    }

    /// The lowest-indexed member.
    #[inline]
    pub const fn first(self) -> Option<Square> {
        if self.0 == 0 {
            None
        } else {
            Square::from_index(self.0.trailing_zeros() as usize)
        }
    }

    #[inline]
    pub(crate) const fn from_cozy(b: cc::BitBoard) -> SquareSet {
        SquareSet(b.0)
    }
}

impl fmt::Debug for SquareSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SquareSet(")?;
        let mut first = true;
        for square in *self {
            if !first {
                f.write_str(" ")?;
            }
            write!(f, "{square}")?;
            first = false;
        }
        f.write_str(")")
    }
}

impl BitAnd for SquareSet {
    type Output = SquareSet;

    #[inline]
    fn bitand(self, rhs: SquareSet) -> SquareSet {
        SquareSet(self.0 & rhs.0)
    }
}

impl BitOr for SquareSet {
    type Output = SquareSet;

    #[inline]
    fn bitor(self, rhs: SquareSet) -> SquareSet {
        SquareSet(self.0 | rhs.0)
    }
}

impl BitXor for SquareSet {
    type Output = SquareSet;

    #[inline]
    fn bitxor(self, rhs: SquareSet) -> SquareSet {
        SquareSet(self.0 ^ rhs.0)
    }
}

impl Sub for SquareSet {
    type Output = SquareSet;

    #[inline]
    fn sub(self, rhs: SquareSet) -> SquareSet {
        SquareSet(self.0 & !rhs.0)
    }
}

impl Not for SquareSet {
    type Output = SquareSet;

    #[inline]
    fn not(self) -> SquareSet {
        SquareSet(!self.0)
    }
}

impl BitAndAssign for SquareSet {
    #[inline]
    fn bitand_assign(&mut self, rhs: SquareSet) {
        self.0 &= rhs.0;
    }
}

impl BitOrAssign for SquareSet {
    #[inline]
    fn bitor_assign(&mut self, rhs: SquareSet) {
        self.0 |= rhs.0;
    }
}

impl BitXorAssign for SquareSet {
    #[inline]
    fn bitxor_assign(&mut self, rhs: SquareSet) {
        self.0 ^= rhs.0;
    }
}

impl SubAssign for SquareSet {
    #[inline]
    fn sub_assign(&mut self, rhs: SquareSet) {
        self.0 &= !rhs.0;
    }
}

impl FromIterator<Square> for SquareSet {
    fn from_iter<I: IntoIterator<Item = Square>>(iter: I) -> SquareSet {
        let mut set = SquareSet::EMPTY;
        for square in iter {
            set.insert(square);
        }
        set
    }
}

/// Iterator over a [`SquareSet`], in ascending square index.
#[derive(Clone, Debug)]
pub struct SquareSetIter(u64);

impl Iterator for SquareSetIter {
    type Item = Square;

    #[inline]
    fn next(&mut self) -> Option<Square> {
        if self.0 == 0 {
            return None;
        }
        let index = self.0.trailing_zeros() as usize;
        self.0 &= self.0 - 1;
        Square::from_index(index)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.0.count_ones() as usize;
        (n, Some(n))
    }
}

impl ExactSizeIterator for SquareSetIter {}

impl IntoIterator for SquareSet {
    type Item = Square;
    type IntoIter = SquareSetIter;

    #[inline]
    fn into_iter(self) -> SquareSetIter {
        SquareSetIter(self.0)
    }
}
