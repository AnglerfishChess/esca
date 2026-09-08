//! Bishops, rooks, knights and queens.

use crate::types::{Colour, File, Role, Square, SquareSet};

use super::scan::{Scan, attackers, attacks_of, between, material_value, order_value};
use super::{PawnFacts, PieceFacts};

/// The roles a trapped unit may have: neither a pawn nor a king.
const TRAPPABLE: [Role; 4] = [Role::Knight, Role::Bishop, Role::Rook, Role::Queen];

/// The squares of a file.
fn file_mask(file: File) -> SquareSet {
    SquareSet::from_bits(0x0101_0101_0101_0101u64 << file.index())
}

/// The starting squares of `colour`'s knights and bishops in classic chess.
fn minor_home_squares(scan: &Scan, colour: Colour) -> SquareSet {
    [File::B, File::C, File::F, File::G]
        .into_iter()
        .map(|file| scan.relative_square(file, 1, colour).to_set())
        .fold(SquareSet::EMPTY, |a, b| a | b)
}

pub(super) fn piece_facts(scan: &Scan, pawns: &PawnFacts) -> PieceFacts {
    let mut facts = PieceFacts::default();

    for colour in Colour::ALL {
        let them = !colour;
        let bishops = scan.role_units.of(colour)[Role::Bishop.index()];
        let rooks = scan.role_units.of(colour)[Role::Rook.index()];
        let knights = scan.role_units.of(colour)[Role::Knight.index()];
        let queens = scan.role_units.of(colour)[Role::Queen.index()];

        let light = bishops & SquareSet::LIGHT;
        let dark = bishops & SquareSet::DARK;
        facts.bishops_light[colour] = light.len().min(255) as u8;
        facts.bishops_dark[colour] = dark.len().min(255) as u8;
        facts.bishop_pair[colour] = super::has_bishop_pair(scan, colour);
        facts.knight_pair[colour] = knights.len() >= 2;

        let mut colours = SquareSet::EMPTY;
        if !light.is_empty() {
            colours |= SquareSet::LIGHT;
        }
        if !dark.is_empty() {
            colours |= SquareSet::DARK;
        }
        facts.pawns_on_bishop_colour[colour] =
            (*pawns.pawns.of(colour) & colours).len().min(255) as u8;
        facts.fixed_pawns_on_bishop_colour[colour] = (*pawns.pawns.of(colour) & colours)
            .into_iter()
            .filter(|pawn| scan.occupied.contains(stop_square(scan, *pawn, colour)))
            .count()
            .min(255) as u8;

        for a in rooks {
            for b in rooks {
                if a >= b {
                    continue;
                }
                if (between(a, b) & scan.occupied).is_empty() {
                    if a.rank() == b.rank() {
                        facts.rooks_connected_rank[colour] = true;
                    }
                    if a.file() == b.file() {
                        facts.rooks_connected_file[colour] = true;
                    }
                }
            }
        }

        for rook in rooks {
            let file = rook.file();
            if pawns.open_files.contains(file) {
                facts.rooks_on_open_file[colour] += 1;
            }
            if pawns.semi_open_files.of(colour).contains(file) {
                facts.rooks_on_semi_open_file[colour] += 1;
            }
            if scan.relative_rank(rook, colour) == 7 {
                facts.rooks_on_seventh_rank[colour] += 1;
            }
            if behind_a_passer(scan, rook, *pawns.passed.of(colour), colour) {
                facts.rook_behind_own_passer[colour] += 1;
            }
            if behind_a_passer(scan, rook, *pawns.passed.of(them), them) {
                facts.rook_behind_enemy_passer[colour] += 1;
            }
        }

        facts.trapped_rook[colour] = trapped_rook(scan, colour, rooks);
        facts.rook_on_seventh_with_king_on_eighth[colour] = *facts.rooks_on_seventh_rank.of(colour)
            > 0
            && scan.relative_rank(*scan.kings.of(them), colour) == 8;

        let (trapped, value) = trapped_pieces(scan, colour);
        facts.trapped_pieces[colour] = trapped;
        facts.trapped_value[colour] = value;

        facts.outposts[colour] = outposts(scan, pawns, colour);
        facts.minors_on_outpost[colour] = ((knights | bishops) & *facts.outposts.of(colour))
            .len()
            .min(255) as u8;
        facts.outpost_squares_free[colour] =
            (*facts.outposts.of(colour) - scan.occupied).len().min(255) as u8;
        facts.knights_on_rim[colour] = knights
            .into_iter()
            .filter(|square| {
                let rank = scan.relative_rank(*square, colour);
                square.file() == File::A || square.file() == File::H || rank == 1 || rank == 8
            })
            .count()
            .min(255) as u8;

        facts.minors_undeveloped[colour] = ((knights | bishops) & minor_home_squares(scan, colour))
            .len()
            .min(255) as u8;
        let queen_home = scan.relative_square(File::D, 1, colour);
        facts.queen_developed[colour] = !(queens - queen_home.to_set()).is_empty();
    }

    facts.opposite_coloured_bishops = facts.bishops_light.white + facts.bishops_dark.white == 1
        && facts.bishops_light.black + facts.bishops_dark.black == 1
        && facts.bishops_light.white != facts.bishops_light.black;

    facts
}

