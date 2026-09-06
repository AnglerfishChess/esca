//! The evidence behind a rules answer.
//!
//! Where [`Facts`](crate::Facts) answers what is true, these types answer
//! why: each distinct reason is its own field carrying the squares it was
//! read off, and every reason that applies is filled in at once.

use crate::facts::scan::{attackers, attacks_of, between, line, order_value};
use crate::game::Game;
use crate::moves::{Move, MoveKind};
use crate::position::Position;
use crate::types::{Colour, File, Piece, Rank, Role, Square, SquareSet};

/// The halfmove clock a player may claim the fifty-move draw at.
const CLAIM_CLOCK: u32 = 100;

/// The halfmove clock the draw becomes automatic at.
const AUTOMATIC_CLOCK: u32 = 150;

/// How often a position must have stood for a claim, and for the automatic
/// draw.
const CLAIM_REPETITIONS: u32 = 3;
const AUTOMATIC_REPETITIONS: u32 = 5;

/// Which castling, named by where the king lands: `Short` on the g-file,
/// `Long` on the c-file.
///
/// Not the wing the rook starts on: a shuffled back rank can put either rook
/// on either side of the centre.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Wing {
    /// The king lands on the g-file, the rook on the f-file.
    Short,
    /// The king lands on the c-file, the rook on the d-file.
    Long,
}

impl Wing {
    /// Both wings, short first.
    pub const ALL: [Wing; 2] = [Wing::Short, Wing::Long];

    /// The file the king lands on.
    const fn king_file(self) -> File {
        match self {
            Wing::Short => File::G,
            Wing::Long => File::C,
        }
    }

    /// The file the rook lands on.
    const fn rook_file(self) -> File {
        match self {
            Wing::Short => File::F,
            Wing::Long => File::D,
        }
    }
}

/// One castling of one colour, and everything standing in its way.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Castling {
    /// The position still holds this castling right.
    pub right: bool,
    /// The rook the right names stands on its square. False without a right,
    /// which names no rook.
    pub rook_present: bool,
    /// The enemy units attacking the king where it stands.
    pub king_in_check_by: SquareSet,
    /// Each square the king crosses or lands on that the enemy covers, with
    /// the units covering it, in ascending square order. The king's own
    /// square is `king_in_check_by`, not a member here, and without the right
    /// there is no path at all.
    pub path_attacked: Vec<(Square, SquareSet)>,
    /// The units standing on squares the king or the rook must pass or land
    /// on, the castling king and rook themselves excepted.
    pub path_blocked: SquareSet,
    /// Nothing above prevents the castling. Whose turn it is is not part of
    /// it, so for the side to move this is exactly legality.
    pub allowed: bool,
}

/// The en-passant capture a position offers the side to move.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EnPassant {
    /// The previous ply was not a double pawn step.
    None,
    /// A pawn skipped `target` on the previous ply.
    Available {
        /// The square it skipped.
        target: Square,
        /// Every pawn of the side to move standing beside it, in ascending
        /// square order.
        captures: Vec<EpCapture>,
    },
}

impl EnPassant {
    /// The square a pawn skipped, if one did.
    pub fn target(&self) -> Option<Square> {
        match self {
            EnPassant::None => None,
            EnPassant::Available { target, .. } => Some(*target),
        }
    }

    /// The pawns that could take it.
    pub fn captures(&self) -> &[EpCapture] {
        match self {
            EnPassant::None => &[],
            EnPassant::Available { captures, .. } => captures,
        }
    }
}

/// One pawn's en-passant capture of the target.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EpCapture {
    /// Where the capturing pawn stands.
    pub from: Square,
    /// The capture is a legal move.
    pub legal: bool,
    /// The first of `InCheck`, `Pinned` and `ExposesKing` that applies;
    /// `None` when the capture is legal.
    pub forbidden_by: Option<EpObstacle>,
}

