//! The rules the built-in variants share: move generation, playing a move,
//! terminal conditions, and move text.

use cozy_chess as cc;

use crate::error::MoveParseError;
use crate::moves::{Move, MoveKind, MoveList};
use crate::position::Position;
use crate::types::{Colour, File, Rank, Role, Square, SquareSet};
use crate::variant::{CastlingOutput, Outcome, Variant};

/// The esca move for a move cozy-chess generated in `position`.
fn classify(position: &Position, mv: cc::Move) -> Move {
    let board = position.board();
    let colour = board.side_to_move();
    let from = Square::from_cozy(mv.from);
    let to = Square::from_cozy(mv.to);
    let promotion = mv.promotion.map(Role::from_cozy);
    if board.colors(colour).has(mv.to) {
        // Castling is generated as the king capturing its own rook.
        return Move::new(from, to, None, MoveKind::Castling);
    }
    let moves_pawn = board.piece_on(mv.from) == Some(cc::Piece::Pawn);
    if moves_pawn && Some(to) == position.en_passant() {
        return Move::new(from, to, None, MoveKind::EnPassant);
    }
    let capture = board.colors(!colour).has(mv.to);
    match promotion {
        Some(_) => Move::new(from, to, promotion, MoveKind::Promotion).with_capture(capture),
        None if capture => Move::new(from, to, None, MoveKind::Capture),
        None => Move::new(from, to, None, MoveKind::Quiet),
    }
}

pub(crate) fn legal_moves(position: &Position, out: &mut MoveList) {
    position.board().generate_moves(|piece_moves| {
        for mv in piece_moves {
            out.push(classify(position, mv));
        }
        false
    });
}

pub(crate) fn is_legal(position: &Position, mv: Move) -> bool {
    position.board().is_legal(mv.to_cozy())
}

pub(crate) fn play(position: &Position, mv: Move) -> Position {
    let mut board = position.board().clone();
    let colour = board.side_to_move();
    let moves_pawn = board.piece_on(mv.from().to_cozy()) == Some(cc::Piece::Pawn);
    let captures = board.colors(!colour).has(mv.to().to_cozy())
        || (moves_pawn && Some(mv.to()) == position.en_passant());
    board.play(mv.to_cozy());
    let halfmove_clock = if moves_pawn || captures {
        0
    } else {
        position.halfmove_clock() + 1
    };
    let fullmove_number = position.fullmove_number() + u32::from(colour == cc::Color::Black);
    Position::from_parts(
        board,
        halfmove_clock,
        fullmove_number,
        position.clocks_known(),
    )
}

pub(crate) fn has_legal_move(position: &Position) -> bool {
    position.board().generate_moves(|_| true)
}

pub(crate) fn outcome(position: &Position) -> Option<Outcome> {
    if !has_legal_move(position) {
        return Some(if position.in_check() {
            Outcome::Checkmate {
                winner: !position.side_to_move(),
            }
        } else {
            Outcome::Stalemate
        });
    }
    if is_insufficient_material(position) {
        return Some(Outcome::InsufficientMaterial);
    }
    if position.halfmove_clock() >= 150 {
        return Some(Outcome::SeventyFiveMoves);
    }
    None
}

/// Whether neither side has material that could ever deliver mate: king
/// against king, king and one minor against king, and bishops of one square
/// colour on both sides.
pub(crate) fn is_insufficient_material(position: &Position) -> bool {
    let heavy =
        position.by_role(Role::Pawn) | position.by_role(Role::Rook) | position.by_role(Role::Queen);
    if !heavy.is_empty() {
        return false;
    }
    let knights = position.by_role(Role::Knight);
    let bishops = position.by_role(Role::Bishop);
    if knights.len() + bishops.len() <= 1 {
        return true;
    }
    knights.is_empty()
        && (bishops.is_subset(SquareSet::DARK) || bishops.is_subset(SquareSet::LIGHT))
}

