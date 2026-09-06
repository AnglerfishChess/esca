//! Pawn structure.

use crate::types::{File, FileSet, Role, Square, SquareSet};

use super::king::shield_files;
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
        .filter(|square| facts.pawns[1].contains(stop_square(scan, *square, Side::Us)))
        .count()
        .min(255) as u8;

    for side in Side::ALL {
        let i = side.index();
        let them = (!side).index();
        let ours = facts.pawns[i];

        facts.chain_max_length[i] = ours
            .into_iter()
            .flat_map(|square| [-1i32, 1].map(|step| chain_run(scan, ours, square, step, side)))
            .max()
            .unwrap_or(0);
        facts.chain_base_attacked[i] = ours.into_iter().any(|square| {
            [-1i32, 1].into_iter().any(|step| {
                chain_run(scan, ours, square, step, side) >= 2
                    && behind(scan, square, step, side).is_none_or(|back| !ours.contains(back))
                    && scan.by[them].contains(square)
            })
        });

        for (wing, mask) in [QUEEN_SIDE, KING_SIDE].into_iter().enumerate() {
            facts.majority_by_wing[i][wing] =
                (ours & mask).len() > (facts.pawns[them] & mask).len();
        }

        facts.holes[i] = holes(scan, ours, side);
        let minors = scan.role_units[them][Role::Knight.index()]
            | scan.role_units[them][Role::Bishop.index()];
        facts.holes_occupied[i] = (minors & facts.holes[i]).len().min(255) as u8;

        facts.fixed_pawns[i] = ours
            .into_iter()
            .filter(|square| scan.occupied.contains(stop_square(scan, *square, side)))
            .count()
            .min(255) as u8;

        let passers = facts.passed[i];
        facts.blocked_passers[i] = passers
            .into_iter()
            .filter(|square| scan.units[them].contains(stop_square(scan, *square, side)))
            .count()
            .min(255) as u8;
        facts.passer_free_path[i] = passers
            .into_iter()
            .filter(|square| {
                let rank = scan.relative_rank(*square, side);
                (file_mask(square.file()) & ahead_of(scan, rank, side) & scan.occupied).is_empty()
            })
            .count()
            .min(255) as u8;

        // The lead passer: the most advanced, and among equals the one nearest
        // file a.
        let lead = passers
            .into_iter()
            .max_by_key(|square| (scan.relative_rank(*square, side), 7 - square.file().index()));
        facts.passer_distance[i] = lead.map(|square| 8 - scan.relative_rank(square, side) as u8);
        facts.passer_king_distance[i] = match lead {
            None => [None; 2],
            Some(square) => {
                let promotion = scan.relative_square(square.file(), 8, side);
                [i, them].map(|king| Some(distance(scan.kings[king], promotion).min(255) as u8))
            }
        };
        facts.passer_in_square[i] = lead.is_some_and(|square| in_square(scan, square, side));

        facts.half_open_at_enemy_king[i] = shield_files(scan.kings[them].file())
            .into_iter()
            .filter(|file| facts.semi_open_files[i].contains(*file))
            .count()
            .min(255) as u8;
        facts.backward_on_semi_open[i] = facts.backward[i]
            .into_iter()
            .filter(|square| facts.semi_open_files[them].contains(square.file()))
            .count()
            .min(255) as u8;
    }

    facts
}

/// Every square of files a to d.
const QUEEN_SIDE: SquareSet = SquareSet::from_bits(0x0F0F_0F0F_0F0F_0F0F);
/// Every square of files e to h.
const KING_SIDE: SquareSet = SquareSet::from_bits(0xF0F0_F0F0_F0F0_F0F0);

/// The square directly ahead of the pawn on `square`, in `side`'s frame.
fn stop_square(scan: &Scan, square: Square, side: Side) -> Square {
    scan.relative_square(square.file(), scan.relative_rank(square, side) + 1, side)
}

/// The square one rank behind `square` on the diagonal `step`, in `side`'s
/// frame; `None` off the board.
fn behind(scan: &Scan, square: Square, step: i32, side: Side) -> Option<Square> {
    let file = square.file().index() as i32 - step;
    let rank = scan.relative_rank(square, side);
    if !(0..8).contains(&file) || rank <= 1 {
        return None;
    }
    Some(scan.relative_square(
        File::from_index(file as usize).expect("a file index below 8"),
        rank - 1,
        side,
    ))
}

/// How many pawns of `ours` stand from `square` on, each defending the next,
/// one file per rank in the direction `step`.
fn chain_run(scan: &Scan, ours: SquareSet, square: Square, step: i32, side: Side) -> u8 {
    let mut length = 1;
    let mut file = square.file().index() as i32;
    let mut rank = scan.relative_rank(square, side);
    loop {
        file += step;
        rank += 1;
        if !(0..8).contains(&file) || rank > 8 {
            return length;
        }
        let file = File::from_index(file as usize).expect("a file index below 8");
        if !ours.contains(scan.relative_square(file, rank, side)) {
            return length;
        }
        length += 1;
    }
}

/// The squares on `side`'s relative ranks 3 to 6 that no pawn of `ours` can
/// ever attack.
fn holes(scan: &Scan, ours: SquareSet, side: Side) -> SquareSet {
    let mut lowest = [9u32; 8];
    for square in ours {
        let file = square.file().index();
        lowest[file] = lowest[file].min(scan.relative_rank(square, side));
    }
    let mut set = SquareSet::EMPTY;
    for square in Square::ALL {
        let rank = scan.relative_rank(square, side);
        if !(3..=6).contains(&rank) {
            continue;
        }
        let attackable = adjacent_files(square.file())
            .into_iter()
            .flatten()
            .any(|file| lowest[file.index()] < rank);
        if !attackable {
            set.insert(square);
        }
    }
    set
}

/// Whether the king defending against the passer on `square` is in its square
/// by the rule of the square.
fn in_square(scan: &Scan, square: Square, side: Side) -> bool {
    let defender = !side;
    let rank = scan.relative_rank(square, side);
    let promotion = scan.relative_square(square.file(), 8, side);
    let tempo = u32::from(defender == Side::Us);
    distance(scan.kings[defender.index()], promotion).saturating_sub(tempo) <= 8 - rank
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
