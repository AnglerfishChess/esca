//! One-ply tactics, and the facts of a single legal move.

use core::cmp::Ordering;

use crate::moves::{Move, MoveList};
use crate::position::Position;
use crate::types::{Colour, File, Role, Square, SquareSet};
use crate::variant::Variant;

use super::scan::{
    Scan, attackers, attacks_of, between, line, material_value, order_value, target_value,
};
use super::{
    AnnotatedMove, AttackFacts, KingFacts, MoveFacts, PawnFacts, Side, TacticsFacts, king, pawns,
};

/// The square the moved unit ends on: for castling the king's destination,
/// which is c1 or g1 in the mover's frame.
fn landing(mv: Move) -> Square {
    if mv.is_castling() {
        let file = if mv.to().file() > mv.from().file() {
            File::G
        } else {
            File::C
        };
        Square::new(file, mv.from().rank())
    } else {
        mv.to()
    }
}

/// The squares the move leaves a unit of the mover on.
fn moved_to(mv: Move) -> SquareSet {
    let mut set = landing(mv).to_set();
    if mv.is_castling() {
        let file = if mv.to().file() > mv.from().file() {
            File::F
        } else {
            File::D
        };
        set.insert(Square::new(file, mv.from().rank()));
    }
    set
}

/// The square the captured unit stands on before the move.
fn victim_square(mv: Move) -> Square {
    if mv.is_en_passant() {
        Square::new(mv.to().file(), mv.from().rank())
    } else {
        mv.to()
    }
}

/// A colour's placement by role.
fn role_units(position: &Position, colour: Colour) -> [SquareSet; 6] {
    let ours = position.by_colour(colour);
    Role::ALL.map(|role| ours & position.by_role(role))
}

/// The units of `side` that an enemy unit attacks and no friendly unit
/// defends. A king is never among them.
fn hanging_of(scan: &Scan, side: Side) -> SquareSet {
    let i = side.index();
    ((scan.units[i] - scan.role_units[i][Role::King.index()]) & scan.by[(!side).index()])
        - scan.by[i]
}

/// The units of `side` worth 3 or more: its knights, bishops, rooks and
/// queens.
fn heavy_units(scan: &Scan, side: Side) -> SquareSet {
    let i = side.index();
    [Role::Knight, Role::Bishop, Role::Rook, Role::Queen]
        .into_iter()
        .fold(SquareSet::EMPTY, |set, role| {
            set | scan.role_units[i][role.index()]
        })
}

/// How many enemy knights, bishops, rooks and queens attack `side`'s king
/// ring.
fn ring_attackers(scan: &Scan, side: Side) -> i32 {
    let king = scan.kings[side.index()];
    let ring = attacks_of(Role::King, king, scan.colour(side), scan.occupied);
    let them = (!side).index();
    [Role::Knight, Role::Bishop, Role::Rook, Role::Queen]
        .into_iter()
        .flat_map(|role| scan.role_units[them][role.index()])
        .filter(|square| !(scan.attacks_from[square.index()] & ring).is_empty())
        .count() as i32
}

/// What a slider of `mover` that `mv` leaves standing gains an attack on.
#[derive(Clone, Copy, Default)]
struct Discovered {
    /// A unit worth 3 or more.
    on_heavy: bool,
    /// The enemy queen.
    on_queen: bool,
}

/// What the sliders of `mover` that `mv` leaves standing come to attack.
fn discovered_attack(
    after: &Position,
    scan: &Scan,
    mover: Side,
    mover_colour: Colour,
    mv: Move,
    enemy_units: SquareSet,
) -> Discovered {
    let occupied = after.occupied();
    let stationary = moved_to(mv) | mv.from().to_set() | mv.to().to_set();
    let mut found = Discovered::default();
    for slider in [Role::Bishop, Role::Rook, Role::Queen] {
        for from in scan.role_units[mover.index()][slider.index()] - stationary {
            let gained =
                attacks_of(slider, from, mover_colour, occupied) - scan.attacks_from[from.index()];
            for target in gained & enemy_units {
                let role = after
                    .piece_at(target)
                    .expect("a target stands on its own square")
                    .role;
                found.on_heavy |= material_value(role) >= 3;
                found.on_queen |= role == Role::Queen;
            }
            if found.on_heavy && found.on_queen {
                return found;
            }
        }
    }
    found
}

/// The units of `colour` an exchange can be read on: all but the king.
fn takeable(position: &Position, colour: Colour) -> SquareSet {
    position.by_colour(colour) - position.by_role(Role::King)
}

