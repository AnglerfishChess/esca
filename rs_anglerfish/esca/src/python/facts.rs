//! The facts of a position, as read-only attribute objects.
//!
//! Every side-paired value is a pair, the side to move first: index it with
//! `esca.US` and `esca.THEM`.

use pyo3::prelude::*;

use crate::facts;
use crate::game::Game;
use crate::position::Position;
use crate::types::{File, FileSet, Square, SquareSet};

use super::board::{PyMove, PySquareSet, PyVariant};
use super::convert::{files_text, role_name, side_from, square_name};

fn pair<T: Copy>(values: [T; 2]) -> (T, T) {
    (values[0], values[1])
}

fn list_pair<T: Copy, const N: usize>(values: [[T; N]; 2]) -> (Vec<T>, Vec<T>) {
    (values[0].to_vec(), values[1].to_vec())
}

/// Counts widened out of `u8`, which Python would otherwise read as bytes.
fn count_pair<const N: usize>(values: [[u8; N]; 2]) -> (Vec<u16>, Vec<u16>) {
    let list = |counts: [u8; N]| counts.iter().copied().map(u16::from).collect();
    (list(values[0]), list(values[1]))
}

fn files_pair(values: [FileSet; 2]) -> (String, String) {
    (files_text(values[0]), files_text(values[1]))
}

fn square_pair(values: [Square; 2]) -> (String, String) {
    (square_name(values[0]), square_name(values[1]))
}

fn file_run_pair(values: [[File; 3]; 2]) -> (String, String) {
    let text = |files: [File; 3]| files.iter().map(|f| f.to_char()).collect::<String>();
    (text(values[0]), text(values[1]))
}

fn set_list_pair(values: [[SquareSet; 6]; 2]) -> (Vec<PySquareSet>, Vec<PySquareSet>) {
    let list = |sets: [SquareSet; 6]| sets.into_iter().map(PySquareSet::new).collect();
    (list(values[0]), list(values[1]))
}

/// What a group's `__reduce__` returns: the callable and its arguments.
type GroupReduce<'py> = PyResult<(Bound<'py, PyAny>, (Py<PyFacts>, &'static str))>;

/// The same for a group that is one of a side-paired two.
type SideGroupReduce<'py> = PyResult<(Bound<'py, PyAny>, (Py<PyFacts>, &'static str, isize))>;

/// The callable a group's `__reduce__` names.
fn group_reconstructor(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
    py.import("esca._esca")?.getattr("_facts_group")
}

/// Rebuilds a facts group from the facts it belongs to.
#[pyfunction]
#[pyo3(name = "_facts_group")]
#[pyo3(signature = (facts, name, index = None))]
pub(crate) fn facts_group<'py>(
    facts: &Bound<'py, PyFacts>,
    name: &str,
    index: Option<isize>,
) -> PyResult<Bound<'py, PyAny>> {
    let group = facts.getattr(name)?;
    match index {
        Some(index) => group.get_item(index),
        None => Ok(group),
    }
}

/// Game-state flags: check, castling rights and the en-passant square.
#[pyclass(frozen, module = "esca", name = "StateFacts")]
pub struct PyStateFacts {
    parent: Py<PyFacts>,
    /// The side to move stands in check.
    #[pyo3(get)]
    in_check: bool,
    /// Two or more units give check.
    #[pyo3(get)]
    double_check: bool,
    /// Each side may still castle short.
    #[pyo3(get)]
    castle_short: (bool, bool),
    /// Each side may still castle long.
    #[pyo3(get)]
    castle_long: (bool, bool),
    /// The file the position names as the en-passant target, if any.
    #[pyo3(get)]
    en_passant: Option<String>,
    /// Some legal move captures en passant.
    #[pyo3(get)]
    ep_capture_legal: bool,
}

impl PyStateFacts {
    fn of(facts: &facts::StateFacts, parent: Py<PyFacts>) -> PyStateFacts {
        PyStateFacts {
            parent,
            in_check: facts.in_check,
            double_check: facts.double_check,
            castle_short: pair(facts.castle_short),
            castle_long: pair(facts.castle_long),
            en_passant: facts.en_passant.map(|file| file.to_char().to_string()),
            ep_capture_legal: facts.ep_capture_legal,
        }
    }
}

#[pymethods]
impl PyStateFacts {
    fn __repr__(&self) -> String {
        format!("<StateFacts in_check={}>", self.in_check)
    }

    fn __reduce__<'py>(slf: &Bound<'py, Self>) -> GroupReduce<'py> {
        let py = slf.py();
        Ok((
            group_reconstructor(py)?,
            (slf.get().parent.clone_ref(py), "state"),
        ))
    }
}

/// What the plies before this position say: the clock, repetition, and how
/// forcing the recent play was.
#[pyclass(frozen, module = "esca", name = "HistoryFacts")]
pub struct PyHistoryFacts {
    parent: Py<PyFacts>,
    /// A position history was supplied.
    #[pyo3(get)]
    known: bool,
    /// Plies since the last capture or pawn move.
    #[pyo3(get)]
    halfmove_clock: u32,
    /// The position carried a halfmove clock.
    #[pyo3(get)]
    halfmove_known: bool,
    /// This position occurred before in the game's history.
    #[pyo3(get)]
    repetition_seen: bool,
    /// Some legal move reaches a position of the history.
    #[pyo3(get)]
    repetition_available: bool,
}

