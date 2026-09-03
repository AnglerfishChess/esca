//! Writing facts out as the `f32` rows a net consumes.

use core::fmt;

use crate::error::FenError;
use crate::position::Position;
use crate::schema::{GroupSet, Schema};
use crate::types::{Colour, FileSet, Role, Square, SquareSet};
use crate::variant::Variant;

use super::pawns::files_of;
use super::{ExchangeFacts, Facts, MoveFacts, Scratch, Side, TacticsFacts};

/// A cursor over the values of one group.
struct Writer<'a> {
    out: &'a mut [f32],
    at: usize,
}

impl<'a> Writer<'a> {
    fn new(out: &'a mut [f32]) -> Writer<'a> {
        Writer { out, at: 0 }
    }

    fn value(&mut self, value: f32) {
        self.out[self.at] = value;
        self.at += 1;
    }

    fn bit(&mut self, set: bool) {
        self.value(if set { 1.0 } else { 0.0 });
    }

    /// `min(n, scale) / scale`.
    fn count(&mut self, n: f32, scale: f32) {
        self.value(n.min(scale).max(0.0) / scale);
    }

    /// `clamp(d / scale, −1, 1)`.
    fn diff(&mut self, d: f32, scale: f32) {
        self.value((d / scale).clamp(-1.0, 1.0));
    }

    /// One 1.0 at `index`, or all zeros.
    fn one_hot(&mut self, index: Option<usize>, width: usize) {
        for slot in 0..width {
            self.bit(index == Some(slot));
        }
    }

    fn file_mask(&mut self, files: FileSet) {
        for index in 0..8 {
            self.bit(files.bits() & (1 << index) != 0);
        }
    }

    fn plane(&mut self, set: SquareSet, us: Colour) {
        for square in Square::ALL {
            let source = Square::new(square.file(), square.rank().relative_to(us));
            self.bit(set.contains(source));
        }
    }
}

/// The bucket a halfmove clock falls in: 0 / 1–3 / 4–9 / 10–19 / 20–39 /
/// 40–69 / 70–89 / 90 and above.
fn halfmove_bucket(clock: u32) -> usize {
    match clock {
        0 => 0,
        1..=3 => 1,
        4..=9 => 2,
        10..=19 => 3,
        20..=39 => 4,
        40..=69 => 5,
        70..=89 => 6,
        _ => 7,
    }
}

fn placement(facts: &Facts, w: &mut Writer) {
    let us = facts.us;
    for side in Side::ALL {
        for role in Role::ALL {
            w.plane(facts.placement.of(side, role), us);
        }
    }
}

fn state(facts: &Facts, w: &mut Writer) {
    let state = &facts.state;
    w.bit(state.in_check);
    w.bit(state.double_check);
    w.bit(state.castle_short[0]);
    w.bit(state.castle_long[0]);
    w.bit(state.castle_short[1]);
    w.bit(state.castle_long[1]);
    w.bit(state.en_passant.is_some());
    w.one_hot(state.en_passant.map(|file| file.index()), 8);
    w.bit(state.ep_capture_legal);
}

fn history(facts: &Facts, w: &mut Writer) {
    let history = &facts.history;
    w.one_hot(Some(halfmove_bucket(history.halfmove_clock)), 8);
    w.bit(history.halfmove_known);
    w.bit(history.repetition_seen);
    w.bit(history.repetition_available);
    w.bit(history.known);
}

fn material(facts: &Facts, w: &mut Writer) {
    let m = &facts.material;
    for side in Side::ALL {
        for role in 0..5 {
            let scale = if role == 0 { 8.0 } else { 4.0 };
            w.count(m.count[side.index()][role] as f32, scale);
        }
    }
    for role in 0..5 {
        w.diff(m.count[0][role] as f32 - m.count[1][role] as f32, 4.0);
    }
    for side in Side::ALL {
        w.count(m.non_pawn_value[side.index()] as f32, 62.0);
    }
    w.diff((m.value[0] - m.value[1]) as f32, 20.0);
    w.value(m.phase);
    let bucket = if m.phase > 0.75 {
        0
    } else if m.phase >= 0.25 {
        1
    } else {
        2
    };
    w.one_hot(Some(bucket), 3);
    w.bit(m.both_queens);
    w.bit(m.pawns_only);
    w.bit(m.insufficient[0]);
    w.bit(m.insufficient[1]);
}

fn pawns(facts: &Facts, w: &mut Writer) {
    let p = &facts.pawns;
    for side in Side::ALL {
        for file in 0..8 {
            w.count(p.count_by_file[side.index()][file] as f32, 3.0);
        }
    }
    for side in Side::ALL {
        for rank in 0..8 {
            w.count(p.count_by_rank[side.index()][rank] as f32, 8.0);
        }
    }
    for set in [
        &p.doubled,
        &p.isolated,
        &p.backward,
        &p.passed,
        &p.candidates,
    ] {
        for side in Side::ALL {
            w.file_mask(files_of(set[side.index()]));
        }
    }
    for side in Side::ALL {
        w.one_hot(
            p.passer_lead_rank[side.index()].map(|rank| rank as usize - 1),
            8,
        );
    }
    for side in Side::ALL {
        w.count(p.passer_protected[side.index()] as f32, 4.0);
    }
    for side in Side::ALL {
        w.bit(p.passers_connected[side.index()]);
    }
    for side in Side::ALL {
        w.bit(p.passer_unstoppable[side.index()]);
    }
    w.file_mask(p.open_files);
    w.file_mask(p.semi_open_files[0]);
    w.file_mask(p.semi_open_files[1]);
    for side in Side::ALL {
        w.count(p.islands[side.index()] as f32, 4.0);
    }
    for side in Side::ALL {
        w.count(p.defended[side.index()].len() as f32, 8.0);
    }
    for side in Side::ALL {
        w.count(p.levers[side.index()] as f32, 4.0);
    }
    w.count(p.rams as f32, 8.0);
}

fn pieces(facts: &Facts, w: &mut Writer) {
    let p = &facts.pieces;
    for side in Side::ALL {
        w.bit(p.bishop_pair[side.index()]);
    }
    for side in Side::ALL {
        w.count(p.bishops_light[side.index()] as f32, 2.0);
        w.count(p.bishops_dark[side.index()] as f32, 2.0);
    }
    w.bit(p.opposite_coloured_bishops);
    for side in Side::ALL {
        w.count(p.pawns_on_bishop_colour[side.index()] as f32, 8.0);
    }
    for side in Side::ALL {
        w.bit(p.rooks_connected_rank[side.index()]);
    }
    for side in Side::ALL {
        w.bit(p.rooks_connected_file[side.index()]);
    }
    for counts in [
        &p.rooks_on_open_file,
        &p.rooks_on_semi_open_file,
        &p.rooks_on_relative_7th,
        &p.rook_behind_own_passer,
        &p.rook_behind_enemy_passer,
    ] {
        for side in Side::ALL {
            w.count(counts[side.index()] as f32, 2.0);
        }
    }
    for side in Side::ALL {
        w.bit(p.trapped_rook[side.index()]);
    }
    for side in Side::ALL {
        w.count(p.minors_on_outpost[side.index()] as f32, 2.0);
    }
    for side in Side::ALL {
        w.count(p.outpost_squares_free[side.index()] as f32, 4.0);
    }
    for side in Side::ALL {
        w.count(p.knights_on_rim[side.index()] as f32, 2.0);
    }
    for side in Side::ALL {
        w.count(p.minors_undeveloped[side.index()] as f32, 4.0);
    }
    for side in Side::ALL {
        w.bit(p.queen_developed[side.index()]);
    }
}

fn king(facts: &Facts, w: &mut Writer) {
    let k = &facts.king;
    let us = facts.us;
    for side in Side::ALL {
        w.one_hot(Some(k.square[side.index()].file().index()), 8);
    }
    for side in Side::ALL {
        let colour = if side == Side::Us { us } else { !us };
        let rank = k.square[side.index()].rank().relative_to(colour).index();
        w.one_hot(Some(rank), 8);
    }
    for side in Side::ALL {
        w.bit(k.on_home_square[side.index()]);
    }
    for side in Side::ALL {
        w.bit(k.castled_queenside[side.index()]);
        w.bit(k.castled_kingside[side.index()]);
    }
    for side in Side::ALL {
        for slot in 0..3 {
            w.one_hot(Some(shelter_bucket(k.shield[side.index()][slot])), 4);
        }
    }
    for side in Side::ALL {
        for slot in 0..3 {
            w.bit(k.file_open[side.index()][slot]);
            w.bit(k.file_semi_open_for_enemy[side.index()][slot]);
        }
    }
    for side in Side::ALL {
        for slot in 0..3 {
            w.one_hot(Some(storm_bucket(k.storm[side.index()][slot])), 4);
        }
    }
    for side in Side::ALL {
        w.count(k.ring_attackers[side.index()] as f32, 6.0);
    }
    for side in Side::ALL {
        w.count(k.ring_attack_weight[side.index()] as f32, 16.0);
    }
    for side in Side::ALL {
        w.count(k.ring_defended[side.index()] as f32, 8.0);
    }
    for side in Side::ALL {
        w.count(k.ring_holes[side.index()] as f32, 8.0);
    }
    for side in Side::ALL {
        w.count(k.escape_squares[side.index()] as f32, 8.0);
    }
    for side in Side::ALL {
        w.bit(k.back_rank_risk[side.index()]);
    }
    // Two kings stand 2 to 7 squares apart; the one-hot has no slot for the
    // distances a legal position cannot show.
    w.one_hot(
        (2..=7)
            .contains(&k.distance)
            .then(|| k.distance as usize - 2),
        6,
    );
    for side in Side::ALL {
        w.count(k.tropism[side.index()], 8.0);
    }
    for side in Side::ALL {
        w.count(k.virtual_mobility[side.index()] as f32, 27.0);
    }
}

/// Which of "one rank ahead / two / three or more / none" a shield distance is.
fn shelter_bucket(distance: Option<u8>) -> usize {
    match distance {
        Some(1) => 0,
        Some(2) => 1,
        Some(_) => 2,
        None => 3,
    }
}

/// Which of "two or nearer / three / four / five or further or none" a storm
/// distance is.
fn storm_bucket(distance: Option<u8>) -> usize {
    match distance {
        Some(d) if d <= 2 => 0,
        Some(3) => 1,
        Some(4) => 2,
        _ => 3,
    }
}

fn mobility(facts: &Facts, w: &mut Writer) {
    let m = &facts.mobility;
    let total = m.total[0] as f32 + m.total[1] as f32;
    w.value(if total == 0.0 {
        0.0
    } else {
        m.total[0] as f32 / total
    });
    for side in Side::ALL {
        for role in 0..5 {
            w.count(m.by_role[side.index()][role] as f32, 16.0);
        }
    }
    for side in Side::ALL {
        for role in 0..5 {
            w.count(m.safe_by_role[side.index()][role] as f32, 16.0);
        }
    }
    for role in 0..5 {
        w.diff(m.by_role[0][role] as f32 - m.by_role[1][role] as f32, 16.0);
    }
    for side in Side::ALL {
        w.count(m.space[side.index()] as f32, 32.0);
    }
    w.count(m.controlled[0] as f32, 48.0);
    w.count(m.controlled[1] as f32, 48.0);
    w.diff(m.controlled[0] as f32 - m.controlled[1] as f32, 48.0);
    for side in Side::ALL {
        w.count(m.centre_control[side.index()] as f32, 4.0);
    }
    for side in Side::ALL {
        w.count(m.extended_centre_control[side.index()] as f32, 16.0);
    }
    for side in Side::ALL {
        w.count(m.immobile_pieces[side.index()] as f32, 4.0);
    }
    for side in Side::ALL {
        w.count(m.total[side.index()] as f32, 96.0);
    }
}

fn attacks(facts: &Facts, w: &mut Writer) {
    let a = &facts.attacks;
    w.count(a.by[0].len() as f32, 48.0);
    w.count(a.by[1].len() as f32, 48.0);
    w.diff(a.by[0].len() as f32 - a.by[1].len() as f32, 48.0);
    for side in Side::ALL {
        w.count(a.attacked[side.index()].len() as f32, 8.0);
    }
    for side in Side::ALL {
        w.count(a.attacked_value[side.index()] as f32, 20.0);
    }
    for side in Side::ALL {
        w.count(a.hanging[side.index()].len() as f32, 4.0);
    }
    for side in Side::ALL {
        w.count(a.hanging_value[side.index()] as f32, 20.0);
    }
    for side in Side::ALL {
        w.count(a.en_prise[side.index()].len() as f32, 4.0);
    }
    for side in Side::ALL {
        w.count(a.en_prise_value[side.index()] as f32, 20.0);
    }
    for side in Side::ALL {
        w.count(a.en_prise_max_value[side.index()] as f32, 9.0);
    }
    for side in Side::ALL {
        w.count(a.pinned[side.index()].len() as f32, 4.0);
    }
    for side in Side::ALL {
        w.count(a.pinned_value[side.index()] as f32, 20.0);
    }
    for side in Side::ALL {
        w.count(a.skewer_candidates[side.index()] as f32, 4.0);
    }
    for side in Side::ALL {
        w.count(a.defended[side.index()].len() as f32, 16.0);
    }
}

fn exchange_block(e: &ExchangeFacts, w: &mut Writer) {
    w.diff(e.see_best_capture as f32, 9.0);
    w.count(e.see_positive_capture_count as f32, 8.0);
    w.count(e.see_equal_capture_count as f32, 8.0);
    w.count(e.see_positive_total as f32, 20.0);
}

fn tactics_block(t: &TacticsFacts, w: &mut Writer) {
    w.bit(t.check_available());
    w.count(t.check_count as f32, 8.0);
    for role in 0..5 {
        w.bit(t.check_by_role[role]);
    }
    w.bit(t.safe_check_available());
    w.count(t.safe_check_count as f32, 8.0);
    for role in 0..5 {
        w.bit(t.safe_check_by_role[role]);
    }
    w.bit(t.double_check_available);
    w.bit(t.discovered_check_available);
    w.bit(t.mate_in_1);
    w.bit(t.stalemate_in_1);
    w.bit(t.promotion_available());
    w.file_mask(t.promotion_files);
    for role in 0..4 {
        w.bit(t.promotion_roles[role]);
    }
    w.bit(t.safe_promotion_available());
    w.file_mask(t.safe_promotion_files);
    w.bit(t.capture_available());
    w.count(t.capture_count as f32, 16.0);
    w.bit(t.winning_capture_available);
    w.count(t.winning_capture_max_gain as f32, 9.0);
    w.bit(t.captures_hanging);
    w.count(t.hanging_victim_max_value as f32, 9.0);
    w.count(t.equal_capture_count as f32, 8.0);
    w.count(t.losing_capture_count as f32, 8.0);
    w.bit(t.fork_available());
    w.count(t.fork_count as f32, 4.0);
    w.count(t.fork_max_value as f32, 9.0);
    w.bit(t.knight_fork_available);
    w.bit(t.royal_fork_available);
    w.bit(t.pin_creation_available());
    w.count(t.pin_creation_count as f32, 4.0);
    w.bit(t.skewer_creation_available);
    w.bit(t.discovered_attack_available);
    w.count(t.legal_move_count as f32, 64.0);
    w.bit(t.only_moves());
    w.bit(t.available);
}

fn planes(facts: &Facts, w: &mut Writer) {
    let p = &facts.planes;
    let us = facts.us;
    for set in [
        p.attacked[0],
        p.attacked[1],
        p.attacked_by_pawns[0],
        p.attacked_by_pawns[1],
        p.hanging[0],
        p.hanging[1],
        p.pinned[0],
        p.pinned[1],
    ] {
        w.plane(set, us);
    }
}

impl Facts {
    /// Writes the selected groups, in schema order, into `out`; returns how
    /// many values were written. A feature not defined for these facts'
    /// variant is written as zeros, so widths and offsets do not depend on the
    /// variant.
    ///
    /// # Panics
    /// If `out` is shorter than `schema.width_of(groups)`.
    pub fn encode_into(&self, schema: &Schema, groups: GroupSet, out: &mut [f32]) -> usize {
        let width = schema.width_of(groups);
        assert!(out.len() >= width, "the output is shorter than the schema");
        let mut at = 0;
        for (index, group) in schema.groups().iter().enumerate() {
            if !groups.contains(index) {
                continue;
            }
            let slice = &mut out[at..at + group.width];
            {
                let mut writer = Writer::new(slice);
                match group.name {
                    // Groups with no features yet: their width is zero, so
                    // nothing is written and no offset moves.
                    "threats" | "endgame" => {}
                    "exchange" => {
                        exchange_block(&self.exchange[0], &mut writer);
                        exchange_block(&self.exchange[1], &mut writer);
                    }
                    "placement" => placement(self, &mut writer),
                    "state" => state(self, &mut writer),
                    "history" => history(self, &mut writer),
                    "material" => material(self, &mut writer),
                    "pawns" => pawns(self, &mut writer),
                    "pieces" => pieces(self, &mut writer),
                    "king" => king(self, &mut writer),
                    "mobility" => mobility(self, &mut writer),
                    "attacks" => attacks(self, &mut writer),
                    "tactics" => {
                        tactics_block(&self.tactics[0], &mut writer);
                        tactics_block(&self.tactics[1], &mut writer);
                    }
                    "planes" => planes(self, &mut writer),
                    other => unreachable!("unknown group {other}"),
                }
                debug_assert_eq!(writer.at, group.width, "group {}", group.name);
            }
            for feature in group.features {
                if !feature.defined_for(self.variant) {
                    slice[feature.offset..feature.offset + feature.width].fill(0.0);
                }
            }
            at += group.width;
        }
        at
    }

