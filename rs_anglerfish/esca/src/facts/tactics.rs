//! One-ply tactics, and the facts of a single legal move.

use core::cmp::Ordering;

use crate::moves::{Move, MoveList};
use crate::position::Position;
use crate::types::{Colour, File, Role, Square, SquareSet};
use crate::variant::Variant;

use super::scan::{Scan, attackers, attacks_of, line, material_value, order_value, target_value};
use super::{AnnotatedMove, AttackFacts, MoveFacts, Side, TacticsFacts};

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

/// The whole `tactics` block for the side to move in `position`.
///
/// `scan` and `attacks` describe the same placement, which a null move leaves
/// unchanged. `annotated`, when given, receives one entry per legal move.
#[expect(clippy::too_many_arguments, reason = "one shared pass, no owning type")]
pub(super) fn tactics(
    variant: &dyn Variant,
    position: &Position,
    scan: &Scan,
    mover: Side,
    attacks: &AttackFacts,
    legal: &MoveList,
    replies: &mut MoveList,
    mut annotated: Option<&mut MoveList<AnnotatedMove>>,
) -> TacticsFacts {
    let enemy = !mover;
    let mover_colour = scan.colour(mover);
    let enemy_colour = !mover_colour;

    let mut facts = TacticsFacts {
        available: true,
        legal_move_count: legal.len().min(u16::MAX as usize) as u16,
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

        let gives_check = after.in_check();
        let victim = if mv.is_capture() {
            position.piece_at(victim_square(mv)).map(|piece| piece.role)
        } else {
            None
        };
        let captures_hanging =
            mv.is_capture() && attacks.hanging[enemy.index()].contains(victim_square(mv));

        if let Some(list) = annotated.as_deref_mut() {
            list.push(AnnotatedMove {
                mv,
                facts: MoveFacts {
                    victim,
                    mover: mover_role,
                    promotion,
                    gives_check,
                    gives_safe_check: gives_check && is_safe,
                    is_safe,
                    captures_hanging,
                    escapes_attack: scan.by[enemy.index()].contains(mv.from()) && is_safe,
                    to_attacked_by_pawn: attacked_by_pawn,
                    is_castling: mv.is_castling(),
                    is_en_passant: mv.is_en_passant(),
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
            }
            let checkers = after.checkers();
            if checkers.len() >= 2 {
                facts.double_check_available = true;
            }
            if !(checkers - moved_to(mv)).is_empty() {
                facts.discovered_check_available = true;
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
        }

        if let Some(role) = victim {
            facts.capture_count += 1;
            let see = position.see_capture(mv);
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

        if !facts.discovered_attack_available {
            let stationary = moved_to(mv) | mv.from().to_set() | mv.to().to_set();
            facts.discovered_attack_available = [Role::Bishop, Role::Rook, Role::Queen]
                .into_iter()
                .any(|role| {
                    (scan.role_units[mover.index()][role.index()] - stationary)
                        .into_iter()
                        .any(|from| {
                            let gained = attacks_of(role, from, mover_colour, occupied)
                                - scan.attacks_from[from.index()];
                            (gained & enemy_units).into_iter().any(|target| {
                                let role = after
                                    .piece_at(target)
                                    .expect("a target stands on its own square")
                                    .role;
                                material_value(role) >= 3
                            })
                        })
                });
        }
    }

    facts
}
