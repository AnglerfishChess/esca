//! What is true about a position and about each of its legal moves.
//!
//! Every fact is side-relative: `Side::Us` is the side to move. Files, ranks
//! and square indices are in the mover's view — the board flipped vertically
//! and the colours swapped when Black is to move — so no fact distinguishes
//! actual White from actual Black.

mod encode;
mod king;
mod pawns;
mod pieces;
mod scan;
mod tactics;

use core::fmt;
use core::ops::Not;

use crate::moves::{Move, MoveList};
use crate::position::Position;
use crate::types::{Colour, File, FileSet, Role, Square, SquareSet};
use crate::variant::Variant;

pub use encode::{RowError, encode_fens, encode_positions};
pub(crate) use scan::Scan;
use scan::{
    CENTRE, EXTENDED_CENTRE, attackers, attacks_of, between, distance, line, material_value,
    order_value,
};

/// Which side a fact is about, relative to the side to move.
///
/// Every `[T; 2]` in this module is indexed by `Side::index()`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Side {
    /// The side to move.
    Us,
    /// The side not to move.
    Them,
}

impl Side {
    /// Both sides, us first.
    pub const ALL: [Side; 2] = [Side::Us, Side::Them];

    /// 0 for us, 1 for them.
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }
}

impl Not for Side {
    type Output = Side;

    #[inline]
    fn not(self) -> Side {
        match self {
            Side::Us => Side::Them,
            Side::Them => Side::Us,
        }
    }
}

/// Game-state flags: check, castling rights, the en-passant square, the
/// halfmove clock and repetition.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct StateFacts {
    /// The side to move stands in check.
    pub in_check: bool,
    /// Two or more units give check.
    pub double_check: bool,
    /// Each side may still castle short.
    pub castle_short: [bool; 2],
    /// Each side may still castle long.
    pub castle_long: [bool; 2],
    /// The file the position names as the en-passant target, if any.
    pub en_passant: Option<File>,
    /// Some legal move captures en passant.
    pub ep_capture_legal: bool,
    /// Plies since the last capture or pawn move.
    pub halfmove_clock: u32,
    /// The position carried a halfmove clock.
    pub halfmove_known: bool,
    /// This position occurred before in the supplied history.
    pub repetition_seen: bool,
    /// Each side can reach a position of the supplied history in one move;
    /// `Side::Them` is judged after a null move.
    pub repetition_available: [bool; 2],
    /// A position history was supplied.
    pub history_known: bool,
}

/// Material and phase.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct MaterialFacts {
    /// Unit counts per side, by role P, N, B, R, Q.
    pub count: [[u8; 5]; 2],
    /// Value of N, B, R and Q per side.
    pub non_pawn_value: [i32; 2],
    /// Value of every unit but the king, per side.
    pub value: [i32; 2],
    /// `4·Q + 2·R + B + N` over both sides, capped at 24, divided by 24.
    pub phase: f32,
    /// Both sides have at least one queen.
    pub both_queens: bool,
    /// Neither side has a unit other than kings and pawns.
    pub pawns_only: bool,
    /// Each side's own material could never deliver mate.
    pub insufficient: [bool; 2],
}