impl PyHistoryFacts {
    fn of(facts: &facts::HistoryFacts, parent: Py<PyFacts>) -> PyHistoryFacts {
        PyHistoryFacts {
            parent,
            known: facts.known,
            halfmove_clock: facts.halfmove_clock,
            halfmove_known: facts.halfmove_known,
            repetition_seen: facts.repetition_seen,
            repetition_available: facts.repetition_available,
        }
    }
}

#[pymethods]
impl PyHistoryFacts {
    fn __repr__(&self) -> String {
        format!("<HistoryFacts known={}>", self.known)
    }

    fn __reduce__<'py>(slf: &Bound<'py, Self>) -> GroupReduce<'py> {
        let py = slf.py();
        Ok((
            group_reconstructor(py)?,
            (slf.get().parent.clone_ref(py), "history"),
        ))
    }
}

/// Material and phase.
#[pyclass(frozen, module = "esca", name = "MaterialFacts")]
pub struct PyMaterialFacts {
    parent: Py<PyFacts>,
    /// Unit counts per side, by role P, N, B, R, Q.
    #[pyo3(get)]
    count: (Vec<u16>, Vec<u16>),
    /// Value of N, B, R and Q per side.
    #[pyo3(get)]
    non_pawn_value: (i32, i32),
    /// Value of every unit but the king, per side.
    #[pyo3(get)]
    value: (i32, i32),
    /// How far from the opening the position is, 1.0 for a full set.
    #[pyo3(get)]
    phase: f32,
    /// Both sides have at least one queen.
    #[pyo3(get)]
    both_queens: bool,
    /// Neither side has a unit other than kings and pawns.
    #[pyo3(get)]
    pawns_only: bool,
    /// Each side's own material could never deliver mate.
    #[pyo3(get)]
    insufficient: (bool, bool),
}

impl PyMaterialFacts {
    fn of(facts: &facts::MaterialFacts, parent: Py<PyFacts>) -> PyMaterialFacts {
        PyMaterialFacts {
            parent,
            count: count_pair(facts.count),
            non_pawn_value: pair(facts.non_pawn_value),
            value: pair(facts.value),
            phase: facts.phase,
            both_queens: facts.both_queens,
            pawns_only: facts.pawns_only,
            insufficient: pair(facts.insufficient),
        }
    }
}

#[pymethods]
impl PyMaterialFacts {
    fn __repr__(&self) -> String {
        format!(
            "<MaterialFacts value={:?} phase={:.2}>",
            self.value, self.phase
        )
    }

    fn __reduce__<'py>(slf: &Bound<'py, Self>) -> GroupReduce<'py> {
        let py = slf.py();
        Ok((
            group_reconstructor(py)?,
            (slf.get().parent.clone_ref(py), "material"),
        ))
    }
}

/// Pawn structure.
#[pyclass(frozen, module = "esca", name = "PawnFacts")]
pub struct PyPawnFacts {
    parent: Py<PyFacts>,
    /// Each side's pawns.
    #[pyo3(get)]
    pawns: (PySquareSet, PySquareSet),
    /// Pawns with no enemy pawn ahead on their own or an adjacent file.
    #[pyo3(get)]
    passed: (PySquareSet, PySquareSet),
    /// Candidate passers.
    #[pyo3(get)]
    candidates: (PySquareSet, PySquareSet),
    /// Pawns sharing a file with another pawn of their own colour.
    #[pyo3(get)]
    doubled: (PySquareSet, PySquareSet),
    /// Pawns with no friendly pawn on either adjacent file.
    #[pyo3(get)]
    isolated: (PySquareSet, PySquareSet),
    /// Backward pawns.
    #[pyo3(get)]
    backward: (PySquareSet, PySquareSet),
    /// Pawns defended by a pawn of their own colour.
    #[pyo3(get)]
    defended: (PySquareSet, PySquareSet),
    /// Pawns per file, file a first.
    #[pyo3(get)]
    count_by_file: (Vec<u16>, Vec<u16>),
    /// Pawns per relative rank, rank 1 first.
    #[pyo3(get)]
    count_by_rank: (Vec<u16>, Vec<u16>),
    /// Files carrying no pawn of either colour.
    #[pyo3(get)]
    open_files: String,
    /// Files carrying no pawn of that side and at least one of the other.
    #[pyo3(get)]
    semi_open_files: (String, String),
    /// Maximal runs of adjacent files carrying a pawn, per side.
    #[pyo3(get)]
    islands: (u8, u8),
    /// Pawns that can capture an enemy pawn, per side.
    #[pyo3(get)]
    levers: (u8, u8),
    /// Pawn pairs blocking each other head on.
    #[pyo3(get)]
    rams: u8,
    /// The relative rank, from 1, of each side's most advanced passer.
    #[pyo3(get)]
    passer_lead_rank: (Option<u8>, Option<u8>),
    /// Passers defended by a friendly pawn, per side.
    #[pyo3(get)]
    passer_protected: (u8, u8),
    /// Two passers on adjacent files, per side.
    #[pyo3(get)]
    passers_connected: (bool, bool),
    /// A passer the enemy king cannot catch, per side.
    #[pyo3(get)]
    passer_unstoppable: (bool, bool),
}