/// What keeps an en-passant capture off the board.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EpObstacle {
    /// The pawn is pinned against its own king and the target is off the
    /// pinning ray.
    Pinned {
        /// Between pinner and king, exclusive.
        ray: SquareSet,
        /// The unit doing the pinning.
        pinner: Square,
    },
    /// Both pawns leave one rank at once and uncover the king: the pin that
    /// binds neither pawn alone.
    ExposesKing {
        /// The unit the two pawns hide.
        attacker: Square,
    },
    /// The side to move is in check and this capture does not answer it.
    InCheck {
        /// The units giving check.
        by: SquareSet,
    },
}

/// A unit that may not move off the line between an enemy slider and its own
/// king.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pin {
    /// The unit that may not move off the ray.
    pub pinned: Square,
    /// The slider holding it there.
    pub pinner: Square,
    /// The king behind it.
    pub king: Square,
    /// Between pinner and king, exclusive.
    pub ray: SquareSet,
}

/// A unit attacked with a less valuable one of the same colour directly
/// behind it on the slider's line.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Skewer {
    /// The slider attacking the front unit.
    pub attacker: Square,
    /// The unit in front, the more valuable of the two.
    pub front: Square,
    /// The unit the front one shields.
    pub behind: Square,
    /// Between attacker and `behind`, exclusive; holds `front`.
    pub ray: SquareSet,
}

/// How often the current position has stood, and what nearly counted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Repetition {
    /// How many of the plies are occurrences.
    pub count: u32,
    /// Every ply the current position occurred at, this one last.
    pub plies: Vec<u32>,
    /// The earlier plies with the same placement that do not count.
    pub near_misses: Vec<NearMiss>,
}

/// An earlier ply with the same placement that is not a repetition.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NearMiss {
    /// The ply it stood at.
    pub ply: u32,
    /// Everything about it that differs, in this enum's order.
    pub differs: Vec<Difference>,
}

/// What can tell two positions of one placement apart.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Difference {
    /// The castlings still available differ.
    CastlingRights,
    /// The en-passant capture on offer differs.
    EnPassant,
    /// The side to move differs.
    SideToMove,
}

/// The halfmove clock, and how far it is from ending the game.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FiftyMove {
    /// Plies since the last capture or pawn move.
    pub clock: u32,
    /// Plies until a player may claim; 0 once one may.
    pub plies_to_claim: u32,
    /// Plies until the draw is automatic; 0 once it is.
    pub plies_to_automatic: u32,
    /// The last move of this game that set the clock to 0. `None` when no
    /// move did, which leaves the clock the start position carried.
    pub last_reset: Option<Reset>,
}

/// The move that last set the halfmove clock to 0.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Reset {
    /// The ply it produced.
    pub ply: u32,
    /// What it did.
    pub kind: ResetKind,
}

/// What resets the halfmove clock. A capturing pawn move is a `Capture`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ResetKind {
    /// The move took a unit, en passant included.
    Capture,
    /// The move advanced a pawn.
    PawnMove,
}

/// Every draw condition that holds, not the first of them. Both lists are
/// empty when the side to move is checkmated.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DrawStatus {
    /// The draws that end the game as they are.
    pub automatic: Vec<AutomaticDraw>,
    /// The draws a player may ask for, the position still playable.
    pub claimable: Vec<ClaimableDraw>,
}

/// A draw that needs no claim.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AutomaticDraw {
    /// The side to move is not in check and has no legal move.
    Stalemate(StalemateDetail),
    /// Neither side has material that could ever deliver mate.
    InsufficientMaterial(MaterialConfig),
    /// The position has stood five times.
    Fivefold(Repetition),
    /// The halfmove clock has reached 150.
    SeventyFiveMoves(FiftyMove),
}

/// A draw a player may ask for.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ClaimableDraw {
    /// The position has stood three times.
    Threefold(Repetition),
    /// The halfmove clock has reached 100.
    FiftyMoves(FiftyMove),
}