/// Where the king lands when a castling move is written as a king move.
fn king_destination(mv: Move) -> Square {
    let file = if mv.to().file() > mv.from().file() {
        File::G
    } else {
        File::C
    };
    Square::new(file, mv.from().rank())
}

pub(crate) fn move_to_uci(mv: Move, style: CastlingOutput, two_squares_allowed: bool) -> String {
    if mv.is_castling() && style == CastlingOutput::KingTwoSquares && two_squares_allowed {
        format!("{}{}", mv.from(), king_destination(mv))
    } else {
        mv.to_string()
    }
}

pub(crate) fn move_from_uci(
    variant: &dyn Variant,
    position: &Position,
    text: &str,
) -> Result<Move, MoveParseError> {
    let bytes = text.as_bytes();
    if bytes.len() != 4 && bytes.len() != 5 {
        return Err(MoveParseError::Syntax);
    }
    let from: Square = text[0..2].parse().map_err(|_| MoveParseError::Syntax)?;
    let to: Square = text[2..4].parse().map_err(|_| MoveParseError::Syntax)?;
    let promotion = match bytes.get(4) {
        None => None,
        Some(&c) => match Role::from_char(c as char) {
            Some(role) if role != Role::Pawn && role != Role::King => Some(role),
            _ => return Err(MoveParseError::Syntax),
        },
    };

    let mut moves = MoveList::new();
    variant.legal_moves(position, &mut moves);
    if let Some(&mv) = moves
        .as_slice()
        .iter()
        .find(|mv| mv.from() == from && mv.to() == to && mv.promotion() == promotion)
    {
        return Ok(mv);
    }
    if promotion.is_none() {
        // The other castling spelling: the king's two-square move.
        if let Some(&mv) = moves
            .as_slice()
            .iter()
            .find(|mv| mv.is_castling() && mv.from() == from && king_destination(**mv) == to)
        {
            return Ok(mv);
        }
    }
    Err(MoveParseError::Illegal)
}

pub(crate) fn move_to_san(variant: &dyn Variant, position: &Position, mv: Move) -> String {
    let mut moves = MoveList::new();
    variant.legal_moves(position, &mut moves);
    let mut text = if mv.is_castling() {
        if mv.to().file() > mv.from().file() {
            "O-O".to_string()
        } else {
            "O-O-O".to_string()
        }
    } else {
        let role = position
            .piece_at(mv.from())
            .expect("a move starts on an occupied square")
            .role;
        let mut text = String::with_capacity(8);
        if role == Role::Pawn {
            if mv.is_capture() {
                text.push(mv.from().file().to_char());
                text.push('x');
            }
        } else {
            text.push(role.to_san_char().expect("a pawn was handled above"));
            text.push_str(&disambiguation(&moves, position, mv, role));
            if mv.is_capture() {
                text.push('x');
            }
        }
        text.push_str(&mv.to().to_string());
        if let Some(promotion) = mv.promotion() {
            text.push('=');
            text.push(
                promotion
                    .to_san_char()
                    .expect("a pawn is not a promotion role"),
            );
        }
        text
    };
    let after = variant.play(position, mv);
    if after.in_check() {
        let mut replies = MoveList::new();
        variant.legal_moves(&after, &mut replies);
        text.push(if replies.is_empty() { '#' } else { '+' });
    }
    text
}

/// As much of the origin square as SAN needs to name `mv` uniquely.
fn disambiguation(moves: &MoveList, position: &Position, mv: Move, role: Role) -> String {
    let rivals: Vec<Move> = moves
        .as_slice()
        .iter()
        .copied()
        .filter(|other| {
            !other.is_castling()
                && other.to() == mv.to()
                && other.from() != mv.from()
                && position.piece_at(other.from()).map(|p| p.role) == Some(role)
        })
        .collect();
    if rivals.is_empty() {
        return String::new();
    }
    if !rivals
        .iter()
        .any(|other| other.from().file() == mv.from().file())
    {
        return mv.from().file().to_char().to_string();
    }
    if !rivals
        .iter()
        .any(|other| other.from().rank() == mv.from().rank())
    {
        return mv.from().rank().to_char().to_string();
    }
    mv.from().to_string()
}

