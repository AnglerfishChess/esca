//! Pawn structure.

use crate::types::{File, FileSet, Role, Square, SquareSet};

use super::scan::{Scan, attacks_of, distance};
use super::{PawnFacts, Side};

/// The squares of `file` and its neighbours, whichever exist.
fn adjacent_files(file: File) -> [Option<File>; 2] {
    [
        File::from_index(file.index().wrapping_sub(1)),
        File::from_index(file.index() + 1),
    ]
}

/// Every square of `file`.
fn file_mask(file: File) -> SquareSet {
    SquareSet::from_bits(0x0101_0101_0101_0101u64 << file.index())
}

/// Every square of `side`'s relative ranks above `rank`.
fn ahead_of(scan: &Scan, rank: u32, side: Side) -> SquareSet {
    let mut set = SquareSet::EMPTY;
    for square in Square::ALL {
        if scan.relative_rank(square, side) > rank {
            set.insert(square);
        }
    }
    set
}

/// Every square of `side`'s relative ranks at or below `rank`.
fn behind_or_on(scan: &Scan, rank: u32, side: Side) -> SquareSet {
    !ahead_of(scan, rank, side)
}

pub(super) fn pawn_facts(scan: &Scan) -> PawnFacts {
    let mut facts = PawnFacts::default();
    for side in Side::ALL {
        facts.pawns[side.index()] = scan.role_units[side.index()][Role::Pawn.index()];
    }

    for side in Side::ALL {
        let i = side.index();
        let ours = facts.pawns[i];
        let theirs = facts.pawns[(!side).index()];

        for square in ours {
            facts.count_by_file[i][square.file().index()] += 1;
            facts.count_by_rank[i][scan.relative_rank(square, side) as usize - 1] += 1;
        }

        for square in ours {
            let file = square.file();
            let rank = scan.relative_rank(square, side);
            let own_file = file_mask(file);
            let neighbours: SquareSet = adjacent_files(file)
                .into_iter()
                .flatten()
                .map(file_mask)
                .fold(SquareSet::EMPTY, |a, b| a | b);
            let ahead = ahead_of(scan, rank, side);
            let behind = behind_or_on(scan, rank, side);

            if !(own_file & (ours - square.to_set())).is_empty() {
                facts.doubled[i].insert(square);
            }
            if (neighbours & ours).is_empty() {
                facts.isolated[i].insert(square);
            }

            let blockers_ahead = (own_file | neighbours) & ahead & theirs;
            let passed = blockers_ahead.is_empty();
            if passed {
                facts.passed[i].insert(square);
            } else if (own_file & ahead & theirs).is_empty() {
                let support = (neighbours & behind & ours).len();
                let opposition = (neighbours & ahead & theirs).len();
                if support >= opposition {
                    facts.candidates[i].insert(square);
                }
            }

            if !passed
                && (neighbours & behind & ours).is_empty()
                && scan.by_role[(!side).index()][Role::Pawn.index()].contains(scan.relative_square(
                    file,
                    rank + 1,
                    side,
                ))
            {
                facts.backward[i].insert(square);
            }

            if !(attacks_of(Role::Pawn, square, scan.colour(side), scan.occupied) & theirs)
                .is_empty()
            {
                facts.levers[i] = facts.levers[i].saturating_add(1);
            }
        }

        facts.defended[i] = ours & scan.by_role[i][Role::Pawn.index()];

        // Islands: maximal runs of adjacent files carrying a pawn.
        let mut previous = false;
        for file in File::ALL {
            let occupied = facts.count_by_file[i][file.index()] > 0;
            if occupied && !previous {
                facts.islands[i] += 1;
            }
            previous = occupied;
        }

        let passers = facts.passed[i];
        facts.passer_lead_rank[i] = passers
            .into_iter()
            .map(|square| scan.relative_rank(square, side) as u8)
            .max();
        facts.passer_protected[i] = (passers & facts.defended[i]).len().min(255) as u8;
        facts.passers_connected[i] = passers.into_iter().any(|square| {
            adjacent_files(square.file())
                .into_iter()
                .flatten()
                .any(|file| !(file_mask(file) & passers).is_empty())
        });
        facts.passer_unstoppable[i] = passers
            .into_iter()
            .any(|square| unstoppable(scan, square, side));
    }

    for file in File::ALL {
        let ours = facts.count_by_file[0][file.index()] > 0;
        let theirs = facts.count_by_file[1][file.index()] > 0;
        if !ours && !theirs {
            facts.open_files.insert(file);
        }
        if !ours && theirs {
            facts.semi_open_files[0].insert(file);
        }
        if !theirs && ours {
            facts.semi_open_files[1].insert(file);
        }
    }

    facts.rams = facts.pawns[0]
        .into_iter()
        .filter(|square| {
            let stop = scan.relative_square(
                square.file(),
                scan.relative_rank(*square, Side::Us) + 1,
                Side::Us,
            );
            facts.pawns[1].contains(stop)
        })
        .count()
        .min(255) as u8;

    facts
}

/// Whether the enemy king cannot catch the passer on `square` by the rule of
/// the square, its own side having no unit but the king to help.
fn unstoppable(scan: &Scan, square: Square, side: Side) -> bool {
    let defender = !side;
    let d = defender.index();
    let has_pieces = [Role::Knight, Role::Bishop, Role::Rook, Role::Queen]
        .iter()
        .any(|role| !scan.role_units[d][role.index()].is_empty());
    if has_pieces {
        return false;
    }
    let rank = scan.relative_rank(square, side);
    let promotion = scan.relative_square(square.file(), 8, side);
    let tempo = u32::from(defender == Side::Us);
    distance(scan.kings[d], promotion).saturating_sub(tempo) > 8 - rank
}

/// The files a set of pawns stands on.
pub(super) fn files_of(pawns: SquareSet) -> FileSet {
    pawns.into_iter().map(Square::file).collect()
}