/// The material a variant calls insufficient, named. Either side may be the
/// one holding the minor.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MaterialConfig {
    /// Kings only.
    KvK,
    /// One knight besides the kings.
    KNvK,
    /// One bishop besides the kings.
    KBvK,
    /// Bishops and kings only, every bishop on one square colour.
    KBvKBSameColour,
}

/// Why the side to move has no move.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StalemateDetail {
    /// The king with nowhere to go.
    pub king: Square,
    /// Each square beside the king that none of its own units holds, with the
    /// enemy units covering it, the king itself out of the way.
    pub escape_squares: Vec<(Square, SquareSet)>,
    /// Every other unit of the side to move, and what holds it.
    pub stuck_units: Vec<(Square, Stuck)>,
}

/// What holds a unit that has no legal move.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stuck {
    /// Pinned against its own king, whatever else is true of it.
    Pinned {
        /// Between pinner and king, exclusive.
        ray: SquareSet,
        /// The unit doing the pinning.
        pinner: Square,
    },
    /// Occupancy leaves it no move at all.
    Blocked,
    /// It has moves, and none of them is legal.
    NoMoves,
}

/// The units of `colour` that attack `square`, sliders seeing `occupied`.
/// Units off `occupied` are gone from the board and attack nothing.
fn attackers_on(
    position: &Position,
    square: Square,
    colour: Colour,
    occupied: SquareSet,
) -> SquareSet {
    let role_units = Role::ALL.map(|role| position.by_colour(colour) & position.by_role(role));
    attackers(square, colour, &role_units, occupied) & occupied
}

/// Placement alone: which colour and which role stands where.
fn placement_of(position: &Position) -> ([SquareSet; 2], [SquareSet; 6]) {
    (
        Colour::ALL.map(|colour| position.by_colour(colour)),
        Role::ALL.map(|role| position.by_role(role)),
    )
}

/// The en-passant square repetition counts, which is one a pawn could take.
fn playable_en_passant(position: &Position) -> Option<Square> {
    let status = position.en_passant_status();
    status
        .target()
        .filter(|_| status.captures().iter().any(|capture| capture.legal))
}

/// The insufficient material `position` holds, if it holds any.
fn material_config(position: &Position) -> Option<MaterialConfig> {
    let heavy =
        position.by_role(Role::Pawn) | position.by_role(Role::Rook) | position.by_role(Role::Queen);
    if !heavy.is_empty() {
        return None;
    }
    let knights = position.by_role(Role::Knight);
    let bishops = position.by_role(Role::Bishop);
    match (knights.len(), bishops.len()) {
        (0, 0) => Some(MaterialConfig::KvK),
        (1, 0) => Some(MaterialConfig::KNvK),
        (0, 1) => Some(MaterialConfig::KBvK),
        (0, _) if bishops.is_subset(SquareSet::DARK) || bishops.is_subset(SquareSet::LIGHT) => {
            Some(MaterialConfig::KBvKBSameColour)
        }
        _ => None,
    }
}

/// The square one step forward of `square` for `colour`.
fn forward(square: Square, colour: Colour) -> Option<Square> {
    let step = match colour {
        Colour::White => 1isize,
        Colour::Black => -1,
    };
    let rank = usize::try_from(square.rank().index() as isize + step).ok()?;
    Rank::from_index(rank).map(|rank| Square::new(square.file(), rank))
}

/// Every square the unit on `square` could move to by its movement rule and
/// by occupancy, whatever its own king would then face.
fn pseudo_moves(position: &Position, square: Square) -> SquareSet {
    let Some(piece) = position.piece_at(square) else {
        return SquareSet::EMPTY;
    };
    let occupied = position.occupied();
    let mine = position.by_colour(piece.colour);
    if piece.role != Role::Pawn {
        return attacks_of(piece.role, square, piece.colour, occupied) - mine;
    }
    let mut moves = SquareSet::EMPTY;
    if let Some(one) = forward(square, piece.colour) {
        if !occupied.contains(one) {
            moves.insert(one);
            if square.rank().relative_to(piece.colour).index() == 1 {
                if let Some(two) = forward(one, piece.colour) {
                    if !occupied.contains(two) {
                        moves.insert(two);
                    }
                }
            }
        }
    }
    let targets = position.by_colour(!piece.colour)
        | position
            .en_passant()
            .map_or(SquareSet::EMPTY, Square::to_set);
    moves | (attacks_of(Role::Pawn, square, piece.colour, occupied) & targets)
}