impl PyPawnFacts {
    fn of(facts: &facts::PawnFacts, parent: Py<PyFacts>) -> PyPawnFacts {
        PyPawnFacts {
            parent,
            pawns: PySquareSet::pair(facts.pawns),
            passed: PySquareSet::pair(facts.passed),
            candidates: PySquareSet::pair(facts.candidates),
            doubled: PySquareSet::pair(facts.doubled),
            isolated: PySquareSet::pair(facts.isolated),
            backward: PySquareSet::pair(facts.backward),
            defended: PySquareSet::pair(facts.defended),
            count_by_file: count_pair(facts.count_by_file),
            count_by_rank: count_pair(facts.count_by_rank),
            open_files: files_text(facts.open_files),
            semi_open_files: files_pair(facts.semi_open_files),
            islands: pair(facts.islands),
            levers: pair(facts.levers),
            rams: facts.rams,
            passer_lead_rank: pair(facts.passer_lead_rank),
            passer_protected: pair(facts.passer_protected),
            passers_connected: pair(facts.passers_connected),
            passer_unstoppable: pair(facts.passer_unstoppable),
        }
    }
}

#[pymethods]
impl PyPawnFacts {
    fn __repr__(&self) -> String {
        format!("<PawnFacts islands={:?} rams={}>", self.islands, self.rams)
    }

    fn __reduce__<'py>(slf: &Bound<'py, Self>) -> GroupReduce<'py> {
        let py = slf.py();
        Ok((
            group_reconstructor(py)?,
            (slf.get().parent.clone_ref(py), "pawns"),
        ))
    }
}

/// Bishops, rooks, knights and queens.
#[pyclass(frozen, module = "esca", name = "PieceFacts")]
pub struct PyPieceFacts {
    parent: Py<PyFacts>,
    /// Bishops on both square colours, per side.
    #[pyo3(get)]
    bishop_pair: (bool, bool),
    /// Bishops on light squares, per side.
    #[pyo3(get)]
    bishops_light: (u8, u8),
    /// Bishops on dark squares, per side.
    #[pyo3(get)]
    bishops_dark: (u8, u8),
    /// Exactly one bishop each, on different square colours.
    #[pyo3(get)]
    opposite_coloured_bishops: bool,
    /// Own pawns standing on a square colour of an own bishop, per side.
    #[pyo3(get)]
    pawns_on_bishop_colour: (u8, u8),
    /// Two rooks share a rank with nothing between, per side.
    #[pyo3(get)]
    rooks_connected_rank: (bool, bool),
    /// Two rooks share a file with nothing between, per side.
    #[pyo3(get)]
    rooks_connected_file: (bool, bool),
    /// Rooks on an open file, per side.
    #[pyo3(get)]
    rooks_on_open_file: (u8, u8),
    /// Rooks on a file semi-open for their own side, per side.
    #[pyo3(get)]
    rooks_on_semi_open_file: (u8, u8),
    /// Rooks on their own relative rank 7, per side.
    #[pyo3(get)]
    rooks_on_relative_7th: (u8, u8),
    /// Rooks behind a passer of their own side, per side.
    #[pyo3(get)]
    rook_behind_own_passer: (u8, u8),
    /// Rooks behind an enemy passer, per side.
    #[pyo3(get)]
    rook_behind_enemy_passer: (u8, u8),
    /// A trapped rook, per side.
    #[pyo3(get)]
    trapped_rook: (bool, bool),
    /// Outpost squares, per side.
    #[pyo3(get)]
    outposts: (PySquareSet, PySquareSet),
    /// Knights and bishops standing on an own outpost square, per side.
    #[pyo3(get)]
    minors_on_outpost: (u8, u8),
    /// Unoccupied outpost squares, per side.
    #[pyo3(get)]
    outpost_squares_free: (u8, u8),
    /// Knights on file a or h, or on relative rank 1 or 8, per side.
    #[pyo3(get)]
    knights_on_rim: (u8, u8),
    /// Knights and bishops still on their starting squares, per side.
    #[pyo3(get)]
    minors_undeveloped: (u8, u8),
    /// A queen stands off its starting square, per side.
    #[pyo3(get)]
    queen_developed: (bool, bool),
}

impl PyPieceFacts {
    fn of(facts: &facts::PieceFacts, parent: Py<PyFacts>) -> PyPieceFacts {
        PyPieceFacts {
            parent,
            bishop_pair: pair(facts.bishop_pair),
            bishops_light: pair(facts.bishops_light),
            bishops_dark: pair(facts.bishops_dark),
            opposite_coloured_bishops: facts.opposite_coloured_bishops,
            pawns_on_bishop_colour: pair(facts.pawns_on_bishop_colour),
            rooks_connected_rank: pair(facts.rooks_connected_rank),
            rooks_connected_file: pair(facts.rooks_connected_file),
            rooks_on_open_file: pair(facts.rooks_on_open_file),
            rooks_on_semi_open_file: pair(facts.rooks_on_semi_open_file),
            rooks_on_relative_7th: pair(facts.rooks_on_relative_7th),
            rook_behind_own_passer: pair(facts.rook_behind_own_passer),
            rook_behind_enemy_passer: pair(facts.rook_behind_enemy_passer),
            trapped_rook: pair(facts.trapped_rook),
            outposts: PySquareSet::pair(facts.outposts),
            minors_on_outpost: pair(facts.minors_on_outpost),
            outpost_squares_free: pair(facts.outpost_squares_free),
            knights_on_rim: pair(facts.knights_on_rim),
            minors_undeveloped: pair(facts.minors_undeveloped),
            queen_developed: pair(facts.queen_developed),
        }
    }
}