/// Pawn structure.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct PawnFacts {
    /// Each side's pawns.
    pub pawns: [SquareSet; 2],
    /// Pawns with no enemy pawn ahead on their own or an adjacent file.
    pub passed: [SquareSet; 2],
    /// Candidate passers.
    pub candidates: [SquareSet; 2],
    /// Pawns sharing a file with another pawn of their own colour.
    pub doubled: [SquareSet; 2],
    /// Pawns with no friendly pawn on either adjacent file.
    pub isolated: [SquareSet; 2],
    /// Backward pawns.
    pub backward: [SquareSet; 2],
    /// Pawns defended by a pawn of their own colour.
    pub defended: [SquareSet; 2],
    /// Pawns per file, file a first.
    pub count_by_file: [[u8; 8]; 2],
    /// Pawns per relative rank, rank 1 first.
    pub count_by_rank: [[u8; 8]; 2],
    /// Files carrying no pawn of either colour.
    pub open_files: FileSet,
    /// Files carrying no pawn of that side and at least one of the other.
    pub semi_open_files: [FileSet; 2],
    /// Maximal runs of adjacent files carrying a pawn, per side.
    pub islands: [u8; 2],
    /// Pawns that can capture an enemy pawn, per side.
    pub levers: [u8; 2],
    /// Pawn pairs blocking each other head on.
    pub rams: u8,
    /// The relative rank, from 1, of each side's most advanced passer.
    pub passer_lead_rank: [Option<u8>; 2],
    /// Passers defended by a friendly pawn, per side.
    pub passer_protected: [u8; 2],
    /// Two passers on adjacent files, per side.
    pub passers_connected: [bool; 2],
    /// A passer the enemy king cannot catch, per side.
    pub passer_unstoppable: [bool; 2],
}

/// Bishops, rooks, knights and queens.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct PieceFacts {
    /// Bishops on both square colours, per side.
    pub bishop_pair: [bool; 2],
    /// Bishops on light squares, per side.
    pub bishops_light: [u8; 2],
    /// Bishops on dark squares, per side.
    pub bishops_dark: [u8; 2],
    /// Exactly one bishop each, on different square colours.
    pub opposite_coloured_bishops: bool,
    /// Own pawns standing on a square colour of an own bishop, per side.
    pub pawns_on_bishop_colour: [u8; 2],
    /// Two rooks share a rank with nothing between, per side.
    pub rooks_connected_rank: [bool; 2],
    /// Two rooks share a file with nothing between, per side.
    pub rooks_connected_file: [bool; 2],
    /// Rooks on an open file, per side.
    pub rooks_on_open_file: [u8; 2],
    /// Rooks on a file semi-open for their own side, per side.
    pub rooks_on_semi_open_file: [u8; 2],
    /// Rooks on their own relative rank 7, per side.
    pub rooks_on_relative_7th: [u8; 2],
    /// Rooks behind a passer of their own side, per side.
    pub rook_behind_own_passer: [u8; 2],
    /// Rooks behind an enemy passer, per side.
    pub rook_behind_enemy_passer: [u8; 2],
    /// A trapped rook, per side.
    pub trapped_rook: [bool; 2],
    /// Outpost squares, per side.
    pub outposts: [SquareSet; 2],
    /// Knights standing on an own outpost square, per side.
    pub knights_on_outpost: [u8; 2],
    /// Unoccupied outpost squares, per side.
    pub outpost_squares_free: [u8; 2],
    /// Knights on file a or h, or on relative rank 1 or 8, per side.
    pub knights_on_rim: [u8; 2],
    /// Knights and bishops still on their starting squares, per side.
    pub minors_undeveloped: [u8; 2],
    /// A queen stands off its starting square, per side.
    pub queen_developed: [bool; 2],
}

/// King safety and shelter. Index 0 is our king, index 1 theirs.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct KingFacts {
    /// Where each king stands.
    pub square: [Square; 2],
    /// The king stands on its own starting square, per side.
    pub on_home_square: [bool; 2],
    /// The king stands on files a to c, per side.
    pub castled_queenside: [bool; 2],
    /// The king stands on files f to h, per side.
    pub castled_kingside: [bool; 2],
    /// The three files a king's shelter is read on, in ascending order.
    pub shield_files: [[File; 3]; 2],
    /// Ranks to the nearest friendly pawn ahead of the king, per shield file.
    pub shield: [[Option<u8>; 3]; 2],
    /// Each shield file carries no pawn of either colour.
    pub file_open: [[bool; 3]; 2],
    /// Each shield file is semi-open for the enemy of that king.
    pub file_semi_open_for_enemy: [[bool; 3]; 2],
    /// Ranks to the nearest enemy pawn ahead of the king, per shield file.
    pub storm: [[Option<u8>; 3]; 2],
    /// The squares adjacent to each king.
    pub ring: [SquareSet; 2],
    /// Enemy knights, bishops, rooks and queens attacking the ring, per side.
    pub ring_attackers: [u8; 2],
    /// Σ over those attackers of N, B = 1, R = 2, Q = 4, per side.
    pub ring_attack_weight: [u8; 2],
    /// Ring squares attacked by the king's own side other than the king
    /// itself, per side.
    pub ring_defended: [u8; 2],
    /// Ring squares attacked by the enemy and not among `ring_defended`, per
    /// side.
    pub ring_holes: [u8; 2],
    /// Adjacent squares empty or capturable and not attacked, per side.
    pub escape_squares: [u8; 2],
    /// King on its relative rank 1 with every forward-adjacent square held by
    /// a friendly unit, per side.
    pub back_rank_risk: [bool; 2],
    /// Chebyshev distance between the kings.
    pub distance: u8,
    /// Mean Chebyshev distance of enemy pieces to this king, per side.
    pub tropism: [f32; 2],
    /// Squares a queen on this king's square would attack, per side.
    pub virtual_mobility: [u8; 2],
}