impl Position {
    /// What stands in the way of `colour` castling on `wing`.
    pub fn castling(&self, colour: Colour, wing: Wing) -> Castling {
        let king = self.king_of(colour);
        let rights = self.castling_rights();
        let file = match wing {
            Wing::Short => rights.short(colour),
            Wing::Long => rights.long(colour),
        };
        let back = Rank::First.relative_to(colour);
        let rook = file.map(|file| Square::new(file, back));
        let rook_present = rook
            .is_some_and(|square| self.piece_at(square) == Some(Piece::new(Role::Rook, colour)));
        let mut castling = Castling {
            right: file.is_some(),
            rook_present,
            king_in_check_by: self.attackers(king, !colour),
            path_attacked: Vec::new(),
            path_blocked: SquareSet::EMPTY,
            allowed: false,
        };
        let Some(rook) = rook.filter(|_| rook_present) else {
            return castling;
        };

        let king_to = Square::new(wing.king_file(), back);
        let rook_to = Square::new(wing.rook_file(), back);
        let king_path = (between(king, king_to) | king_to.to_set()) - king.to_set();
        let rook_path = between(rook, rook_to) | rook_to.to_set();
        castling.path_blocked =
            ((king_path | rook_path) & self.occupied()) - king.to_set() - rook.to_set();
        for square in king_path {
            let by = self.attackers(square, !colour);
            if !by.is_empty() {
                castling.path_attacked.push((square, by));
            }
        }
        castling.allowed = castling.king_in_check_by.is_empty()
            && castling.path_attacked.is_empty()
            && castling.path_blocked.is_empty();
        castling
    }

    /// The en-passant capture this position offers the side to move.
    pub fn en_passant_status(&self) -> EnPassant {
        let Some(target) = self.en_passant() else {
            return EnPassant::None;
        };
        let colour = self.side_to_move();
        let pawn = Piece::new(Role::Pawn, colour);
        let rank = Rank::Fifth.relative_to(colour);
        let mut captures = Vec::new();
        for offset in [-1isize, 1] {
            let file = target
                .file()
                .index()
                .checked_add_signed(offset)
                .and_then(File::from_index);
            let Some(file) = file else { continue };
            let from = Square::new(file, rank);
            if self.piece_at(from) != Some(pawn) {
                continue;
            }
            let legal = self.allows(Move::new(from, target, None, MoveKind::EnPassant));
            let forbidden_by = if legal {
                None
            } else {
                self.en_passant_obstacle(from, target, colour)
            };
            captures.push(EpCapture {
                from,
                legal,
                forbidden_by,
            });
        }
        EnPassant::Available { target, captures }
    }

    /// Why the pawn on `from` may not take `target` en passant.
    fn en_passant_obstacle(
        &self,
        from: Square,
        target: Square,
        colour: Colour,
    ) -> Option<EpObstacle> {
        let checkers = self.checkers();
        if !checkers.is_empty() {
            return Some(EpObstacle::InCheck { by: checkers });
        }
        let pin = self.pins(colour).into_iter().find(|pin| pin.pinned == from);
        if let Some(pin) = pin {
            if !(pin.ray | pin.pinner.to_set()).contains(target) {
                return Some(EpObstacle::Pinned {
                    ray: pin.ray,
                    pinner: pin.pinner,
                });
            }
        }
        // Both pawns leave the rank at once, so neither is pinned on its own.
        let taken = Square::new(target.file(), from.rank());
        let after = ((self.occupied() - from.to_set()) - taken.to_set()) | target.to_set();
        let king = self.king_of(colour);
        let revealed = attackers_on(self, king, !colour, after) - self.attackers(king, !colour);
        revealed
            .first()
            .map(|attacker| EpObstacle::ExposesKing { attacker })
    }