#[pymethods]
impl PyPieceFacts {
    fn __repr__(&self) -> String {
        format!("<PieceFacts bishop_pair={:?}>", self.bishop_pair)
    }

    fn __reduce__<'py>(slf: &Bound<'py, Self>) -> GroupReduce<'py> {
        let py = slf.py();
        Ok((
            group_reconstructor(py)?,
            (slf.get().parent.clone_ref(py), "pieces"),
        ))
    }
}

/// King safety and shelter.
#[pyclass(frozen, module = "esca", name = "KingFacts")]
pub struct PyKingFacts {
    parent: Py<PyFacts>,
    /// Where each king stands.
    #[pyo3(get)]
    square: (String, String),
    /// The king stands on its own starting square, per side.
    #[pyo3(get)]
    on_home_square: (bool, bool),
    /// The king stands on files a to c, per side.
    #[pyo3(get)]
    castled_queenside: (bool, bool),
    /// The king stands on files f to h, per side.
    #[pyo3(get)]
    castled_kingside: (bool, bool),
    /// The three files a king's shelter is read on, in ascending order.
    #[pyo3(get)]
    shield_files: (String, String),
    /// Ranks to the nearest friendly pawn ahead of the king, per shield file.
    #[pyo3(get)]
    shield: (Vec<Option<u8>>, Vec<Option<u8>>),
    /// Each shield file carries no pawn of either colour.
    #[pyo3(get)]
    file_open: (Vec<bool>, Vec<bool>),
    /// Each shield file is semi-open for the enemy of that king.
    #[pyo3(get)]
    file_semi_open_for_enemy: (Vec<bool>, Vec<bool>),
    /// Ranks to the nearest enemy pawn ahead of the king, per shield file.
    #[pyo3(get)]
    storm: (Vec<Option<u8>>, Vec<Option<u8>>),
    /// The squares adjacent to each king.
    #[pyo3(get)]
    ring: (PySquareSet, PySquareSet),
    /// Enemy knights, bishops, rooks and queens attacking the ring, per side.
    #[pyo3(get)]
    ring_attackers: (u8, u8),
    /// Sum over those attackers of N, B = 1, R = 2, Q = 4, per side.
    #[pyo3(get)]
    ring_attack_weight: (u8, u8),
    /// Ring squares attacked by the king's own side, per side.
    #[pyo3(get)]
    ring_defended: (u8, u8),
    /// Ring squares attacked by the enemy and not defended, per side.
    #[pyo3(get)]
    ring_holes: (u8, u8),
    /// Adjacent squares empty or capturable and not attacked, per side.
    #[pyo3(get)]
    escape_squares: (u8, u8),
    /// King on its relative rank 1 with every forward-adjacent square held by
    /// a friendly unit, per side.
    #[pyo3(get)]
    back_rank_risk: (bool, bool),
    /// Chebyshev distance between the kings.
    #[pyo3(get)]
    distance: u8,
    /// Mean Chebyshev distance of enemy pieces to this king, per side.
    #[pyo3(get)]
    tropism: (f32, f32),
    /// Squares a queen on this king's square would attack, per side.
    #[pyo3(get)]
    virtual_mobility: (u8, u8),
}

impl PyKingFacts {
    fn of(facts: &facts::KingFacts, parent: Py<PyFacts>) -> PyKingFacts {
        PyKingFacts {
            parent,
            square: square_pair(facts.square),
            on_home_square: pair(facts.on_home_square),
            castled_queenside: pair(facts.castled_queenside),
            castled_kingside: pair(facts.castled_kingside),
            shield_files: file_run_pair(facts.shield_files),
            shield: list_pair(facts.shield),
            file_open: list_pair(facts.file_open),
            file_semi_open_for_enemy: list_pair(facts.file_semi_open_for_enemy),
            storm: list_pair(facts.storm),
            ring: PySquareSet::pair(facts.ring),
            ring_attackers: pair(facts.ring_attackers),
            ring_attack_weight: pair(facts.ring_attack_weight),
            ring_defended: pair(facts.ring_defended),
            ring_holes: pair(facts.ring_holes),
            escape_squares: pair(facts.escape_squares),
            back_rank_risk: pair(facts.back_rank_risk),
            distance: facts.distance,
            tropism: pair(facts.tropism),
            virtual_mobility: pair(facts.virtual_mobility),
        }
    }
}

#[pymethods]
impl PyKingFacts {
    fn __repr__(&self) -> String {
        format!("<KingFacts square={:?}>", self.square)
    }

    fn __reduce__<'py>(slf: &Bound<'py, Self>) -> GroupReduce<'py> {
        let py = slf.py();
        Ok((
            group_reconstructor(py)?,
            (slf.get().parent.clone_ref(py), "king"),
        ))
    }
}

/// Mobility and space.
#[pyclass(frozen, module = "esca", name = "MobilityFacts")]
pub struct PyMobilityFacts {
    parent: Py<PyFacts>,
    /// Attacked squares not held by own units, per side, by role P, N, B, R, Q.
    #[pyo3(get)]
    by_role: (Vec<u16>, Vec<u16>),
    /// The same, minus squares attacked by an enemy pawn.
    #[pyo3(get)]
    safe_by_role: (Vec<u16>, Vec<u16>),
    /// Sum of `by_role`, per side.
    #[pyo3(get)]
    total: (u16, u16),
    /// Attacked squares in the opponent's half, per side.
    #[pyo3(get)]
    space: (u16, u16),
    /// Attacked squares, per side.
    #[pyo3(get)]
    controlled: (u16, u16),
    /// Attacks on d4, e4, d5 and e5, per side.
    #[pyo3(get)]
    centre_control: (u8, u8),
    /// Attacks on c3 to f6, per side.
    #[pyo3(get)]
    extended_centre_control: (u8, u8),
    /// Units other than pawns and kings with no destination, per side.
    #[pyo3(get)]
    immobile_pieces: (u8, u8),
}

