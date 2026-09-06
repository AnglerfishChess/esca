//! One-ply tactics, and the facts of a single legal move.

use core::cmp::Ordering;

use crate::moves::{Move, MoveList};
use crate::position::Position;
use crate::types::{ByColour, Colour, File, Role, Square, SquareSet};
use crate::variant::Variant;

use super::scan::{
    Scan, attackers, attacks_of, between, line, material_value, order_value, target_value,
};
use super::{
    AnnotatedMove, AttackFacts, KingFacts, MoveFacts, PawnFacts, TacticsFacts, king, pawns,
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

/// The units of `colour` that an enemy unit attacks and no friendly unit
/// defends. A king is never among them.
fn hanging_of(scan: &Scan, colour: Colour) -> SquareSet {
    ((scan.units.of(colour) - scan.role_units.of(colour)[Role::King.index()]) & scan.by.of(!colour))
        - scan.by.of(colour)
}

/// The units of `colour` worth 3 or more: its knights, bishops, rooks and
/// queens.
fn heavy_units(scan: &Scan, colour: Colour) -> SquareSet {
    [Role::Knight, Role::Bishop, Role::Rook, Role::Queen]
        .into_iter()
        .fold(SquareSet::EMPTY, |set, role| {
            set | scan.role_units.of(colour)[role.index()]
        })
}

/// How many enemy knights, bishops, rooks and queens attack `colour`'s king
/// ring.
fn ring_attackers(scan: &Scan, colour: Colour) -> i32 {
    let king = scan.kings.of(colour);
    let ring = attacks_of(Role::King, king, colour, scan.occupied);
    [Role::Knight, Role::Bishop, Role::Rook, Role::Queen]
        .into_iter()
        .flat_map(|role| scan.role_units.of(!colour)[role.index()])
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
    mover: Colour,
    mv: Move,
    enemy_units: SquareSet,
) -> Discovered {
    let occupied = after.occupied();
    let stationary = moved_to(mv) | mv.from().to_set() | mv.to().to_set();
    let mut found = Discovered::default();
    for slider in [Role::Bishop, Role::Rook, Role::Queen] {
        for from in scan.role_units.of(mover)[slider.index()] - stationary {
            let gained =
                attacks_of(slider, from, mover, occupied) - scan.attacks_from[from.index()];
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
    mover: Colour,
}

/// What one move does to the position, read from the position after it.
struct After {
    threat_created_max: i32,
    creates_passer: bool,
    creates_isolated: bool,
    creates_doubled: bool,
    creates_backward: bool,
    opens_file_at_enemy_king: bool,
    enemy_king_attackers_delta: i32,
    mover_king_attackers_delta: i32,
    mover_hanging_delta: i32,
    enemy_hanging_delta: i32,
    leaves_unit_hanging: bool,
}

impl Before<'_> {
    /// The facts of `mv`, whose mover is `mover_role`, that only the position
    /// after it settles.
    fn after(&self, after: &Position, mv: Move, mover_role: Role) -> After {
        let mover = self.mover;
        let enemy = !mover;
        let scan = Scan::new(after);
        // Only a pawn move or a capture can move a pawn of either side, so
        // every other move leaves the whole structure where it stood.
        let structure =
            (mover_role == Role::Pawn || mv.is_capture()).then(|| pawns::pawn_facts(&scan));

        let counts = |set: SquareSet| set.len() as i32;
        let hanging_mover = hanging_of(&scan, mover);
        let hanging_enemy = hanging_of(&scan, enemy);
        let heavy_before = self.attacks.hanging.of(mover) & heavy_units(self.scan, mover);
        let more = |pick: fn(&PawnFacts, Colour) -> SquareSet| match &structure {
            Some(structure) => counts(pick(structure, mover)) > counts(pick(self.pawns, mover)),
            None => false,
        };

        After {
            // A unit no unit of the mover's attacks has an exchange value of 0.
            threat_created_max: (scan.by.of(mover)
                & (scan.units.of(enemy) - scan.role_units.of(enemy)[Role::King.index()]))
            .into_iter()
            .map(|square| after.see(square))
            .max()
            .unwrap_or(0)
            .max(0),
            creates_passer: more(|facts, colour| facts.passed.of(colour)),
            creates_isolated: more(|facts, colour| facts.isolated.of(colour)),
            creates_doubled: more(|facts, colour| facts.doubled.of(colour)),
            creates_backward: more(|facts, colour| facts.backward.of(colour)),
            opens_file_at_enemy_king: structure.is_some_and(|structure| {
                king::shield_files(self.scan.kings.of(enemy).file())
                    .into_iter()
                    .any(|file| {
                        self.pawns.count_by_file.of(mover).of(file) > 0
                            && structure.count_by_file.of(mover).of(file) == 0
                    })
            }),
            enemy_king_attackers_delta: ring_attackers(&scan, enemy)
                - ring_attackers(self.scan, enemy),
            mover_king_attackers_delta: ring_attackers(&scan, mover)
                - ring_attackers(self.scan, mover),
            mover_hanging_delta: counts(hanging_mover) - counts(self.attacks.hanging.of(mover)),
            enemy_hanging_delta: counts(hanging_enemy) - counts(self.attacks.hanging.of(enemy)),
            leaves_unit_hanging: !((hanging_mover & heavy_units(&scan, mover)) - heavy_before)
                .is_empty(),
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
    mover: Colour,
    attacks: &AttackFacts,
    pawns: &PawnFacts,
    king: &KingFacts,
    legal: &MoveList,
    replies: &mut MoveList,
    mut annotated: Option<&mut MoveList<AnnotatedMove>>,
    facts: &mut TacticsFacts,
) {
    let enemy = !mover;
    let annotate = annotated.is_some();
    let before = Before {
        scan,
        attacks,
        pawns,
        mover,
    };
    // The back rank the enemy king may be mated on, and how much the mover
    // already stands to win: both are read before any move is played.
    let back_rank_open = king.back_rank_risk.of(enemy);
    let their_king = scan.kings.of(enemy);
    let threat_now = max_threat(position, enemy);

    facts.available[mover] = true;
    facts.legal_move_count[mover] = legal.len().min(u16::MAX as usize) as u16;
    facts.no_safe_moves[mover] = true;

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
        let mine = role_units(&after, mover);
        let theirs = role_units(&after, enemy);
        let enemy_units = after.by_colour(enemy);

        let attacked_by_pawn =
            !(attacks_of(Role::Pawn, to, mover, occupied) & theirs[Role::Pawn.index()]).is_empty();
        let enemy_attackers = attackers(to, enemy, &theirs, occupied);
        let defenders = attackers(to, mover, &mine, occupied);
        let cheaper = enemy_attackers.into_iter().any(|from| {
            let role = after
                .piece_at(from)
                .expect("an attacker stands on its own square")
                .role;
            order_value(role) < order_value(landed_role)
        });
        let is_safe =
            !attacked_by_pawn && !cheaper && (enemy_attackers.is_empty() || !defenders.is_empty());
        facts.no_safe_moves[mover] &= !is_safe;

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
            mv.is_capture() && attacks.hanging.of(enemy).contains(victim_square(mv));

        let discovers = if annotate
            || !facts.discovered_attack_available[mover]
            || !facts.discovered_attack_on_queen[mover]
        {
            discovered_attack(&after, scan, mover, mv, enemy_units)
        } else {
            Discovered::default()
        };
        facts.discovered_attack_available[mover] |= discovers.on_heavy;
        facts.discovered_attack_on_queen[mover] |= discovers.on_queen;

        if victim.is_none()
            && !facts.quiet_threat_available[mover]
            && threatens_more_than(&after, enemy, threat_now)
        {
            facts.quiet_threat_available[mover] = true;
        }

        if annotate {
            let moves_attacked_unit = scan.by.of(enemy).contains(mv.from());
            let checkers = position.checkers();
            let blocks_check = checkers.len() == 1
                && between(
                    checkers.first().expect("one checker is a checker"),
                    scan.kings.of(mover),
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
                            && pawns.passed.of(mover).contains(mv.from()),
                        creates_passer: delta.creates_passer,
                        creates_isolated: delta.creates_isolated,
                        creates_doubled: delta.creates_doubled,
                        creates_backward: delta.creates_backward,
                        opens_file_at_enemy_king: delta.opens_file_at_enemy_king,
                        enemy_king_attackers_delta: delta.enemy_king_attackers_delta,
                        mover_king_attackers_delta: delta.mover_king_attackers_delta,
                        mover_hanging_delta: delta.mover_hanging_delta,
                        enemy_hanging_delta: delta.enemy_hanging_delta,
                        leaves_unit_hanging: delta.leaves_unit_hanging,
                        gives_discovered_attack: discovers.on_heavy,
                    },
                });
        }

        if gives_check {
            facts.check_count[mover] += 1;
            set_by_role(
                &mut facts.check_by_pawn,
                &mut facts.check_by_knight,
                &mut facts.check_by_bishop,
                &mut facts.check_by_rook,
                &mut facts.check_by_queen,
                mover,
                mover_role,
            );
            if is_safe {
                facts.safe_check_count[mover] += 1;
                set_by_role(
                    &mut facts.safe_check_by_pawn,
                    &mut facts.safe_check_by_knight,
                    &mut facts.safe_check_by_bishop,
                    &mut facts.safe_check_by_rook,
                    &mut facts.safe_check_by_queen,
                    mover,
                    mover_role,
                );
                facts.safe_check_capturing[mover] |= victim.is_some();
            }
            let checkers = after.checkers();
            if checkers.len() >= 2 {
                facts.double_check_available[mover] = true;
            }
            if !(checkers - moved_to(mv)).is_empty() {
                facts.discovered_check_available[mover] = true;
            }
            if back_rank_open {
                facts.back_rank_mate_threat[mover] |= checkers.into_iter().any(|from| {
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
                facts.mate_in_1[mover] = true;
            } else {
                facts.stalemate_in_1[mover] = true;
            }
        }

        if let Some(role) = promotion {
            facts.promotion_files[mover].insert(mv.to().file());
            match role {
                Role::Queen => facts.promotes_to_queen[mover] = true,
                Role::Rook => facts.promotes_to_rook[mover] = true,
                Role::Bishop => facts.promotes_to_bishop[mover] = true,
                _ => facts.promotes_to_knight[mover] = true,
            }
            if is_safe {
                facts.safe_promotion_files[mover].insert(mv.to().file());
            }
            facts.promotion_see_positive[mover] |= see > 0;
        }

        if let Some(role) = victim {
            facts.capture_count[mover] += 1;
            facts.winning_capture_available[mover] |= see > 0;
            facts.winning_capture_max_gain[mover] =
                facts.winning_capture_max_gain.of(mover).max(see.max(0));
            if captures_hanging {
                facts.captures_hanging[mover] = true;
                facts.hanging_victim_max_value[mover] = facts
                    .hanging_victim_max_value
                    .of(mover)
                    .max(material_value(role));
            }
            match see.cmp(&0) {
                Ordering::Equal => facts.equal_capture_count[mover] += 1,
                Ordering::Less => facts.losing_capture_count[mover] += 1,
                Ordering::Greater => {}
            }
        }

        let moved_attacks = attacks_of(landed_role, to, mover, occupied);
        let targets = moved_attacks & enemy_units;
        let mut forked = 0u32;
        let mut fork_value = 0i32;
        let mut royal = false;
        for target in targets {
            let role = after
                .piece_at(target)
                .expect("a target stands on its own square")
                .role;
            let undefended = attackers(target, enemy, &theirs, occupied).is_empty();
            if order_value(role) > order_value(landed_role) || undefended {
                forked += 1;
                fork_value = fork_value.max(target_value(role));
                royal |= role == Role::King;
            }
        }
        if forked >= 2 {
            facts.fork_count[mover] += 1;
            facts.fork_max_value[mover] = facts.fork_max_value.of(mover).max(fork_value);
            facts.knight_fork_available[mover] |= landed_role == Role::Knight;
            facts.royal_fork_available[mover] |= royal;
        }

        if matches!(landed_role, Role::Bishop | Role::Rook | Role::Queen) {
            let mut pins = false;
            for front in targets {
                let front_role = after
                    .piece_at(front)
                    .expect("a target stands on its own square")
                    .role;
                let xray = attacks_of(landed_role, to, mover, occupied - front.to_set());
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
                        facts.skewer_creation_available[mover] = true;
                    }
                }
            }
            if pins {
                facts.pin_creation_count[mover] += 1;
            }
        }
    }
}

/// Marks the flag of `role` for `mover`; a king move marks none.
#[allow(clippy::too_many_arguments, reason = "one flag per role, by reference")]
fn set_by_role(
    pawn: &mut ByColour<bool>,
    knight: &mut ByColour<bool>,
    bishop: &mut ByColour<bool>,
    rook: &mut ByColour<bool>,
    queen: &mut ByColour<bool>,
    mover: Colour,
    role: Role,
) {
    match role {
        Role::Pawn => pawn[mover] = true,
        Role::Knight => knight[mover] = true,
        Role::Bishop => bishop[mover] = true,
        Role::Rook => rook[mover] = true,
        Role::Queen => queen[mover] = true,
        Role::King => {}
    }
}