    /// The units of `colour` attacking `square`, pins ignored.
    pub fn attackers(&self, square: Square, colour: Colour) -> SquareSet {
        attackers_on(self, square, colour, self.occupied())
    }

    /// The squares strictly between `a` and `b`; empty when they share no
    /// rank, file or diagonal.
    pub fn between(&self, a: Square, b: Square) -> SquareSet {
        between(a, b)
    }

    /// The absolute pins on `colour`'s units: what stands behind a pinned
    /// unit is its own king. Relative pins are not counted.
    pub fn pins(&self, colour: Colour) -> Vec<Pin> {
        let king = self.king_of(colour);
        let theirs = self.by_colour(!colour);
        let mine = self.by_colour(colour);
        let bishops = (self.by_role(Role::Bishop) | self.by_role(Role::Queen)) & theirs;
        let rooks = (self.by_role(Role::Rook) | self.by_role(Role::Queen)) & theirs;
        let candidates = (attacks_of(Role::Bishop, king, colour, SquareSet::EMPTY) & bishops)
            | (attacks_of(Role::Rook, king, colour, SquareSet::EMPTY) & rooks);
        let mut pins = Vec::new();
        for pinner in candidates {
            let ray = between(pinner, king);
            let blockers = ray & self.occupied();
            if blockers.len() == 1 && blockers.is_subset(mine) {
                let pinned = blockers.first().expect("one blocker stands on the ray");
                pins.push(Pin {
                    pinned,
                    pinner,
                    king,
                    ray,
                });
            }
        }
        pins.sort_by_key(|pin| pin.pinned.index());
        pins
    }

    /// The skewers on `colour`'s units: the more valuable one in front, and
    /// the next unit of the same colour on the slider's line behind it.
    pub fn skewers(&self, colour: Colour) -> Vec<Skewer> {
        let occupied = self.occupied();
        let mine = self.by_colour(colour);
        let theirs = self.by_colour(!colour);
        let mut skewers = Vec::new();
        for role in [Role::Bishop, Role::Rook, Role::Queen] {
            for attacker in theirs & self.by_role(role) {
                let attacks = attacks_of(role, attacker, !colour, occupied);
                for front in attacks & mine {
                    let xray = attacks_of(role, attacker, !colour, occupied - front.to_set());
                    let behind = (xray - attacks) & line(attacker, front) & mine;
                    let Some(behind) = behind.first() else {
                        continue;
                    };
                    let value = |square: Square| {
                        order_value(
                            self.piece_at(square)
                                .expect("a unit stands on its own square")
                                .role,
                        )
                    };
                    if value(front) > value(behind) {
                        skewers.push(Skewer {
                            attacker,
                            front,
                            behind,
                            ray: between(attacker, behind),
                        });
                    }
                }
            }
        }
        skewers.sort_by_key(|skewer| skewer.front.index());
        skewers
    }
}

impl Game {
    /// How often the current position has stood, and which earlier plies
    /// share its placement without counting.
    pub fn repetition_status(&self) -> Repetition {
        let now = self.position();
        let placement = placement_of(now);
        let en_passant = playable_en_passant(now);
        let rights = now.castling_rights();
        let side = now.side_to_move();
        let mut plies = Vec::new();
        let mut near_misses = Vec::new();
        for (ply, position) in self.positions().enumerate() {
            if placement_of(position) != placement {
                continue;
            }
            let mut differs = Vec::new();
            if position.castling_rights() != rights {
                differs.push(Difference::CastlingRights);
            }
            if playable_en_passant(position) != en_passant {
                differs.push(Difference::EnPassant);
            }
            if position.side_to_move() != side {
                differs.push(Difference::SideToMove);
            }
            if differs.is_empty() {
                plies.push(ply as u32);
            } else {
                near_misses.push(NearMiss {
                    ply: ply as u32,
                    differs,
                });
            }
        }
        Repetition {
            count: plies.len() as u32,
            plies,
            near_misses,
        }
    }

