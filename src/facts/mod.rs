//! What is true about a position and about each of its legal moves.
//!
//! Every fact names the colour it is about: a value that differs between the
//! two sides is a [`ByColour`], read as `.white`, `.black` or `.of(colour)`.
//! Where a fact is about the side to move — that it stands in check, say — it
//! is a plain value and its meaning says so.
//!
//! [`catalogue`] lists every group, field, type and meaning as data.

pub mod catalogue;
mod endgame;
mod exchange;
mod history;
mod king;
mod pawns;
mod pieces;
pub(crate) mod scan;
#[cfg(feature = "serde")]
pub mod schema;
#[cfg(feature = "serde")]
pub mod stub;
mod tactics;
#[cfg(feature = "tensors")]
pub mod tensors;
mod threats;
pub mod types;

use core::fmt;

use crate::moves::{Move, MoveList};
use crate::position::Position;
use crate::types::{ByColour, Colour, File, FileSet, PerFile, PerRank, Role, Square, SquareSet};
use crate::variant::Variant;

use catalogue::{Group, facts_group};
pub(crate) use history::from_history;
pub(crate) use scan::Scan;
use scan::{
    CENTRE, EXTENDED_CENTRE, attacks_of, between, distance, line, material_value, order_value,
};

facts_group! {
    /// Where the units stand.
    derive(Clone, Copy, PartialEq, Eq, Debug, Default);
    pub struct PlacementFacts as "placement" {
        /// Every unit of the side.
        pub units: ByColour<SquareSet>,
        /// The side's pawns.
        pub pawns: ByColour<SquareSet>,
        /// The side's knights.
        pub knights: ByColour<SquareSet>,
        /// The side's bishops.
        pub bishops: ByColour<SquareSet>,
        /// The side's rooks.
        pub rooks: ByColour<SquareSet>,
        /// The side's queens.
        pub queens: ByColour<SquareSet>,
        /// The side's king.
        pub kings: ByColour<SquareSet>,
    }
}

impl PlacementFacts {
    /// The units of `colour` with `role`.
    #[inline]
    pub fn of(&self, colour: Colour, role: Role) -> SquareSet {
        match role {
            Role::Pawn => self.pawns.of(colour),
            Role::Knight => self.knights.of(colour),
            Role::Bishop => self.bishops.of(colour),
            Role::Rook => self.rooks.of(colour),
            Role::Queen => self.queens.of(colour),
            Role::King => self.kings.of(colour),
        }
    }
}

facts_group! {
    /// Check, castling rights and the en-passant square.
    derive(Clone, Copy, PartialEq, Eq, Debug, Default);
    pub struct StateFacts as "state" {
        /// The side to move stands in check.
        pub in_check: bool,
        /// Two or more units give check to the side to move.
        pub double_check: bool,
        /// The side may still castle short.
        pub can_castle_short: ByColour<bool>,
        /// The side may still castle long.
        pub can_castle_long: ByColour<bool>,
        /// The file the position names as the en-passant target, if any.
        pub en_passant: Option<File>,
        /// Some legal move captures en passant.
        pub en_passant_capture_legal: bool,
    }
}

facts_group! {
    /// What the plies before this position say: the clock, repetition, and how
    /// forcing the recent play was.
    ///
    /// Everything but the halfmove clock, which the position itself carries, is
    /// zero unless `known` is true.
    derive(Clone, Copy, PartialEq, Eq, Debug, Default);
    pub struct HistoryFacts as "history" {
        /// A position history was supplied.
        pub known: bool,
        /// Plies since the last capture or pawn move.
        pub halfmove_clock: u32,
        /// The position carried a halfmove clock.
        pub halfmove_known: bool,
        /// This position occurred before in the supplied history.
        pub repetition_seen: bool,
        /// Some legal move reaches a position of the supplied history.
        pub repetition_available: bool,
        /// Captures among the last eight plies of the supplied history.
        pub captures_in_last_8: u8,
        /// Plies among the last eight that gave check.
        pub checks_in_last_8: u8,
        /// Plies since the last one that captured or gave check; the whole
        /// history when it holds none.
        pub quiet_plies: u32,
        /// The material balance now, less the balance eight plies ago, White
        /// less Black.
        pub material_trend: i32,
        /// The role the last move captured, the pawn for an en-passant capture.
        pub last_move_victim: Option<Role>,
        /// The role that made the last move; for castling, the king.
        pub last_move_mover: Option<Role>,
    }
}

facts_group! {
    /// Material and phase.
    derive(Clone, Copy, PartialEq, Eq, Debug, Default);
    pub struct MaterialFacts as "material" {
        /// The side's pawns.
        pub pawns: ByColour<u8>,
        /// The side's knights.
        pub knights: ByColour<u8>,
        /// The side's bishops.
        pub bishops: ByColour<u8>,
        /// The side's rooks.
        pub rooks: ByColour<u8>,
        /// The side's queens.
        pub queens: ByColour<u8>,
        /// The value of the side's knights, bishops, rooks and queens.
        pub non_pawn_value: ByColour<i32>,
        /// The value of every unit of the side but the king.
        pub value: ByColour<i32>,
        /// `4·Q + 2·R + B + N` over both sides: 24 at the full opening array,
        /// 0 with nothing but kings and pawns left.
        pub phase_points: u8,
        /// Both sides have at least one queen.
        pub both_queens: bool,
        /// Neither side has a unit other than kings and pawns.
        pub pawns_only: bool,
        /// The side's own material could never deliver mate.
        pub insufficient_material: ByColour<bool>,
    }
}

