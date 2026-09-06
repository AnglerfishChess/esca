//! Pawn structure.

use crate::types::{Colour, File, Role, Square, SquareSet};

use super::PawnFacts;
use super::king::shield_files;
use super::scan::{Scan, attacks_of, distance};

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

/// Every square above `rank`, counted from `colour`'s own back rank.
fn ahead_of(scan: &Scan, rank: u32, colour: Colour) -> SquareSet {
    let mut set = SquareSet::EMPTY;
    for square in Square::ALL {
        if scan.relative_rank(square, colour) > rank {
            set.insert(square);
        }
    }
    set
}

/// Every square at or below `rank`, counted from `colour`'s own back rank.
fn behind_or_on(scan: &Scan, rank: u32, colour: Colour) -> SquareSet {
    !ahead_of(scan, rank, colour)
}

pub(super) fn pawn_facts(scan: &Scan) -> PawnFacts {
    let mut facts = PawnFacts::default();
    for colour in Colour::ALL {
        facts.pawns[colour] = scan.role_units.of(colour)[Role::Pawn.index()];
    }

    for colour in Colour::ALL {
        let them = !colour;
        let ours = facts.pawns.of(colour);
        let theirs = facts.pawns.of(them);

        for square in ours {
            facts.count_by_file[colour][square.file().index()] += 1;
            facts.count_by_rank[colour][scan.relative_rank(square, colour) as usize - 1] += 1;
        }

        for square in ours {
            let file = square.file();
            let rank = scan.relative_rank(square, colour);
            let own_file = file_mask(file);
            let neighbours: SquareSet = adjacent_files(file)
                .into_iter()
                .flatten()
                .map(file_mask)
                .fold(SquareSet::EMPTY, |a, b| a | b);
            let ahead = ahead_of(scan, rank, colour);
            let behind = behind_or_on(scan, rank, colour);

            if !(own_file & (ours - square.to_set())).is_empty() {
                facts.doubled[colour].insert(square);
            }
            if (neighbours & ours).is_empty() {
                facts.isolated[colour].insert(square);
            }

            let blockers_ahead = (own_file | neighbours) & ahead & theirs;
            let passed = blockers_ahead.is_empty();
            if passed {
                facts.passed[colour].insert(square);
            } else if (own_file & ahead & theirs).is_empty() {
                let support = (neighbours & behind & ours).len();
                let opposition = (neighbours & ahead & theirs).len();
                if support >= opposition {
                    facts.candidates[colour].insert(square);
                }
            }

            if !passed
                && (neighbours & behind & ours).is_empty()
                && scan.by_role.of(them)[Role::Pawn.index()].contains(scan.relative_square(
                    file,
                    rank + 1,
                    colour,
                ))
            {
                facts.backward[colour].insert(square);
            }

            if !(attacks_of(Role::Pawn, square, colour, scan.occupied) & theirs).is_empty() {
                facts.levers[colour] = facts.levers.of(colour).saturating_add(1);
            }
        }

        facts.defended[colour] = ours & scan.by_role.of(colour)[Role::Pawn.index()];

        // Islands: maximal runs of adjacent files carrying a pawn.
        let mut previous = false;
        for file in File::ALL {
            let occupied = facts.count_by_file.of(colour)[file.index()] > 0;
            if occupied && !previous {
                facts.islands[colour] += 1;
            }
            previous = occupied;
        }

        let passers = facts.passed.of(colour);
        facts.passer_lead_rank[colour] = passers
            .into_iter()
            .map(|square| scan.relative_rank(square, colour) as u8)
            .max();
        facts.passer_protected[colour] = (passers & facts.defended.of(colour)).len().min(255) as u8;
        facts.passers_connected[colour] = passers.into_iter().any(|square| {
            adjacent_files(square.file())
                .into_iter()
                .flatten()
                .any(|file| !(file_mask(file) & passers).is_empty())
        });
        facts.passer_unstoppable[colour] = passers
            .into_iter()
            .any(|square| unstoppable(scan, square, colour));
    }

    for file in File::ALL {
        let white = facts.count_by_file.white[file.index()] > 0;
        let black = facts.count_by_file.black[file.index()] > 0;
        if !white && !black {
            facts.open_files.insert(file);
        }
        if !white && black {
            facts.semi_open_files.white.insert(file);
        }
        if !black && white {
            facts.semi_open_files.black.insert(file);
        }
    }

    facts.rams = facts
        .pawns
        .white
        .into_iter()
        .filter(|square| {
            facts
                .pawns
                .black
                .contains(stop_square(scan, *square, Colour::White))
        })
        .count()
        .min(255) as u8;

    for colour in Colour::ALL {
        let them = !colour;
        let ours = facts.pawns.of(colour);

        facts.chain_max_length[colour] = ours
            .into_iter()
            .flat_map(|square| [-1i32, 1].map(|step| chain_run(scan, ours, square, step, colour)))
            .max()
            .unwrap_or(0);
        facts.chain_base_attacked[colour] = ours.into_iter().any(|square| {
            [-1i32, 1].into_iter().any(|step| {
                chain_run(scan, ours, square, step, colour) >= 2
                    && behind(scan, square, step, colour).is_none_or(|back| !ours.contains(back))
                    && scan.by.of(them).contains(square)
            })
        });

        facts.queenside_majority[colour] =
            (ours & QUEEN_SIDE).len() > (facts.pawns.of(them) & QUEEN_SIDE).len();
        facts.kingside_majority[colour] =
            (ours & KING_SIDE).len() > (facts.pawns.of(them) & KING_SIDE).len();

        facts.holes[colour] = holes(scan, ours, colour);
        let minors = scan.role_units.of(them)[Role::Knight.index()]
            | scan.role_units.of(them)[Role::Bishop.index()];
        facts.holes_occupied[colour] = (minors & facts.holes.of(colour)).len().min(255) as u8;

        facts.fixed_pawns[colour] = ours
            .into_iter()
            .filter(|square| scan.occupied.contains(stop_square(scan, *square, colour)))
            .count()
            .min(255) as u8;

        let passers = facts.passed.of(colour);
        facts.blocked_passers[colour] = passers
            .into_iter()
            .filter(|square| {
                scan.units
                    .of(them)
                    .contains(stop_square(scan, *square, colour))
            })
            .count()
            .min(255) as u8;
        facts.passer_free_path[colour] = passers
            .into_iter()
            .filter(|square| {
                let rank = scan.relative_rank(*square, colour);
                (file_mask(square.file()) & ahead_of(scan, rank, colour) & scan.occupied).is_empty()
            })
            .count()
            .min(255) as u8;

        // The lead passer: the most advanced, and among equals the one nearest
        // file a.
        let lead = passers.into_iter().max_by_key(|square| {
            (
                scan.relative_rank(*square, colour),
                7 - square.file().index(),
            )
        });
        facts.passer_distance[colour] =
            lead.map(|square| 8 - scan.relative_rank(square, colour) as u8);
        if let Some(square) = lead {
            let promotion = scan.relative_square(square.file(), 8, colour);
            facts.passer_own_king_distance[colour] =
                Some(distance(scan.kings.of(colour), promotion).min(255) as u8);
            facts.passer_enemy_king_distance[colour] =
                Some(distance(scan.kings.of(them), promotion).min(255) as u8);
        }
        facts.passer_in_square[colour] = lead.is_some_and(|square| in_square(scan, square, colour));

        facts.half_open_at_enemy_king[colour] = shield_files(scan.kings.of(them).file())
            .into_iter()
            .filter(|file| facts.semi_open_files.of(colour).contains(*file))
            .count()
            .min(255) as u8;
        facts.backward_on_semi_open[colour] = facts
            .backward
            .of(colour)
            .into_iter()
            .filter(|square| facts.semi_open_files.of(them).contains(square.file()))
            .count()
            .min(255) as u8;
    }

    facts
}