/// Mobility and space.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct MobilityFacts {
    /// Attacked squares not held by own units, per side, by role P, N, B, R, Q.
    pub by_role: [[u16; 5]; 2],
    /// The same, minus squares attacked by an enemy pawn.
    pub safe_by_role: [[u16; 5]; 2],
    /// Sum of `by_role`, per side.
    pub total: [u16; 2],
    /// Attacked squares in the opponent's half, per side.
    pub space: [u16; 2],
    /// Attacked squares, per side.
    pub controlled: [u16; 2],
    /// Attacks on d4, e4, d5 and e5, per side.
    pub centre_control: [u8; 2],
    /// Attacks on c3 to f6, per side.
    pub extended_centre_control: [u8; 2],
    /// Units other than pawns and kings with no destination, per side.
    pub immobile_pieces: [u8; 2],
}

/// The attack maps and what they say about the units standing on the board.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AttackFacts {
    /// Each side's whole attack map.
    pub by: [SquareSet; 2],
    /// Each side's pawn attacks.
    pub by_pawns: [SquareSet; 2],
    /// Each side's attack map by role.
    pub by_role: [[SquareSet; 6]; 2],
    /// Units attacked by the opponent and not defended; never a king.
    pub hanging: [SquareSet; 2],
    /// Units hanging or attacked by a cheaper enemy unit; never a king.
    pub en_prise: [SquareSet; 2],
    /// Units under an absolute pin.
    pub pinned: [SquareSet; 2],
    /// Units standing on a square their own side attacks.
    pub defended: [SquareSet; 2],
    /// Value sum of the hanging units, per side.
    pub hanging_value: [i32; 2],
    /// Largest value en prise, per side.
    pub en_prise_max_value: [i32; 2],
    /// Enemy unit pairs this side's sliders skewer, per side.
    pub skewer_candidates: [u8; 2],
    units: [SquareSet; 2],
    role_units: [[SquareSet; 6]; 2],
    occupied: SquareSet,
    us: Colour,
}

impl AttackFacts {
    /// The units of `side` that attack `square`.
    pub fn attackers_of(&self, square: Square, side: Side) -> SquareSet {
        let colour = match side {
            Side::Us => self.us,
            Side::Them => !self.us,
        };
        attackers(
            square,
            colour,
            &self.role_units[side.index()],
            self.occupied,
        )
    }

    /// Whether the unit on `square`, of either colour, is hanging.
    pub fn is_hanging(&self, square: Square) -> bool {
        self.hanging[0].contains(square) || self.hanging[1].contains(square)
    }

    /// The units of `side`.
    pub fn units(&self, side: Side) -> SquareSet {
        self.units[side.index()]
    }
}