facts_group! {
    /// Pawn structure.
    derive(Clone, Copy, PartialEq, Eq, Debug, Default);
    pub struct PawnFacts as "pawns" {
        /// The side's pawns.
        pub pawns: ByColour<SquareSet>,
        /// The side's pawns with no enemy pawn ahead on their own or an
        /// adjacent file.
        pub passed: ByColour<SquareSet>,
        /// The side's candidate passers: pawns with no enemy pawn ahead on
        /// their own file, backed by at least as many friendly pawns behind on
        /// the adjacent files as there are enemy pawns ahead on them.
        pub candidates: ByColour<SquareSet>,
        /// The side's pawns sharing a file with another pawn of their own
        /// colour.
        pub doubled: ByColour<SquareSet>,
        /// The side's pawns with no friendly pawn on either adjacent file.
        pub isolated: ByColour<SquareSet>,
        /// The side's pawns that cannot advance without being taken and cannot
        /// be defended by a pawn.
        pub backward: ByColour<SquareSet>,
        /// The side's pawns defended by a pawn of their own colour.
        pub defended: ByColour<SquareSet>,
        /// The side's pawns per file, file a first.
        pub count_by_file: ByColour<PerFile<u8>>,
        /// The side's pawns per rank counted from its own back rank, rank 1
        /// first.
        pub count_by_rank: ByColour<PerRank<u8>>,
        /// Files carrying no pawn of either colour.
        pub open_files: FileSet,
        /// Files carrying no pawn of the side and at least one of the other.
        pub semi_open_files: ByColour<FileSet>,
        /// The side's maximal runs of adjacent files carrying a pawn.
        pub islands: ByColour<u8>,
        /// The side's pawns that can capture an enemy pawn.
        pub levers: ByColour<u8>,
        /// Pawn pairs blocking each other head on.
        pub rams: u8,
        /// The rank, counted from the side's own back rank, of its most
        /// advanced passer.
        pub passer_lead_rank: ByColour<Option<u8>>,
        /// The side's passers defended by a friendly pawn.
        pub passer_protected: ByColour<u8>,
        /// The side has two passers on adjacent files.
        pub passers_connected: ByColour<bool>,
        /// The side has a passer the enemy king cannot catch.
        pub passer_unstoppable: ByColour<bool>,
        /// The side's longest run of pawns each defending the next.
        pub chain_max_length: ByColour<u8>,
        /// An enemy unit attacks the base of one of the side's chains of two or
        /// more.
        pub chain_base_attacked: ByColour<bool>,
        /// The side has more pawns than the enemy on files a to d.
        pub queenside_majority: ByColour<bool>,
        /// The side has more pawns than the enemy on files e to h.
        pub kingside_majority: ByColour<bool>,
        /// Squares on the side's ranks 3 to 6 that no pawn of the side can ever
        /// attack.
        pub holes: ByColour<SquareSet>,
        /// Enemy knights and bishops standing on those squares.
        pub holes_occupied: ByColour<u8>,
        /// The side's pawns whose stop square holds a unit of either colour.
        pub fixed_pawns: ByColour<u8>,
        /// The side's passers whose stop square holds an enemy unit.
        pub blocked_passers: ByColour<u8>,
        /// The pushes the side's lead passer still needs.
        pub passer_distance: ByColour<Option<u8>>,
        /// The distance from the side's own king to its lead passer's promotion
        /// square.
        pub passer_own_king_distance: ByColour<Option<u8>>,
        /// The distance from the enemy king to that same square.
        pub passer_enemy_king_distance: ByColour<Option<u8>>,
        /// The enemy king stands in the square of the side's lead passer.
        pub passer_in_square: ByColour<bool>,
        /// The side's passers whose whole front span is empty.
        pub passer_free_path: ByColour<u8>,
        /// Files semi-open for the side among the enemy king's three shield
        /// files.
        pub half_open_at_enemy_king: ByColour<u8>,
        /// The side's backward pawns on a file semi-open for the enemy.
        pub backward_on_semi_open: ByColour<u8>,
    }
}

facts_group! {
    /// Bishops, rooks, knights and queens.
    derive(Clone, Copy, PartialEq, Eq, Debug, Default);
    pub struct PieceFacts as "pieces" {
        /// The side holds bishops on both square colours.
        pub bishop_pair: ByColour<bool>,
        /// The side holds two or more knights.
        pub knight_pair: ByColour<bool>,
        /// The side's bishops on light squares.
        pub bishops_light: ByColour<u8>,
        /// The side's bishops on dark squares.
        pub bishops_dark: ByColour<u8>,
        /// Exactly one bishop each, on different square colours.
        pub opposite_coloured_bishops: bool,
        /// The side's pawns standing on a square colour of one of its own
        /// bishops.
        pub pawns_on_bishop_colour: ByColour<u8>,
        /// Two of the side's rooks share a rank with nothing between.
        pub rooks_connected_rank: ByColour<bool>,
        /// Two of the side's rooks share a file with nothing between.
        pub rooks_connected_file: ByColour<bool>,
        /// The side's rooks on an open file.
        pub rooks_on_open_file: ByColour<u8>,
        /// The side's rooks on a file semi-open for their own side.
        pub rooks_on_semi_open_file: ByColour<u8>,
        /// The side's rooks on its seventh rank, counted from its own back
        /// rank.
        pub rooks_on_seventh_rank: ByColour<u8>,
        /// The side's rooks behind a passer of their own side.
        pub rook_behind_own_passer: ByColour<u8>,
        /// The side's rooks behind an enemy passer.
        pub rook_behind_enemy_passer: ByColour<u8>,
        /// The side has a rook shut in by its own king with no file to leave
        /// by.
        pub trapped_rook: ByColour<bool>,
        /// The side's outpost squares: squares on its ranks 4 to 6 that a
        /// friendly pawn defends and no enemy pawn can ever attack.
        pub outposts: ByColour<SquareSet>,
        /// The side's knights and bishops standing on one of them.
        pub minors_on_outpost: ByColour<u8>,
        /// Its unoccupied outpost squares.
        pub outpost_squares_free: ByColour<u8>,
        /// The side's knights on file a or h, or on its rank 1 or 8.
        pub knights_on_rim: ByColour<u8>,
        /// The side's knights and bishops still on their starting squares.
        pub minors_undeveloped: ByColour<u8>,
        /// A queen of the side stands off its starting square.
        pub queen_developed: ByColour<bool>,
        /// The side's fixed pawns standing on a square colour of one of its own
        /// bishops.
        pub fixed_pawns_on_bishop_colour: ByColour<u8>,
        /// The side has a rook on its seventh rank with the enemy king on its
        /// eighth.
        pub rook_on_seventh_with_king_on_eighth: ByColour<bool>,
        /// The side's units that are neither pawns nor kings and have no safe
        /// destination.
        pub trapped_pieces: ByColour<u8>,
        /// The value sum of those.
        pub trapped_value: ByColour<u8>,
    }
}

