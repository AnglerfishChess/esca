//! King safety and shelter.

use crate::types::{File, Rank, Role, Square, SquareSet};

use super::scan::{Scan, attacks_of, distance};
use super::{CastledSide, KingFacts, Side, tropism};

/// The eight directions a ray leaves a king by, as (file, rank) steps.
const RAYS: [(i32, i32); 8] = [
    (0, 1),
    (1, 1),
    (1, 0),
    (1, -1),
    (0, -1),
    (-1, -1),
    (-1, 0),
    (-1, 1),
];

/// The squares of a file.
fn file_mask(file: File) -> SquareSet {
    SquareSet::from_bits(0x0101_0101_0101_0101u64 << file.index())
}

/// The three files a king's shelter is read on: its own file clamped to b–g,
/// and the two neighbours of that, in ascending order.
pub(super) fn shield_files(king: File) -> [File; 3] {
    let centre = king.index().clamp(1, 6);
    [centre - 1, centre, centre + 1].map(|i| File::from_index(i).expect("b to g has neighbours"))
}

/// The squares `side` attacks with anything but its king.
fn guarded_by(scan: &Scan, side: Side) -> SquareSet {
    Role::ALL
        .into_iter()
        .filter(|&role| role != Role::King)
        .fold(SquareSet::EMPTY, |set, role| {
            set | scan.by_role[side.index()][role.index()]
        })
}

/// The weight a ring attacker of `role` carries.
fn ring_weight(role: Role) -> u8 {
    match role {
        Role::Knight | Role::Bishop => 1,
        Role::Rook => 2,
        Role::Queen => 4,
        _ => 0,
    }
}

pub(super) fn king_facts(scan: &Scan) -> KingFacts {
    let mut facts = KingFacts {
        square: scan.kings,
        on_home_square: [false; 2],
        castled_queenside: [false; 2],
        castled_kingside: [false; 2],
        shield_files: [
            shield_files(scan.kings[0].file()),
            shield_files(scan.kings[1].file()),
        ],
        shield: [[None; 3]; 2],
        file_open: [[false; 3]; 2],
        file_semi_open_for_enemy: [[false; 3]; 2],
        storm: [[None; 3]; 2],
        ring: [SquareSet::EMPTY; 2],
        ring_attackers: [0; 2],
        ring_attack_weight: [0; 2],
        ring_defended: [0; 2],
        ring_holes: [0; 2],
        escape_squares: [0; 2],
        back_rank_risk: [false; 2],
        distance: distance(scan.kings[0], scan.kings[1]).min(255) as u8,
        tropism: [0.0; 2],
        virtual_mobility: [0; 2],
        ring_defenders: [0; 2],
        ring_defence_weight: [0; 2],
        open_rays: [0; 2],
        luft: [false; 2],
        castled_side: [None; 2],
        opposite_side_castling: queen_side(scan.kings[0]) != queen_side(scan.kings[1]),
    };

    for side in Side::ALL {
        let i = side.index();
        let them = (!side).index();
        let king = scan.kings[i];
        let rank = scan.relative_rank(king, side);
        let ours = scan.role_units[i][Role::Pawn.index()];
        let theirs = scan.role_units[them][Role::Pawn.index()];

        facts.on_home_square[i] = king.file() == File::E && rank == 1;
        facts.castled_queenside[i] = king.file() <= File::C;
        facts.castled_kingside[i] = king.file() >= File::F;

        for (slot, &file) in facts.shield_files[i].iter().enumerate() {
            let mask = file_mask(file);
            facts.file_open[i][slot] = (mask & (ours | theirs)).is_empty();
            facts.file_semi_open_for_enemy[i][slot] =
                (mask & theirs).is_empty() && !(mask & ours).is_empty();
            facts.shield[i][slot] = nearest_ahead(scan, mask & ours, rank, side);
            facts.storm[i][slot] = nearest_ahead(scan, mask & theirs, rank, side);
        }

        let ring = attacks_of(Role::King, king, scan.colour(side), scan.occupied);
        facts.ring[i] = ring;
        for role in [Role::Knight, Role::Bishop, Role::Rook, Role::Queen] {
            for square in scan.role_units[them][role.index()] {
                if !(scan.attacks_from[square.index()] & ring).is_empty() {
                    facts.ring_attackers[i] = facts.ring_attackers[i].saturating_add(1);
                    facts.ring_attack_weight[i] =
                        facts.ring_attack_weight[i].saturating_add(ring_weight(role));
                }
            }
            for square in scan.role_units[i][role.index()] {
                if !(scan.attacks_from[square.index()] & ring).is_empty() {
                    facts.ring_defenders[i] = facts.ring_defenders[i].saturating_add(1);
                    facts.ring_defence_weight[i] =
                        facts.ring_defence_weight[i].saturating_add(ring_weight(role));
                }
            }
        }
        let guarded = guarded_by(scan, side);
        facts.ring_defended[i] = (ring & guarded).len().min(255) as u8;
        facts.ring_holes[i] = ((ring & scan.by[them]) - guarded).len().min(255) as u8;
        facts.escape_squares[i] = ((ring - scan.units[i]) - scan.by[them]).len().min(255) as u8;

        let ahead = ahead_of_the_back_rank(scan, king, side);
        facts.back_rank_risk[i] = rank == 1 && ahead.is_subset(scan.units[i]);
        facts.luft[i] = rank == 1 && !((ahead - scan.occupied) - scan.by[them]).is_empty();

        facts.castled_side[i] = if scan.castling[i] {
            None
        } else if king.file() >= File::G {
            Some(CastledSide::Short)
        } else if king.file() <= File::C {
            Some(CastledSide::Long)
        } else {
            None
        };

        facts.open_rays[i] = open_rays(king, scan.occupied);
        facts.tropism[i] = tropism(scan, king, !side);
        facts.virtual_mobility[i] = attacks_of(Role::Queen, king, scan.colour(side), scan.occupied)
            .len()
            .min(255) as u8;
    }

    facts
}

