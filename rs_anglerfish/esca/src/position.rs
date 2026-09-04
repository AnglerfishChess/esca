//! The position: placement and state, with no rules attached.

use core::fmt;

use cozy_chess as cc;

use crate::error::FenError;
use crate::fen;
use crate::moves::Move;
use crate::types::{Colour, File, Piece, Rank, Role, Square, SquareSet};

/// A Zobrist key.
///
/// Valid as an identity within one process run: the constants it is built
/// from are fixed for the run, not across runs, so a key is not a value to
/// store or to send.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Key(u64);

impl Key {
    /// The key as a number.
    #[inline]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

/// An evaluation of a position. Positive favours the side to move; `Mate(n)`
/// is a forced mate in *n* moves, negative when it is against the side to
/// move.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Score {
    /// Centipawns.
    Cp(i32),
    /// Moves to a forced mate.
    Mate(i32),
}

impl fmt::Display for Score {
    /// The UCI spelling: `cp 25`, `mate -3`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Score::Cp(cp) => write!(f, "cp {cp}"),
            Score::Mate(n) => write!(f, "mate {n}"),
        }
    }
}

/// Which castlings remain available, each named by its rook's starting file.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct CastlingRights {
    /// Indexed by colour, then by 0 for short and 1 for long.
    files: [[Option<File>; 2]; 2],
}

impl CastlingRights {
    /// No rights at all.
    pub const EMPTY: CastlingRights = CastlingRights {
        files: [[None; 2]; 2],
    };

    /// The rights of a classic starting array.
    pub const CLASSIC: CastlingRights = CastlingRights {
        files: [[Some(File::H), Some(File::A)]; 2],
    };

    /// The file of the rook `colour` may castle short with.
    #[inline]
    pub fn short(&self, colour: Colour) -> Option<File> {
        self.files[colour.index()][0]
    }

    /// The file of the rook `colour` may castle long with.
    #[inline]
    pub fn long(&self, colour: Colour) -> Option<File> {
        self.files[colour.index()][1]
    }

    /// Whether `colour` may still castle either way.
    #[inline]
    pub fn any(&self, colour: Colour) -> bool {
        self.short(colour).is_some() || self.long(colour).is_some()
    }

    /// Whether neither side may castle.
    #[inline]
    pub fn is_empty(&self) -> bool {
        !self.any(Colour::White) && !self.any(Colour::Black)
    }

    /// The FEN castling field: `KQkq` when every remaining right names a
    /// classic rook file, the Shredder form `AHah` otherwise, `-` when there
    /// are none.
    pub fn to_fen_field(&self) -> String {
        if self.is_empty() {
            return "-".to_string();
        }
        let classic = Colour::ALL.iter().all(|&colour| {
            self.short(colour).is_none_or(|f| f == File::H)
                && self.long(colour).is_none_or(|f| f == File::A)
        });
        let mut out = String::with_capacity(4);
        for &colour in &Colour::ALL {
            for (side, letter) in [(0usize, 'k'), (1usize, 'q')] {
                if let Some(file) = self.files[colour.index()][side] {
                    let c = if classic { letter } else { file.to_char() };
                    out.push(match colour {
                        Colour::White => c.to_ascii_uppercase(),
                        Colour::Black => c,
                    });
                }
            }
        }
        out
    }

    pub(crate) fn set(&mut self, colour: Colour, short: bool, file: Option<File>) {
        self.files[colour.index()][usize::from(!short)] = file;
    }

    pub(crate) fn to_cozy(self) -> [cc::CastleRights; 2] {
        Colour::ALL.map(|colour| cc::CastleRights {
            short: self.short(colour).map(File::to_cozy),
            long: self.long(colour).map(File::to_cozy),
        })
    }

    fn from_cozy(board: &cc::Board) -> CastlingRights {
        let mut rights = CastlingRights::EMPTY;
        for &colour in &Colour::ALL {
            let cozy = board.castle_rights(colour.to_cozy());
            rights.set(colour, true, cozy.short.map(File::from_cozy));
            rights.set(colour, false, cozy.long.map(File::from_cozy));
        }
        rights
    }
}

impl fmt::Display for CastlingRights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_fen_field())
    }
}