/// The side a king's square and its spent castling rights read as. A side that
/// still holds a castling right has castled neither way.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CastledSide {
    /// The king stands on file g or h.
    Short,
    /// The king stands on file a, b or c.
    Long,
}

impl CastledSide {
    /// The name the side carries in text: `"short"` or `"long"`.
    pub const fn name(self) -> &'static str {
        match self {
            CastledSide::Short => "short",
            CastledSide::Long => "long",
        }
    }
}

facts_group! {
    /// King safety and shelter.
    derive(Clone, Copy, PartialEq, Debug);
    pub struct KingFacts as "king" {
        /// Where the side's king stands.
        pub square: ByColour<Square>,
        /// It stands on its own starting square.
        pub on_home_square: ByColour<bool>,
        /// It stands on files a to c.
        pub castled_queenside: ByColour<bool>,
        /// It stands on files f to h.
        pub castled_kingside: ByColour<bool>,
        /// The three files the king's shelter is read on, queenside first.
        pub shield_files: ByColour<[File; 3]>,
        /// Ranks to the nearest friendly pawn ahead of the king, one per file of
        /// the three around the king, queenside first; `shield_files` names them.
        pub shield: ByColour<[Option<u8>; 3]>,
        /// Whether each of the three files around the king carries no pawn of
        /// either colour, queenside first; `shield_files` names them.
        pub shield_file_open: ByColour<[bool; 3]>,
        /// Whether each of those three files is semi-open for the enemy of that
        /// king, queenside first; `shield_files` names them.
        pub shield_file_semi_open_for_enemy: ByColour<[bool; 3]>,
        /// Ranks to the nearest enemy pawn ahead of the king, one per file of the
        /// three around the king, queenside first; `shield_files` names them.
        pub storm: ByColour<[Option<u8>; 3]>,
        /// The squares adjacent to the king.
        pub ring: ByColour<SquareSet>,
        /// Enemy knights, bishops, rooks and queens attacking that ring.
        pub ring_attackers: ByColour<u8>,
        /// Σ over those attackers of N, B = 1, R = 2, Q = 4.
        pub ring_attack_weight: ByColour<u8>,
        /// Ring squares the king's own side attacks, the king itself apart.
        pub ring_defended: ByColour<u8>,
        /// Ring squares the enemy attacks and `ring_defended` does not count.
        pub ring_holes: ByColour<u8>,
        /// Squares adjacent to the king that are empty or capturable and not
        /// attacked.
        pub escape_squares: ByColour<u8>,
        /// The king stands on its rank 1 with every forward-adjacent square
        /// held by a friendly unit.
        pub back_rank_risk: ByColour<bool>,
        /// Chebyshev distance between the kings.
        pub distance: u8,
        /// Mean Chebyshev distance of the enemy's knights, bishops, rooks and
        /// queens to this king.
        pub tropism: ByColour<f32>,
        /// Squares a queen on this king's square would attack.
        pub virtual_mobility: ByColour<u8>,
        /// The king's own knights, bishops, rooks and queens attacking its
        /// ring.
        pub ring_defenders: ByColour<u8>,
        /// Σ over those defenders of N, B = 1, R = 2, Q = 4.
        pub ring_defence_weight: ByColour<u8>,
        /// Directions from the king holding at least one square, all of them
        /// empty out to the edge.
        pub open_rays: ByColour<u8>,
        /// The king stands on its rank 1 with a square ahead of it empty and
        /// unattacked.
        pub luft: ByColour<bool>,
        /// The side the king's square and its spent castling rights read as.
        pub castled_side: ByColour<Option<CastledSide>>,
        /// The two kings stand on opposite wings.
        pub opposite_side_castling: bool,
    }
}

impl KingFacts {
    /// The weight besieging each king's ring, less the weight defending it:
    /// negative for a king whose own side outweighs the attackers.
    #[inline]
    pub fn ring_attacker_surplus(&self) -> ByColour<i16> {
        ByColour::from_fn(|colour| {
            i16::from(self.ring_attack_weight.of(colour))
                - i16::from(self.ring_defence_weight.of(colour))
        })
    }
}

facts_group! {
    /// Mobility and space.
    derive(Clone, Copy, PartialEq, Eq, Debug, Default);
    pub struct MobilityFacts as "mobility" {
        /// Squares the side's pawns attack that its own units do not hold.
        pub pawns: ByColour<u16>,
        /// The same for its knights.
        pub knights: ByColour<u16>,
        /// The same for its bishops.
        pub bishops: ByColour<u16>,
        /// The same for its rooks.
        pub rooks: ByColour<u16>,
        /// The same for its queens.
        pub queens: ByColour<u16>,
        /// Of the pawn squares, those no enemy pawn attacks.
        pub safe_pawns: ByColour<u16>,
        /// The same for its knights.
        pub safe_knights: ByColour<u16>,
        /// The same for its bishops.
        pub safe_bishops: ByColour<u16>,
        /// The same for its rooks.
        pub safe_rooks: ByColour<u16>,
        /// The same for its queens.
        pub safe_queens: ByColour<u16>,
        /// The five counts added up.
        pub total: ByColour<u16>,
        /// Squares the side attacks in the enemy's half.
        pub space: ByColour<u16>,
        /// The side's attacks on d4, e4, d5 and e5.
        pub centre_control: ByColour<u8>,
        /// The side's attacks on c3 to f6.
        pub extended_centre_control: ByColour<u8>,
        /// The side's units other than pawns and kings with no destination.
        pub immobile_pieces: ByColour<u8>,
    }
}