/// Whether some unit of `colour` has an SEE of a unit above `best`.
fn threatens_more_than(position: &Position, colour: Colour, best: i32) -> bool {
    takeable(position, colour)
        .into_iter()
        .any(|square| position.see(square) > best)
}

/// The largest SEE of a unit over the units of `colour`.
fn max_threat(position: &Position, colour: Colour) -> i32 {
    takeable(position, colour)
        .into_iter()
        .map(|square| position.see(square))
        .max()
        .unwrap_or(0)
}

/// What the position before a move says about it: the reading every move of
/// one call to [`tactics`] shares.
struct Before<'a> {
    scan: &'a Scan,
    attacks: &'a AttackFacts,
    pawns: &'a PawnFacts,
    mover: Side,
    mover_colour: Colour,
}

/// What one move does to the position, read from the position after it.
struct After {
    threat_created_max: i32,
    creates_passer: bool,
    creates_isolated: bool,
    creates_doubled: bool,
    creates_backward: bool,
    opens_file_at_enemy_king: bool,
    our_ring_attackers_delta: i32,
    their_ring_attackers_delta: i32,
    own_hanging_delta: i32,
    their_hanging_delta: i32,
    leaves_unit_hanging: bool,
}

impl Before<'_> {
    /// The facts of `mv`, whose mover is `mover_role`, that only the position
    /// after it settles.
    fn after(&self, after: &Position, mv: Move, mover_role: Role) -> After {
        let mover = self.mover;
        let enemy = !mover;
        let scan = Scan::new(after);
        let us = if scan.us == self.mover_colour {
            Side::Us
        } else {
            Side::Them
        };
        let them = !us;
        // Only a pawn move or a capture can move a pawn of either side, so
        // every other move leaves the whole structure where it stood.
        let structure =
            (mover_role == Role::Pawn || mv.is_capture()).then(|| pawns::pawn_facts(&scan));

        let counts = |set: SquareSet| set.len() as i32;
        let hanging = [hanging_of(&scan, us), hanging_of(&scan, them)];
        let heavy_before = self.attacks.hanging[mover.index()] & heavy_units(self.scan, mover);
        let more = |after: fn(&PawnFacts, usize) -> SquareSet| match &structure {
            Some(structure) => {
                counts(after(structure, us.index())) > counts(after(self.pawns, mover.index()))
            }
            None => false,
        };

        After {
            // A unit no unit of ours attacks has an exchange value of 0.
            threat_created_max: (scan.by[us.index()]
                & (scan.units[them.index()] - scan.role_units[them.index()][Role::King.index()]))
            .into_iter()
            .map(|square| after.see(square))
            .max()
            .unwrap_or(0)
            .max(0),
            creates_passer: more(|facts, side| facts.passed[side]),
            creates_isolated: more(|facts, side| facts.isolated[side]),
            creates_doubled: more(|facts, side| facts.doubled[side]),
            creates_backward: more(|facts, side| facts.backward[side]),
            opens_file_at_enemy_king: structure.is_some_and(|structure| {
                king::shield_files(self.scan.kings[enemy.index()].file())
                    .into_iter()
                    .any(|file| {
                        self.pawns.count_by_file[mover.index()][file.index()] > 0
                            && structure.count_by_file[us.index()][file.index()] == 0
                    })
            }),
            our_ring_attackers_delta: ring_attackers(&scan, them)
                - ring_attackers(self.scan, enemy),
            their_ring_attackers_delta: ring_attackers(&scan, us)
                - ring_attackers(self.scan, mover),
            own_hanging_delta: counts(hanging[0]) - counts(self.attacks.hanging[mover.index()]),
            their_hanging_delta: counts(hanging[1]) - counts(self.attacks.hanging[enemy.index()]),
            leaves_unit_hanging: !((hanging[0] & heavy_units(&scan, us)) - heavy_before).is_empty(),
        }
    }
}