    /// The selected groups as a fresh vector.
    pub fn encode(&self, schema: &Schema, groups: GroupSet) -> Vec<f32> {
        let mut out = vec![0.0; schema.width_of(groups)];
        self.encode_into(schema, groups, &mut out);
        out
    }
}

impl MoveFacts {
    /// How many values one move occupies.
    pub const WIDTH: usize = 24;

    /// Writes the move's values into `out`.
    ///
    /// # Panics
    /// If `out` is shorter than [`MoveFacts::WIDTH`].
    pub fn encode_into(&self, out: &mut [f32]) {
        assert!(
            out.len() >= MoveFacts::WIDTH,
            "the output is shorter than a move"
        );
        let mut w = Writer::new(&mut out[..MoveFacts::WIDTH]);
        w.bit(self.victim.is_some());
        w.one_hot(self.victim.map(Role::index), 5);
        w.one_hot(Some(self.mover.index()), 6);
        w.one_hot(self.promotion.map(promotion_slot), 4);
        w.bit(self.gives_check);
        w.bit(self.gives_safe_check);
        w.bit(self.is_safe);
        w.bit(self.captures_hanging);
        w.bit(self.escapes_attack);
        w.bit(self.to_attacked_by_pawn);
        w.bit(self.is_castling);
        w.bit(self.is_en_passant);
        debug_assert_eq!(w.at, MoveFacts::WIDTH);
    }
}

/// The slot a promotion role takes in the Q, R, B, N order.
fn promotion_slot(role: Role) -> usize {
    match role {
        Role::Queen => 0,
        Role::Rook => 1,
        Role::Bishop => 2,
        _ => 3,
    }
}

/// Which row of a batch a failure belongs to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RowError {
    /// The row index, from 0.
    pub row: usize,
    /// Why the row could not be read.
    pub source: FenError,
}