facts_group! {
    /// The attack maps and what they say about the units standing on the board.
    derive(Clone, Copy, PartialEq, Eq, Debug, Default);
    pub struct AttackFacts as "attacks" {
        /// Every square the side attacks.
        pub controlled: ByColour<SquareSet>,
        /// The squares the side's pawns attack.
        pub by_pawns: ByColour<SquareSet>,
        /// The squares its knights attack.
        pub by_knights: ByColour<SquareSet>,
        /// The squares its bishops attack.
        pub by_bishops: ByColour<SquareSet>,
        /// The squares its rooks attack.
        pub by_rooks: ByColour<SquareSet>,
        /// The squares its queens attack.
        pub by_queens: ByColour<SquareSet>,
        /// The squares its king attacks.
        pub by_kings: ByColour<SquareSet>,
        /// The side's units the enemy attacks, defended or not; never a king.
        pub attacked: ByColour<SquareSet>,
        /// Those of them their own side does not defend.
        pub hanging: ByColour<SquareSet>,
        /// Those hanging or attacked by a cheaper enemy unit.
        pub en_prise: ByColour<SquareSet>,
        /// The side's units under an absolute pin.
        pub pinned: ByColour<SquareSet>,
        /// The side's units standing on a square their own side attacks.
        pub defended: ByColour<SquareSet>,
        /// The value sum of the attacked units.
        pub attacked_value: ByColour<i32>,
        /// The value sum of the hanging units.
        pub hanging_value: ByColour<i32>,
        /// The value sum of the units en prise.
        pub en_prise_value: ByColour<i32>,
        /// The largest value en prise.
        pub en_prise_max_value: ByColour<i32>,
        /// The value sum of the absolutely pinned units.
        pub pinned_value: ByColour<i32>,
        /// Enemy unit pairs the side's sliders skewer.
        pub skewer_candidates: ByColour<u8>,
    }
}

facts_group! {
    /// Each side's captures, judged by the exchange they start.
    ///
    /// The values for the side not to move are read after a null move; when the
    /// side to move is in check that move does not exist and they are zero.
    derive(Clone, Copy, PartialEq, Eq, Debug, Default);
    pub struct ExchangeFacts as "exchange" {
        /// The largest static exchange evaluation over the side's captures; 0
        /// when it has none.
        pub see_best_capture: ByColour<i32>,
        /// Its captures whose evaluation is above 0.
        pub see_positive_capture_count: ByColour<u16>,
        /// Its captures whose evaluation is 0.
        pub see_equal_capture_count: ByColour<u16>,
        /// The sum of the evaluations above 0.
        pub see_positive_total: ByColour<i32>,
    }
}

facts_group! {
    /// What each side stands to lose, and the slider geometry behind it.
    ///
    /// Every set is read on the units it is about, and holds no king.
    derive(Clone, Copy, PartialEq, Eq, Debug, Default);
    pub struct ThreatFacts as "threats" {
        /// The side's units the enemy wins material by taking.
        pub threatened: ByColour<SquareSet>,
        /// The value sum of those.
        pub threatened_value: ByColour<i32>,
        /// The largest such gain over the side's own units.
        pub threat_max_gain: ByColour<i32>,
        /// The side's units an enemy unit of strictly lower value attacks.
        pub attacked_by_lesser: ByColour<SquareSet>,
        /// One of those units is a queen.
        pub queen_attacked_by_lesser: ByColour<bool>,
        /// The side's sole defenders of two or more attacked friendly units.
        pub overloaded_defenders: ByColour<SquareSet>,
        /// Its sole defenders the enemy can capture without loss.
        pub removable_defenders: ByColour<SquareSet>,
        /// The side's units no unit of their own side defends.
        pub loose: ByColour<SquareSet>,
        /// The side's units more enemy units attack than its own defend.
        pub attacker_surplus: ByColour<SquareSet>,
        /// The side's x-rays onto an enemy unit through one enemy unit.
        pub xray_through_enemy: ByColour<u8>,
        /// The side's batteries: two of its own sliders on one line, agreeing
        /// in direction.
        pub battery_count: ByColour<u8>,
        /// One of them bears on a square of the enemy king's ring.
        pub battery_at_king: ByColour<bool>,
    }
}

/// Which opposition the kings stand in. The side not to move holds it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Opposition {
    /// One empty square stands between the kings.
    Direct,
    /// Three or five.
    Distant,
}

impl Opposition {
    /// The name the opposition carries in text: `"direct"` or `"distant"`.
    pub const fn name(self) -> &'static str {
        match self {
            Opposition::Direct => "direct",
            Opposition::Distant => "distant",
        }
    }
}

/// A material configuration that draws although one side is ahead. The three
/// exclude one another.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DrawishMaterial {
    /// Two knights against a bare king.
    TwoKnights,
    /// A wrong-colour bishop with its rook pawns against a bare king.
    WrongBishop,
    /// One bishop each, on opposite square colours, with no other piece.
    OppositeBishops,
}

impl DrawishMaterial {
    /// The name the configuration carries in text.
    pub const fn name(self) -> &'static str {
        match self {
            DrawishMaterial::TwoKnights => "two_knights",
            DrawishMaterial::WrongBishop => "wrong_bishop",
            DrawishMaterial::OppositeBishops => "opposite_bishops",
        }
    }
}

facts_group! {
    /// The square sets a reader most often wants side by side: what each side
    /// attacks, and what of it stands hanging, pinned or threatened.
    ///
    /// Every one of them is also a field of the `attacks` or the `threats`
    /// group; this group is where they are read together.
    derive(Clone, Copy, PartialEq, Eq, Debug, Default);
    pub struct MapFacts as "maps" {
        /// Every square the side attacks.
        pub controlled: ByColour<SquareSet>,
        /// The squares the side's pawns attack.
        pub by_pawns: ByColour<SquareSet>,
        /// The side's units the enemy attacks and their own side does not
        /// defend.
        pub hanging: ByColour<SquareSet>,
        /// The side's units under an absolute pin.
        pub pinned: ByColour<SquareSet>,
        /// The side's units the enemy wins material by taking.
        pub threatened: ByColour<SquareSet>,
    }
}

facts_group! {
    /// Where the kings stand, how the pawn race runs, and which drawn
    /// configuration the material is.
    derive(Clone, Copy, PartialEq, Eq, Debug, Default);
    pub struct EndgameFacts as "endgame" {
        /// Chebyshev distance from the side's king to the nearest central
        /// square.
        pub king_centralisation: ByColour<u8>,
        /// Plies the side's most advanced passer needs to promote unopposed; 8
        /// for a side with no passer.
        pub race_plies: ByColour<u8>,
        /// The opposition the kings stand in, if any.
        pub opposition: Option<Opposition>,
        /// The side's king stands on a key square of one of its own passers.
        pub key_square_occupied: ByColour<bool>,
        /// The side's bishops all stand on the colour none of its pawns
        /// promotes on, every one of those pawns being a rook pawn.
        pub wrong_colour_bishop: ByColour<bool>,
        /// The drawn configuration the material is, if it is one of the three.
        pub drawish_material: Option<DrawishMaterial>,
    }
}

