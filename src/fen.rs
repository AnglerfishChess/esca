//! FEN and EPD text for a [`Position`].
//!
//! The castling field is read in both dialects: `KQkq`, where a letter means
//! the outermost rook on that wing, and the file letters `AHah` of
//! X-FEN/Shredder-FEN.

use cozy_chess as cc;

use crate::error::FenError;
use crate::position::{CastlingRights, Position};
use crate::types::{Colour, File, Piece, Rank, Square};

/// Reads a four- or six-field FEN.
pub(crate) fn parse(text: &str) -> Result<Position, FenError> {
    let fields: Vec<&str> = text.split_whitespace().collect();
    if fields.len() != 4 && fields.len() != 6 {
        return Err(FenError::FieldCount);
    }
    let clocks_known = fields.len() == 6;

    let mut builder = cc::BoardBuilder::empty();
    parse_placement(&mut builder, fields[0])?;

    builder.side_to_move = match fields[1] {
        "w" => cc::Color::White,
        "b" => cc::Color::Black,
        _ => return Err(FenError::SideToMove),
    };

    let rights = parse_castling(&builder, fields[2])?;
    builder.castle_rights = rights.to_cozy();

    if fields[3] != "-" {
        let square: Square = fields[3].parse().map_err(|_| FenError::EnPassant)?;
        let expected = Rank::Sixth.relative_to(Colour::from_cozy(builder.side_to_move));
        if square.rank() != expected {
            return Err(FenError::EnPassant);
        }
        builder.en_passant = Some(square.to_cozy());
    }

    let (halfmove_clock, fullmove_number) = if clocks_known {
        let halfmove: u32 = fields[4].parse().map_err(|_| FenError::HalfmoveClock)?;
        let fullmove: u32 = fields[5].parse().map_err(|_| FenError::FullmoveNumber)?;
        if fullmove == 0 {
            return Err(FenError::FullmoveNumber);
        }
        (halfmove, fullmove)
    } else {
        (0, 1)
    };

    let board = builder.build().map_err(|error| match error {
        cc::BoardBuilderError::InvalidCastlingRights => FenError::Castling,
        cc::BoardBuilderError::InvalidEnPassant => FenError::EnPassant,
        _ => FenError::Position,
    })?;
    Ok(Position::from_parts(
        board,
        halfmove_clock,
        fullmove_number,
        clocks_known,
    ))
}

fn parse_placement(builder: &mut cc::BoardBuilder, field: &str) -> Result<(), FenError> {
    let mut rows = field.split('/');
    for rank in Rank::ALL.iter().rev() {
        let row = rows.next().ok_or(FenError::Placement)?;
        let mut file = 0usize;
        for c in row.chars() {
            if let Some(skip) = c.to_digit(10) {
                if skip == 0 {
                    return Err(FenError::Placement);
                }
                file += skip as usize;
            } else {
                let piece = Piece::from_char(c).ok_or(FenError::Placement)?;
                let square = Square::new(File::from_index(file).ok_or(FenError::Placement)?, *rank);
                builder.board[square.index()] =
                    Some((piece.role.to_cozy(), piece.colour.to_cozy()));
                file += 1;
            }
        }
        if file != 8 {
            return Err(FenError::Placement);
        }
    }
    if rows.next().is_some() {
        return Err(FenError::Placement);
    }
    Ok(())
}

fn parse_castling(builder: &cc::BoardBuilder, field: &str) -> Result<CastlingRights, FenError> {
    let mut rights = CastlingRights::EMPTY;
    if field == "-" {
        return Ok(rights);
    }
    for c in field.chars() {
        let colour = if c.is_ascii_uppercase() {
            Colour::White
        } else {
            Colour::Black
        };
        let king = king_file(builder, colour).ok_or(FenError::Castling)?;
        let (short, file) = match c.to_ascii_lowercase() {
            'k' => (true, outermost_rook(builder, colour, king, true)),
            'q' => (false, outermost_rook(builder, colour, king, false)),
            other => {
                let file = File::from_char(other).ok_or(FenError::Castling)?;
                (file > king, Some(file))
            }
        };
        let file = file.ok_or(FenError::Castling)?;
        let already_set = if short {
            rights.short(colour).is_some()
        } else {
            rights.long(colour).is_some()
        };
        if already_set {
            return Err(FenError::Castling);
        }
        rights.set(colour, short, Some(file));
    }
    Ok(rights)
}

fn king_file(builder: &cc::BoardBuilder, colour: Colour) -> Option<File> {
    let back_rank = Rank::First.relative_to(colour);
    File::ALL.into_iter().find(|&file| {
        builder.board[Square::new(file, back_rank).index()]
            == Some((cc::Piece::King, colour.to_cozy()))
    })
}

/// The rook of `colour` on its back rank furthest from the king on the wing
/// `short` names: what `K`, `Q`, `k` and `q` mean in X-FEN.
fn outermost_rook(
    builder: &cc::BoardBuilder,
    colour: Colour,
    king: File,
    short: bool,
) -> Option<File> {
    let back_rank = Rank::First.relative_to(colour);
    let mut found = None;
    for file in File::ALL {
        let on_wing = if short { file > king } else { file < king };
        let is_rook = builder.board[Square::new(file, back_rank).index()]
            == Some((cc::Piece::Rook, colour.to_cozy()));
        if on_wing && is_rook && (short || found.is_none()) {
            found = Some(file);
        }
    }
    found
}

/// Writes the six FEN fields, or only the first four when `clocks` is false.
pub(crate) fn format(position: &Position, clocks: bool) -> String {
    let mut out = String::with_capacity(if clocks { 90 } else { 80 });
    for rank in Rank::ALL.iter().rev() {
        let mut empty = 0;
        for file in File::ALL {
            match position.piece_at(Square::new(file, *rank)) {
                Some(piece) => {
                    if empty > 0 {
                        out.push_str(&empty.to_string());
                        empty = 0;
                    }
                    out.push(piece.to_char());
                }
                None => empty += 1,
            }
        }
        if empty > 0 {
            out.push_str(&empty.to_string());
        }
        if *rank != Rank::First {
            out.push('/');
        }
    }
    out.push(' ');
    out.push(position.side_to_move().to_char());
    out.push(' ');
    out.push_str(&position.castling_rights().to_fen_field());
    out.push(' ');
    match position.en_passant() {
        Some(square) => out.push_str(&square.to_string()),
        None => out.push('-'),
    }
    if clocks {
        out.push_str(&format!(
            " {} {}",
            position.halfmove_clock(),
            position.fullmove_number()
        ));
    }
    out
}