/// An immutable, variant-agnostic snapshot: placement, side to move, castling
/// rights, en-passant square and clocks.
///
/// Every question that needs rules is asked of a `Variant` or of a `Game`.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Position {
    board: cc::Board,
    halfmove_clock: u32,
    fullmove_number: u32,
    clocks_known: bool,
}

impl Position {
    pub(crate) fn from_parts(
        board: cc::Board,
        halfmove_clock: u32,
        fullmove_number: u32,
        clocks_known: bool,
    ) -> Position {
        // The clocks live here, uncapped; the board keeps its own at the
        // fixed values so that board identity is placement and state only.
        let mut board = board;
        board.set_halfmove_clock(0);
        board.set_fullmove_number(1);
        Position {
            board,
            halfmove_clock,
            fullmove_number,
            clocks_known,
        }
    }

    pub(crate) fn board(&self) -> &cc::Board {
        &self.board
    }

    /// Reads a six-field FEN, or a four-field one (an EPD without
    /// operations), which takes halfmove clock 0, full move 1 and leaves the
    /// clocks marked unknown.
    pub fn from_fen(text: &str) -> Result<Position, FenError> {
        fen::parse(text)
    }

    /// The six-field FEN.
    pub fn fen(&self) -> String {
        fen::format(self, true)
    }

    /// The first four FEN fields.
    pub fn epd(&self) -> String {
        fen::format(self, false)
    }

    /// Whose turn it is.
    #[inline]
    pub fn side_to_move(&self) -> Colour {
        Colour::from_cozy(self.board.side_to_move())
    }