impl PyMobilityFacts {
    fn of(facts: &facts::MobilityFacts, parent: Py<PyFacts>) -> PyMobilityFacts {
        PyMobilityFacts {
            parent,
            by_role: list_pair(facts.by_role),
            safe_by_role: list_pair(facts.safe_by_role),
            total: pair(facts.total),
            space: pair(facts.space),
            controlled: pair(facts.controlled),
            centre_control: pair(facts.centre_control),
            extended_centre_control: pair(facts.extended_centre_control),
            immobile_pieces: pair(facts.immobile_pieces),
        }
    }
}

#[pymethods]
impl PyMobilityFacts {
    fn __repr__(&self) -> String {
        format!("<MobilityFacts total={:?}>", self.total)
    }

    fn __reduce__<'py>(slf: &Bound<'py, Self>) -> GroupReduce<'py> {
        let py = slf.py();
        Ok((
            group_reconstructor(py)?,
            (slf.get().parent.clone_ref(py), "mobility"),
        ))
    }
}

/// The attack maps and what they say about the units on the board.
#[pyclass(frozen, module = "esca", name = "AttackFacts")]
pub struct PyAttackFacts {
    parent: Py<PyFacts>,
    inner: facts::AttackFacts,
    /// Each side's whole attack map.
    #[pyo3(get)]
    by: (PySquareSet, PySquareSet),
    /// Each side's pawn attacks.
    #[pyo3(get)]
    by_pawns: (PySquareSet, PySquareSet),
    /// Each side's attack map by role P, N, B, R, Q, K.
    #[pyo3(get)]
    by_role: (Vec<PySquareSet>, Vec<PySquareSet>),
    /// Units the opponent attacks, defended or not; never a king.
    #[pyo3(get)]
    attacked: (PySquareSet, PySquareSet),
    /// Units attacked by the opponent and not defended; never a king.
    #[pyo3(get)]
    hanging: (PySquareSet, PySquareSet),
    /// Units hanging or attacked by a cheaper enemy unit; never a king.
    #[pyo3(get)]
    en_prise: (PySquareSet, PySquareSet),
    /// Units under an absolute pin.
    #[pyo3(get)]
    pinned: (PySquareSet, PySquareSet),
    /// Units standing on a square their own side attacks.
    #[pyo3(get)]
    defended: (PySquareSet, PySquareSet),
    /// Value sum of the attacked units, per side.
    #[pyo3(get)]
    attacked_value: (i32, i32),
    /// Value sum of the hanging units, per side.
    #[pyo3(get)]
    hanging_value: (i32, i32),
    /// Value sum of the units en prise, per side.
    #[pyo3(get)]
    en_prise_value: (i32, i32),
    /// Largest value en prise, per side.
    #[pyo3(get)]
    en_prise_max_value: (i32, i32),
    /// Value sum of the absolutely pinned units, per side.
    #[pyo3(get)]
    pinned_value: (i32, i32),
    /// Enemy unit pairs this side's sliders skewer, per side.
    #[pyo3(get)]
    skewer_candidates: (u8, u8),
}

impl PyAttackFacts {
    fn of(facts: &facts::AttackFacts, parent: Py<PyFacts>) -> PyAttackFacts {
        PyAttackFacts {
            parent,
            inner: *facts,
            by: PySquareSet::pair(facts.by),
            by_pawns: PySquareSet::pair(facts.by_pawns),
            by_role: set_list_pair(facts.by_role),
            attacked: PySquareSet::pair(facts.attacked),
            hanging: PySquareSet::pair(facts.hanging),
            en_prise: PySquareSet::pair(facts.en_prise),
            pinned: PySquareSet::pair(facts.pinned),
            defended: PySquareSet::pair(facts.defended),
            attacked_value: pair(facts.attacked_value),
            hanging_value: pair(facts.hanging_value),
            en_prise_value: pair(facts.en_prise_value),
            en_prise_max_value: pair(facts.en_prise_max_value),
            pinned_value: pair(facts.pinned_value),
            skewer_candidates: pair(facts.skewer_candidates),
        }
    }
}

#[pymethods]
impl PyAttackFacts {
    /// The units of `side` that attack `square`.
    fn attackers_of(&self, square: &str, side: isize) -> PyResult<PySquareSet> {
        let square = super::convert::square_from(square)?;
        Ok(PySquareSet::new(
            self.inner.attackers_of(square, side_from(side)?),
        ))
    }

    /// Whether the unit on `square`, of either colour, is hanging.
    fn is_hanging(&self, square: &str) -> PyResult<bool> {
        Ok(self.inner.is_hanging(super::convert::square_from(square)?))
    }

    /// The units of `side`.
    fn units(&self, side: isize) -> PyResult<PySquareSet> {
        Ok(PySquareSet::new(self.inner.units(side_from(side)?)))
    }

    fn __repr__(&self) -> String {
        format!("<AttackFacts hanging_value={:?}>", self.hanging_value)
    }

