//! King safety and shelter.

use crate::types::{ByColour, Colour, File, Rank, Role, Square, SquareSet};

use super::scan::{Scan, attacks_of, distance};
use super::{CastledSide, KingFacts, tropism};

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

/// The squares `colour` attacks with anything but its king.
fn guarded_by(scan: &Scan, colour: Colour) -> SquareSet {
    Role::ALL
        .into_iter()
        .filter(|&role| role != Role::King)
        .fold(SquareSet::EMPTY, |set, role| {
            set | scan.by_role.of(colour)[role.index()]
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
        on_home_square: ByColour::default(),
        castled_queenside: ByColour::default(),
        castled_kingside: ByColour::default(),
        shield_files: scan.kings.map(|king| shield_files(king.file())),
        shield: ByColour::new([None; 3], [None; 3]),
        shield_file_open: ByColour::default(),
        shield_file_semi_open_for_enemy: ByColour::default(),
        storm: ByColour::new([None; 3], [None; 3]),
        ring: ByColour::default(),
        ring_attackers: ByColour::default(),
        ring_attack_weight: ByColour::default(),
        ring_defended: ByColour::default(),
        ring_holes: ByColour::default(),
        escape_squares: ByColour::default(),
        back_rank_risk: ByColour::default(),
        distance: distance(scan.kings.white, scan.kings.black).min(255) as u8,
        tropism: ByColour::default(),
        virtual_mobility: ByColour::default(),
        ring_defenders: ByColour::default(),
        ring_defence_weight: ByColour::default(),
        open_rays: ByColour::default(),
        luft: ByColour::default(),
        castled_side: ByColour::default(),
        opposite_side_castling: queen_side(scan.kings.white) != queen_side(scan.kings.black),
    };

    for colour in Colour::ALL {
        let them = !colour;
        let king = scan.kings.of(colour);
        let rank = scan.relative_rank(*king, colour);
        let ours = scan.role_units.of(colour)[Role::Pawn.index()];
        let theirs = scan.role_units.of(them)[Role::Pawn.index()];

        facts.on_home_square[colour] = king.file() == File::E && rank == 1;
        facts.castled_queenside[colour] = king.file() <= File::C;
        facts.castled_kingside[colour] = king.file() >= File::F;

        for (slot, file) in facts.shield_files.of(colour).iter().enumerate() {
            let mask = file_mask(*file);
            facts.shield_file_open[colour][slot] = (mask & (ours | theirs)).is_empty();
            facts.shield_file_semi_open_for_enemy[colour][slot] =
                (mask & theirs).is_empty() && !(mask & ours).is_empty();
            facts.shield[colour][slot] = nearest_ahead(scan, mask & ours, rank, colour);
            facts.storm[colour][slot] = nearest_ahead(scan, mask & theirs, rank, colour);
        }

        let ring = attacks_of(Role::King, *king, colour, scan.occupied);
        facts.ring[colour] = ring;
        for role in [Role::Knight, Role::Bishop, Role::Rook, Role::Queen] {
            for square in scan.role_units.of(them)[role.index()] {
                if !(scan.attacks_from[square.index()] & ring).is_empty() {
                    facts.ring_attackers[colour] =
                        facts.ring_attackers.of(colour).saturating_add(1);
                    facts.ring_attack_weight[colour] = facts
                        .ring_attack_weight
                        .of(colour)
                        .saturating_add(ring_weight(role));
                }
            }
            for square in scan.role_units.of(colour)[role.index()] {
                if !(scan.attacks_from[square.index()] & ring).is_empty() {
                    facts.ring_defenders[colour] =
                        facts.ring_defenders.of(colour).saturating_add(1);
                    facts.ring_defence_weight[colour] = facts
                        .ring_defence_weight
                        .of(colour)
                        .saturating_add(ring_weight(role));
                }
            }
        }
        let guarded = guarded_by(scan, colour);
        facts.ring_defended[colour] = (ring & guarded).len().min(255) as u8;
        facts.ring_holes[colour] = ((ring & *scan.by.of(them)) - guarded).len().min(255) as u8;
        facts.escape_squares[colour] = ((ring - *scan.units.of(colour)) - *scan.by.of(them))
            .len()
            .min(255) as u8;

        let ahead = ahead_of_the_back_rank(scan, *king, colour);
        facts.back_rank_risk[colour] = rank == 1 && ahead.is_subset(*scan.units.of(colour));
        facts.luft[colour] = rank == 1 && !((ahead - scan.occupied) - *scan.by.of(them)).is_empty();

        facts.castled_side[colour] = if *scan.castling.of(colour) {
            None
        } else if king.file() >= File::G {
            Some(CastledSide::Short)
        } else if king.file() <= File::C {
            Some(CastledSide::Long)
        } else {
            None
        };

        facts.open_rays[colour] = open_rays(*king, scan.occupied);
        facts.tropism[colour] = tropism(scan, *king, them);
        facts.virtual_mobility[colour] = attacks_of(Role::Queen, *king, colour, scan.occupied)
            .len()
            .min(255) as u8;
    }

    facts
}

/// Whether a square falls on the queen-side, files a to d.
fn queen_side(square: Square) -> bool {
    square.file() <= File::D
}

/// The up-to-three squares of `colour`'s rank 2 adjacent to `king`.
/// Meaningful for a king on its own rank 1.
fn ahead_of_the_back_rank(scan: &Scan, king: Square, colour: Colour) -> SquareSet {
    [
        File::from_index(king.file().index().wrapping_sub(1)),
        Some(king.file()),
        File::from_index(king.file().index() + 1),
    ]
    .into_iter()
    .flatten()
    .map(|file| scan.relative_square(file, 2, colour).to_set())
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

/// Ranks from a king of `colour` at its rank `rank` to the nearest pawn of
/// `pawns` ahead of it.
fn nearest_ahead(scan: &Scan, pawns: SquareSet, rank: u32, colour: Colour) -> Option<u8> {
    pawns
        .into_iter()
        .map(|square| scan.relative_rank(square, colour))
        .filter(|&pawn_rank| pawn_rank > rank)
        .min()
        .map(|pawn_rank| (pawn_rank - rank) as u8)
}