/// One side's one-ply tactical options.
///
/// The block for `Side::Them` is computed after a null move; when the side to
/// move is in check that move does not exist and the whole block is zero, with
/// `available` false.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct TacticsFacts {
    /// The block could be computed.
    pub available: bool,
    /// Moves giving check.
    pub check_count: u16,
    /// A checking move exists per moving role P, N, B, R, Q.
    pub check_by_role: [bool; 5],
    /// Checking moves whose destination is safe.
    pub safe_check_count: u16,
    /// A safe checking move exists per moving role P, N, B, R, Q.
    pub safe_check_by_role: [bool; 5],
    /// A move gives check from two units at once.
    pub double_check_available: bool,
    /// A move gives check with a unit that did not move.
    pub discovered_check_available: bool,
    /// A move leaves the opponent checkmated.
    pub mate_in_1: bool,
    /// A move leaves the opponent stalemated.
    pub stalemate_in_1: bool,
    /// Files a legal move promotes on.
    pub promotion_files: FileSet,
    /// A promotion to each of Q, R, B, N is available.
    pub promotion_roles: [bool; 4],
    /// Files a legal promotion with a safe destination lands on.
    pub safe_promotion_files: FileSet,
    /// Capturing moves.
    pub capture_count: u16,
    /// A capture whose victim outvalues the capturer or is undefended.
    pub winning_capture_available: bool,
    /// The largest `victim − capturer` over the captures, at least 0.
    pub winning_capture_max_gain: i32,
    /// A capture of a hanging unit.
    pub captures_hanging: bool,
    /// The largest value among the hanging units capturable now.
    pub hanging_victim_max_value: i32,
    /// Captures of a defended unit of equal value.
    pub equal_capture_count: u16,
    /// Captures of a defended unit of lower value.
    pub losing_capture_count: u16,
    /// Moves after which the moved unit forks.
    pub fork_count: u16,
    /// The largest single forked value.
    pub fork_max_value: i32,
    /// A fork by a knight.
    pub knight_fork_available: bool,
    /// A fork one of whose targets is the king.
    pub royal_fork_available: bool,
    /// Moves creating an absolute or a relative pin.
    pub pin_creation_count: u16,
    /// A move creating a skewer.
    pub skewer_creation_available: bool,
    /// A move uncovering a slider's attack on a unit of value 3 or more.
    pub discovered_attack_available: bool,
    /// Legal moves.
    pub legal_move_count: u16,
}

impl TacticsFacts {
    /// Whether a checking move exists.
    #[inline]
    pub fn check_available(&self) -> bool {
        self.check_count > 0
    }

    /// Whether a checking move with a safe destination exists.
    #[inline]
    pub fn safe_check_available(&self) -> bool {
        self.safe_check_count > 0
    }

    /// Whether a promotion is available.
    #[inline]
    pub fn promotion_available(&self) -> bool {
        !self.promotion_files.is_empty()
    }

    /// Whether a promotion with a safe destination is available.
    #[inline]
    pub fn safe_promotion_available(&self) -> bool {
        !self.safe_promotion_files.is_empty()
    }

    /// Whether a capture is available.
    #[inline]
    pub fn capture_available(&self) -> bool {
        self.capture_count > 0
    }

    /// Whether a forking move is available.
    #[inline]
    pub fn fork_available(&self) -> bool {
        self.fork_count > 0
    }

    /// Whether a move creating a pin is available.
    #[inline]
    pub fn pin_creation_available(&self) -> bool {
        self.pin_creation_count > 0
    }

    /// Whether the side to move has at most two legal moves.
    #[inline]
    pub fn only_moves(&self) -> bool {
        self.legal_move_count <= 2
    }
}

/// The eight square sets the `planes` group emits.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct PlaneFacts {
    /// Each side's attack map.
    pub attacked: [SquareSet; 2],
    /// Each side's pawn attacks.
    pub attacked_by_pawns: [SquareSet; 2],
    /// Each side's hanging units.
    pub hanging: [SquareSet; 2],
    /// Each side's absolutely pinned units.
    pub pinned: [SquareSet; 2],
}