/// Whether a square falls on the queen-side, files a to d.
fn queen_side(square: Square) -> bool {
    square.file() <= File::D
}

/// The up-to-three squares of relative rank 2 adjacent to `king`, in `side`'s
/// frame. Meaningful for a king on its relative rank 1.
fn ahead_of_the_back_rank(scan: &Scan, king: Square, side: Side) -> SquareSet {
    [
        File::from_index(king.file().index().wrapping_sub(1)),
        Some(king.file()),
        File::from_index(king.file().index() + 1),
    ]
    .into_iter()
    .flatten()
    .map(|file| scan.relative_square(file, 2, side).to_set())
    .fold(SquareSet::EMPTY, |a, b| a | b)
}

/// Directions from `king` holding at least one square of the board, none of
/// them occupied.
fn open_rays(king: Square, occupied: SquareSet) -> u8 {
    let mut open = 0;
    for (step_file, step_rank) in RAYS {
        let mut file = king.file().index() as i32 + step_file;
        let mut rank = king.rank().index() as i32 + step_rank;
        let mut on_board = false;
        let mut clear = true;
        while let (Some(f), Some(r)) = (
            usize::try_from(file).ok().and_then(File::from_index),
            usize::try_from(rank).ok().and_then(Rank::from_index),
        ) {
            on_board = true;
            if occupied.contains(Square::new(f, r)) {
                clear = false;
                break;
            }
            file += step_file;
            rank += step_rank;
        }
        open += u8::from(on_board && clear);
    }
    open
}

/// Ranks from a king at relative rank `rank` to the nearest pawn of `pawns`
/// ahead of it, in `side`'s frame.
fn nearest_ahead(scan: &Scan, pawns: SquareSet, rank: u32, side: Side) -> Option<u8> {
    pawns
        .into_iter()
        .map(|square| scan.relative_rank(square, side))
        .filter(|&pawn_rank| pawn_rank > rank)
        .min()
        .map(|pawn_rank| (pawn_rank - rank) as u8)
}