facts_group! {
    /// Each side's one-ply tactical options.
    ///
    /// The values for the side not to move are read after a null move; when the
    /// side to move is in check that move does not exist and they are zero,
    /// which `available` reports.
    derive(Clone, Copy, PartialEq, Eq, Debug, Default);
    pub struct TacticsFacts as "tactics" {
        /// The side's values could be read.
        pub available: ByColour<bool>,
        /// Its moves giving check.
        pub check_count: ByColour<u16>,
        /// A pawn move of the side gives check.
        pub check_by_pawn: ByColour<bool>,
        /// A knight move gives check.
        pub check_by_knight: ByColour<bool>,
        /// A bishop move gives check.
        pub check_by_bishop: ByColour<bool>,
        /// A rook move gives check.
        pub check_by_rook: ByColour<bool>,
        /// A queen move gives check.
        pub check_by_queen: ByColour<bool>,
        /// Its checking moves whose destination is safe.
        pub safe_check_count: ByColour<u16>,
        /// A pawn move of the side gives check from a safe square.
        pub safe_check_by_pawn: ByColour<bool>,
        /// A knight move does.
        pub safe_check_by_knight: ByColour<bool>,
        /// A bishop move does.
        pub safe_check_by_bishop: ByColour<bool>,
        /// A rook move does.
        pub safe_check_by_rook: ByColour<bool>,
        /// A queen move does.
        pub safe_check_by_queen: ByColour<bool>,
        /// A move gives check from two units at once.
        pub double_check_available: ByColour<bool>,
        /// A move gives check with a unit that did not move.
        pub discovered_check_available: ByColour<bool>,
        /// A move leaves the opponent checkmated.
        pub mate_in_1: ByColour<bool>,
        /// A move leaves the opponent stalemated.
        pub stalemate_in_1: ByColour<bool>,
        /// Files a legal move of the side promotes on.
        pub promotion_files: ByColour<FileSet>,
        /// A promotion to a queen is available.
        pub promotes_to_queen: ByColour<bool>,
        /// One to a rook is.
        pub promotes_to_rook: ByColour<bool>,
        /// One to a bishop is.
        pub promotes_to_bishop: ByColour<bool>,
        /// One to a knight is.
        pub promotes_to_knight: ByColour<bool>,
        /// Files a legal promotion with a safe destination lands on.
        pub safe_promotion_files: ByColour<FileSet>,
        /// The side's capturing moves.
        pub capture_count: ByColour<u16>,
        /// One of them takes a unit that outvalues the capturer or is
        /// undefended.
        pub winning_capture_available: ByColour<bool>,
        /// The largest victim value less capturer value over the captures, at
        /// least 0.
        pub winning_capture_max_gain: ByColour<i32>,
        /// A capture takes a hanging unit.
        pub captures_hanging: ByColour<bool>,
        /// The largest value among the hanging units capturable now.
        pub hanging_victim_max_value: ByColour<i32>,
        /// Captures of a defended unit of equal value.
        pub equal_capture_count: ByColour<u16>,
        /// Captures of a defended unit of lower value.
        pub losing_capture_count: ByColour<u16>,
        /// Moves after which the moved unit forks.
        pub fork_count: ByColour<u16>,
        /// The largest single forked value.
        pub fork_max_value: ByColour<i32>,
        /// A fork by a knight.
        pub knight_fork_available: ByColour<bool>,
        /// A fork one of whose targets is the king.
        pub royal_fork_available: ByColour<bool>,
        /// Moves creating an absolute or a relative pin.
        pub pin_creation_count: ByColour<u16>,
        /// A move creating a skewer.
        pub skewer_creation_available: ByColour<bool>,
        /// A move uncovering a slider's attack on a unit of value 3 or more.
        pub discovered_attack_available: ByColour<bool>,
        /// The side's legal moves.
        pub legal_move_count: ByColour<u16>,
        /// A checking move that captures and whose destination is safe.
        pub safe_check_capturing: ByColour<bool>,
        /// A move uncovering a slider's attack on the enemy queen.
        pub discovered_attack_on_queen: ByColour<bool>,
        /// A move after which a rook or queen of the side checks the enemy king
        /// along that king's own rank, its back rank being blocked.
        pub back_rank_mate_threat: ByColour<bool>,
        /// A quiet move after which the largest gain the side threatens is
        /// greater than it is now.
        pub quiet_threat_available: ByColour<bool>,
        /// No legal move of the side has a safe destination.
        pub no_safe_moves: ByColour<bool>,
        /// A promotion of the side wins material.
        pub promotion_see_positive: ByColour<bool>,
    }
}

impl TacticsFacts {
    /// Whether the side has a checking move.
    #[inline]
    pub fn check_available(&self) -> ByColour<bool> {
        self.check_count.map(|count| count > 0)
    }

    /// Whether it has a checking move with a safe destination.
    #[inline]
    pub fn safe_check_available(&self) -> ByColour<bool> {
        self.safe_check_count.map(|count| count > 0)
    }

    /// Whether it has a promotion.
    #[inline]
    pub fn promotion_available(&self) -> ByColour<bool> {
        self.promotion_files.map(|files| !files.is_empty())
    }

    /// Whether it has a promotion with a safe destination.
    #[inline]
    pub fn safe_promotion_available(&self) -> ByColour<bool> {
        self.safe_promotion_files.map(|files| !files.is_empty())
    }

    /// Whether it has a capture.
    #[inline]
    pub fn capture_available(&self) -> ByColour<bool> {
        self.capture_count.map(|count| count > 0)
    }

    /// Whether it has a forking move.
    #[inline]
    pub fn fork_available(&self) -> ByColour<bool> {
        self.fork_count.map(|count| count > 0)
    }

    /// Whether it has a move creating a pin.
    #[inline]
    pub fn pin_creation_available(&self) -> ByColour<bool> {
        self.pin_creation_count.map(|count| count > 0)
    }