/// What one legal move does, beyond what the move itself says.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MoveFacts {
    /// The role captured, the pawn for an en-passant capture.
    pub victim: Option<Role>,
    /// The role that moves.
    pub mover: Role,
    /// The role a promoting pawn becomes.
    pub promotion: Option<Role>,
    /// The move gives check.
    pub gives_check: bool,
    /// The move gives check and its destination is safe.
    pub gives_safe_check: bool,
    /// The destination is a safe destination.
    pub is_safe: bool,
    /// The move captures a hanging unit.
    pub captures_hanging: bool,
    /// The origin is attacked by the opponent and the destination is safe.
    pub escapes_attack: bool,
    /// The destination is attacked by an enemy pawn.
    pub to_attacked_by_pawn: bool,
    /// The move is a castling.
    pub is_castling: bool,
    /// The move is an en-passant capture.
    pub is_en_passant: bool,
}

impl Default for MoveFacts {
    fn default() -> MoveFacts {
        MoveFacts {
            victim: None,
            mover: Role::Pawn,
            promotion: None,
            gives_check: false,
            gives_safe_check: false,
            is_safe: false,
            captures_hanging: false,
            escapes_attack: false,
            to_attacked_by_pawn: false,
            is_castling: false,
            is_en_passant: false,
        }
    }
}

/// A legal move and its facts.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct AnnotatedMove {
    /// The move.
    pub mv: Move,
    /// What it does.
    pub facts: MoveFacts,
}

/// Reusable buffers for fact extraction. One per thread, or one per node of a
/// search stack.
#[derive(Clone, Default)]
pub struct Scratch {
    moves: MoveList,
    their_moves: MoveList,
    replies: MoveList,
}

impl Scratch {
    /// Empty buffers.
    pub fn new() -> Scratch {
        Scratch {
            moves: MoveList::new(),
            their_moves: MoveList::new(),
            replies: MoveList::new(),
        }
    }
}

impl fmt::Debug for Scratch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Scratch")
    }
}

/// Everything the v0 schema says about one position, plus its annotated legal
/// moves.
#[derive(Clone, PartialEq, Debug)]
pub struct Facts {
    /// Game-state flags.
    pub state: StateFacts,
    /// Material and phase.
    pub material: MaterialFacts,
    /// Pawn structure.
    pub pawns: PawnFacts,
    /// Bishops, rooks, knights and queens.
    pub pieces: PieceFacts,
    /// King safety and shelter.
    pub king: KingFacts,
    /// Mobility and space.
    pub mobility: MobilityFacts,
    /// Attack maps and what stands under them.
    pub attacks: AttackFacts,
    /// One-ply tactics, ours then theirs.
    pub tactics: [TacticsFacts; 2],
    /// The square sets the `planes` group emits.
    pub planes: PlaneFacts,
    /// Every legal move, annotated.
    pub moves: MoveList<AnnotatedMove>,
    variant: &'static str,
    us: Colour,
}

impl Facts {
    /// The variant these facts were computed under.
    #[inline]
    pub fn variant(&self) -> &'static str {
        self.variant
    }

    /// The colour that plays `Side::Us`.
    #[inline]
    pub fn side_to_move(&self) -> Colour {
        self.us
    }

    /// The side `colour` plays: the index into every side-paired fact.
    #[inline]
    pub fn side(&self, colour: Colour) -> Side {
        if colour == self.us {
            Side::Us
        } else {
            Side::Them
        }
    }

    /// Material, structure, king safety and threats, for a human reader. The
    /// text is not a stable format.
    pub fn summary(&self) -> String {
        let mut out = String::new();
        let phase = self.material.phase;
        out.push_str(&format!(
            "material us {} them {}, phase {phase:.2}\n",
            self.material.value[0], self.material.value[1],
        ));
        out.push_str(&format!(
            "pawns: passed {:?} / {:?}, islands {} / {}, open files {:?}\n",
            self.pawns.passed[0],
            self.pawns.passed[1],
            self.pawns.islands[0],
            self.pawns.islands[1],
            self.pawns.open_files,
        ));
        out.push_str(&format!(
            "kings: {} / {}, ring holes {} / {}, attack weight {} / {}\n",
            self.king.square[0],
            self.king.square[1],
            self.king.ring_holes[0],
            self.king.ring_holes[1],
            self.king.ring_attack_weight[0],
            self.king.ring_attack_weight[1],
        ));
        out.push_str(&format!(
            "hanging {:?} / {:?}, pinned {:?} / {:?}\n",
            self.attacks.hanging[0],
            self.attacks.hanging[1],
            self.attacks.pinned[0],
            self.attacks.pinned[1],
        ));
        out.push_str(&format!(
            "tactics us: {} legal, {} checks, {} captures, {} forks{}\n",
            self.tactics[0].legal_move_count,
            self.tactics[0].check_count,
            self.tactics[0].capture_count,
            self.tactics[0].fork_count,
            if self.tactics[0].mate_in_1 {
                ", mate in 1"
            } else {
                ""
            },
        ));
        out
    }
}

