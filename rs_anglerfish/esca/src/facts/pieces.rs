//! Bishops, rooks, knights and queens.

use crate::types::{File, Role, Square, SquareSet};

use super::scan::{Scan, between};
use super::{PawnFacts, PieceFacts, Side};

/// The squares of a file.
fn file_mask(file: File) -> SquareSet {
    SquareSet::from_bits(0x0101_0101_0101_0101u64 << file.index())
}

/// The starting squares of `side`'s knights and bishops in classic chess.
fn minor_home_squares(scan: &Scan, side: Side) -> SquareSet {
    [File::B, File::C, File::F, File::G]
        .into_iter()
        .map(|file| scan.relative_square(file, 1, side).to_set())
        .fold(SquareSet::EMPTY, |a, b| a | b)
}

pub(super) fn piece_facts(scan: &Scan, pawns: &PawnFacts) -> PieceFacts {
    let mut facts = PieceFacts::default();

    for side in Side::ALL {
        let i = side.index();
        let bishops = scan.role_units[i][Role::Bishop.index()];
        let rooks = scan.role_units[i][Role::Rook.index()];
        let knights = scan.role_units[i][Role::Knight.index()];
        let queens = scan.role_units[i][Role::Queen.index()];

        let light = bishops & scan.view_light();
        let dark = bishops & scan.view_dark();
        facts.bishops_light[i] = light.len().min(255) as u8;
        facts.bishops_dark[i] = dark.len().min(255) as u8;
        facts.bishop_pair[i] = !light.is_empty() && !dark.is_empty();

        let mut colours = SquareSet::EMPTY;
        if !light.is_empty() {
            colours |= scan.view_light();
        }
        if !dark.is_empty() {
            colours |= scan.view_dark();
        }
        facts.pawns_on_bishop_colour[i] = (pawns.pawns[i] & colours).len().min(255) as u8;

        for a in rooks {
            for b in rooks {
                if a >= b {
                    continue;
                }
                if (between(a, b) & scan.occupied).is_empty() {
                    if a.rank() == b.rank() {
                        facts.rooks_connected_rank[i] = true;
                    }
                    if a.file() == b.file() {
                        facts.rooks_connected_file[i] = true;
                    }
                }
            }
        }

        for rook in rooks {
            let file = rook.file();
            if pawns.open_files.contains(file) {
                facts.rooks_on_open_file[i] += 1;
            }
            if pawns.semi_open_files[i].contains(file) {
                facts.rooks_on_semi_open_file[i] += 1;
            }
            if scan.relative_rank(rook, side) == 7 {
                facts.rooks_on_relative_7th[i] += 1;
            }
            if behind_a_passer(scan, rook, pawns.passed[i], side) {
                facts.rook_behind_own_passer[i] += 1;
            }
            if behind_a_passer(scan, rook, pawns.passed[(!side).index()], !side) {
                facts.rook_behind_enemy_passer[i] += 1;
            }
        }

        facts.trapped_rook[i] = trapped_rook(scan, side, rooks);

        facts.outposts[i] = outposts(scan, pawns, side);
        facts.knights_on_outpost[i] = (knights & facts.outposts[i]).len().min(255) as u8;
        facts.outpost_squares_free[i] = (facts.outposts[i] - scan.occupied).len().min(255) as u8;
        facts.knights_on_rim[i] = knights
            .into_iter()
            .filter(|square| {
                let rank = scan.relative_rank(*square, side);
                square.file() == File::A || square.file() == File::H || rank == 1 || rank == 8
            })
            .count()
            .min(255) as u8;

        facts.minors_undeveloped[i] = ((knights | bishops) & minor_home_squares(scan, side))
            .len()
            .min(255) as u8;
        let queen_home = scan.relative_square(File::D, 1, side);
        facts.queen_developed[i] = !(queens - queen_home.to_set()).is_empty();
    }

    facts.opposite_coloured_bishops = facts.bishops_light[0] + facts.bishops_dark[0] == 1
        && facts.bishops_light[1] + facts.bishops_dark[1] == 1
        && facts.bishops_light[0] != facts.bishops_light[1];

    facts
}

/// Whether `rook` stands on the file of one of `passers` and behind it in the
/// passer owner's frame.
fn behind_a_passer(scan: &Scan, rook: Square, passers: SquareSet, owner: Side) -> bool {
    (passers & file_mask(rook.file()))
        .into_iter()
        .any(|passer| scan.relative_rank(rook, owner) < scan.relative_rank(passer, owner))
}

/// Whether a rook of `side` is boxed in: at most two non-capture destinations,
/// beyond its own king on the wing the king stands on, the king having lost
/// its castling rights.
fn trapped_rook(scan: &Scan, side: Side, rooks: SquareSet) -> bool {
    let i = side.index();
    if scan.castling[i] {
        return false;
    }
    let king = scan.kings[i];
    rooks.into_iter().any(|rook| {
        let outside = if king.file() >= File::E {
            rook.file() > king.file()
        } else {
            rook.file() < king.file()
        };
        outside && (scan.attacks_from[rook.index()] - scan.occupied).len() <= 2
    })
}

/// The squares on `side`'s relative ranks 4 to 6 that one of its pawns attacks
/// and no enemy pawn can ever attack.
fn outposts(scan: &Scan, pawns: &PawnFacts, side: Side) -> SquareSet {
    let theirs = pawns.pawns[(!side).index()];
    let mut set = SquareSet::EMPTY;
    for square in scan.by_role[side.index()][Role::Pawn.index()] {
        let rank = scan.relative_rank(square, side);
        if !(4..=6).contains(&rank) {
            continue;
        }
        let attackable = [
            File::from_index(square.file().index().wrapping_sub(1)),
            File::from_index(square.file().index() + 1),
        ]
        .into_iter()
        .flatten()
        .any(|file| {
            (theirs & file_mask(file))
                .into_iter()
                .any(|pawn| scan.relative_rank(pawn, side) >= rank)
        });
        if !attackable {
            set.insert(square);
        }
    }
    set
}