    /// The unit standing on `square`.
    #[inline]
    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        let sq = square.to_cozy();
        let role = self.board.piece_on(sq)?;
        let colour = self.board.color_on(sq)?;
        Some(Piece::new(Role::from_cozy(role), Colour::from_cozy(colour)))
    }

    /// Every square holding a unit of `role`, of either colour.
    #[inline]
    pub fn by_role(&self, role: Role) -> SquareSet {
        SquareSet::from_cozy(self.board.pieces(role.to_cozy()))
    }

    /// Every square holding a unit of `colour`.
    #[inline]
    pub fn by_colour(&self, colour: Colour) -> SquareSet {
        SquareSet::from_cozy(self.board.colors(colour.to_cozy()))
    }

    /// Every square holding exactly `piece`.
    #[inline]
    pub fn by_piece(&self, piece: Piece) -> SquareSet {
        SquareSet::from_cozy(
            self.board
                .colored_pieces(piece.colour.to_cozy(), piece.role.to_cozy()),
        )
    }

    /// Every square holding a unit.
    #[inline]
    pub fn occupied(&self) -> SquareSet {
        SquareSet::from_cozy(self.board.occupied())
    }

    /// Where `colour`'s king stands.
    #[inline]
    pub fn king_of(&self, colour: Colour) -> Square {
        Square::from_cozy(self.board.king(colour.to_cozy()))
    }

    /// The castlings still available.
    #[inline]
    pub fn castling_rights(&self) -> CastlingRights {
        CastlingRights::from_cozy(&self.board)
    }

    /// The square a pawn skipped on the previous ply.
    #[inline]
    pub fn en_passant(&self) -> Option<Square> {
        let file = self.board.en_passant()?;
        let rank = Rank::Sixth.relative_to(self.side_to_move());
        Some(Square::new(File::from_cozy(file), rank))
    }

    /// Plies since the last capture or pawn move.
    #[inline]
    pub fn halfmove_clock(&self) -> u32 {
        self.halfmove_clock
    }

    /// The ordinal of the current full move, from 1.
    #[inline]
    pub fn fullmove_number(&self) -> u32 {
        self.fullmove_number
    }

    /// False when the position came from a four-field FEN.
    #[inline]
    pub fn clocks_known(&self) -> bool {
        self.clocks_known
    }

    /// Whether the side to move stands in check.
    #[inline]
    pub fn in_check(&self) -> bool {
        !self.board.checkers().is_empty()
    }

    /// The units giving check to the side to move.
    #[inline]
    pub fn checkers(&self) -> SquareSet {
        SquareSet::from_cozy(self.board.checkers())
    }

    /// Whether `mv` is legal here by the rules the built-in variants share.
    #[inline]
    pub(crate) fn allows(&self, mv: Move) -> bool {
        self.board.is_legal(mv.to_cozy())
    }

    /// The same placement with the other side to move and no en-passant
    /// square; `None` when the side to move stands in check.
    pub(crate) fn null_move(&self) -> Option<Position> {
        let board = self.board.null_move()?;
        Some(Position::from_parts(
            board,
            self.halfmove_clock,
            self.fullmove_number,
            self.clocks_known,
        ))
    }

    /// The Zobrist key: equal for equal placement, side to move, castling
    /// rights and en-passant square, and independent of the clocks.
    #[inline]
    pub fn key(&self) -> Key {
        Key(self.board.hash())
    }

    /// The key repetition is counted by: the en-passant square is part of it
    /// only when a pawn could legally take that way, as the FIDE rule says.
    pub(crate) fn repetition_key(&self) -> u64 {
        match self.en_passant() {
            Some(square) if self.en_passant_is_playable(square) => self.board.hash(),
            _ => self.board.hash_without_ep(),
        }
    }

    fn en_passant_is_playable(&self, square: Square) -> bool {
        let colour = self.side_to_move();
        let rank = Rank::Fifth.relative_to(colour);
        [-1isize, 1]
            .into_iter()
            .filter_map(|offset| {
                File::from_index(square.file().index().checked_add_signed(offset)?)
            })
            .any(|file| {
                let from = Square::new(file, rank);
                self.piece_at(from) == Some(Piece::new(Role::Pawn, colour))
                    && self.board.is_legal(cc::Move {
                        from: from.to_cozy(),
                        to: square.to_cozy(),
                        promotion: None,
                    })
            })
    }

    /// The position with the colours swapped and the ranks flipped.
    pub fn mirrored(&self) -> Position {
        let mut builder = cc::BoardBuilder::empty();
        for square in Square::ALL {
            if let Some(piece) = self.piece_at(square) {
                let mirrored = Piece::new(piece.role, !piece.colour);
                builder.board[square.flip_rank().index()] =
                    Some((mirrored.role.to_cozy(), mirrored.colour.to_cozy()));
            }
        }
        builder.side_to_move = (!self.side_to_move()).to_cozy();
        let rights = self.castling_rights();
        let mut mirrored_rights = CastlingRights::EMPTY;
        for &colour in &Colour::ALL {
            mirrored_rights.set(!colour, true, rights.short(colour));
            mirrored_rights.set(!colour, false, rights.long(colour));
        }
        builder.castle_rights = mirrored_rights.to_cozy();
        builder.en_passant = self.en_passant().map(|sq| sq.flip_rank().to_cozy());
        let board = builder.build().expect("a mirrored legal position is legal");
        Position::from_parts(
            board,
            self.halfmove_clock,
            self.fullmove_number,
            self.clocks_known,
        )
    }

    /// Board, side to move and state, for a human reader. The text is not a
    /// stable format.
    pub fn summary(&self) -> String {
        let mut out = String::new();
        for rank in Rank::ALL.iter().rev() {
            out.push(rank.to_char());
            out.push(' ');
            for file in File::ALL {
                out.push(match self.piece_at(Square::new(file, *rank)) {
                    Some(piece) => piece.to_char(),
                    None => '.',
                });
                out.push(' ');
            }
            out.push('\n');
        }
        out.push_str("  a b c d e f g h\n");
        out.push_str(match self.side_to_move() {
            Colour::White => "White to move",
            Colour::Black => "Black to move",
        });
        if self.in_check() {
            out.push_str(", in check");
        }
        out.push('\n');
        let ep = match self.en_passant() {
            Some(square) => square.to_string(),
            None => "-".to_string(),
        };
        out.push_str(&format!(
            "castling {}, en passant {}, halfmove clock {}, move {}\n",
            self.castling_rights().to_fen_field(),
            ep,
            self.halfmove_clock,
            self.fullmove_number,
        ));
        out
    }
}

impl fmt::Display for Position {
    /// The six-field FEN.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.fen())
    }
}