impl Position {
    /// The facts of this position under `variant`.
    ///
    /// The repetition and history values of `state` are zero: a position has
    /// no history.
    pub fn facts(&self, variant: &dyn Variant) -> Facts {
        let mut scratch = Scratch::new();
        self.facts_in(variant, &mut scratch)
    }

    /// The facts of this position under `variant`, reusing `scratch`. Nothing
    /// is allocated.
    pub fn facts_in(&self, variant: &dyn Variant, scratch: &mut Scratch) -> Facts {
        compute(self, variant, scratch)
    }
}

/// The whole extraction, in the order the groups are written.
fn compute(position: &Position, variant: &dyn Variant, scratch: &mut Scratch) -> Facts {
    let Scratch {
        moves: legal,
        their_moves,
        replies,
    } = scratch;

    let scan = Scan::new(position);
    let attacks = attack_facts(position, &scan);
    let pawns = pawns::pawn_facts(&scan);
    let pieces = pieces::piece_facts(&scan, &pawns);
    let king = king::king_facts(&scan);
    let mobility = mobility_facts(&scan);
    let material = material_facts(&scan);

    legal.clear();
    variant.legal_moves(position, legal);
    let state = state_facts(position, &scan, legal);

    let mut moves = MoveList::new();
    let ours = tactics::tactics(
        variant,
        position,
        &scan,
        Side::Us,
        &attacks,
        legal,
        replies,
        Some(&mut moves),
    );
    let theirs = match position.null_move() {
        Some(null) => {
            their_moves.clear();
            variant.legal_moves(&null, their_moves);
            tactics::tactics(
                variant,
                &null,
                &scan,
                Side::Them,
                &attacks,
                their_moves,
                replies,
                None,
            )
        }
        None => TacticsFacts::default(),
    };

    let planes = PlaneFacts {
        attacked: attacks.by,
        attacked_by_pawns: attacks.by_pawns,
        hanging: attacks.hanging,
        pinned: attacks.pinned,
    };

    Facts {
        state,
        material,
        pawns,
        pieces,
        king,
        mobility,
        attacks,
        tactics: [ours, theirs],
        planes,
        moves,
        variant: variant.name(),
        us: scan.us,
    }
}

fn state_facts(position: &Position, scan: &Scan, legal: &MoveList) -> StateFacts {
    let rights = position.castling_rights();
    StateFacts {
        in_check: position.in_check(),
        double_check: position.checkers().len() >= 2,
        castle_short: Side::ALL.map(|side| rights.short(scan.colour(side)).is_some()),
        castle_long: Side::ALL.map(|side| rights.long(scan.colour(side)).is_some()),
        en_passant: position.en_passant().map(|square| square.file()),
        ep_capture_legal: legal.iter().any(|mv| mv.is_en_passant()),
        halfmove_clock: position.halfmove_clock(),
        halfmove_known: position.clocks_known(),
        repetition_seen: false,
        repetition_available: [false; 2],
        history_known: false,
    }
}