    /// The halfmove clock, what it is counting towards, and what last set it
    /// to 0.
    pub fn fifty_move_status(&self) -> FiftyMove {
        let clock = self.position().halfmove_clock();
        let mut last_reset = None;
        for (index, (before, &mv)) in self.positions().zip(self.moves()).enumerate() {
            let kind = if mv.is_capture() {
                Some(ResetKind::Capture)
            } else if before
                .piece_at(mv.from())
                .is_some_and(|p| p.role == Role::Pawn)
            {
                Some(ResetKind::PawnMove)
            } else {
                None
            };
            if let Some(kind) = kind {
                last_reset = Some(Reset {
                    ply: index as u32 + 1,
                    kind,
                });
            }
        }
        FiftyMove {
            clock,
            plies_to_claim: CLAIM_CLOCK.saturating_sub(clock),
            plies_to_automatic: AUTOMATIC_CLOCK.saturating_sub(clock),
            last_reset,
        }
    }

    /// Every draw condition that holds now.
    pub fn draw_status(&self) -> DrawStatus {
        let mut status = DrawStatus {
            automatic: Vec::new(),
            claimable: Vec::new(),
        };
        let position = self.position();
        let stuck = self.legal_moves().is_empty();
        if stuck && position.in_check() {
            return status;
        }
        if stuck {
            status
                .automatic
                .push(AutomaticDraw::Stalemate(self.stalemate_detail()));
        }
        if let Some(config) = material_config(position) {
            status
                .automatic
                .push(AutomaticDraw::InsufficientMaterial(config));
        }
        let repetition = self.repetition_status();
        let fifty = self.fifty_move_status();
        if repetition.count >= AUTOMATIC_REPETITIONS {
            status
                .automatic
                .push(AutomaticDraw::Fivefold(repetition.clone()));
        }
        if fifty.clock >= AUTOMATIC_CLOCK {
            status
                .automatic
                .push(AutomaticDraw::SeventyFiveMoves(fifty.clone()));
        }
        if repetition.count >= CLAIM_REPETITIONS {
            status.claimable.push(ClaimableDraw::Threefold(repetition));
        }
        if fifty.clock >= CLAIM_CLOCK {
            status.claimable.push(ClaimableDraw::FiftyMoves(fifty));
        }
        status
    }

    /// What could be claimed once `mv` is played. Empty when `mv` is not
    /// legal here.
    pub fn claims_after(&self, mv: Move) -> Vec<ClaimableDraw> {
        let mut next = self.clone();
        if next.play(mv).is_err() {
            return Vec::new();
        }
        next.draw_status().claimable
    }

    /// Why the side to move has no move, its king first.
    fn stalemate_detail(&self) -> StalemateDetail {
        let position = self.position();
        let colour = position.side_to_move();
        let king = position.king_of(colour);
        let mine = position.by_colour(colour);
        let without_king = position.occupied() - king.to_set();
        let escape_squares = (attacks_of(Role::King, king, colour, without_king) - mine)
            .into_iter()
            .map(|square| {
                (
                    square,
                    attackers_on(position, square, !colour, without_king),
                )
            })
            .collect();
        let pins = position.pins(colour);
        let stuck_units = (mine - king.to_set())
            .into_iter()
            .map(|square| {
                let stuck = match pins.iter().find(|pin| pin.pinned == square) {
                    Some(pin) => Stuck::Pinned {
                        ray: pin.ray,
                        pinner: pin.pinner,
                    },
                    None if pseudo_moves(position, square).is_empty() => Stuck::Blocked,
                    None => Stuck::NoMoves,
                };
                (square, stuck)
            })
            .collect();
        StalemateDetail {
            king,
            escape_squares,
            stuck_units,
        }
    }
}