impl fmt::Display for RowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "row {}: {}", self.row, self.source)
    }
}

impl std::error::Error for RowError {}

/// Writes one row per position, row-major, into `out`.
///
/// # Panics
/// If `out` is shorter than `positions.len() * schema.width_of(groups)`.
pub fn encode_positions(
    variant: &dyn Variant,
    positions: &[Position],
    schema: &Schema,
    groups: GroupSet,
    out: &mut [f32],
) {
    let width = schema.width_of(groups);
    assert!(
        out.len() >= positions.len() * width,
        "the output is shorter than the batch"
    );
    let mut scratch = Scratch::new();
    for (row, position) in positions.iter().enumerate() {
        let facts = position.facts_in(variant, &mut scratch);
        facts.encode_into(schema, groups, &mut out[row * width..(row + 1) * width]);
    }
}

/// As [`encode_positions`], reading each row's position from FEN text.
///
/// # Panics
/// If `out` is shorter than `fens.len() * schema.width_of(groups)`.
pub fn encode_fens(
    variant: &dyn Variant,
    fens: &[&str],
    schema: &Schema,
    groups: GroupSet,
    out: &mut [f32],
) -> Result<(), RowError> {
    let width = schema.width_of(groups);
    assert!(
        out.len() >= fens.len() * width,
        "the output is shorter than the batch"
    );
    let mut scratch = Scratch::new();
    for (row, fen) in fens.iter().enumerate() {
        let position = Position::from_fen(fen).map_err(|source| RowError { row, source })?;
        let facts = position.facts_in(variant, &mut scratch);
        facts.encode_into(schema, groups, &mut out[row * width..(row + 1) * width]);
    }
    Ok(())
}