fn material_facts(scan: &Scan) -> MaterialFacts {
    let mut facts = MaterialFacts::default();
    let mut phase_points = 0u32;
    for side in Side::ALL {
        let i = side.index();
        for role in [
            Role::Pawn,
            Role::Knight,
            Role::Bishop,
            Role::Rook,
            Role::Queen,
        ] {
            let n = scan.role_units[i][role.index()].len();
            facts.count[i][role.index()] = n.min(255) as u8;
            let value = material_value(role) * n as i32;
            facts.value[i] += value;
            if role != Role::Pawn {
                facts.non_pawn_value[i] += value;
            }
            phase_points += n * match role {
                Role::Queen => 4,
                Role::Rook => 2,
                Role::Bishop | Role::Knight => 1,
                _ => 0,
            };
        }
        facts.insufficient[i] = insufficient(&scan.role_units[i]);
    }
    facts.phase = phase_points.min(24) as f32 / 24.0;
    facts.both_queens =
        facts.count[0][Role::Queen.index()] > 0 && facts.count[1][Role::Queen.index()] > 0;
    facts.pawns_only = Side::ALL.iter().all(|side| {
        let i = side.index();
        [Role::Knight, Role::Bishop, Role::Rook, Role::Queen]
            .iter()
            .all(|role| scan.role_units[i][role.index()].is_empty())
    });
    facts
}

/// Whether a side's own material could never deliver mate: a bare king, a
/// king and one minor, or a king and bishops of a single square colour.
fn insufficient(role_units: &[SquareSet; 6]) -> bool {
    let heavy = role_units[Role::Pawn.index()]
        | role_units[Role::Rook.index()]
        | role_units[Role::Queen.index()];
    if !heavy.is_empty() {
        return false;
    }
    let knights = role_units[Role::Knight.index()];
    let bishops = role_units[Role::Bishop.index()];
    if knights.len() + bishops.len() <= 1 {
        return true;
    }
    knights.is_empty()
        && (bishops.is_subset(SquareSet::DARK) || bishops.is_subset(SquareSet::LIGHT))
}

fn attack_facts(position: &Position, scan: &Scan) -> AttackFacts {
    let mut facts = AttackFacts {
        by: scan.by,
        by_pawns: [
            scan.by_role[0][Role::Pawn.index()],
            scan.by_role[1][Role::Pawn.index()],
        ],
        by_role: scan.by_role,
        hanging: [SquareSet::EMPTY; 2],
        en_prise: [SquareSet::EMPTY; 2],
        pinned: [SquareSet::EMPTY; 2],
        defended: [SquareSet::EMPTY; 2],
        hanging_value: [0; 2],
        en_prise_max_value: [0; 2],
        skewer_candidates: [0; 2],
        units: scan.units,
        role_units: scan.role_units,
        occupied: scan.occupied,
        us: scan.us,
    };

    for side in Side::ALL {
        let i = side.index();
        let them = !side;
        facts.defended[i] = scan.units[i] & scan.by[i];
        for square in scan.units[i] - scan.role_units[i][Role::King.index()] {
            let role = position
                .piece_at(square)
                .expect("a unit stands on its own square")
                .role;
            let attackers = scan.attackers_of(square, them);
            if attackers.is_empty() {
                continue;
            }
            let defended = scan.by[i].contains(square);
            let cheaper = attackers.into_iter().any(|from| {
                let attacker = position
                    .piece_at(from)
                    .expect("an attacker stands on its own square")
                    .role;
                order_value(attacker) < order_value(role)
            });
            if !defended {
                facts.hanging[i].insert(square);
                facts.hanging_value[i] += material_value(role);
            }
            if !defended || cheaper {
                facts.en_prise[i].insert(square);
                facts.en_prise_max_value[i] = facts.en_prise_max_value[i].max(material_value(role));
            }
        }
        facts.pinned[i] = absolute_pins(scan, side);
        facts.skewer_candidates[i] = skewers(position, scan, side);
    }
    facts
}

