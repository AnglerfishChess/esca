//! What is about to be lost: the exchange each unit faces, the defenders that
//! do not hold, and the slider geometry a threat is made of.

use crate::position::Position;
use crate::types::{Colour, Role, Square, SquareSet};

use super::ThreatFacts;
use super::exchange::signed_see;
use super::scan::{Scan, attacks_of, between, line, material_value, order_value};

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
    for colour in Colour::ALL {
        let them = !colour;
        // A king is never captured, so it is in none of these sets.
        let units = scan.units.of(colour) - scan.role_units.of(colour)[Role::King.index()];
        // The sole defenders of an attacked friendly unit; the second such unit
        // a defender holds makes it overloaded.
        let mut sole = SquareSet::EMPTY;

        for square in units {
            let role = role_at(position, square);
            let attackers = scan.attackers_of(square, them);
            let defenders = scan.attackers_of(square, colour);

            let see = signed_see(position, square).unwrap_or(0).max(0);
            if see > 0 {
                facts.threatened[colour].insert(square);
                facts.threatened_value[colour] += material_value(role);
            }
            facts.threat_max_gain[colour] = facts.threat_max_gain.of(colour).max(see);

            if attackers
                .into_iter()
                .any(|from| order_value(role_at(position, from)) < order_value(role))
            {
                facts.attacked_by_lesser[colour].insert(square);
                facts.queen_attacked_by_lesser[colour] |= role == Role::Queen;
            }

            if defenders.is_empty() {
                facts.loose[colour].insert(square);
            }

            let limit = order_value(role);
            let cheap = |set: SquareSet| {
                set.into_iter()
                    .filter(|from| order_value(role_at(position, *from)) <= limit)
                    .count()
            };
            if cheap(attackers) > cheap(defenders) {
                facts.attacker_surplus[colour].insert(square);
            }

            if !attackers.is_empty() && defenders.len() == 1 {
                let defender = defenders.first().expect("a set of one has a member");
                if sole.contains(defender) {
                    facts.overloaded_defenders[colour].insert(defender);
                }
                sole.insert(defender);
            }
        }

        facts.removable_defenders[colour] = sole
            .into_iter()
            .filter(|&defender| signed_see(position, defender).is_some_and(|see| see >= 0))
            .collect();

        facts.xray_through_enemy[colour] = xrays_through_enemy(scan, colour);
        let (batteries, at_king) = batteries(position, scan, colour);
        facts.battery_count[colour] = batteries;
        facts.battery_at_king[colour] = at_king;
    }
    facts
}

/// How many (slider, target) pairs of `colour` x-ray an enemy unit through one
/// enemy unit.
fn xrays_through_enemy(scan: &Scan, colour: Colour) -> u8 {
    let them = !colour;
    let mut count = 0u8;
    for role in [Role::Bishop, Role::Rook, Role::Queen] {
        for slider in scan.role_units.of(colour)[role.index()] {
            let attacks = scan.attacks_from[slider.index()];
            for front in attacks & scan.units.of(them) {
                let xray = attacks_of(role, slider, colour, scan.occupied - front.to_set());
                let behind = (xray - attacks) & line(slider, front) & scan.units.of(them);
                if !behind.is_empty() {
                    count = count.saturating_add(1);
                }
            }
        }
    }
    count
}

/// `colour`'s batteries, and whether one of their lines holds a square of the
/// enemy king ring.
fn batteries(position: &Position, scan: &Scan, colour: Colour) -> (u8, bool) {
    let sliders = scan.role_units.of(colour)[Role::Bishop.index()]
        | scan.role_units.of(colour)[Role::Rook.index()]
        | scan.role_units.of(colour)[Role::Queen.index()];
    let ring = attacks_of(
        Role::King,
        scan.kings.of(!colour),
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