    /// Whether the side has at most two legal moves. False for a side whose
    /// values could not be read.
    #[inline]
    pub fn only_moves(&self) -> ByColour<bool> {
        ByColour::from_fn(|colour| {
            self.available.of(colour) && self.legal_move_count.of(colour) <= 2
        })
    }
}

facts_group! {
    /// What one legal move does, beyond what the move itself says.
    ///
    /// The mover is the side to move; the enemy is the other side.
    derive(Clone, Copy, PartialEq, Eq, Debug);
    pub struct MoveFacts as "move" {
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
        /// The origin is attacked by the enemy and the destination is safe.
        pub escapes_attack: bool,
        /// The destination is attacked by an enemy pawn.
        pub to_attacked_by_pawn: bool,
        /// The move is a castling.
        pub is_castling: bool,
        /// The move is an en-passant capture.
        pub is_en_passant: bool,
        /// The static exchange evaluation of the move, signed.
        pub see: i32,
        /// The largest static exchange evaluation of one of the enemy's units
        /// after the move; 0 when the move threatens nothing.
        pub threat_created_max: i32,
        /// The origin is attacked by the enemy.
        pub moves_attacked_unit: bool,
        /// The destination interposes on the ray of the one unit giving check.
        pub blocks_check: bool,
        /// The moved unit is a passed pawn.
        pub advances_passer: bool,
        /// The mover has more passers after the move than before.
        pub creates_passer: bool,
        /// The mover has more isolated pawns after the move than before.
        pub creates_isolated: bool,
        /// The mover has more doubled pawns after the move than before.
        pub creates_doubled: bool,
        /// The mover has more backward pawns after the move than before.
        pub creates_backward: bool,
        /// One of the enemy king's shield files carried a pawn of the mover's
        /// and carries none after.
        pub opens_file_at_enemy_king: bool,
        /// Change in the number of the mover's units attacking the enemy king
        /// ring.
        pub enemy_king_attackers_delta: i32,
        /// Change in the number of the enemy's units attacking the mover's king
        /// ring.
        pub mover_king_attackers_delta: i32,
        /// Change in the number of the mover's hanging units.
        pub mover_hanging_delta: i32,
        /// Change in the number of the enemy's hanging units.
        pub enemy_hanging_delta: i32,
        /// A square carries a hanging unit of the mover's worth 3 or more after
        /// the move and carried none before.
        pub leaves_unit_hanging: bool,
        /// A slider of the mover's that the move leaves standing gains an
        /// attack on an enemy unit worth 3 or more.
        pub gives_discovered_attack: bool,
    }
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
            see: 0,
            threat_created_max: 0,
            moves_attacked_unit: false,
            blocks_check: false,
            advances_passer: false,
            creates_passer: false,
            creates_isolated: false,
            creates_doubled: false,
            creates_backward: false,
            opens_file_at_enemy_king: false,
            enemy_king_attackers_delta: 0,
            mover_king_attackers_delta: 0,
            mover_hanging_delta: 0,
            enemy_hanging_delta: 0,
            leaves_unit_hanging: false,
            gives_discovered_attack: false,
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

/// Every group of facts about one position, plus its annotated legal moves.
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Facts {
    /// Where the units stand.
    pub placement: PlacementFacts,
    /// Check, castling rights and the en-passant square.
    pub state: StateFacts,
    /// What the plies before this position say.
    pub history: HistoryFacts,
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
    /// Captures by exchange.
    pub exchange: ExchangeFacts,
    /// What each side stands to lose.
    pub threats: ThreatFacts,
    /// One-ply tactics.
    pub tactics: TacticsFacts,
    /// Kings, races and drawn material.
    pub endgame: EndgameFacts,
    /// The square sets read side by side.
    pub maps: MapFacts,
    /// Every legal move, annotated.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub moves: MoveList<AnnotatedMove>,
    #[cfg_attr(feature = "serde", serde(skip))]
    variant: &'static str,
    #[cfg_attr(feature = "serde", serde(skip))]
    us: Colour,
}

/// Every group, in the order [`Facts`] declares them.
pub fn catalogue() -> &'static [Group] {
    const GROUPS: [Group; 14] = [
        PlacementFacts::ENTRY,
        StateFacts::ENTRY,
        HistoryFacts::ENTRY,
        MaterialFacts::ENTRY,
        PawnFacts::ENTRY,
        PieceFacts::ENTRY,
        KingFacts::ENTRY,
        MobilityFacts::ENTRY,
        AttackFacts::ENTRY,
        ExchangeFacts::ENTRY,
        ThreatFacts::ENTRY,
        TacticsFacts::ENTRY,
        EndgameFacts::ENTRY,
        MapFacts::ENTRY,
    ];
    &GROUPS
}