/// The units of `side` that may not legally move because their own king would
/// be exposed.
fn absolute_pins(scan: &Scan, side: Side) -> SquareSet {
    let king = scan.kings[side.index()];
    let them = (!side).index();
    let mut pinned = SquareSet::EMPTY;
    let bishops =
        scan.role_units[them][Role::Bishop.index()] | scan.role_units[them][Role::Queen.index()];
    let rooks =
        scan.role_units[them][Role::Rook.index()] | scan.role_units[them][Role::Queen.index()];
    let candidates = (attacks_of(Role::Bishop, king, scan.us, SquareSet::EMPTY) & bishops)
        | (attacks_of(Role::Rook, king, scan.us, SquareSet::EMPTY) & rooks);
    for slider in candidates {
        let blockers = between(slider, king) & scan.occupied;
        if blockers.len() == 1 && blockers.is_subset(scan.units[side.index()]) {
            pinned |= blockers;
        }
    }
    pinned
}

/// How many enemy pairs `side`'s sliders skewer: an attacked unit with a unit
/// of lower or equal value directly behind it on the same ray.
fn skewers(position: &Position, scan: &Scan, side: Side) -> u8 {
    let i = side.index();
    let them = (!side).index();
    let mut count = 0u8;
    for role in [Role::Bishop, Role::Rook, Role::Queen] {
        for slider in scan.role_units[i][role.index()] {
            let attacks = scan.attacks_from[slider.index()];
            for front in attacks & scan.units[them] {
                let front_role = position
                    .piece_at(front)
                    .expect("a unit stands on its own square")
                    .role;
                let xray = attacks_of(
                    role,
                    slider,
                    scan.colour(side),
                    scan.occupied - front.to_set(),
                );
                let behind = (xray - attacks) & line(slider, front) & scan.units[them];
                for back in behind {
                    let back_role = position
                        .piece_at(back)
                        .expect("a unit stands on its own square")
                        .role;
                    if order_value(back_role) <= order_value(front_role) {
                        count = count.saturating_add(1);
                    }
                }
            }
        }
    }
    count
}

fn mobility_facts(scan: &Scan) -> MobilityFacts {
    let mut facts = MobilityFacts::default();
    for side in Side::ALL {
        let i = side.index();
        let enemy_pawns = scan.by_role[(!side).index()][Role::Pawn.index()];
        for role in [
            Role::Pawn,
            Role::Knight,
            Role::Bishop,
            Role::Rook,
            Role::Queen,
        ] {
            let free = scan.by_role[i][role.index()] - scan.units[i];
            facts.by_role[i][role.index()] = free.len() as u16;
            facts.safe_by_role[i][role.index()] = (free - enemy_pawns).len() as u16;
            facts.total[i] += free.len() as u16;
        }
        facts.space[i] = (scan.by[i] & scan.own_half(!side)).len() as u16;
        facts.controlled[i] = scan.by[i].len() as u16;
        facts.centre_control[i] = (scan.by[i] & CENTRE).len() as u8;
        facts.extended_centre_control[i] = (scan.by[i] & EXTENDED_CENTRE).len() as u8;
        let pieces = scan.units[i]
            - scan.role_units[i][Role::Pawn.index()]
            - scan.role_units[i][Role::King.index()];
        facts.immobile_pieces[i] = pieces
            .into_iter()
            .filter(|square| (scan.attacks_from[square.index()] - scan.units[i]).is_empty())
            .count()
            .min(255) as u8;
    }
    facts
}

/// The mean Chebyshev distance of `side`'s knights, bishops, rooks and queens
/// to `square`, or 0 when it has none.
fn tropism(scan: &Scan, square: Square, side: Side) -> f32 {
    let i = side.index();
    let pieces = scan.role_units[i][Role::Knight.index()]
        | scan.role_units[i][Role::Bishop.index()]
        | scan.role_units[i][Role::Rook.index()]
        | scan.role_units[i][Role::Queen.index()];
    if pieces.is_empty() {
        return 0.0;
    }
    let total: u32 = pieces.into_iter().map(|from| distance(from, square)).sum();
    total as f32 / pieces.len() as f32
}