    fn __reduce__<'py>(slf: &Bound<'py, Self>) -> GroupReduce<'py> {
        let py = slf.py();
        Ok((
            group_reconstructor(py)?,
            (slf.get().parent.clone_ref(py), "attacks"),
        ))
    }
}

/// One side's one-ply tactical options.
#[pyclass(frozen, module = "esca", name = "TacticsFacts")]
pub struct PyTacticsFacts {
    parent: Py<PyFacts>,
    side: isize,
    /// The block could be computed.
    #[pyo3(get)]
    available: bool,
    /// Moves giving check.
    #[pyo3(get)]
    check_count: u16,
    /// A checking move exists per moving role P, N, B, R, Q.
    #[pyo3(get)]
    check_by_role: Vec<bool>,
    /// Checking moves whose destination is safe.
    #[pyo3(get)]
    safe_check_count: u16,
    /// A safe checking move exists per moving role P, N, B, R, Q.
    #[pyo3(get)]
    safe_check_by_role: Vec<bool>,
    /// A move gives check from two units at once.
    #[pyo3(get)]
    double_check_available: bool,
    /// A move gives check with a unit that did not move.
    #[pyo3(get)]
    discovered_check_available: bool,
    /// A move leaves the opponent checkmated.
    #[pyo3(get)]
    mate_in_1: bool,
    /// A move leaves the opponent stalemated.
    #[pyo3(get)]
    stalemate_in_1: bool,
    /// Files a legal move promotes on.
    #[pyo3(get)]
    promotion_files: String,
    /// A promotion to each of Q, R, B, N is available.
    #[pyo3(get)]
    promotion_roles: Vec<bool>,
    /// Files a legal promotion with a safe destination lands on.
    #[pyo3(get)]
    safe_promotion_files: String,
    /// Capturing moves.
    #[pyo3(get)]
    capture_count: u16,
    /// A capture whose victim outvalues the capturer or is undefended.
    #[pyo3(get)]
    winning_capture_available: bool,
    /// The largest victim minus capturer over the captures, at least 0.
    #[pyo3(get)]
    winning_capture_max_gain: i32,
    /// A capture of a hanging unit.
    #[pyo3(get)]
    captures_hanging: bool,
    /// The largest value among the hanging units capturable now.
    #[pyo3(get)]
    hanging_victim_max_value: i32,
    /// Captures of a defended unit of equal value.
    #[pyo3(get)]
    equal_capture_count: u16,
    /// Captures of a defended unit of lower value.
    #[pyo3(get)]
    losing_capture_count: u16,
    /// Moves after which the moved unit forks.
    #[pyo3(get)]
    fork_count: u16,
    /// The largest single forked value.
    #[pyo3(get)]
    fork_max_value: i32,
    /// A fork by a knight.
    #[pyo3(get)]
    knight_fork_available: bool,
    /// A fork one of whose targets is the king.
    #[pyo3(get)]
    royal_fork_available: bool,
    /// Moves creating an absolute or a relative pin.
    #[pyo3(get)]
    pin_creation_count: u16,
    /// A move creating a skewer.
    #[pyo3(get)]
    skewer_creation_available: bool,
    /// A move uncovering a slider's attack on a unit of value 3 or more.
    #[pyo3(get)]
    discovered_attack_available: bool,
    /// Legal moves.
    #[pyo3(get)]
    legal_move_count: u16,
}

impl PyTacticsFacts {
    fn of(facts: &facts::TacticsFacts, parent: Py<PyFacts>, side: isize) -> PyTacticsFacts {
        PyTacticsFacts {
            parent,
            side,
            available: facts.available,
            check_count: facts.check_count,
            check_by_role: facts.check_by_role.to_vec(),
            safe_check_count: facts.safe_check_count,
            safe_check_by_role: facts.safe_check_by_role.to_vec(),
            double_check_available: facts.double_check_available,
            discovered_check_available: facts.discovered_check_available,
            mate_in_1: facts.mate_in_1,
            stalemate_in_1: facts.stalemate_in_1,
            promotion_files: files_text(facts.promotion_files),
            promotion_roles: facts.promotion_roles.to_vec(),
            safe_promotion_files: files_text(facts.safe_promotion_files),
            capture_count: facts.capture_count,
            winning_capture_available: facts.winning_capture_available,
            winning_capture_max_gain: facts.winning_capture_max_gain,
            captures_hanging: facts.captures_hanging,
            hanging_victim_max_value: facts.hanging_victim_max_value,
            equal_capture_count: facts.equal_capture_count,
            losing_capture_count: facts.losing_capture_count,
            fork_count: facts.fork_count,
            fork_max_value: facts.fork_max_value,
            knight_fork_available: facts.knight_fork_available,
            royal_fork_available: facts.royal_fork_available,
            pin_creation_count: facts.pin_creation_count,
            skewer_creation_available: facts.skewer_creation_available,
            discovered_attack_available: facts.discovered_attack_available,
            legal_move_count: facts.legal_move_count,
        }
    }
}

#[pymethods]
impl PyTacticsFacts {
    /// Whether a checking move exists.
    #[getter]
    fn check_available(&self) -> bool {
        self.check_count > 0
    }

    /// Whether a checking move with a safe destination exists.
    #[getter]
    fn safe_check_available(&self) -> bool {
        self.safe_check_count > 0
    }

    /// Whether a promotion is available.
    #[getter]
    fn promotion_available(&self) -> bool {
        !self.promotion_files.is_empty()
    }