impl Facts {
    /// The variant these facts were computed under.
    #[inline]
    pub fn variant(&self) -> &'static str {
        self.variant
    }

    /// The colour to move.
    #[inline]
    pub fn side_to_move(&self) -> Colour {
        self.us
    }

    /// The names of the groups, in the order they are declared.
    pub fn groups() -> impl Iterator<Item = &'static str> {
        catalogue().iter().map(|group| group.name)
    }

    /// Material, structure, king safety and threats, for a human reader. The
    /// text is not a stable format.
    pub fn summary(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "{} to move, material {} v {}, phase {}/24\n",
            name(self.us),
            self.material.value.white,
            self.material.value.black,
            self.material.phase_points,
        ));
        out.push_str(&format!(
            "pawns: passed {} / {}, islands {} / {}, open files {}\n",
            listed(self.pawns.passed.white),
            listed(self.pawns.passed.black),
            self.pawns.islands.white,
            self.pawns.islands.black,
            files_listed(self.pawns.open_files),
        ));
        out.push_str(&format!(
            "kings: {} / {}, ring holes {} / {}, attack weight {} / {}\n",
            self.king.square.white,
            self.king.square.black,
            self.king.ring_holes.white,
            self.king.ring_holes.black,
            self.king.ring_attack_weight.white,
            self.king.ring_attack_weight.black,
        ));
        out.push_str(&format!(
            "hanging {} / {}, pinned {} / {}\n",
            listed(self.attacks.hanging.white),
            listed(self.attacks.hanging.black),
            listed(self.attacks.pinned.white),
            listed(self.attacks.pinned.black),
        ));
        out.push_str(&format!(
            "{}: {} legal, {} checks, {} captures, {} forks{}\n",
            name(self.us),
            self.tactics.legal_move_count.of(self.us),
            self.tactics.check_count.of(self.us),
            self.tactics.capture_count.of(self.us),
            self.tactics.fork_count.of(self.us),
            if self.tactics.mate_in_1.of(self.us) {
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
    /// The repetition and history values are zero: a position has no history.
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

/// A colour, spelled out.
fn name(colour: Colour) -> &'static str {
    match colour {
        Colour::White => "White",
        Colour::Black => "Black",
    }
}

/// The squares of a set, space separated; an em dash for none.
fn listed(set: SquareSet) -> String {
    if set.is_empty() {
        return "\u{2014}".to_string();
    }
    set.into_iter()
        .map(|square| square.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The same for a set of files.
fn files_listed(files: FileSet) -> String {
    if files.is_empty() {
        return "\u{2014}".to_string();
    }
    files.into_iter().map(File::to_char).collect()
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
    let history = HistoryFacts {
        halfmove_clock: position.halfmove_clock(),
        halfmove_known: position.clocks_known(),
        ..HistoryFacts::default()
    };

    let us = scan.us;
    let mut moves = MoveList::new();
    let mut tactics = TacticsFacts::default();
    let mut exchange = ExchangeFacts::default();
    tactics::tactics(
        variant,
        position,
        &scan,
        us,
        &attacks,
        &pawns,
        &king,
        legal,
        replies,
        Some(&mut moves),
        &mut tactics,
    );
    exchange::exchange_facts(position, legal, us, &mut exchange);
    if let Some(null) = position.null_move() {
        their_moves.clear();
        variant.legal_moves(&null, their_moves);
        tactics::tactics(
            variant,
            &null,
            &scan,
            !us,
            &attacks,
            &pawns,
            &king,
            their_moves,
            replies,
            None,
            &mut tactics,
        );
        exchange::exchange_facts(&null, their_moves, !us, &mut exchange);
    }

    let threats = threats::threat_facts(position, &scan);
    let endgame = endgame::endgame_facts(&scan, &pawns, &pieces);

    Facts {
        placement: placement_facts(&scan),
        state,
        history,
        material,
        pawns,
        pieces,
        king,
        mobility,
        attacks,
        exchange,
        threats,
        tactics,
        endgame,
        maps: MapFacts {
            controlled: attacks.controlled,
            by_pawns: attacks.by_pawns,
            hanging: attacks.hanging,
            pinned: attacks.pinned,
            threatened: threats.threatened,
        },
        moves,
        variant: variant.name(),
        us,
    }
}

fn placement_facts(scan: &Scan) -> PlacementFacts {
    let of = |role: Role| ByColour::from_fn(|colour| scan.role_units.of(colour)[role.index()]);
    PlacementFacts {
        units: scan.units,
        pawns: of(Role::Pawn),
        knights: of(Role::Knight),
        bishops: of(Role::Bishop),
        rooks: of(Role::Rook),
        queens: of(Role::Queen),
        kings: of(Role::King),
    }
}

fn state_facts(position: &Position, _scan: &Scan, legal: &MoveList) -> StateFacts {
    let rights = position.castling_rights();
    StateFacts {
        in_check: position.in_check(),
        double_check: position.checkers().len() >= 2,
        can_castle_short: ByColour::from_fn(|colour| rights.short(colour).is_some()),
        can_castle_long: ByColour::from_fn(|colour| rights.long(colour).is_some()),
        en_passant: position.en_passant().map(|square| square.file()),
        en_passant_capture_legal: legal.iter().any(|mv| mv.is_en_passant()),
    }
}

/// Whether `colour` holds bishops of both square colours.
fn has_bishop_pair(scan: &Scan, colour: Colour) -> bool {
    let bishops = scan.role_units.of(colour)[Role::Bishop.index()];
    !(bishops & SquareSet::LIGHT).is_empty() && !(bishops & SquareSet::DARK).is_empty()
}

fn material_facts(scan: &Scan) -> MaterialFacts {
    let mut facts = MaterialFacts::default();
    let mut phase_points = 0u32;
    for colour in Colour::ALL {
        for role in [
            Role::Pawn,
            Role::Knight,
            Role::Bishop,
            Role::Rook,
            Role::Queen,
        ] {
            let n = scan.role_units.of(colour)[role.index()].len();
            let count = n.min(255) as u8;
            match role {
                Role::Pawn => facts.pawns[colour] = count,
                Role::Knight => facts.knights[colour] = count,
                Role::Bishop => facts.bishops[colour] = count,
                Role::Rook => facts.rooks[colour] = count,
                _ => facts.queens[colour] = count,
            }
            let value = material_value(role) * n as i32;
            facts.value[colour] += value;
            if role != Role::Pawn {
                facts.non_pawn_value[colour] += value;
            }
            phase_points += n * match role {
                Role::Queen => 4,
                Role::Rook => 2,
                Role::Bishop | Role::Knight => 1,
                _ => 0,
            };
        }
        facts.insufficient_material[colour] = insufficient(&scan.role_units.of(colour));
    }
    facts.phase_points = phase_points.min(255) as u8;
    facts.both_queens = facts.queens.white > 0 && facts.queens.black > 0;
    facts.pawns_only = Colour::ALL.iter().all(|&colour| {
        [Role::Knight, Role::Bishop, Role::Rook, Role::Queen]
            .iter()
            .all(|role| scan.role_units.of(colour)[role.index()].is_empty())
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
    let by_role = |role: Role| ByColour::from_fn(|colour| scan.by_role.of(colour)[role.index()]);
    let mut facts = AttackFacts {
        controlled: scan.by,
        by_pawns: by_role(Role::Pawn),
        by_knights: by_role(Role::Knight),
        by_bishops: by_role(Role::Bishop),
        by_rooks: by_role(Role::Rook),
        by_queens: by_role(Role::Queen),
        by_kings: by_role(Role::King),
        ..AttackFacts::default()
    };

    for colour in Colour::ALL {
        let them = !colour;
        facts.defended[colour] = scan.units.of(colour) & scan.by.of(colour);
        for square in scan.units.of(colour) - scan.role_units.of(colour)[Role::King.index()] {
            let role = position
                .piece_at(square)
                .expect("a unit stands on its own square")
                .role;
            let attackers = scan.attackers_of(square, them);
            if attackers.is_empty() {
                continue;
            }
            facts.attacked[colour].insert(square);
            facts.attacked_value[colour] += material_value(role);
            let defended = scan.by.of(colour).contains(square);
            let cheaper = attackers.into_iter().any(|from| {
                let attacker = position
                    .piece_at(from)
                    .expect("an attacker stands on its own square")
                    .role;
                order_value(attacker) < order_value(role)
            });
            if !defended {
                facts.hanging[colour].insert(square);
                facts.hanging_value[colour] += material_value(role);
            }
            if !defended || cheaper {
                facts.en_prise[colour].insert(square);
                facts.en_prise_value[colour] += material_value(role);
                facts.en_prise_max_value[colour] =
                    facts.en_prise_max_value[colour].max(material_value(role));
            }
        }
        facts.pinned[colour] = absolute_pins(scan, colour);
        facts.pinned_value[colour] = facts.pinned[colour]
            .into_iter()
            .map(|square| {
                let role = position
                    .piece_at(square)
                    .expect("a pinned unit stands on its own square")
                    .role;
                material_value(role)
            })
            .sum();
        facts.skewer_candidates[colour] = skewers(position, scan, colour);
    }
    facts
}

/// The units of `colour` that may not legally move because their own king would
/// be exposed.
fn absolute_pins(scan: &Scan, colour: Colour) -> SquareSet {
    let king = scan.kings.of(colour);
    let them = !colour;
    let mut pinned = SquareSet::EMPTY;
    let bishops = scan.role_units.of(them)[Role::Bishop.index()]
        | scan.role_units.of(them)[Role::Queen.index()];
    let rooks = scan.role_units.of(them)[Role::Rook.index()]
        | scan.role_units.of(them)[Role::Queen.index()];
    let candidates = (attacks_of(Role::Bishop, king, colour, SquareSet::EMPTY) & bishops)
        | (attacks_of(Role::Rook, king, colour, SquareSet::EMPTY) & rooks);
    for slider in candidates {
        let blockers = between(slider, king) & scan.occupied;
        if blockers.len() == 1 && blockers.is_subset(scan.units.of(colour)) {
            pinned |= blockers;
        }
    }
    pinned
}

/// How many enemy pairs `colour`'s sliders skewer: an attacked unit with a unit
/// of lower or equal value directly behind it on the same ray.
fn skewers(position: &Position, scan: &Scan, colour: Colour) -> u8 {
    let them = !colour;
    let mut count = 0u8;
    for role in [Role::Bishop, Role::Rook, Role::Queen] {
        for slider in scan.role_units.of(colour)[role.index()] {
            let attacks = scan.attacks_from[slider.index()];
            for front in attacks & scan.units.of(them) {
                let front_role = position
                    .piece_at(front)
                    .expect("a unit stands on its own square")
                    .role;
                let xray = attacks_of(role, slider, colour, scan.occupied - front.to_set());
                let behind = (xray - attacks) & line(slider, front) & scan.units.of(them);
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
    for colour in Colour::ALL {
        let enemy_pawns = scan.by_role.of(!colour)[Role::Pawn.index()];
        for role in [
            Role::Pawn,
            Role::Knight,
            Role::Bishop,
            Role::Rook,
            Role::Queen,
        ] {
            let free = scan.by_role.of(colour)[role.index()] - scan.units.of(colour);
            let all = free.len() as u16;
            let safe = (free - enemy_pawns).len() as u16;
            match role {
                Role::Pawn => (facts.pawns[colour], facts.safe_pawns[colour]) = (all, safe),
                Role::Knight => (facts.knights[colour], facts.safe_knights[colour]) = (all, safe),
                Role::Bishop => (facts.bishops[colour], facts.safe_bishops[colour]) = (all, safe),
                Role::Rook => (facts.rooks[colour], facts.safe_rooks[colour]) = (all, safe),
                _ => (facts.queens[colour], facts.safe_queens[colour]) = (all, safe),
            }
            facts.total[colour] += all;
        }
        facts.space[colour] = (scan.by.of(colour) & scan.own_half(!colour)).len() as u16;
        facts.centre_control[colour] = (scan.by.of(colour) & CENTRE).len() as u8;
        facts.extended_centre_control[colour] = (scan.by.of(colour) & EXTENDED_CENTRE).len() as u8;
        let pieces = scan.units.of(colour)
            - scan.role_units.of(colour)[Role::Pawn.index()]
            - scan.role_units.of(colour)[Role::King.index()];
        facts.immobile_pieces[colour] = pieces
            .into_iter()
            .filter(|square| (scan.attacks_from[square.index()] - scan.units.of(colour)).is_empty())
            .count()
            .min(255) as u8;
    }
    facts
}

/// The mean Chebyshev distance of `colour`'s knights, bishops, rooks and queens
/// to `square`, or 0 when it has none.
fn tropism(scan: &Scan, square: Square, colour: Colour) -> f32 {
    let units = scan.role_units.of(colour);
    let pieces = units[Role::Knight.index()]
        | units[Role::Bishop.index()]
        | units[Role::Rook.index()]
        | units[Role::Queen.index()];
    if pieces.is_empty() {
        return 0.0;
    }
    let total: u32 = pieces.into_iter().map(|from| distance(from, square)).sum();
    total as f32 / pieces.len() as f32
}

/// The three named enums serialise as their snake-case names, the same text
/// the Python surface uses.
#[cfg(feature = "serde")]
mod serde_impls {
    use serde::ser::{Serialize, Serializer};

    use super::{CastledSide, DrawishMaterial, Opposition};

    impl Serialize for CastledSide {
        fn serialize<S: Serializer>(&self, out: S) -> Result<S::Ok, S::Error> {
            out.serialize_str(self.name())
        }
    }

    impl Serialize for Opposition {
        fn serialize<S: Serializer>(&self, out: S) -> Result<S::Ok, S::Error> {
            out.serialize_str(self.name())
        }
    }

    impl Serialize for DrawishMaterial {
        fn serialize<S: Serializer>(&self, out: S) -> Result<S::Ok, S::Error> {
            out.serialize_str(self.name())
        }
    }
}
