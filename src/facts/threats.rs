//! What is about to be lost: the exchange each unit faces, the defenders that
//! do not hold, and the slider geometry a threat is made of.

use crate::position::Position;
use crate::types::{Role, Square, SquareSet};

use super::exchange::signed_see;
use super::scan::{Scan, attacks_of, between, line, material_value, order_value};
use super::{Side, ThreatFacts};

/// The role of the unit standing on `square`.
fn role_at(position: &Position, square: Square) -> Role {
    position
        .piece_at(square)
        .expect("a unit stands on its own square")
        .role
}

/// The `threats` group of `position`.
pub(super) fn threat_facts(position: &Position, scan: &Scan) -> ThreatFacts {
    let mut facts = ThreatFacts::default();
    for side in Side::ALL {
        let i = side.index();
        let them = !side;
        // A king is never captured, so it is in none of these sets.
        let units = scan.units[i] - scan.role_units[i][Role::King.index()];
        // The sole defenders of an attacked friendly unit; the second such unit
        // a defender holds makes it overloaded.
        let mut sole = SquareSet::EMPTY;

        for square in units {
            let role = role_at(position, square);
            let attackers = scan.attackers_of(square, them);
            let defenders = scan.attackers_of(square, side);

            let see = signed_see(position, square).unwrap_or(0).max(0);
            if see > 0 {
                facts.threatened[i].insert(square);
                facts.threatened_value[i] += material_value(role);
            }
            facts.threat_max_gain[i] = facts.threat_max_gain[i].max(see);

            if attackers
                .into_iter()
                .any(|from| order_value(role_at(position, from)) < order_value(role))
            {
                facts.attacked_by_lesser[i].insert(square);
                facts.queen_attacked_by_lesser[i] |= role == Role::Queen;
            }

            if defenders.is_empty() {
                facts.loose[i].insert(square);
            }

            let limit = order_value(role);
            let cheap = |set: SquareSet| {
                set.into_iter()
                    .filter(|from| order_value(role_at(position, *from)) <= limit)
                    .count()
            };
            if cheap(attackers) > cheap(defenders) {
                facts.attacker_surplus[i].insert(square);
            }

            if !attackers.is_empty() && defenders.len() == 1 {
                let defender = defenders.first().expect("a set of one has a member");
                if sole.contains(defender) {
                    facts.overloaded_defenders[i].insert(defender);
                }
                sole.insert(defender);
            }
        }

        facts.removable_defenders[i] = sole
            .into_iter()
            .filter(|&defender| signed_see(position, defender).is_some_and(|see| see >= 0))
            .collect();

        facts.xray_through_enemy[i] = xrays_through_enemy(scan, side);
        let (batteries, at_king) = batteries(position, scan, side);
        facts.battery_count[i] = batteries;
        facts.battery_at_king[i] = at_king;
    }
    facts
}

/// How many (slider, target) pairs of `side` x-ray an enemy unit through one
/// enemy unit.
fn xrays_through_enemy(scan: &Scan, side: Side) -> u8 {
    let i = side.index();
    let them = (!side).index();
    let mut count = 0u8;
    for role in [Role::Bishop, Role::Rook, Role::Queen] {
        for slider in scan.role_units[i][role.index()] {
            let attacks = scan.attacks_from[slider.index()];
            for front in attacks & scan.units[them] {
                let xray = attacks_of(
                    role,
                    slider,
                    scan.colour(side),
                    scan.occupied - front.to_set(),
                );
                let behind = (xray - attacks) & line(slider, front) & scan.units[them];
                if !behind.is_empty() {
                    count = count.saturating_add(1);
                }
            }
        }
    }
    count
}

/// `side`'s batteries, and whether one of their lines holds a square of the
/// enemy king ring.
fn batteries(position: &Position, scan: &Scan, side: Side) -> (u8, bool) {
    let i = side.index();
    let colour = scan.colour(side);
    let sliders = scan.role_units[i][Role::Bishop.index()]
        | scan.role_units[i][Role::Rook.index()]
        | scan.role_units[i][Role::Queen.index()];
    let ring = attacks_of(
        Role::King,
        scan.kings[(!side).index()],
        !colour,
        SquareSet::EMPTY,
    );
    let mut count = 0u8;
    let mut at_king = false;
    for first in sliders {
        for second in sliders {
            if second.index() <= first.index() {
                continue;
            }
            let ray = line(first, second);
            if ray.is_empty() {
                continue;
            }
            let straight = attacks_of(Role::Rook, first, colour, SquareSet::EMPTY).contains(second);
            let holds = |square: Square| {
                sliders.contains(square) && moves_along(role_at(position, square), straight)
            };
            if !holds(first) || !holds(second) {
                continue;
            }
            if (between(first, second) & scan.occupied)
                .into_iter()
                .any(|square| !holds(square))
            {
                continue;
            }
            count = count.saturating_add(1);
            at_king |= !(ray & ring).is_empty();
        }
    }
    (count, at_king)
}

/// Whether a role moves along a rank or file, or along a diagonal.
fn moves_along(role: Role, straight: bool) -> bool {
    match role {
        Role::Queen => true,
        Role::Rook => straight,
        Role::Bishop => !straight,
        _ => false,
    }
}