    /// Whether a promotion with a safe destination is available.
    #[getter]
    fn safe_promotion_available(&self) -> bool {
        !self.safe_promotion_files.is_empty()
    }

    /// Whether a capture is available.
    #[getter]
    fn capture_available(&self) -> bool {
        self.capture_count > 0
    }

    /// Whether a forking move is available.
    #[getter]
    fn fork_available(&self) -> bool {
        self.fork_count > 0
    }

    /// Whether a move creating a pin is available.
    #[getter]
    fn pin_creation_available(&self) -> bool {
        self.pin_creation_count > 0
    }

    /// Whether the side has at most two legal moves. False for a block that
    /// could not be computed, which has no side to move.
    #[getter]
    fn only_moves(&self) -> bool {
        self.available && self.legal_move_count <= 2
    }

    fn __repr__(&self) -> String {
        format!("<TacticsFacts legal_move_count={}>", self.legal_move_count)
    }

    fn __reduce__<'py>(slf: &Bound<'py, Self>) -> SideGroupReduce<'py> {
        let py = slf.py();
        Ok((
            group_reconstructor(py)?,
            (slf.get().parent.clone_ref(py), "tactics", slf.get().side),
        ))
    }
}

/// The eight square sets the `planes` group emits.
#[pyclass(frozen, module = "esca", name = "PlaneFacts")]
pub struct PyPlaneFacts {
    parent: Py<PyFacts>,
    /// Each side's attack map.
    #[pyo3(get)]
    attacked: (PySquareSet, PySquareSet),
    /// Each side's pawn attacks.
    #[pyo3(get)]
    attacked_by_pawns: (PySquareSet, PySquareSet),
    /// Each side's hanging units.
    #[pyo3(get)]
    hanging: (PySquareSet, PySquareSet),
    /// Each side's absolutely pinned units.
    #[pyo3(get)]
    pinned: (PySquareSet, PySquareSet),
}

impl PyPlaneFacts {
    fn of(facts: &facts::PlaneFacts, parent: Py<PyFacts>) -> PyPlaneFacts {
        PyPlaneFacts {
            parent,
            attacked: PySquareSet::pair(facts.attacked),
            attacked_by_pawns: PySquareSet::pair(facts.attacked_by_pawns),
            hanging: PySquareSet::pair(facts.hanging),
            pinned: PySquareSet::pair(facts.pinned),
        }
    }
}

#[pymethods]
impl PyPlaneFacts {
    fn __repr__(&self) -> String {
        "<PlaneFacts>".to_string()
    }

    fn __reduce__<'py>(slf: &Bound<'py, Self>) -> GroupReduce<'py> {
        let py = slf.py();
        Ok((
            group_reconstructor(py)?,
            (slf.get().parent.clone_ref(py), "planes"),
        ))
    }
}

/// What one legal move does, beyond what the move itself says.
#[pyclass(frozen, skip_from_py_object, module = "esca", name = "MoveFacts")]
#[derive(Clone)]
pub struct PyMoveFacts {
    /// The role captured, the pawn for an en-passant capture.
    #[pyo3(get)]
    victim: Option<String>,
    /// The role that moves.
    #[pyo3(get)]
    mover: String,
    /// The role a promoting pawn becomes.
    #[pyo3(get)]
    promotion: Option<String>,
    /// The move gives check.
    #[pyo3(get)]
    gives_check: bool,
    /// The move gives check and its destination is safe.
    #[pyo3(get)]
    gives_safe_check: bool,
    /// The destination is a safe destination.
    #[pyo3(get)]
    is_safe: bool,
    /// The move captures a hanging unit.
    #[pyo3(get)]
    captures_hanging: bool,
    /// The origin is attacked by the opponent and the destination is safe.
    #[pyo3(get)]
    escapes_attack: bool,
    /// The destination is attacked by an enemy pawn.
    #[pyo3(get)]
    to_attacked_by_pawn: bool,
    /// The move is a castling.
    #[pyo3(get)]
    is_castling: bool,
    /// The move is an en-passant capture.
    #[pyo3(get)]
    is_en_passant: bool,
}

impl PyMoveFacts {
    fn of(facts: &facts::MoveFacts) -> PyMoveFacts {
        PyMoveFacts {
            victim: facts.victim.map(role_name),
            mover: role_name(facts.mover),
            promotion: facts.promotion.map(role_name),
            gives_check: facts.gives_check,
            gives_safe_check: facts.gives_safe_check,
            is_safe: facts.is_safe,
            captures_hanging: facts.captures_hanging,
            escapes_attack: facts.escapes_attack,
            to_attacked_by_pawn: facts.to_attacked_by_pawn,
            is_castling: facts.is_castling,
            is_en_passant: facts.is_en_passant,
        }
    }
}

#[pymethods]
impl PyMoveFacts {
    fn __repr__(&self) -> String {
        format!("<MoveFacts mover={}>", self.mover)
    }
}

/// A legal move and its facts.
#[pyclass(frozen, module = "esca", name = "AnnotatedMove")]
pub struct PyAnnotatedMove {
    inner: facts::AnnotatedMove,
}

impl PyAnnotatedMove {
    pub(crate) fn new(inner: facts::AnnotatedMove) -> PyAnnotatedMove {
        PyAnnotatedMove { inner }
    }
}

#[pymethods]
impl PyAnnotatedMove {
    /// The move.
    #[getter]
    #[pyo3(name = "move")]
    fn get_move(&self) -> PyMove {
        PyMove::new(self.inner.mv)
    }