/// The square directly ahead of the pawn of `colour` on `square`.
fn stop_square(scan: &Scan, square: Square, colour: Colour) -> Square {
    scan.relative_square(
        square.file(),
        scan.relative_rank(square, colour) + 1,
        colour,
    )
}

/// How many of `colour`'s units are trapped, and what they are worth.
fn trapped_pieces(scan: &Scan, colour: Colour) -> (u8, u8) {
    let mut count = 0u32;
    let mut value = 0i32;
    for role in TRAPPABLE {
        for square in scan.role_units.of(colour)[role.index()] {
            let destinations = scan.attacks_from[square.index()] - *scan.units.of(colour);
            if destinations
                .into_iter()
                .any(|to| is_safe(scan, colour, role, square, to))
            {
                continue;
            }
            count += 1;
            value += material_value(role);
        }
    }
    (count.min(255) as u8, value.clamp(0, 255) as u8)
}

/// Whether `to` is a safe destination for the unit of `colour` and `role`
/// standing on `from`, read in the position that move would leave.
fn is_safe(scan: &Scan, colour: Colour, role: Role, from: Square, to: Square) -> bool {
    let them = !colour;
    let occupied = (scan.occupied - from.to_set()) | to.to_set();

    // The move takes whatever stood on `to` and leaves `from` empty.
    let mut their_units = *scan.role_units.of(them);
    for set in &mut their_units {
        *set -= to.to_set();
    }
    let mut our_units = *scan.role_units.of(colour);
    our_units[role.index()] = (our_units[role.index()] - from.to_set()) | to.to_set();

    let by_pawn = attacks_of(Role::Pawn, to, colour, occupied);
    if !(by_pawn & their_units[Role::Pawn.index()]).is_empty() {
        return false;
    }
    let enemy = attackers(to, them, &their_units, occupied);
    if enemy.is_empty() {
        return true;
    }
    let cheaper = TRAPPABLE
        .into_iter()
        .filter(|other| order_value(*other) < order_value(role))
        .any(|other| !(enemy & their_units[other.index()]).is_empty());
    !cheaper && !attackers(to, colour, &our_units, occupied).is_empty()
}

/// Whether `rook` stands on the file of one of `owner`'s `passers` and behind
/// it from the owner's point of view.
fn behind_a_passer(scan: &Scan, rook: Square, passers: SquareSet, owner: Colour) -> bool {
    (passers & file_mask(rook.file()))
        .into_iter()
        .any(|passer| scan.relative_rank(rook, owner) < scan.relative_rank(passer, owner))
}

/// Whether a rook of `colour` is boxed in: at most two non-capture
/// destinations, beyond its own king on the wing the king stands on, the king
/// having lost its castling rights.
fn trapped_rook(scan: &Scan, colour: Colour, rooks: SquareSet) -> bool {
    if *scan.castling.of(colour) {
        return false;
    }
    let king = scan.kings.of(colour);
    rooks.into_iter().any(|rook| {
        let outside = if king.file() >= File::E {
            rook.file() > king.file()
        } else {
            rook.file() < king.file()
        };
        outside && (scan.attacks_from[rook.index()] - scan.occupied).len() <= 2
    })
}

/// The squares on `colour`'s ranks 4 to 6 that one of its pawns attacks and no
/// enemy pawn can ever attack.
fn outposts(scan: &Scan, pawns: &PawnFacts, colour: Colour) -> SquareSet {
    let theirs = pawns.pawns.of(!colour);
    let mut set = SquareSet::EMPTY;
    for square in scan.by_role.of(colour)[Role::Pawn.index()] {
        let rank = scan.relative_rank(square, colour);
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
            (*theirs & file_mask(file))
                .into_iter()
                .any(|pawn| scan.relative_rank(pawn, colour) >= rank)
        });
        if !attackable {
            set.insert(square);
        }
    }
    set
}