/// The whole `tactics` block for the side to move in `position`.
///
/// `scan`, `attacks` and `king` describe the same placement, which a null move
/// leaves unchanged. `annotated`, when given, receives one entry per legal move.
#[expect(clippy::too_many_arguments, reason = "one shared pass, no owning type")]
pub(super) fn tactics(
    variant: &dyn Variant,
    position: &Position,
    scan: &Scan,
    mover: Side,
    attacks: &AttackFacts,
    pawns: &PawnFacts,
    king: &KingFacts,
    legal: &MoveList,
    replies: &mut MoveList,
    mut annotated: Option<&mut MoveList<AnnotatedMove>>,
) -> TacticsFacts {
    let enemy = !mover;
    let mover_colour = scan.colour(mover);
    let enemy_colour = !mover_colour;
    let annotate = annotated.is_some();
    let before = Before {
        scan,
        attacks,
        pawns,
        mover,
        mover_colour,
    };
    // The back rank the enemy king may be mated on, and how much of theirs we
    // already stand to win: both are read before any move is played.
    let back_rank_open = king.back_rank_risk[enemy.index()];
    let their_king = scan.kings[enemy.index()];
    let threat_now = max_threat(position, enemy_colour);

    let mut facts = TacticsFacts {
        available: true,
        legal_move_count: legal.len().min(u16::MAX as usize) as u16,
        no_safe_moves: true,
        ..TacticsFacts::default()
    };

    for &mv in legal.as_slice() {
        let after = variant.play(position, mv);
        let to = landing(mv);
        let mover_role = position
            .piece_at(mv.from())
            .expect("a move starts on an occupied square")
            .role;
        let promotion = mv.promotion();
        let landed_role = promotion.unwrap_or(mover_role);

        let occupied = after.occupied();
        let mine = role_units(&after, mover_colour);
        let theirs = role_units(&after, enemy_colour);
        let enemy_units = after.by_colour(enemy_colour);

        let attacked_by_pawn = !(attacks_of(Role::Pawn, to, mover_colour, occupied)
            & theirs[Role::Pawn.index()])
        .is_empty();
        let enemy_attackers = attackers(to, enemy_colour, &theirs, occupied);
        let defenders = attackers(to, mover_colour, &mine, occupied);
        let cheaper = enemy_attackers.into_iter().any(|from| {
            let role = after
                .piece_at(from)
                .expect("an attacker stands on its own square")
                .role;
            order_value(role) < order_value(landed_role)
        });
        let is_safe =
            !attacked_by_pawn && !cheaper && (enemy_attackers.is_empty() || !defenders.is_empty());
        facts.no_safe_moves &= !is_safe;

        let gives_check = after.in_check();
        let victim = if mv.is_capture() {
            position.piece_at(victim_square(mv)).map(|piece| piece.role)
        } else {
            None
        };
        let see = if annotate || victim.is_some() || promotion.is_some() {
            position.see_capture(mv)
        } else {
            0
        };
        let captures_hanging =
            mv.is_capture() && attacks.hanging[enemy.index()].contains(victim_square(mv));

        let discovers = if annotate
            || !facts.discovered_attack_available
            || !facts.discovered_attack_on_queen
        {
            discovered_attack(&after, scan, mover, mover_colour, mv, enemy_units)
        } else {
            Discovered::default()
        };
        facts.discovered_attack_available |= discovers.on_heavy;
        facts.discovered_attack_on_queen |= discovers.on_queen;

        if victim.is_none()
            && !facts.quiet_threat_available
            && threatens_more_than(&after, enemy_colour, threat_now)
        {
            facts.quiet_threat_available = true;
        }

        if annotate {
            let moves_attacked_unit = scan.by[enemy.index()].contains(mv.from());
            let checkers = position.checkers();
            let blocks_check = checkers.len() == 1
                && between(
                    checkers.first().expect("one checker is a checker"),
                    scan.kings[mover.index()],
                )
                .contains(to);
            let delta = before.after(&after, mv, mover_role);
            annotated
                .as_deref_mut()
                .expect("annotate is whether the list is there")
                .push(AnnotatedMove {
                    mv,
                    facts: MoveFacts {
                        victim,
                        mover: mover_role,
                        promotion,
                        gives_check,
                        gives_safe_check: gives_check && is_safe,
                        is_safe,
                        captures_hanging,
                        escapes_attack: moves_attacked_unit && is_safe,
                        to_attacked_by_pawn: attacked_by_pawn,
                        is_castling: mv.is_castling(),
                        is_en_passant: mv.is_en_passant(),
                        see,
                        threat_created_max: delta.threat_created_max,
                        moves_attacked_unit,
                        blocks_check,
                        advances_passer: mover_role == Role::Pawn
                            && pawns.passed[mover.index()].contains(mv.from()),
                        creates_passer: delta.creates_passer,
                        creates_isolated: delta.creates_isolated,
                        creates_doubled: delta.creates_doubled,
                        creates_backward: delta.creates_backward,
                        opens_file_at_enemy_king: delta.opens_file_at_enemy_king,
                        our_ring_attackers_delta: delta.our_ring_attackers_delta,
                        their_ring_attackers_delta: delta.their_ring_attackers_delta,
                        own_hanging_delta: delta.own_hanging_delta,
                        their_hanging_delta: delta.their_hanging_delta,
                        leaves_unit_hanging: delta.leaves_unit_hanging,
                        gives_discovered_attack: discovers.on_heavy,
                    },
                });
        }

        if gives_check {
            facts.check_count += 1;
            if mover_role != Role::King {
                facts.check_by_role[mover_role.index()] = true;
            }
            if is_safe {
                facts.safe_check_count += 1;
                if mover_role != Role::King {
                    facts.safe_check_by_role[mover_role.index()] = true;
                }
                facts.safe_check_capturing |= victim.is_some();
            }
            let checkers = after.checkers();
            if checkers.len() >= 2 {
                facts.double_check_available = true;
            }
            if !(checkers - moved_to(mv)).is_empty() {
                facts.discovered_check_available = true;
            }
            if back_rank_open {
                facts.back_rank_mate_threat |= checkers.into_iter().any(|from| {
                    from.rank() == their_king.rank()
                        && matches!(
                            after
                                .piece_at(from)
                                .expect("a checker stands on its own square")
                                .role,
                            Role::Rook | Role::Queen
                        )
                });
            }
        }

        replies.clear();
        variant.legal_moves(&after, replies);
        if replies.is_empty() {
            if gives_check {
                facts.mate_in_1 = true;
            } else {
                facts.stalemate_in_1 = true;
            }
        }

        if let Some(role) = promotion {
            facts.promotion_files.insert(mv.to().file());
            let slot = match role {
                Role::Queen => 0,
                Role::Rook => 1,
                Role::Bishop => 2,
                _ => 3,
            };
            facts.promotion_roles[slot] = true;
            if is_safe {
                facts.safe_promotion_files.insert(mv.to().file());
            }
            facts.promotion_see_positive |= see > 0;
        }

        if let Some(role) = victim {
            facts.capture_count += 1;
            facts.winning_capture_available |= see > 0;
            facts.winning_capture_max_gain = facts.winning_capture_max_gain.max(see.max(0));
            if captures_hanging {
                facts.captures_hanging = true;
                facts.hanging_victim_max_value =
                    facts.hanging_victim_max_value.max(material_value(role));
            }
            match see.cmp(&0) {
                Ordering::Equal => facts.equal_capture_count += 1,
                Ordering::Less => facts.losing_capture_count += 1,
                Ordering::Greater => {}
            }
        }

        let moved_attacks = attacks_of(landed_role, to, mover_colour, occupied);
        let targets = moved_attacks & enemy_units;
        let mut forked = 0u32;
        let mut fork_value = 0i32;
        let mut royal = false;
        for target in targets {
            let role = after
                .piece_at(target)
                .expect("a target stands on its own square")
                .role;
            let undefended = attackers(target, enemy_colour, &theirs, occupied).is_empty();
            if order_value(role) > order_value(landed_role) || undefended {
                forked += 1;
                fork_value = fork_value.max(target_value(role));
                royal |= role == Role::King;
            }
        }
        if forked >= 2 {
            facts.fork_count += 1;
            facts.fork_max_value = facts.fork_max_value.max(fork_value);
            facts.knight_fork_available |= landed_role == Role::Knight;
            facts.royal_fork_available |= royal;
        }

        if matches!(landed_role, Role::Bishop | Role::Rook | Role::Queen) {
            let mut pins = false;
            for front in targets {
                let front_role = after
                    .piece_at(front)
                    .expect("a target stands on its own square")
                    .role;
                let xray = attacks_of(landed_role, to, mover_colour, occupied - front.to_set());
                let behind = (xray - moved_attacks) & line(to, front) & enemy_units;
                for back in behind {
                    let back_role = after
                        .piece_at(back)
                        .expect("a unit stands on its own square")
                        .role;
                    if back_role == Role::King || order_value(back_role) > order_value(front_role) {
                        pins = true;
                    }
                    if order_value(back_role) <= order_value(front_role) {
                        facts.skewer_creation_available = true;
                    }
                }
            }
            if pins {
                facts.pin_creation_count += 1;
            }
        }
    }

    facts
}