pub(crate) fn move_from_san(
    variant: &dyn Variant,
    position: &Position,
    text: &str,
) -> Result<Move, MoveParseError> {
    let core = text.trim_end_matches(['+', '#', '!', '?']);
    let mut moves = MoveList::new();
    variant.legal_moves(position, &mut moves);

    let short = matches!(core, "O-O" | "0-0" | "o-o");
    let long = matches!(core, "O-O-O" | "0-0-0" | "o-o-o");
    if short || long {
        return moves
            .as_slice()
            .iter()
            .copied()
            .find(|mv| mv.is_castling() && (mv.to().file() > mv.from().file()) == short)
            .ok_or(MoveParseError::Illegal);
    }

    let (role, rest) = match core.chars().next() {
        Some(c @ ('N' | 'B' | 'R' | 'Q' | 'K')) => (
            Role::from_char(c).expect("a SAN role letter is a role letter"),
            &core[1..],
        ),
        Some(_) => (Role::Pawn, core),
        None => return Err(MoveParseError::Syntax),
    };

    let (rest, promotion) = match rest.split_once('=') {
        Some((head, tail)) => {
            let mut chars = tail.chars();
            let role = chars
                .next()
                .and_then(Role::from_char)
                .filter(|r| *r != Role::Pawn && *r != Role::King)
                .ok_or(MoveParseError::Syntax)?;
            if chars.next().is_some() {
                return Err(MoveParseError::Syntax);
            }
            (head, Some(role))
        }
        None => (rest, None),
    };

    // `x` and `-` between origin and destination carry no information.
    let rest: String = rest.chars().filter(|c| !matches!(c, 'x' | '-')).collect();
    if rest.len() < 2 {
        return Err(MoveParseError::Syntax);
    }
    let to: Square = rest[rest.len() - 2..]
        .parse()
        .map_err(|_| MoveParseError::Syntax)?;
    let (mut from_file, mut from_rank) = (None, None);
    for c in rest[..rest.len() - 2].chars() {
        if let Some(file) = File::from_char(c) {
            if from_file.replace(file).is_some() {
                return Err(MoveParseError::Syntax);
            }
        } else if let Some(rank) = Rank::from_char(c) {
            if from_rank.replace(rank).is_some() {
                return Err(MoveParseError::Syntax);
            }
        } else {
            return Err(MoveParseError::Syntax);
        }
    }

    let mut found = None;
    for &mv in moves.as_slice() {
        let matches = !mv.is_castling()
            && mv.to() == to
            && mv.promotion() == promotion
            && position.piece_at(mv.from()).map(|p| p.role) == Some(role)
            && from_file.is_none_or(|f| mv.from().file() == f)
            && from_rank.is_none_or(|r| mv.from().rank() == r);
        if matches {
            if found.is_some() {
                return Err(MoveParseError::Ambiguous);
            }
            found = Some(mv);
        }
    }
    found.ok_or(MoveParseError::Illegal)
}

/// Whether the position's castling rights are ones a classic starting array
/// can produce: the king on the e-file, its rooks on the a- and h-files.
pub(crate) fn has_classic_castling(position: &Position) -> bool {
    let rights = position.castling_rights();
    Colour::ALL.iter().all(|&colour| {
        if !rights.any(colour) {
            return true;
        }
        let king = position.king_of(colour);
        king.file() == File::E
            && king.rank() == Rank::First.relative_to(colour)
            && rights.short(colour).is_none_or(|f| f == File::H)
            && rights.long(colour).is_none_or(|f| f == File::A)
    })
}