/// Every square of files a to d.
const QUEEN_SIDE: SquareSet = SquareSet::from_bits(0x0F0F_0F0F_0F0F_0F0F);
/// Every square of files e to h.
const KING_SIDE: SquareSet = SquareSet::from_bits(0xF0F0_F0F0_F0F0_F0F0);

/// The square directly ahead of the pawn of `colour` on `square`.
fn stop_square(scan: &Scan, square: Square, colour: Colour) -> Square {
    scan.relative_square(
        square.file(),
        scan.relative_rank(square, colour) + 1,
        colour,
    )
}

/// The square one rank behind `square` on the diagonal `step`, from `colour`'s
/// point of view; `None` off the board.
fn behind(scan: &Scan, square: Square, step: i32, colour: Colour) -> Option<Square> {
    let file = square.file().index() as i32 - step;
    let rank = scan.relative_rank(square, colour);
    if !(0..8).contains(&file) || rank <= 1 {
        return None;
    }
    Some(scan.relative_square(
        File::from_index(file as usize).expect("a file index below 8"),
        rank - 1,
        colour,
    ))
}

/// How many pawns of `ours` stand from `square` on, each defending the next,
/// one file per rank in the direction `step`.
fn chain_run(scan: &Scan, ours: SquareSet, square: Square, step: i32, colour: Colour) -> u8 {
    let mut length = 1;
    let mut file = square.file().index() as i32;
    let mut rank = scan.relative_rank(square, colour);
    loop {
        file += step;
        rank += 1;
        if !(0..8).contains(&file) || rank > 8 {
            return length;
        }
        let file = File::from_index(file as usize).expect("a file index below 8");
        if !ours.contains(scan.relative_square(file, rank, colour)) {
            return length;
        }
        length += 1;
    }
}

/// The squares on `colour`'s ranks 3 to 6 that no pawn of `ours` can ever
/// attack.
fn holes(scan: &Scan, ours: SquareSet, colour: Colour) -> SquareSet {
    let mut lowest = [9u32; 8];
    for square in ours {
        let file = square.file().index();
        lowest[file] = lowest[file].min(scan.relative_rank(square, colour));
    }
    let mut set = SquareSet::EMPTY;
    for square in Square::ALL {
        let rank = scan.relative_rank(square, colour);
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

/// Whether the king defending against `colour`'s passer on `square` is in its
/// square by the rule of the square.
fn in_square(scan: &Scan, square: Square, colour: Colour) -> bool {
    let defender = !colour;
    let rank = scan.relative_rank(square, colour);
    let promotion = scan.relative_square(square.file(), 8, colour);
    let tempo = u32::from(defender == scan.us);
    distance(scan.kings.of(defender), promotion).saturating_sub(tempo) <= 8 - rank
}

/// Whether the enemy king cannot catch `colour`'s passer on `square` by the
/// rule of the square, its own side having no unit but the king to help.
fn unstoppable(scan: &Scan, square: Square, colour: Colour) -> bool {
    let defender = !colour;
    let has_pieces = [Role::Knight, Role::Bishop, Role::Rook, Role::Queen]
        .iter()
        .any(|role| !scan.role_units.of(defender)[role.index()].is_empty());
    if has_pieces {
        return false;
    }
    let rank = scan.relative_rank(square, colour);
    let promotion = scan.relative_square(square.file(), 8, colour);
    let tempo = u32::from(defender == scan.us);
    distance(scan.kings.of(defender), promotion).saturating_sub(tempo) > 8 - rank
}
