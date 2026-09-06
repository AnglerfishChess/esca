//! Endgame facts: where the kings stand, how the pawn race runs, and the
//! material configurations that draw.

use crate::types::{File, Role, Square, SquareSet};

use super::scan::{CENTRE, Scan, between, distance};
use super::{DrawishMaterial, EndgameFacts, Opposition, PawnFacts, PieceFacts, Side};

/// The race plies of a side with no passer: one more than the longest race a
/// passer can run, so the count saturates at its scale.
const NO_RACE: u8 = 8;

/// The files a rook pawn stands on.
const ROOK_FILES: SquareSet = SquareSet::from_bits(0x8181_8181_8181_8181);

pub(super) fn endgame_facts(scan: &Scan, pawns: &PawnFacts, pieces: &PieceFacts) -> EndgameFacts {
    let mut facts = EndgameFacts::default();
    for side in Side::ALL {
        let i = side.index();
        facts.king_centralisation[i] = centralisation(scan.kings[i]);
        facts.race_plies[i] = race_plies(pawns.passer_lead_rank[i], side);
        facts.key_square_occupied[i] = on_own_key_square(scan, pawns, side);
        facts.wrong_colour_bishop[i] = wrong_colour_bishop(scan, side);
    }
    facts.opposition = opposition(scan);
    facts.drawish_material = drawish_material(scan, pieces, facts.wrong_colour_bishop);
    facts
}

/// The Chebyshev distance from `square` to the nearest of d4, e4, d5 and e5.
fn centralisation(square: Square) -> u8 {
    CENTRE
        .into_iter()
        .map(|centre| distance(square, centre) as u8)
        .min()
        .expect("the centre holds four squares")
}

/// The plies `side`'s most advanced passer needs to promote unopposed, or
/// [`NO_RACE`] when it has no passer.
fn race_plies(lead_rank: Option<u8>, side: Side) -> u8 {
    match lead_rank {
        Some(rank) => (8 - rank).saturating_sub(u8::from(side == Side::Us)),
        None => NO_RACE,
    }
}

/// Which opposition the kings stand in, if any: the squares between them are
/// empty and their number is odd.
fn opposition(scan: &Scan) -> Option<Opposition> {
    let corridor = between(scan.kings[0], scan.kings[1]);
    if !(corridor & scan.occupied).is_empty() {
        return None;
    }
    match corridor.len() {
        1 => Some(Opposition::Direct),
        3 | 5 => Some(Opposition::Distant),
        _ => None,
    }
}

/// Whether `side`'s king stands on a key square of one of `side`'s passers.
fn on_own_key_square(scan: &Scan, pawns: &PawnFacts, side: Side) -> bool {
    let king = scan.kings[side.index()];
    pawns.passed[side.index()]
        .into_iter()
        .any(|pawn| key_squares(scan, pawn, side).contains(king))
}

/// The key squares of the pawn on `square`: the three squares two relative
/// ranks ahead of a pawn on rank 4 or below, one rank ahead above that. A rook
/// pawn has none.
fn key_squares(scan: &Scan, square: Square, side: Side) -> SquareSet {
    let file = square.file();
    if file == File::A || file == File::H {
        return SquareSet::EMPTY;
    }
    let rank = scan.relative_rank(square, side);
    let ahead = if rank <= 4 { rank + 2 } else { rank + 1 };
    let mut set = SquareSet::EMPTY;
    for index in file.index() - 1..=file.index() + 1 {
        let file = File::from_index(index).expect("a pawn with key squares stands on files b to g");
        set.insert(scan.relative_square(file, ahead, side));
    }
    set
}

/// Whether `side`'s bishops all stand on the square colour none of its pawns
/// promotes on, every one of those pawns being a rook pawn.
fn wrong_colour_bishop(scan: &Scan, side: Side) -> bool {
    let i = side.index();
    let bishops = scan.role_units[i][Role::Bishop.index()];
    let pawns = scan.role_units[i][Role::Pawn.index()];
    if bishops.is_empty() || pawns.is_empty() || !pawns.is_subset(ROOK_FILES) {
        return false;
    }
    let dark = bishops.is_subset(SquareSet::DARK);
    if !dark && !bishops.is_subset(SquareSet::LIGHT) {
        return false;
    }
    pawns
        .into_iter()
        .all(|pawn| scan.relative_square(pawn.file(), 8, side).is_dark() != dark)
}

/// Which of the three drawn configurations the material is, if any.
fn drawish_material(
    scan: &Scan,
    pieces: &PieceFacts,
    wrong_colour_bishop: [bool; 2],
) -> Option<DrawishMaterial> {
    for side in Side::ALL {
        let i = side.index();
        if !bare_king(scan, !side) {
            continue;
        }
        let units = &scan.role_units[i];
        let count = |role: Role| units[role.index()].len();
        if count(Role::Knight) == 2
            && count(Role::Pawn) == 0
            && count(Role::Bishop) == 0
            && count(Role::Rook) == 0
            && count(Role::Queen) == 0
        {
            return Some(DrawishMaterial::TwoKnights);
        }
        if wrong_colour_bishop[i]
            && count(Role::Knight) == 0
            && count(Role::Rook) == 0
            && count(Role::Queen) == 0
        {
            return Some(DrawishMaterial::WrongBishop);
        }
    }
    let no_other_pieces = Side::ALL.iter().all(|side| {
        [Role::Knight, Role::Rook, Role::Queen]
            .iter()
            .all(|role| scan.role_units[side.index()][role.index()].is_empty())
    });
    if pieces.opposite_coloured_bishops && no_other_pieces {
        return Some(DrawishMaterial::OppositeBishops);
    }
    None
}

/// Whether `side` has no unit but its king.
fn bare_king(scan: &Scan, side: Side) -> bool {
    let i = side.index();
    scan.units[i] == scan.role_units[i][Role::King.index()]
}