    /// What it does.
    #[getter]
    fn facts(&self) -> PyMoveFacts {
        PyMoveFacts::of(&self.inner.facts)
    }

    fn __repr__(&self) -> String {
        format!("<AnnotatedMove {}>", self.inner.mv)
    }
}

/// Everything the v1 schema says about one position, plus its annotated legal
/// moves.
///
/// Pickling re-reads the facts from the position they were computed for, so
/// the history flags a game supplies do not survive a round trip.
#[pyclass(frozen, module = "esca", name = "Facts")]
pub struct PyFacts {
    pub(crate) inner: facts::Facts,
    text: String,
    variant: PyVariant,
}

impl PyFacts {
    pub(crate) fn of_position(position: &Position, variant: PyVariant) -> PyFacts {
        let inner = position.facts(variant.rules());
        PyFacts {
            inner,
            text: position.fen(),
            variant,
        }
    }

    pub(crate) fn of_game(game: &Game, variant: PyVariant) -> PyFacts {
        PyFacts {
            inner: game.facts(),
            text: game.position().fen(),
            variant,
        }
    }
}

#[pymethods]
impl PyFacts {
    #[new]
    #[pyo3(signature = (fen, *, variant = None))]
    fn py_new(fen: &str, variant: Option<PyVariant>) -> PyResult<PyFacts> {
        let variant = variant.unwrap_or_else(super::default_variant);
        let position = Position::from_fen(fen).map_err(super::convert::value_error)?;
        Ok(PyFacts::of_position(&position, variant))
    }

    /// The variant the facts were computed under.
    #[getter]
    fn variant(&self) -> PyVariant {
        self.variant.clone()
    }

    /// The colour that plays `esca.US`.
    #[getter]
    fn side_to_move(&self) -> String {
        super::convert::colour_name(self.inner.side_to_move())
    }

    /// The side the colour `"w"` or `"b"` plays: `esca.US` or `esca.THEM`.
    fn side(&self, colour: &str) -> PyResult<usize> {
        Ok(self
            .inner
            .side(super::convert::colour_from(colour)?)
            .index())
    }

    /// The position the facts were computed for.
    #[getter]
    fn position(&self) -> PyResult<super::board::PyPosition> {
        Position::from_fen(&self.text)
            .map(super::board::PyPosition::new)
            .map_err(super::convert::value_error)
    }

    #[getter]
    fn state(slf: &Bound<'_, Self>) -> PyStateFacts {
        PyStateFacts::of(&slf.get().inner.state, slf.clone().unbind())
    }

    #[getter]
    fn history(slf: &Bound<'_, Self>) -> PyHistoryFacts {
        PyHistoryFacts::of(&slf.get().inner.history, slf.clone().unbind())
    }

    #[getter]
    fn material(slf: &Bound<'_, Self>) -> PyMaterialFacts {
        PyMaterialFacts::of(&slf.get().inner.material, slf.clone().unbind())
    }

    #[getter]
    fn pawns(slf: &Bound<'_, Self>) -> PyPawnFacts {
        PyPawnFacts::of(&slf.get().inner.pawns, slf.clone().unbind())
    }

    #[getter]
    fn pieces(slf: &Bound<'_, Self>) -> PyPieceFacts {
        PyPieceFacts::of(&slf.get().inner.pieces, slf.clone().unbind())
    }

    #[getter]
    fn king(slf: &Bound<'_, Self>) -> PyKingFacts {
        PyKingFacts::of(&slf.get().inner.king, slf.clone().unbind())
    }

    #[getter]
    fn mobility(slf: &Bound<'_, Self>) -> PyMobilityFacts {
        PyMobilityFacts::of(&slf.get().inner.mobility, slf.clone().unbind())
    }

    #[getter]
    fn attacks(slf: &Bound<'_, Self>) -> PyAttackFacts {
        PyAttackFacts::of(&slf.get().inner.attacks, slf.clone().unbind())
    }

    /// One-ply tactics, ours then theirs.
    #[getter]
    fn tactics(slf: &Bound<'_, Self>) -> (PyTacticsFacts, PyTacticsFacts) {
        let facts = &slf.get().inner.tactics;
        (
            PyTacticsFacts::of(&facts[0], slf.clone().unbind(), 0),
            PyTacticsFacts::of(&facts[1], slf.clone().unbind(), 1),
        )
    }

    #[getter]
    fn planes(slf: &Bound<'_, Self>) -> PyPlaneFacts {
        PyPlaneFacts::of(&slf.get().inner.planes, slf.clone().unbind())
    }

    /// Every legal move, annotated.
    #[getter]
    fn moves(&self) -> Vec<PyAnnotatedMove> {
        self.inner
            .moves
            .iter()
            .copied()
            .map(PyAnnotatedMove::new)
            .collect()
    }

    /// Material, structure, king safety and threats, for a human reader. Not a
    /// stable format.
    fn summary(&self) -> String {
        self.inner.summary()
    }

    fn __repr__(&self) -> String {
        format!("<Facts {} {}>", self.variant.rules().name(), self.text)
    }

    fn __getnewargs_ex__(&self) -> ((String,), std::collections::HashMap<String, PyVariant>) {
        let mut kwargs = std::collections::HashMap::new();
        kwargs.insert("variant".to_string(), self.variant.clone());
        ((self.text.clone(),), kwargs)
    }
}
