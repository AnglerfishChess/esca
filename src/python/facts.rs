//! The facts of a position, as read-only attribute objects.
//!
//! Every value that differs between the two sides is a [`PyByColour`]: read it
//! as `.white`, `.black` or `.of(colour)`, and a fixed run of values is a
//! tuple in the order its type or its meaning states. `to_dict()` writes a
//! group as the crate's own JSON, parsed back, so the two surfaces cannot
//! answer differently.

use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::facts::{self, CastledSide, DrawishMaterial, Opposition};
use crate::game::Game;
use crate::position::Position;

use super::board::{PyMove, PySquareSet, PyVariant};
use super::convert::{colour_name, file_name, role_name, square_name};
use super::values::{Fixed, PyByColour, PyFileSet, hash_of, plain, repr_of};

/// A run of eight counts, widened out of `u8`, which Python would otherwise
/// read as bytes.
fn counts<I: IntoIterator<Item = u8>>(values: I) -> Fixed<u16> {
    Fixed(values.into_iter().map(u16::from).collect())
}

/// What a group's `__reduce__` returns: the callable and its arguments.
type GroupReduce<'py> = PyResult<(Bound<'py, PyAny>, (Py<PyFacts>, &'static str))>;

/// The callable a group's `__reduce__` names.
fn group_reconstructor(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
    py.import("esca._esca")?.getattr("_facts_group")
}

/// Rebuilds a facts group from the facts it belongs to.
#[pyfunction]
#[pyo3(name = "_facts_group")]
pub(crate) fn facts_group<'py>(
    facts: &Bound<'py, PyFacts>,
    name: &str,
) -> PyResult<Bound<'py, PyAny>> {
    facts.getattr(name)
}

/// Where the units stand.
#[pyclass(frozen, module = "esca", name = "PlacementFacts")]
pub struct PyPlacementFacts {
    parent: Py<PyFacts>,
    /// Every unit of the side.
    #[pyo3(get)]
    units: Py<PyByColour>,
    /// The side's pawns.
    #[pyo3(get)]
    pawns: Py<PyByColour>,
    /// The side's knights.
    #[pyo3(get)]
    knights: Py<PyByColour>,
    /// The side's bishops.
    #[pyo3(get)]
    bishops: Py<PyByColour>,
    /// The side's rooks.
    #[pyo3(get)]
    rooks: Py<PyByColour>,
    /// The side's queens.
    #[pyo3(get)]
    queens: Py<PyByColour>,
    /// The side's king.
    #[pyo3(get)]
    kings: Py<PyByColour>,
}

impl PyPlacementFacts {
    fn of(
        py: Python<'_>,
        facts: &facts::PlacementFacts,
        parent: Py<PyFacts>,
    ) -> PyResult<PyPlacementFacts> {
        Ok(PyPlacementFacts {
            parent,
            units: Py::new(py, PyByColour::pair(py, facts.units.map(PySquareSet::new))?)?,
            pawns: Py::new(py, PyByColour::pair(py, facts.pawns.map(PySquareSet::new))?)?,
            knights: Py::new(
                py,
                PyByColour::pair(py, facts.knights.map(PySquareSet::new))?,
            )?,
            bishops: Py::new(
                py,
                PyByColour::pair(py, facts.bishops.map(PySquareSet::new))?,
            )?,
            rooks: Py::new(py, PyByColour::pair(py, facts.rooks.map(PySquareSet::new))?)?,
            queens: Py::new(
                py,
                PyByColour::pair(py, facts.queens.map(PySquareSet::new))?,
            )?,
            kings: Py::new(py, PyByColour::pair(py, facts.kings.map(PySquareSet::new))?)?,
        })
    }
}

#[pymethods]
impl PyPlacementFacts {
    /// The group as plain data, in the one JSON form `docs/features.md`
    /// describes.
    fn to_dict<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        plain(slf.py(), &slf.get().parent.get().inner.placement)
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        repr_of(slf.as_any(), "PlacementFacts", FIELDS_PLACEMENT)
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        match other.cast::<Self>() {
            Ok(other) => {
                self.parent.get().inner.placement == other.get().parent.get().inner.placement
            }
            Err(_) => false,
        }
    }

    fn __hash__(&self) -> u64 {
        hash_of(&self.parent.get().inner.placement)
    }

    fn __reduce__<'py>(slf: &Bound<'py, Self>) -> GroupReduce<'py> {
        let py = slf.py();
        Ok((
            group_reconstructor(py)?,
            (slf.get().parent.clone_ref(py), "placement"),
        ))
    }
}

/// Check, castling rights and the en-passant square.
#[pyclass(frozen, module = "esca", name = "StateFacts")]
pub struct PyStateFacts {
    parent: Py<PyFacts>,
    /// The side to move stands in check.
    #[pyo3(get)]
    in_check: bool,
    /// Two or more units give check to the side to move.
    #[pyo3(get)]
    double_check: bool,
    /// The side may still castle short.
    #[pyo3(get)]
    can_castle_short: Py<PyByColour>,
    /// The side may still castle long.
    #[pyo3(get)]
    can_castle_long: Py<PyByColour>,
    /// The file the position names as the en-passant target, if any.
    #[pyo3(get)]
    en_passant: Option<String>,
    /// Some legal move captures en passant.
    #[pyo3(get)]
    en_passant_capture_legal: bool,
}

impl PyStateFacts {
    fn of(
        py: Python<'_>,
        facts: &facts::StateFacts,
        parent: Py<PyFacts>,
    ) -> PyResult<PyStateFacts> {
        Ok(PyStateFacts {
            parent,
            in_check: facts.in_check,
            double_check: facts.double_check,
            can_castle_short: Py::new(py, PyByColour::pair(py, facts.can_castle_short)?)?,
            can_castle_long: Py::new(py, PyByColour::pair(py, facts.can_castle_long)?)?,
            en_passant: facts.en_passant.map(file_name),
            en_passant_capture_legal: facts.en_passant_capture_legal,
        })
    }
}

#[pymethods]
impl PyStateFacts {
    /// The group as plain data, in the one JSON form `docs/features.md`
    /// describes.
    fn to_dict<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        plain(slf.py(), &slf.get().parent.get().inner.state)
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        repr_of(slf.as_any(), "StateFacts", FIELDS_STATE)
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        match other.cast::<Self>() {
            Ok(other) => self.parent.get().inner.state == other.get().parent.get().inner.state,
            Err(_) => false,
        }
    }

    fn __hash__(&self) -> u64 {
        hash_of(&self.parent.get().inner.state)
    }

    fn __reduce__<'py>(slf: &Bound<'py, Self>) -> GroupReduce<'py> {
        let py = slf.py();
        Ok((
            group_reconstructor(py)?,
            (slf.get().parent.clone_ref(py), "state"),
        ))
    }
}

/// What the plies before this position say: the clock, repetition, and how forcing the recent play was. Everything but the halfmove clock, which the position itself carries, is zero unless `known` is true.
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
    /// This position occurred before in the supplied history.
    #[pyo3(get)]
    repetition_seen: bool,
    /// Some legal move reaches a position of the supplied history.
    #[pyo3(get)]
    repetition_available: bool,
    /// Captures among the last eight plies of the supplied history.
    #[pyo3(get)]
    captures_in_last_8: u8,
    /// Plies among the last eight that gave check.
    #[pyo3(get)]
    checks_in_last_8: u8,
    /// Plies since the last one that captured or gave check; the whole history when it holds none.
    #[pyo3(get)]
    quiet_plies: u32,
    /// The material balance now, less the balance eight plies ago, White less Black.
    #[pyo3(get)]
    material_trend: i32,
    /// The role the last move captured, the pawn for an en-passant capture.
    #[pyo3(get)]
    last_move_victim: Option<String>,
    /// The role that made the last move; for castling, the king.
    #[pyo3(get)]
    last_move_mover: Option<String>,
}

impl PyHistoryFacts {
    fn of(
        _py: Python<'_>,
        facts: &facts::HistoryFacts,
        parent: Py<PyFacts>,
    ) -> PyResult<PyHistoryFacts> {
        Ok(PyHistoryFacts {
            parent,
            known: facts.known,
            halfmove_clock: facts.halfmove_clock,
            halfmove_known: facts.halfmove_known,
            repetition_seen: facts.repetition_seen,
            repetition_available: facts.repetition_available,
            captures_in_last_8: facts.captures_in_last_8,
            checks_in_last_8: facts.checks_in_last_8,
            quiet_plies: facts.quiet_plies,
            material_trend: facts.material_trend,
            last_move_victim: facts.last_move_victim.map(role_name),
            last_move_mover: facts.last_move_mover.map(role_name),
        })
    }
}

#[pymethods]
impl PyHistoryFacts {
    /// The group as plain data, in the one JSON form `docs/features.md`
    /// describes.
    fn to_dict<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        plain(slf.py(), &slf.get().parent.get().inner.history)
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        repr_of(slf.as_any(), "HistoryFacts", FIELDS_HISTORY)
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        match other.cast::<Self>() {
            Ok(other) => self.parent.get().inner.history == other.get().parent.get().inner.history,
            Err(_) => false,
        }
    }

    fn __hash__(&self) -> u64 {
        hash_of(&self.parent.get().inner.history)
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
    /// The side's pawns.
    #[pyo3(get)]
    pawns: Py<PyByColour>,
    /// The side's knights.
    #[pyo3(get)]
    knights: Py<PyByColour>,
    /// The side's bishops.
    #[pyo3(get)]
    bishops: Py<PyByColour>,
    /// The side's rooks.
    #[pyo3(get)]
    rooks: Py<PyByColour>,
    /// The side's queens.
    #[pyo3(get)]
    queens: Py<PyByColour>,
    /// The value of the side's knights, bishops, rooks and queens.
    #[pyo3(get)]
    non_pawn_value: Py<PyByColour>,
    /// The value of every unit of the side but the king.
    #[pyo3(get)]
    value: Py<PyByColour>,
    /// `4·Q + 2·R + B + N` over both sides: 24 at the full opening array, 0 with nothing but kings and pawns left.
    #[pyo3(get)]
    phase_points: u8,
    /// Both sides have at least one queen.
    #[pyo3(get)]
    both_queens: bool,
    /// Neither side has a unit other than kings and pawns.
    #[pyo3(get)]
    pawns_only: bool,
    /// The side's own material could never deliver mate.
    #[pyo3(get)]
    insufficient_material: Py<PyByColour>,
}

impl PyMaterialFacts {
    fn of(
        py: Python<'_>,
        facts: &facts::MaterialFacts,
        parent: Py<PyFacts>,
    ) -> PyResult<PyMaterialFacts> {
        Ok(PyMaterialFacts {
            parent,
            pawns: Py::new(py, PyByColour::pair(py, facts.pawns)?)?,
            knights: Py::new(py, PyByColour::pair(py, facts.knights)?)?,
            bishops: Py::new(py, PyByColour::pair(py, facts.bishops)?)?,
            rooks: Py::new(py, PyByColour::pair(py, facts.rooks)?)?,
            queens: Py::new(py, PyByColour::pair(py, facts.queens)?)?,
            non_pawn_value: Py::new(py, PyByColour::pair(py, facts.non_pawn_value)?)?,
            value: Py::new(py, PyByColour::pair(py, facts.value)?)?,
            phase_points: facts.phase_points,
            both_queens: facts.both_queens,
            pawns_only: facts.pawns_only,
            insufficient_material: Py::new(py, PyByColour::pair(py, facts.insufficient_material)?)?,
        })
    }
}

#[pymethods]
impl PyMaterialFacts {
    /// The group as plain data, in the one JSON form `docs/features.md`
    /// describes.
    fn to_dict<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        plain(slf.py(), &slf.get().parent.get().inner.material)
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        repr_of(slf.as_any(), "MaterialFacts", FIELDS_MATERIAL)
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        match other.cast::<Self>() {
            Ok(other) => {
                self.parent.get().inner.material == other.get().parent.get().inner.material
            }
            Err(_) => false,
        }
    }

    fn __hash__(&self) -> u64 {
        hash_of(&self.parent.get().inner.material)
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
    /// The side's pawns.
    #[pyo3(get)]
    pawns: Py<PyByColour>,
    /// The side's pawns with no enemy pawn ahead on their own or an adjacent file.
    #[pyo3(get)]
    passed: Py<PyByColour>,
    /// The side's candidate passers: pawns on a file the enemy has left whose own side leads the pawn count on that wing's neighbouring files.
    #[pyo3(get)]
    candidates: Py<PyByColour>,
    /// The side's pawns sharing a file with another pawn of their own colour.
    #[pyo3(get)]
    doubled: Py<PyByColour>,
    /// The side's pawns with no friendly pawn on either adjacent file.
    #[pyo3(get)]
    isolated: Py<PyByColour>,
    /// The side's pawns that cannot advance without being taken and cannot be defended by a pawn.
    #[pyo3(get)]
    backward: Py<PyByColour>,
    /// The side's pawns defended by a pawn of their own colour.
    #[pyo3(get)]
    defended: Py<PyByColour>,
    /// The side's pawns per file, file a first.
    #[pyo3(get)]
    count_by_file: Py<PyByColour>,
    /// The side's pawns per rank counted from its own back rank, rank 1 first.
    #[pyo3(get)]
    count_by_rank: Py<PyByColour>,
    /// Files carrying no pawn of either colour.
    #[pyo3(get)]
    open_files: PyFileSet,
    /// Files carrying no pawn of the side and at least one of the other.
    #[pyo3(get)]
    semi_open_files: Py<PyByColour>,
    /// The side's maximal runs of adjacent files carrying a pawn.
    #[pyo3(get)]
    islands: Py<PyByColour>,
    /// The side's pawns that can capture an enemy pawn.
    #[pyo3(get)]
    levers: Py<PyByColour>,
    /// Pawn pairs blocking each other head on.
    #[pyo3(get)]
    rams: u8,
    /// The rank, counted from the side's own back rank, of its most advanced passer.
    #[pyo3(get)]
    passer_lead_rank: Py<PyByColour>,
    /// The side's passers defended by a friendly pawn.
    #[pyo3(get)]
    passer_protected: Py<PyByColour>,
    /// The side has two passers on adjacent files.
    #[pyo3(get)]
    passers_connected: Py<PyByColour>,
    /// The side has a passer the enemy king cannot catch.
    #[pyo3(get)]
    passer_unstoppable: Py<PyByColour>,
    /// The side's longest run of pawns each defending the next.
    #[pyo3(get)]
    chain_max_length: Py<PyByColour>,
    /// An enemy unit attacks the base of one of the side's chains of two or more.
    #[pyo3(get)]
    chain_base_attacked: Py<PyByColour>,
    /// The side has more pawns than the enemy on files a to d.
    #[pyo3(get)]
    queenside_majority: Py<PyByColour>,
    /// The side has more pawns than the enemy on files e to h.
    #[pyo3(get)]
    kingside_majority: Py<PyByColour>,
    /// Squares on the side's ranks 3 to 6 that no pawn of the side can ever attack.
    #[pyo3(get)]
    holes: Py<PyByColour>,
    /// Enemy knights and bishops standing on those squares.
    #[pyo3(get)]
    holes_occupied: Py<PyByColour>,
    /// The side's pawns whose stop square holds a unit of either colour.
    #[pyo3(get)]
    fixed_pawns: Py<PyByColour>,
    /// The side's passers whose stop square holds an enemy unit.
    #[pyo3(get)]
    blocked_passers: Py<PyByColour>,
    /// The pushes the side's lead passer still needs.
    #[pyo3(get)]
    passer_distance: Py<PyByColour>,
    /// The distance from the side's own king to its lead passer's promotion square.
    #[pyo3(get)]
    passer_own_king_distance: Py<PyByColour>,
    /// The distance from the enemy king to that same square.
    #[pyo3(get)]
    passer_enemy_king_distance: Py<PyByColour>,
    /// The enemy king stands in the square of the side's lead passer.
    #[pyo3(get)]
    passer_in_square: Py<PyByColour>,
    /// The side's passers whose whole front span is empty.
    #[pyo3(get)]
    passer_free_path: Py<PyByColour>,
    /// Files semi-open for the side among the enemy king's three shield files.
    #[pyo3(get)]
    half_open_at_enemy_king: Py<PyByColour>,
    /// The side's backward pawns on a file semi-open for the enemy.
    #[pyo3(get)]
    backward_on_semi_open: Py<PyByColour>,
}

impl PyPawnFacts {
    fn of(py: Python<'_>, facts: &facts::PawnFacts, parent: Py<PyFacts>) -> PyResult<PyPawnFacts> {
        Ok(PyPawnFacts {
            parent,
            pawns: Py::new(py, PyByColour::pair(py, facts.pawns.map(PySquareSet::new))?)?,
            passed: Py::new(
                py,
                PyByColour::pair(py, facts.passed.map(PySquareSet::new))?,
            )?,
            candidates: Py::new(
                py,
                PyByColour::pair(py, facts.candidates.map(PySquareSet::new))?,
            )?,
            doubled: Py::new(
                py,
                PyByColour::pair(py, facts.doubled.map(PySquareSet::new))?,
            )?,
            isolated: Py::new(
                py,
                PyByColour::pair(py, facts.isolated.map(PySquareSet::new))?,
            )?,
            backward: Py::new(
                py,
                PyByColour::pair(py, facts.backward.map(PySquareSet::new))?,
            )?,
            defended: Py::new(
                py,
                PyByColour::pair(py, facts.defended.map(PySquareSet::new))?,
            )?,
            count_by_file: Py::new(py, PyByColour::pair(py, facts.count_by_file.map(counts))?)?,
            count_by_rank: Py::new(py, PyByColour::pair(py, facts.count_by_rank.map(counts))?)?,
            open_files: PyFileSet::new(facts.open_files),
            semi_open_files: Py::new(
                py,
                PyByColour::pair(py, facts.semi_open_files.map(PyFileSet::new))?,
            )?,
            islands: Py::new(py, PyByColour::pair(py, facts.islands)?)?,
            levers: Py::new(py, PyByColour::pair(py, facts.levers)?)?,
            rams: facts.rams,
            passer_lead_rank: Py::new(py, PyByColour::pair(py, facts.passer_lead_rank)?)?,
            passer_protected: Py::new(py, PyByColour::pair(py, facts.passer_protected)?)?,
            passers_connected: Py::new(py, PyByColour::pair(py, facts.passers_connected)?)?,
            passer_unstoppable: Py::new(py, PyByColour::pair(py, facts.passer_unstoppable)?)?,
            chain_max_length: Py::new(py, PyByColour::pair(py, facts.chain_max_length)?)?,
            chain_base_attacked: Py::new(py, PyByColour::pair(py, facts.chain_base_attacked)?)?,
            queenside_majority: Py::new(py, PyByColour::pair(py, facts.queenside_majority)?)?,
            kingside_majority: Py::new(py, PyByColour::pair(py, facts.kingside_majority)?)?,
            holes: Py::new(py, PyByColour::pair(py, facts.holes.map(PySquareSet::new))?)?,
            holes_occupied: Py::new(py, PyByColour::pair(py, facts.holes_occupied)?)?,
            fixed_pawns: Py::new(py, PyByColour::pair(py, facts.fixed_pawns)?)?,
            blocked_passers: Py::new(py, PyByColour::pair(py, facts.blocked_passers)?)?,
            passer_distance: Py::new(py, PyByColour::pair(py, facts.passer_distance)?)?,
            passer_own_king_distance: Py::new(
                py,
                PyByColour::pair(py, facts.passer_own_king_distance)?,
            )?,
            passer_enemy_king_distance: Py::new(
                py,
                PyByColour::pair(py, facts.passer_enemy_king_distance)?,
            )?,
            passer_in_square: Py::new(py, PyByColour::pair(py, facts.passer_in_square)?)?,
            passer_free_path: Py::new(py, PyByColour::pair(py, facts.passer_free_path)?)?,
            half_open_at_enemy_king: Py::new(
                py,
                PyByColour::pair(py, facts.half_open_at_enemy_king)?,
            )?,
            backward_on_semi_open: Py::new(py, PyByColour::pair(py, facts.backward_on_semi_open)?)?,
        })
    }
}

#[pymethods]
impl PyPawnFacts {
    /// The group as plain data, in the one JSON form `docs/features.md`
    /// describes.
    fn to_dict<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        plain(slf.py(), &slf.get().parent.get().inner.pawns)
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        repr_of(slf.as_any(), "PawnFacts", FIELDS_PAWNS)
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        match other.cast::<Self>() {
            Ok(other) => self.parent.get().inner.pawns == other.get().parent.get().inner.pawns,
            Err(_) => false,
        }
    }

    fn __hash__(&self) -> u64 {
        hash_of(&self.parent.get().inner.pawns)
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
    /// The side holds bishops on both square colours.
    #[pyo3(get)]
    bishop_pair: Py<PyByColour>,
    /// The side holds two or more knights.
    #[pyo3(get)]
    knight_pair: Py<PyByColour>,
    /// The side's bishops on light squares.
    #[pyo3(get)]
    bishops_light: Py<PyByColour>,
    /// The side's bishops on dark squares.
    #[pyo3(get)]
    bishops_dark: Py<PyByColour>,
    /// Exactly one bishop each, on different square colours.
    #[pyo3(get)]
    opposite_coloured_bishops: bool,
    /// The side's pawns standing on a square colour of one of its own bishops.
    #[pyo3(get)]
    pawns_on_bishop_colour: Py<PyByColour>,
    /// Two of the side's rooks share a rank with nothing between.
    #[pyo3(get)]
    rooks_connected_rank: Py<PyByColour>,
    /// Two of the side's rooks share a file with nothing between.
    #[pyo3(get)]
    rooks_connected_file: Py<PyByColour>,
    /// The side's rooks on an open file.
    #[pyo3(get)]
    rooks_on_open_file: Py<PyByColour>,
    /// The side's rooks on a file semi-open for their own side.
    #[pyo3(get)]
    rooks_on_semi_open_file: Py<PyByColour>,
    /// The side's rooks on its seventh rank, counted from its own back rank.
    #[pyo3(get)]
    rooks_on_seventh_rank: Py<PyByColour>,
    /// The side's rooks behind a passer of their own side.
    #[pyo3(get)]
    rook_behind_own_passer: Py<PyByColour>,
    /// The side's rooks behind an enemy passer.
    #[pyo3(get)]
    rook_behind_enemy_passer: Py<PyByColour>,
    /// The side has a rook shut in by its own king with no file to leave by.
    #[pyo3(get)]
    trapped_rook: Py<PyByColour>,
    /// The side's outpost squares: squares on its ranks 4 to 6 that a friendly pawn defends and no enemy pawn can ever attack.
    #[pyo3(get)]
    outposts: Py<PyByColour>,
    /// The side's knights and bishops standing on one of them.
    #[pyo3(get)]
    minors_on_outpost: Py<PyByColour>,
    /// Its unoccupied outpost squares.
    #[pyo3(get)]
    outpost_squares_free: Py<PyByColour>,
    /// The side's knights on file a or h, or on its rank 1 or 8.
    #[pyo3(get)]
    knights_on_rim: Py<PyByColour>,
    /// The side's knights and bishops still on their starting squares.
    #[pyo3(get)]
    minors_undeveloped: Py<PyByColour>,
    /// A queen of the side stands off its starting square.
    #[pyo3(get)]
    queen_developed: Py<PyByColour>,
    /// The side's fixed pawns standing on a square colour of one of its own bishops.
    #[pyo3(get)]
    fixed_pawns_on_bishop_colour: Py<PyByColour>,
    /// The side has a rook on its seventh rank with the enemy king on its eighth.
    #[pyo3(get)]
    rook_on_seventh_with_king_on_eighth: Py<PyByColour>,
    /// The side's units that are neither pawns nor kings and have no safe destination.
    #[pyo3(get)]
    trapped_pieces: Py<PyByColour>,
    /// The value sum of those.
    #[pyo3(get)]
    trapped_value: Py<PyByColour>,
}

impl PyPieceFacts {
    fn of(
        py: Python<'_>,
        facts: &facts::PieceFacts,
        parent: Py<PyFacts>,
    ) -> PyResult<PyPieceFacts> {
        Ok(PyPieceFacts {
            parent,
            bishop_pair: Py::new(py, PyByColour::pair(py, facts.bishop_pair)?)?,
            knight_pair: Py::new(py, PyByColour::pair(py, facts.knight_pair)?)?,
            bishops_light: Py::new(py, PyByColour::pair(py, facts.bishops_light)?)?,
            bishops_dark: Py::new(py, PyByColour::pair(py, facts.bishops_dark)?)?,
            opposite_coloured_bishops: facts.opposite_coloured_bishops,
            pawns_on_bishop_colour: Py::new(
                py,
                PyByColour::pair(py, facts.pawns_on_bishop_colour)?,
            )?,
            rooks_connected_rank: Py::new(py, PyByColour::pair(py, facts.rooks_connected_rank)?)?,
            rooks_connected_file: Py::new(py, PyByColour::pair(py, facts.rooks_connected_file)?)?,
            rooks_on_open_file: Py::new(py, PyByColour::pair(py, facts.rooks_on_open_file)?)?,
            rooks_on_semi_open_file: Py::new(
                py,
                PyByColour::pair(py, facts.rooks_on_semi_open_file)?,
            )?,
            rooks_on_seventh_rank: Py::new(py, PyByColour::pair(py, facts.rooks_on_seventh_rank)?)?,
            rook_behind_own_passer: Py::new(
                py,
                PyByColour::pair(py, facts.rook_behind_own_passer)?,
            )?,
            rook_behind_enemy_passer: Py::new(
                py,
                PyByColour::pair(py, facts.rook_behind_enemy_passer)?,
            )?,
            trapped_rook: Py::new(py, PyByColour::pair(py, facts.trapped_rook)?)?,
            outposts: Py::new(
                py,
                PyByColour::pair(py, facts.outposts.map(PySquareSet::new))?,
            )?,
            minors_on_outpost: Py::new(py, PyByColour::pair(py, facts.minors_on_outpost)?)?,
            outpost_squares_free: Py::new(py, PyByColour::pair(py, facts.outpost_squares_free)?)?,
            knights_on_rim: Py::new(py, PyByColour::pair(py, facts.knights_on_rim)?)?,
            minors_undeveloped: Py::new(py, PyByColour::pair(py, facts.minors_undeveloped)?)?,
            queen_developed: Py::new(py, PyByColour::pair(py, facts.queen_developed)?)?,
            fixed_pawns_on_bishop_colour: Py::new(
                py,
                PyByColour::pair(py, facts.fixed_pawns_on_bishop_colour)?,
            )?,
            rook_on_seventh_with_king_on_eighth: Py::new(
                py,
                PyByColour::pair(py, facts.rook_on_seventh_with_king_on_eighth)?,
            )?,
            trapped_pieces: Py::new(py, PyByColour::pair(py, facts.trapped_pieces)?)?,
            trapped_value: Py::new(py, PyByColour::pair(py, facts.trapped_value)?)?,
        })
    }
}

#[pymethods]
impl PyPieceFacts {
    /// The group as plain data, in the one JSON form `docs/features.md`
    /// describes.
    fn to_dict<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        plain(slf.py(), &slf.get().parent.get().inner.pieces)
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        repr_of(slf.as_any(), "PieceFacts", FIELDS_PIECES)
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        match other.cast::<Self>() {
            Ok(other) => self.parent.get().inner.pieces == other.get().parent.get().inner.pieces,
            Err(_) => false,
        }
    }

    fn __hash__(&self) -> u64 {
        hash_of(&self.parent.get().inner.pieces)
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
    inner: facts::KingFacts,
    /// Where the side's king stands.
    #[pyo3(get)]
    square: Py<PyByColour>,
    /// It stands on its own starting square.
    #[pyo3(get)]
    on_home_square: Py<PyByColour>,
    /// It stands on files a to c.
    #[pyo3(get)]
    castled_queenside: Py<PyByColour>,
    /// It stands on files f to h.
    #[pyo3(get)]
    castled_kingside: Py<PyByColour>,
    /// The three files the king's shelter is read on, in ascending order.
    #[pyo3(get)]
    shield_files: Py<PyByColour>,
    /// Ranks to the nearest friendly pawn ahead of the king, per shield file.
    #[pyo3(get)]
    shield: Py<PyByColour>,
    /// Each shield file carries no pawn of either colour.
    #[pyo3(get)]
    shield_file_open: Py<PyByColour>,
    /// Each shield file is semi-open for the enemy of that king.
    #[pyo3(get)]
    shield_file_semi_open_for_enemy: Py<PyByColour>,
    /// Ranks to the nearest enemy pawn ahead of the king, per shield file.
    #[pyo3(get)]
    storm: Py<PyByColour>,
    /// The squares adjacent to the king.
    #[pyo3(get)]
    ring: Py<PyByColour>,
    /// Enemy knights, bishops, rooks and queens attacking that ring.
    #[pyo3(get)]
    ring_attackers: Py<PyByColour>,
    /// Σ over those attackers of N, B = 1, R = 2, Q = 4.
    #[pyo3(get)]
    ring_attack_weight: Py<PyByColour>,
    /// Ring squares the king's own side attacks, the king itself apart.
    #[pyo3(get)]
    ring_defended: Py<PyByColour>,
    /// Ring squares the enemy attacks and `ring_defended` does not count.
    #[pyo3(get)]
    ring_holes: Py<PyByColour>,
    /// Squares adjacent to the king that are empty or capturable and not attacked.
    #[pyo3(get)]
    escape_squares: Py<PyByColour>,
    /// The king stands on its rank 1 with every forward-adjacent square held by a friendly unit.
    #[pyo3(get)]
    back_rank_risk: Py<PyByColour>,
    /// Chebyshev distance between the kings.
    #[pyo3(get)]
    distance: u8,
    /// Mean Chebyshev distance of the enemy's knights, bishops, rooks and queens to this king.
    #[pyo3(get)]
    tropism: Py<PyByColour>,
    /// Squares a queen on this king's square would attack.
    #[pyo3(get)]
    virtual_mobility: Py<PyByColour>,
    /// The king's own knights, bishops, rooks and queens attacking its ring.
    #[pyo3(get)]
    ring_defenders: Py<PyByColour>,
    /// Σ over those defenders of N, B = 1, R = 2, Q = 4.
    #[pyo3(get)]
    ring_defence_weight: Py<PyByColour>,
    /// Directions from the king holding at least one square, all of them empty out to the edge.
    #[pyo3(get)]
    open_rays: Py<PyByColour>,
    /// The king stands on its rank 1 with a square ahead of it empty and unattacked.
    #[pyo3(get)]
    luft: Py<PyByColour>,
    /// The side the king's square and its spent castling rights read as.
    #[pyo3(get)]
    castled_side: Py<PyByColour>,
    /// The two kings stand on opposite wings.
    #[pyo3(get)]
    opposite_side_castling: bool,
}

impl PyKingFacts {
    fn of(py: Python<'_>, facts: &facts::KingFacts, parent: Py<PyFacts>) -> PyResult<PyKingFacts> {
        Ok(PyKingFacts {
            parent,
            inner: *facts,
            square: Py::new(py, PyByColour::pair(py, facts.square.map(square_name))?)?,
            on_home_square: Py::new(py, PyByColour::pair(py, facts.on_home_square)?)?,
            castled_queenside: Py::new(py, PyByColour::pair(py, facts.castled_queenside)?)?,
            castled_kingside: Py::new(py, PyByColour::pair(py, facts.castled_kingside)?)?,
            shield_files: Py::new(
                py,
                PyByColour::pair(
                    py,
                    facts
                        .shield_files
                        .map(|files| Fixed(files.map(file_name).to_vec())),
                )?,
            )?,
            shield: Py::new(
                py,
                PyByColour::pair(py, facts.shield.map(|values| Fixed(values.to_vec())))?,
            )?,
            shield_file_open: Py::new(
                py,
                PyByColour::pair(
                    py,
                    facts.shield_file_open.map(|values| Fixed(values.to_vec())),
                )?,
            )?,
            shield_file_semi_open_for_enemy: Py::new(
                py,
                PyByColour::pair(
                    py,
                    facts
                        .shield_file_semi_open_for_enemy
                        .map(|values| Fixed(values.to_vec())),
                )?,
            )?,
            storm: Py::new(
                py,
                PyByColour::pair(py, facts.storm.map(|values| Fixed(values.to_vec())))?,
            )?,
            ring: Py::new(py, PyByColour::pair(py, facts.ring.map(PySquareSet::new))?)?,
            ring_attackers: Py::new(py, PyByColour::pair(py, facts.ring_attackers)?)?,
            ring_attack_weight: Py::new(py, PyByColour::pair(py, facts.ring_attack_weight)?)?,
            ring_defended: Py::new(py, PyByColour::pair(py, facts.ring_defended)?)?,
            ring_holes: Py::new(py, PyByColour::pair(py, facts.ring_holes)?)?,
            escape_squares: Py::new(py, PyByColour::pair(py, facts.escape_squares)?)?,
            back_rank_risk: Py::new(py, PyByColour::pair(py, facts.back_rank_risk)?)?,
            distance: facts.distance,
            tropism: Py::new(py, PyByColour::pair(py, facts.tropism)?)?,
            virtual_mobility: Py::new(py, PyByColour::pair(py, facts.virtual_mobility)?)?,
            ring_defenders: Py::new(py, PyByColour::pair(py, facts.ring_defenders)?)?,
            ring_defence_weight: Py::new(py, PyByColour::pair(py, facts.ring_defence_weight)?)?,
            open_rays: Py::new(py, PyByColour::pair(py, facts.open_rays)?)?,
            luft: Py::new(py, PyByColour::pair(py, facts.luft)?)?,
            castled_side: Py::new(
                py,
                PyByColour::pair(
                    py,
                    facts.castled_side.map(|side| side.map(CastledSide::name)),
                )?,
            )?,
            opposite_side_castling: facts.opposite_side_castling,
        })
    }
}

#[pymethods]
impl PyKingFacts {
    /// The weight besieging each king's ring, less the weight defending it.
    #[getter]
    fn ring_attacker_surplus(&self, py: Python<'_>) -> PyResult<PyByColour> {
        PyByColour::pair(py, self.inner.ring_attacker_surplus())
    }

    /// The group as plain data, in the one JSON form `docs/features.md`
    /// describes.
    fn to_dict<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        plain(slf.py(), &slf.get().parent.get().inner.king)
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        repr_of(slf.as_any(), "KingFacts", FIELDS_KING)
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        match other.cast::<Self>() {
            Ok(other) => self.parent.get().inner.king == other.get().parent.get().inner.king,
            Err(_) => false,
        }
    }

    fn __hash__(&self) -> u64 {
        hash_of(&self.parent.get().inner.king)
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
    /// Squares the side's pawns attack that its own units do not hold.
    #[pyo3(get)]
    pawns: Py<PyByColour>,
    /// The same for its knights.
    #[pyo3(get)]
    knights: Py<PyByColour>,
    /// The same for its bishops.
    #[pyo3(get)]
    bishops: Py<PyByColour>,
    /// The same for its rooks.
    #[pyo3(get)]
    rooks: Py<PyByColour>,
    /// The same for its queens.
    #[pyo3(get)]
    queens: Py<PyByColour>,
    /// Of the pawn squares, those no enemy pawn attacks.
    #[pyo3(get)]
    safe_pawns: Py<PyByColour>,
    /// The same for its knights.
    #[pyo3(get)]
    safe_knights: Py<PyByColour>,
    /// The same for its bishops.
    #[pyo3(get)]
    safe_bishops: Py<PyByColour>,
    /// The same for its rooks.
    #[pyo3(get)]
    safe_rooks: Py<PyByColour>,
    /// The same for its queens.
    #[pyo3(get)]
    safe_queens: Py<PyByColour>,
    /// The five counts added up.
    #[pyo3(get)]
    total: Py<PyByColour>,
    /// Squares the side attacks in the enemy's half.
    #[pyo3(get)]
    space: Py<PyByColour>,
    /// The side's attacks on d4, e4, d5 and e5.
    #[pyo3(get)]
    centre_control: Py<PyByColour>,
    /// The side's attacks on c3 to f6.
    #[pyo3(get)]
    extended_centre_control: Py<PyByColour>,
    /// The side's units other than pawns and kings with no destination.
    #[pyo3(get)]
    immobile_pieces: Py<PyByColour>,
}

impl PyMobilityFacts {
    fn of(
        py: Python<'_>,
        facts: &facts::MobilityFacts,
        parent: Py<PyFacts>,
    ) -> PyResult<PyMobilityFacts> {
        Ok(PyMobilityFacts {
            parent,
            pawns: Py::new(py, PyByColour::pair(py, facts.pawns)?)?,
            knights: Py::new(py, PyByColour::pair(py, facts.knights)?)?,
            bishops: Py::new(py, PyByColour::pair(py, facts.bishops)?)?,
            rooks: Py::new(py, PyByColour::pair(py, facts.rooks)?)?,
            queens: Py::new(py, PyByColour::pair(py, facts.queens)?)?,
            safe_pawns: Py::new(py, PyByColour::pair(py, facts.safe_pawns)?)?,
            safe_knights: Py::new(py, PyByColour::pair(py, facts.safe_knights)?)?,
            safe_bishops: Py::new(py, PyByColour::pair(py, facts.safe_bishops)?)?,
            safe_rooks: Py::new(py, PyByColour::pair(py, facts.safe_rooks)?)?,
            safe_queens: Py::new(py, PyByColour::pair(py, facts.safe_queens)?)?,
            total: Py::new(py, PyByColour::pair(py, facts.total)?)?,
            space: Py::new(py, PyByColour::pair(py, facts.space)?)?,
            centre_control: Py::new(py, PyByColour::pair(py, facts.centre_control)?)?,
            extended_centre_control: Py::new(
                py,
                PyByColour::pair(py, facts.extended_centre_control)?,
            )?,
            immobile_pieces: Py::new(py, PyByColour::pair(py, facts.immobile_pieces)?)?,
        })
    }
}

#[pymethods]
impl PyMobilityFacts {
    /// The group as plain data, in the one JSON form `docs/features.md`
    /// describes.
    fn to_dict<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        plain(slf.py(), &slf.get().parent.get().inner.mobility)
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        repr_of(slf.as_any(), "MobilityFacts", FIELDS_MOBILITY)
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        match other.cast::<Self>() {
            Ok(other) => {
                self.parent.get().inner.mobility == other.get().parent.get().inner.mobility
            }
            Err(_) => false,
        }
    }

    fn __hash__(&self) -> u64 {
        hash_of(&self.parent.get().inner.mobility)
    }

    fn __reduce__<'py>(slf: &Bound<'py, Self>) -> GroupReduce<'py> {
        let py = slf.py();
        Ok((
            group_reconstructor(py)?,
            (slf.get().parent.clone_ref(py), "mobility"),
        ))
    }
}

/// The attack maps and what they say about the units standing on the board.
#[pyclass(frozen, module = "esca", name = "AttackFacts")]
pub struct PyAttackFacts {
    parent: Py<PyFacts>,
    /// Every square the side attacks.
    #[pyo3(get)]
    controlled: Py<PyByColour>,
    /// The squares the side's pawns attack.
    #[pyo3(get)]
    by_pawns: Py<PyByColour>,
    /// The squares its knights attack.
    #[pyo3(get)]
    by_knights: Py<PyByColour>,
    /// The squares its bishops attack.
    #[pyo3(get)]
    by_bishops: Py<PyByColour>,
    /// The squares its rooks attack.
    #[pyo3(get)]
    by_rooks: Py<PyByColour>,
    /// The squares its queens attack.
    #[pyo3(get)]
    by_queens: Py<PyByColour>,
    /// The squares its king attacks.
    #[pyo3(get)]
    by_kings: Py<PyByColour>,
    /// The side's units the enemy attacks, defended or not; never a king.
    #[pyo3(get)]
    attacked: Py<PyByColour>,
    /// Those of them their own side does not defend.
    #[pyo3(get)]
    hanging: Py<PyByColour>,
    /// Those hanging or attacked by a cheaper enemy unit.
    #[pyo3(get)]
    en_prise: Py<PyByColour>,
    /// The side's units under an absolute pin.
    #[pyo3(get)]
    pinned: Py<PyByColour>,
    /// The side's units standing on a square their own side attacks.
    #[pyo3(get)]
    defended: Py<PyByColour>,
    /// The value sum of the attacked units.
    #[pyo3(get)]
    attacked_value: Py<PyByColour>,
    /// The value sum of the hanging units.
    #[pyo3(get)]
    hanging_value: Py<PyByColour>,
    /// The value sum of the units en prise.
    #[pyo3(get)]
    en_prise_value: Py<PyByColour>,
    /// The largest value en prise.
    #[pyo3(get)]
    en_prise_max_value: Py<PyByColour>,
    /// The value sum of the absolutely pinned units.
    #[pyo3(get)]
    pinned_value: Py<PyByColour>,
    /// Enemy unit pairs the side's sliders skewer.
    #[pyo3(get)]
    skewer_candidates: Py<PyByColour>,
}

impl PyAttackFacts {
    fn of(
        py: Python<'_>,
        facts: &facts::AttackFacts,
        parent: Py<PyFacts>,
    ) -> PyResult<PyAttackFacts> {
        Ok(PyAttackFacts {
            parent,
            controlled: Py::new(
                py,
                PyByColour::pair(py, facts.controlled.map(PySquareSet::new))?,
            )?,
            by_pawns: Py::new(
                py,
                PyByColour::pair(py, facts.by_pawns.map(PySquareSet::new))?,
            )?,
            by_knights: Py::new(
                py,
                PyByColour::pair(py, facts.by_knights.map(PySquareSet::new))?,
            )?,
            by_bishops: Py::new(
                py,
                PyByColour::pair(py, facts.by_bishops.map(PySquareSet::new))?,
            )?,
            by_rooks: Py::new(
                py,
                PyByColour::pair(py, facts.by_rooks.map(PySquareSet::new))?,
            )?,
            by_queens: Py::new(
                py,
                PyByColour::pair(py, facts.by_queens.map(PySquareSet::new))?,
            )?,
            by_kings: Py::new(
                py,
                PyByColour::pair(py, facts.by_kings.map(PySquareSet::new))?,
            )?,
            attacked: Py::new(
                py,
                PyByColour::pair(py, facts.attacked.map(PySquareSet::new))?,
            )?,
            hanging: Py::new(
                py,
                PyByColour::pair(py, facts.hanging.map(PySquareSet::new))?,
            )?,
            en_prise: Py::new(
                py,
                PyByColour::pair(py, facts.en_prise.map(PySquareSet::new))?,
            )?,
            pinned: Py::new(
                py,
                PyByColour::pair(py, facts.pinned.map(PySquareSet::new))?,
            )?,
            defended: Py::new(
                py,
                PyByColour::pair(py, facts.defended.map(PySquareSet::new))?,
            )?,
            attacked_value: Py::new(py, PyByColour::pair(py, facts.attacked_value)?)?,
            hanging_value: Py::new(py, PyByColour::pair(py, facts.hanging_value)?)?,
            en_prise_value: Py::new(py, PyByColour::pair(py, facts.en_prise_value)?)?,
            en_prise_max_value: Py::new(py, PyByColour::pair(py, facts.en_prise_max_value)?)?,
            pinned_value: Py::new(py, PyByColour::pair(py, facts.pinned_value)?)?,
            skewer_candidates: Py::new(py, PyByColour::pair(py, facts.skewer_candidates)?)?,
        })
    }
}

#[pymethods]
impl PyAttackFacts {
    /// The group as plain data, in the one JSON form `docs/features.md`
    /// describes.
    fn to_dict<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        plain(slf.py(), &slf.get().parent.get().inner.attacks)
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        repr_of(slf.as_any(), "AttackFacts", FIELDS_ATTACKS)
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        match other.cast::<Self>() {
            Ok(other) => self.parent.get().inner.attacks == other.get().parent.get().inner.attacks,
            Err(_) => false,
        }
    }

    fn __hash__(&self) -> u64 {
        hash_of(&self.parent.get().inner.attacks)
    }

    fn __reduce__<'py>(slf: &Bound<'py, Self>) -> GroupReduce<'py> {
        let py = slf.py();
        Ok((
            group_reconstructor(py)?,
            (slf.get().parent.clone_ref(py), "attacks"),
        ))
    }
}

/// Each side's captures, judged by the exchange they start. The values for the side not to move are read after a null move; when the side to move is in check that move does not exist and they are zero.
#[pyclass(frozen, module = "esca", name = "ExchangeFacts")]
pub struct PyExchangeFacts {
    parent: Py<PyFacts>,
    /// The largest static exchange evaluation over the side's captures; 0 when it has none.
    #[pyo3(get)]
    see_best_capture: Py<PyByColour>,
    /// Its captures whose evaluation is above 0.
    #[pyo3(get)]
    see_positive_capture_count: Py<PyByColour>,
    /// Its captures whose evaluation is 0.
    #[pyo3(get)]
    see_equal_capture_count: Py<PyByColour>,
    /// The sum of the evaluations above 0.
    #[pyo3(get)]
    see_positive_total: Py<PyByColour>,
}

impl PyExchangeFacts {
    fn of(
        py: Python<'_>,
        facts: &facts::ExchangeFacts,
        parent: Py<PyFacts>,
    ) -> PyResult<PyExchangeFacts> {
        Ok(PyExchangeFacts {
            parent,
            see_best_capture: Py::new(py, PyByColour::pair(py, facts.see_best_capture)?)?,
            see_positive_capture_count: Py::new(
                py,
                PyByColour::pair(py, facts.see_positive_capture_count)?,
            )?,
            see_equal_capture_count: Py::new(
                py,
                PyByColour::pair(py, facts.see_equal_capture_count)?,
            )?,
            see_positive_total: Py::new(py, PyByColour::pair(py, facts.see_positive_total)?)?,
        })
    }
}

#[pymethods]
impl PyExchangeFacts {
    /// The group as plain data, in the one JSON form `docs/features.md`
    /// describes.
    fn to_dict<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        plain(slf.py(), &slf.get().parent.get().inner.exchange)
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        repr_of(slf.as_any(), "ExchangeFacts", FIELDS_EXCHANGE)
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        match other.cast::<Self>() {
            Ok(other) => {
                self.parent.get().inner.exchange == other.get().parent.get().inner.exchange
            }
            Err(_) => false,
        }
    }

    fn __hash__(&self) -> u64 {
        hash_of(&self.parent.get().inner.exchange)
    }

    fn __reduce__<'py>(slf: &Bound<'py, Self>) -> GroupReduce<'py> {
        let py = slf.py();
        Ok((
            group_reconstructor(py)?,
            (slf.get().parent.clone_ref(py), "exchange"),
        ))
    }
}

/// What each side stands to lose, and the slider geometry behind it. Every set is read on the units it is about, and holds no king.
#[pyclass(frozen, module = "esca", name = "ThreatFacts")]
pub struct PyThreatFacts {
    parent: Py<PyFacts>,
    /// The side's units the enemy wins material by taking.
    #[pyo3(get)]
    threatened: Py<PyByColour>,
    /// The value sum of those.
    #[pyo3(get)]
    threatened_value: Py<PyByColour>,
    /// The largest such gain over the side's own units.
    #[pyo3(get)]
    threat_max_gain: Py<PyByColour>,
    /// The side's units an enemy unit of strictly lower value attacks.
    #[pyo3(get)]
    attacked_by_lesser: Py<PyByColour>,
    /// One of those units is a queen.
    #[pyo3(get)]
    queen_attacked_by_lesser: Py<PyByColour>,
    /// The side's sole defenders of two or more attacked friendly units.
    #[pyo3(get)]
    overloaded_defenders: Py<PyByColour>,
    /// Its sole defenders the enemy can capture without loss.
    #[pyo3(get)]
    removable_defenders: Py<PyByColour>,
    /// The side's units no unit of their own side defends.
    #[pyo3(get)]
    loose: Py<PyByColour>,
    /// The side's units more enemy units attack than its own defend.
    #[pyo3(get)]
    attacker_surplus: Py<PyByColour>,
    /// The side's x-rays onto an enemy unit through one enemy unit.
    #[pyo3(get)]
    xray_through_enemy: Py<PyByColour>,
    /// The side's batteries: two of its own sliders on one line, agreeing in direction.
    #[pyo3(get)]
    battery_count: Py<PyByColour>,
    /// One of them bears on a square of the enemy king's ring.
    #[pyo3(get)]
    battery_at_king: Py<PyByColour>,
}

impl PyThreatFacts {
    fn of(
        py: Python<'_>,
        facts: &facts::ThreatFacts,
        parent: Py<PyFacts>,
    ) -> PyResult<PyThreatFacts> {
        Ok(PyThreatFacts {
            parent,
            threatened: Py::new(
                py,
                PyByColour::pair(py, facts.threatened.map(PySquareSet::new))?,
            )?,
            threatened_value: Py::new(py, PyByColour::pair(py, facts.threatened_value)?)?,
            threat_max_gain: Py::new(py, PyByColour::pair(py, facts.threat_max_gain)?)?,
            attacked_by_lesser: Py::new(
                py,
                PyByColour::pair(py, facts.attacked_by_lesser.map(PySquareSet::new))?,
            )?,
            queen_attacked_by_lesser: Py::new(
                py,
                PyByColour::pair(py, facts.queen_attacked_by_lesser)?,
            )?,
            overloaded_defenders: Py::new(
                py,
                PyByColour::pair(py, facts.overloaded_defenders.map(PySquareSet::new))?,
            )?,
            removable_defenders: Py::new(
                py,
                PyByColour::pair(py, facts.removable_defenders.map(PySquareSet::new))?,
            )?,
            loose: Py::new(py, PyByColour::pair(py, facts.loose.map(PySquareSet::new))?)?,
            attacker_surplus: Py::new(
                py,
                PyByColour::pair(py, facts.attacker_surplus.map(PySquareSet::new))?,
            )?,
            xray_through_enemy: Py::new(py, PyByColour::pair(py, facts.xray_through_enemy)?)?,
            battery_count: Py::new(py, PyByColour::pair(py, facts.battery_count)?)?,
            battery_at_king: Py::new(py, PyByColour::pair(py, facts.battery_at_king)?)?,
        })
    }
}

#[pymethods]
impl PyThreatFacts {
    /// The group as plain data, in the one JSON form `docs/features.md`
    /// describes.
    fn to_dict<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        plain(slf.py(), &slf.get().parent.get().inner.threats)
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        repr_of(slf.as_any(), "ThreatFacts", FIELDS_THREATS)
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        match other.cast::<Self>() {
            Ok(other) => self.parent.get().inner.threats == other.get().parent.get().inner.threats,
            Err(_) => false,
        }
    }

    fn __hash__(&self) -> u64 {
        hash_of(&self.parent.get().inner.threats)
    }

    fn __reduce__<'py>(slf: &Bound<'py, Self>) -> GroupReduce<'py> {
        let py = slf.py();
        Ok((
            group_reconstructor(py)?,
            (slf.get().parent.clone_ref(py), "threats"),
        ))
    }
}

/// The square sets a reader most often wants side by side: what each side attacks, and what of it stands hanging, pinned or threatened. Every one of them is also a field of the `attacks` or the `threats` group; this group is where they are read together.
#[pyclass(frozen, module = "esca", name = "MapFacts")]
pub struct PyMapFacts {
    parent: Py<PyFacts>,
    /// Every square the side attacks.
    #[pyo3(get)]
    controlled: Py<PyByColour>,
    /// The squares the side's pawns attack.
    #[pyo3(get)]
    by_pawns: Py<PyByColour>,
    /// The side's units the enemy attacks and their own side does not defend.
    #[pyo3(get)]
    hanging: Py<PyByColour>,
    /// The side's units under an absolute pin.
    #[pyo3(get)]
    pinned: Py<PyByColour>,
    /// The side's units the enemy wins material by taking.
    #[pyo3(get)]
    threatened: Py<PyByColour>,
}

impl PyMapFacts {
    fn of(py: Python<'_>, facts: &facts::MapFacts, parent: Py<PyFacts>) -> PyResult<PyMapFacts> {
        Ok(PyMapFacts {
            parent,
            controlled: Py::new(
                py,
                PyByColour::pair(py, facts.controlled.map(PySquareSet::new))?,
            )?,
            by_pawns: Py::new(
                py,
                PyByColour::pair(py, facts.by_pawns.map(PySquareSet::new))?,
            )?,
            hanging: Py::new(
                py,
                PyByColour::pair(py, facts.hanging.map(PySquareSet::new))?,
            )?,
            pinned: Py::new(
                py,
                PyByColour::pair(py, facts.pinned.map(PySquareSet::new))?,
            )?,
            threatened: Py::new(
                py,
                PyByColour::pair(py, facts.threatened.map(PySquareSet::new))?,
            )?,
        })
    }
}

#[pymethods]
impl PyMapFacts {
    /// The group as plain data, in the one JSON form `docs/features.md`
    /// describes.
    fn to_dict<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        plain(slf.py(), &slf.get().parent.get().inner.maps)
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        repr_of(slf.as_any(), "MapFacts", FIELDS_MAPS)
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        match other.cast::<Self>() {
            Ok(other) => self.parent.get().inner.maps == other.get().parent.get().inner.maps,
            Err(_) => false,
        }
    }

    fn __hash__(&self) -> u64 {
        hash_of(&self.parent.get().inner.maps)
    }

    fn __reduce__<'py>(slf: &Bound<'py, Self>) -> GroupReduce<'py> {
        let py = slf.py();
        Ok((
            group_reconstructor(py)?,
            (slf.get().parent.clone_ref(py), "maps"),
        ))
    }
}

/// Where the kings stand, how the pawn race runs, and which drawn configuration the material is.
#[pyclass(frozen, module = "esca", name = "EndgameFacts")]
pub struct PyEndgameFacts {
    parent: Py<PyFacts>,
    /// Chebyshev distance from the side's king to the nearest central square.
    #[pyo3(get)]
    king_centralisation: Py<PyByColour>,
    /// Plies the side's most advanced passer needs to promote unopposed; 8 for a side with no passer.
    #[pyo3(get)]
    race_plies: Py<PyByColour>,
    /// The opposition the kings stand in, if any.
    #[pyo3(get)]
    opposition: Option<&'static str>,
    /// The side's king stands on a key square of one of its own passers.
    #[pyo3(get)]
    key_square_occupied: Py<PyByColour>,
    /// The side's bishops all stand on the colour none of its pawns promotes on, every one of those pawns being a rook pawn.
    #[pyo3(get)]
    wrong_colour_bishop: Py<PyByColour>,
    /// The drawn configuration the material is, if it is one of the three.
    #[pyo3(get)]
    drawish_material: Option<&'static str>,
}

impl PyEndgameFacts {
    fn of(
        py: Python<'_>,
        facts: &facts::EndgameFacts,
        parent: Py<PyFacts>,
    ) -> PyResult<PyEndgameFacts> {
        Ok(PyEndgameFacts {
            parent,
            king_centralisation: Py::new(py, PyByColour::pair(py, facts.king_centralisation)?)?,
            race_plies: Py::new(py, PyByColour::pair(py, facts.race_plies)?)?,
            opposition: facts.opposition.map(Opposition::name),
            key_square_occupied: Py::new(py, PyByColour::pair(py, facts.key_square_occupied)?)?,
            wrong_colour_bishop: Py::new(py, PyByColour::pair(py, facts.wrong_colour_bishop)?)?,
            drawish_material: facts.drawish_material.map(DrawishMaterial::name),
        })
    }
}

#[pymethods]
impl PyEndgameFacts {
    /// The group as plain data, in the one JSON form `docs/features.md`
    /// describes.
    fn to_dict<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        plain(slf.py(), &slf.get().parent.get().inner.endgame)
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        repr_of(slf.as_any(), "EndgameFacts", FIELDS_ENDGAME)
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        match other.cast::<Self>() {
            Ok(other) => self.parent.get().inner.endgame == other.get().parent.get().inner.endgame,
            Err(_) => false,
        }
    }

    fn __hash__(&self) -> u64 {
        hash_of(&self.parent.get().inner.endgame)
    }

    fn __reduce__<'py>(slf: &Bound<'py, Self>) -> GroupReduce<'py> {
        let py = slf.py();
        Ok((
            group_reconstructor(py)?,
            (slf.get().parent.clone_ref(py), "endgame"),
        ))
    }
}

/// Each side's one-ply tactical options. The values for the side not to move are read after a null move; when the side to move is in check that move does not exist and they are zero, which `available` reports.
#[pyclass(frozen, module = "esca", name = "TacticsFacts")]
pub struct PyTacticsFacts {
    parent: Py<PyFacts>,
    inner: facts::TacticsFacts,
    /// The side's values could be read.
    #[pyo3(get)]
    available: Py<PyByColour>,
    /// Its moves giving check.
    #[pyo3(get)]
    check_count: Py<PyByColour>,
    /// A pawn move of the side gives check.
    #[pyo3(get)]
    check_by_pawn: Py<PyByColour>,
    /// A knight move gives check.
    #[pyo3(get)]
    check_by_knight: Py<PyByColour>,
    /// A bishop move gives check.
    #[pyo3(get)]
    check_by_bishop: Py<PyByColour>,
    /// A rook move gives check.
    #[pyo3(get)]
    check_by_rook: Py<PyByColour>,
    /// A queen move gives check.
    #[pyo3(get)]
    check_by_queen: Py<PyByColour>,
    /// Its checking moves whose destination is safe.
    #[pyo3(get)]
    safe_check_count: Py<PyByColour>,
    /// A pawn move of the side gives check from a safe square.
    #[pyo3(get)]
    safe_check_by_pawn: Py<PyByColour>,
    /// A knight move does.
    #[pyo3(get)]
    safe_check_by_knight: Py<PyByColour>,
    /// A bishop move does.
    #[pyo3(get)]
    safe_check_by_bishop: Py<PyByColour>,
    /// A rook move does.
    #[pyo3(get)]
    safe_check_by_rook: Py<PyByColour>,
    /// A queen move does.
    #[pyo3(get)]
    safe_check_by_queen: Py<PyByColour>,
    /// A move gives check from two units at once.
    #[pyo3(get)]
    double_check_available: Py<PyByColour>,
    /// A move gives check with a unit that did not move.
    #[pyo3(get)]
    discovered_check_available: Py<PyByColour>,
    /// A move leaves the opponent checkmated.
    #[pyo3(get)]
    mate_in_1: Py<PyByColour>,
    /// A move leaves the opponent stalemated.
    #[pyo3(get)]
    stalemate_in_1: Py<PyByColour>,
    /// Files a legal move of the side promotes on.
    #[pyo3(get)]
    promotion_files: Py<PyByColour>,
    /// A promotion to a queen is available.
    #[pyo3(get)]
    promotes_to_queen: Py<PyByColour>,
    /// One to a rook is.
    #[pyo3(get)]
    promotes_to_rook: Py<PyByColour>,
    /// One to a bishop is.
    #[pyo3(get)]
    promotes_to_bishop: Py<PyByColour>,
    /// One to a knight is.
    #[pyo3(get)]
    promotes_to_knight: Py<PyByColour>,
    /// Files a legal promotion with a safe destination lands on.
    #[pyo3(get)]
    safe_promotion_files: Py<PyByColour>,
    /// The side's capturing moves.
    #[pyo3(get)]
    capture_count: Py<PyByColour>,
    /// One of them takes a unit that outvalues the capturer or is undefended.
    #[pyo3(get)]
    winning_capture_available: Py<PyByColour>,
    /// The largest `victim − capturer` over the captures, at least 0.
    #[pyo3(get)]
    winning_capture_max_gain: Py<PyByColour>,
    /// A capture takes a hanging unit.
    #[pyo3(get)]
    captures_hanging: Py<PyByColour>,
    /// The largest value among the hanging units capturable now.
    #[pyo3(get)]
    hanging_victim_max_value: Py<PyByColour>,
    /// Captures of a defended unit of equal value.
    #[pyo3(get)]
    equal_capture_count: Py<PyByColour>,
    /// Captures of a defended unit of lower value.
    #[pyo3(get)]
    losing_capture_count: Py<PyByColour>,
    /// Moves after which the moved unit forks.
    #[pyo3(get)]
    fork_count: Py<PyByColour>,
    /// The largest single forked value.
    #[pyo3(get)]
    fork_max_value: Py<PyByColour>,
    /// A fork by a knight.
    #[pyo3(get)]
    knight_fork_available: Py<PyByColour>,
    /// A fork one of whose targets is the king.
    #[pyo3(get)]
    royal_fork_available: Py<PyByColour>,
    /// Moves creating an absolute or a relative pin.
    #[pyo3(get)]
    pin_creation_count: Py<PyByColour>,
    /// A move creating a skewer.
    #[pyo3(get)]
    skewer_creation_available: Py<PyByColour>,
    /// A move uncovering a slider's attack on a unit of value 3 or more.
    #[pyo3(get)]
    discovered_attack_available: Py<PyByColour>,
    /// The side's legal moves.
    #[pyo3(get)]
    legal_move_count: Py<PyByColour>,
    /// A checking move that captures and whose destination is safe.
    #[pyo3(get)]
    safe_check_capturing: Py<PyByColour>,
    /// A move uncovering a slider's attack on the enemy queen.
    #[pyo3(get)]
    discovered_attack_on_queen: Py<PyByColour>,
    /// A move after which a rook or queen of the side checks the enemy king along that king's own rank, its back rank being blocked.
    #[pyo3(get)]
    back_rank_mate_threat: Py<PyByColour>,
    /// A quiet move after which the largest gain the side threatens is greater than it is now.
    #[pyo3(get)]
    quiet_threat_available: Py<PyByColour>,
    /// No legal move of the side has a safe destination.
    #[pyo3(get)]
    no_safe_moves: Py<PyByColour>,
    /// A promotion of the side wins material.
    #[pyo3(get)]
    promotion_see_positive: Py<PyByColour>,
}

impl PyTacticsFacts {
    fn of(
        py: Python<'_>,
        facts: &facts::TacticsFacts,
        parent: Py<PyFacts>,
    ) -> PyResult<PyTacticsFacts> {
        Ok(PyTacticsFacts {
            parent,
            inner: *facts,
            available: Py::new(py, PyByColour::pair(py, facts.available)?)?,
            check_count: Py::new(py, PyByColour::pair(py, facts.check_count)?)?,
            check_by_pawn: Py::new(py, PyByColour::pair(py, facts.check_by_pawn)?)?,
            check_by_knight: Py::new(py, PyByColour::pair(py, facts.check_by_knight)?)?,
            check_by_bishop: Py::new(py, PyByColour::pair(py, facts.check_by_bishop)?)?,
            check_by_rook: Py::new(py, PyByColour::pair(py, facts.check_by_rook)?)?,
            check_by_queen: Py::new(py, PyByColour::pair(py, facts.check_by_queen)?)?,
            safe_check_count: Py::new(py, PyByColour::pair(py, facts.safe_check_count)?)?,
            safe_check_by_pawn: Py::new(py, PyByColour::pair(py, facts.safe_check_by_pawn)?)?,
            safe_check_by_knight: Py::new(py, PyByColour::pair(py, facts.safe_check_by_knight)?)?,
            safe_check_by_bishop: Py::new(py, PyByColour::pair(py, facts.safe_check_by_bishop)?)?,
            safe_check_by_rook: Py::new(py, PyByColour::pair(py, facts.safe_check_by_rook)?)?,
            safe_check_by_queen: Py::new(py, PyByColour::pair(py, facts.safe_check_by_queen)?)?,
            double_check_available: Py::new(
                py,
                PyByColour::pair(py, facts.double_check_available)?,
            )?,
            discovered_check_available: Py::new(
                py,
                PyByColour::pair(py, facts.discovered_check_available)?,
            )?,
            mate_in_1: Py::new(py, PyByColour::pair(py, facts.mate_in_1)?)?,
            stalemate_in_1: Py::new(py, PyByColour::pair(py, facts.stalemate_in_1)?)?,
            promotion_files: Py::new(
                py,
                PyByColour::pair(py, facts.promotion_files.map(PyFileSet::new))?,
            )?,
            promotes_to_queen: Py::new(py, PyByColour::pair(py, facts.promotes_to_queen)?)?,
            promotes_to_rook: Py::new(py, PyByColour::pair(py, facts.promotes_to_rook)?)?,
            promotes_to_bishop: Py::new(py, PyByColour::pair(py, facts.promotes_to_bishop)?)?,
            promotes_to_knight: Py::new(py, PyByColour::pair(py, facts.promotes_to_knight)?)?,
            safe_promotion_files: Py::new(
                py,
                PyByColour::pair(py, facts.safe_promotion_files.map(PyFileSet::new))?,
            )?,
            capture_count: Py::new(py, PyByColour::pair(py, facts.capture_count)?)?,
            winning_capture_available: Py::new(
                py,
                PyByColour::pair(py, facts.winning_capture_available)?,
            )?,
            winning_capture_max_gain: Py::new(
                py,
                PyByColour::pair(py, facts.winning_capture_max_gain)?,
            )?,
            captures_hanging: Py::new(py, PyByColour::pair(py, facts.captures_hanging)?)?,
            hanging_victim_max_value: Py::new(
                py,
                PyByColour::pair(py, facts.hanging_victim_max_value)?,
            )?,
            equal_capture_count: Py::new(py, PyByColour::pair(py, facts.equal_capture_count)?)?,
            losing_capture_count: Py::new(py, PyByColour::pair(py, facts.losing_capture_count)?)?,
            fork_count: Py::new(py, PyByColour::pair(py, facts.fork_count)?)?,
            fork_max_value: Py::new(py, PyByColour::pair(py, facts.fork_max_value)?)?,
            knight_fork_available: Py::new(py, PyByColour::pair(py, facts.knight_fork_available)?)?,
            royal_fork_available: Py::new(py, PyByColour::pair(py, facts.royal_fork_available)?)?,
            pin_creation_count: Py::new(py, PyByColour::pair(py, facts.pin_creation_count)?)?,
            skewer_creation_available: Py::new(
                py,
                PyByColour::pair(py, facts.skewer_creation_available)?,
            )?,
            discovered_attack_available: Py::new(
                py,
                PyByColour::pair(py, facts.discovered_attack_available)?,
            )?,
            legal_move_count: Py::new(py, PyByColour::pair(py, facts.legal_move_count)?)?,
            safe_check_capturing: Py::new(py, PyByColour::pair(py, facts.safe_check_capturing)?)?,
            discovered_attack_on_queen: Py::new(
                py,
                PyByColour::pair(py, facts.discovered_attack_on_queen)?,
            )?,
            back_rank_mate_threat: Py::new(py, PyByColour::pair(py, facts.back_rank_mate_threat)?)?,
            quiet_threat_available: Py::new(
                py,
                PyByColour::pair(py, facts.quiet_threat_available)?,
            )?,
            no_safe_moves: Py::new(py, PyByColour::pair(py, facts.no_safe_moves)?)?,
            promotion_see_positive: Py::new(
                py,
                PyByColour::pair(py, facts.promotion_see_positive)?,
            )?,
        })
    }
}

#[pymethods]
impl PyTacticsFacts {
    /// Whether the side has a checking move.
    #[getter]
    fn check_available(&self, py: Python<'_>) -> PyResult<PyByColour> {
        PyByColour::pair(py, self.inner.check_available())
    }

    /// Whether it has a checking move with a safe destination.
    #[getter]
    fn safe_check_available(&self, py: Python<'_>) -> PyResult<PyByColour> {
        PyByColour::pair(py, self.inner.safe_check_available())
    }

    /// Whether it has a promotion.
    #[getter]
    fn promotion_available(&self, py: Python<'_>) -> PyResult<PyByColour> {
        PyByColour::pair(py, self.inner.promotion_available())
    }

    /// Whether it has a promotion with a safe destination.
    #[getter]
    fn safe_promotion_available(&self, py: Python<'_>) -> PyResult<PyByColour> {
        PyByColour::pair(py, self.inner.safe_promotion_available())
    }

    /// Whether it has a capture.
    #[getter]
    fn capture_available(&self, py: Python<'_>) -> PyResult<PyByColour> {
        PyByColour::pair(py, self.inner.capture_available())
    }

    /// Whether it has a forking move.
    #[getter]
    fn fork_available(&self, py: Python<'_>) -> PyResult<PyByColour> {
        PyByColour::pair(py, self.inner.fork_available())
    }

    /// Whether it has a move creating a pin.
    #[getter]
    fn pin_creation_available(&self, py: Python<'_>) -> PyResult<PyByColour> {
        PyByColour::pair(py, self.inner.pin_creation_available())
    }

    /// Whether the side has at most two legal moves.
    #[getter]
    fn only_moves(&self, py: Python<'_>) -> PyResult<PyByColour> {
        PyByColour::pair(py, self.inner.only_moves())
    }

    /// The group as plain data, in the one JSON form `docs/features.md`
    /// describes.
    fn to_dict<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        plain(slf.py(), &slf.get().parent.get().inner.tactics)
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        repr_of(slf.as_any(), "TacticsFacts", FIELDS_TACTICS)
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        match other.cast::<Self>() {
            Ok(other) => self.parent.get().inner.tactics == other.get().parent.get().inner.tactics,
            Err(_) => false,
        }
    }

    fn __hash__(&self) -> u64 {
        hash_of(&self.parent.get().inner.tactics)
    }

    fn __reduce__<'py>(slf: &Bound<'py, Self>) -> GroupReduce<'py> {
        let py = slf.py();
        Ok((
            group_reconstructor(py)?,
            (slf.get().parent.clone_ref(py), "tactics"),
        ))
    }
}

/// What one legal move does, beyond what the move itself says. The mover is the side to move; the enemy is the other side.
#[pyclass(frozen, skip_from_py_object, module = "esca", name = "MoveFacts")]
pub struct PyMoveFacts {
    inner: facts::MoveFacts,
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
    /// The origin is attacked by the enemy and the destination is safe.
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
    /// The static exchange evaluation of the move, signed.
    #[pyo3(get)]
    see: i32,
    /// The largest static exchange evaluation of one of the enemy's units after the move; 0 when the move threatens nothing.
    #[pyo3(get)]
    threat_created_max: i32,
    /// The origin is attacked by the enemy.
    #[pyo3(get)]
    moves_attacked_unit: bool,
    /// The destination interposes on the ray of the one unit giving check.
    #[pyo3(get)]
    blocks_check: bool,
    /// The moved unit is a passed pawn.
    #[pyo3(get)]
    advances_passer: bool,
    /// The mover has more passers after the move than before.
    #[pyo3(get)]
    creates_passer: bool,
    /// The mover has more isolated pawns after the move than before.
    #[pyo3(get)]
    creates_isolated: bool,
    /// The mover has more doubled pawns after the move than before.
    #[pyo3(get)]
    creates_doubled: bool,
    /// The mover has more backward pawns after the move than before.
    #[pyo3(get)]
    creates_backward: bool,
    /// One of the enemy king's shield files carried a pawn of the mover's and carries none after.
    #[pyo3(get)]
    opens_file_at_enemy_king: bool,
    /// Change in the number of the mover's units attacking the enemy king ring.
    #[pyo3(get)]
    enemy_king_attackers_delta: i32,
    /// Change in the number of the enemy's units attacking the mover's king ring.
    #[pyo3(get)]
    mover_king_attackers_delta: i32,
    /// Change in the number of the mover's hanging units.
    #[pyo3(get)]
    mover_hanging_delta: i32,
    /// Change in the number of the enemy's hanging units.
    #[pyo3(get)]
    enemy_hanging_delta: i32,
    /// A square carries a hanging unit of the mover's worth 3 or more after the move and carried none before.
    #[pyo3(get)]
    leaves_unit_hanging: bool,
    /// A slider of the mover's that the move leaves standing gains an attack on an enemy unit worth 3 or more.
    #[pyo3(get)]
    gives_discovered_attack: bool,
}

impl PyMoveFacts {
    pub(crate) fn of(_py: Python<'_>, facts: &facts::MoveFacts) -> PyResult<PyMoveFacts> {
        Ok(PyMoveFacts {
            inner: *facts,
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
            see: facts.see,
            threat_created_max: facts.threat_created_max,
            moves_attacked_unit: facts.moves_attacked_unit,
            blocks_check: facts.blocks_check,
            advances_passer: facts.advances_passer,
            creates_passer: facts.creates_passer,
            creates_isolated: facts.creates_isolated,
            creates_doubled: facts.creates_doubled,
            creates_backward: facts.creates_backward,
            opens_file_at_enemy_king: facts.opens_file_at_enemy_king,
            enemy_king_attackers_delta: facts.enemy_king_attackers_delta,
            mover_king_attackers_delta: facts.mover_king_attackers_delta,
            mover_hanging_delta: facts.mover_hanging_delta,
            enemy_hanging_delta: facts.enemy_hanging_delta,
            leaves_unit_hanging: facts.leaves_unit_hanging,
            gives_discovered_attack: facts.gives_discovered_attack,
        })
    }
}

#[pymethods]
impl PyMoveFacts {
    /// What the move does, as plain data, in the one JSON form
    /// `docs/features.md` describes.
    fn to_dict<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        plain(slf.py(), &slf.get().inner)
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        repr_of(slf.as_any(), "MoveFacts", FIELDS_MOVE)
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        match other.cast::<Self>() {
            Ok(other) => self.inner == other.get().inner,
            Err(_) => false,
        }
    }

    fn __hash__(&self) -> u64 {
        hash_of(&self.inner)
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
    fn facts(&self, py: Python<'_>) -> PyResult<PyMoveFacts> {
        PyMoveFacts::of(py, &self.inner.facts)
    }

    fn __repr__(&self) -> String {
        format!("<AnnotatedMove {:?}>", self.inner.mv)
    }
}

/// Every group of facts about one position.
///
/// What each *move* of the position does is a separate answer:
/// `Position.annotated_moves()` and `Game.annotated_moves()`. Pickling
/// re-reads the facts from the position they were computed for, so the
/// history flags a game supplies do not survive a round trip.
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

    /// The colour to move: `"white"` or `"black"`.
    #[getter]
    fn side_to_move(&self) -> String {
        colour_name(self.inner.side_to_move())
    }

    /// The position the facts were computed for.
    #[getter]
    fn position(&self) -> PyResult<super::board::PyPosition> {
        Position::from_fen(&self.text)
            .map(super::board::PyPosition::new)
            .map_err(super::convert::value_error)
    }

    /// The names of the groups, in the order they are declared.
    #[staticmethod]
    fn groups() -> Vec<&'static str> {
        facts::catalogue().iter().map(|group| group.name).collect()
    }

    /// The group of that name.
    fn group<'py>(slf: &Bound<'py, Self>, name: &str) -> PyResult<Bound<'py, PyAny>> {
        if !facts::catalogue().iter().any(|group| group.name == name) {
            return Err(super::convert::value_error(format!(
                "not a facts group: {name:?}"
            )));
        }
        slf.getattr(name)
    }

    /// Every group, by name, as the plain data of the one JSON form
    /// `docs/features.md` describes.
    fn to_dict<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        plain(slf.py(), &slf.get().inner)
    }

    #[getter]
    fn placement(slf: &Bound<'_, Self>) -> PyResult<PyPlacementFacts> {
        PyPlacementFacts::of(slf.py(), &slf.get().inner.placement, slf.clone().unbind())
    }

    #[getter]
    fn state(slf: &Bound<'_, Self>) -> PyResult<PyStateFacts> {
        PyStateFacts::of(slf.py(), &slf.get().inner.state, slf.clone().unbind())
    }

    #[getter]
    fn history(slf: &Bound<'_, Self>) -> PyResult<PyHistoryFacts> {
        PyHistoryFacts::of(slf.py(), &slf.get().inner.history, slf.clone().unbind())
    }

    #[getter]
    fn material(slf: &Bound<'_, Self>) -> PyResult<PyMaterialFacts> {
        PyMaterialFacts::of(slf.py(), &slf.get().inner.material, slf.clone().unbind())
    }

    #[getter]
    fn pawns(slf: &Bound<'_, Self>) -> PyResult<PyPawnFacts> {
        PyPawnFacts::of(slf.py(), &slf.get().inner.pawns, slf.clone().unbind())
    }

    #[getter]
    fn pieces(slf: &Bound<'_, Self>) -> PyResult<PyPieceFacts> {
        PyPieceFacts::of(slf.py(), &slf.get().inner.pieces, slf.clone().unbind())
    }

    #[getter]
    fn king(slf: &Bound<'_, Self>) -> PyResult<PyKingFacts> {
        PyKingFacts::of(slf.py(), &slf.get().inner.king, slf.clone().unbind())
    }

    #[getter]
    fn mobility(slf: &Bound<'_, Self>) -> PyResult<PyMobilityFacts> {
        PyMobilityFacts::of(slf.py(), &slf.get().inner.mobility, slf.clone().unbind())
    }

    #[getter]
    fn attacks(slf: &Bound<'_, Self>) -> PyResult<PyAttackFacts> {
        PyAttackFacts::of(slf.py(), &slf.get().inner.attacks, slf.clone().unbind())
    }

    #[getter]
    fn exchange(slf: &Bound<'_, Self>) -> PyResult<PyExchangeFacts> {
        PyExchangeFacts::of(slf.py(), &slf.get().inner.exchange, slf.clone().unbind())
    }

    #[getter]
    fn threats(slf: &Bound<'_, Self>) -> PyResult<PyThreatFacts> {
        PyThreatFacts::of(slf.py(), &slf.get().inner.threats, slf.clone().unbind())
    }

    #[getter]
    fn tactics(slf: &Bound<'_, Self>) -> PyResult<PyTacticsFacts> {
        PyTacticsFacts::of(slf.py(), &slf.get().inner.tactics, slf.clone().unbind())
    }

    #[getter]
    fn endgame(slf: &Bound<'_, Self>) -> PyResult<PyEndgameFacts> {
        PyEndgameFacts::of(slf.py(), &slf.get().inner.endgame, slf.clone().unbind())
    }

    #[getter]
    fn maps(slf: &Bound<'_, Self>) -> PyResult<PyMapFacts> {
        PyMapFacts::of(slf.py(), &slf.get().inner.maps, slf.clone().unbind())
    }

    /// Material, structure, king safety and threats, for a human reader. Not a
    /// stable format.
    fn summary(&self) -> String {
        self.inner.summary()
    }

    fn __repr__(&self) -> String {
        format!(
            "Facts({:?}, variant={})",
            self.text,
            self.variant.rules().name()
        )
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        match other.cast::<Self>() {
            Ok(other) => {
                let other = other.get();
                self.variant.rules().name() == other.variant.rules().name()
                    && self.inner == other.inner
            }
            Err(_) => false,
        }
    }

    fn __hash__(&self) -> u64 {
        hash_of(&(self.variant.rules().name(), &self.text, &self.inner))
    }

    fn __getnewargs_ex__(&self) -> ((String,), std::collections::HashMap<String, PyVariant>) {
        let mut kwargs = std::collections::HashMap::new();
        kwargs.insert("variant".to_string(), self.variant.clone());
        ((self.text.clone(),), kwargs)
    }
}

/// Every group, field, type and meaning, as data.
///
/// One entry per group: `name`, `meaning` and `fields`, each field carrying its
/// `name`, `type` and `meaning`.
#[pyfunction]
pub(crate) fn facts_catalogue(py: Python<'_>) -> PyResult<Vec<Py<PyDict>>> {
    let mut groups = Vec::new();
    for group in facts::catalogue() {
        let entry = PyDict::new(py);
        entry.set_item("name", group.name)?;
        entry.set_item("meaning", group.meaning())?;
        let fields = Vec::from_iter(group.fields.iter().map(|field| {
            let out = PyDict::new(py);
            out.set_item("name", field.name)?;
            out.set_item("type", field.type_name())?;
            out.set_item("meaning", field.meaning())?;
            PyResult::Ok(out)
        }));
        let fields: PyResult<Vec<_>> = fields.into_iter().collect();
        entry.set_item("fields", fields?)?;
        groups.push(entry.unbind());
    }
    Ok(groups)
}

/// The tensor layout: one entry per fact, in the order the groups declare
/// them.
///
/// Each entry carries the array's `name` (`group.field`), its `dtype` and
/// `shape` with the batch axis left off, an `order` note naming every axis,
/// the `packed_dtype` and `packed_shape` of the packed form, the fact's
/// `meaning`, and the `kind` tree of the type it was declared with.
#[pyfunction]
pub(crate) fn facts_layout(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
    let entries: Vec<serde_json::Value> = facts::tensors::layout()
        .entries
        .iter()
        .map(facts::tensors::Entry::to_json)
        .collect();
    let text = serde_json::to_string(&entries).map_err(super::convert::value_error)?;
    py.import("json")?.call_method1("loads", (text,))
}

/// The fields of `MoveFacts`, each with its type and meaning.
#[pyfunction]
pub(crate) fn move_facts_catalogue(py: Python<'_>) -> PyResult<Vec<Py<PyDict>>> {
    let mut fields = Vec::new();
    for field in facts::MoveFacts::FIELDS {
        let out = PyDict::new(py);
        out.set_item("name", field.name)?;
        out.set_item("type", field.type_name())?;
        out.set_item("meaning", field.meaning())?;
        fields.push(out.unbind());
    }
    Ok(fields)
}

/// The field names of each group, in declaration order.
const FIELDS_PLACEMENT: &[&str] = &[
    "units", "pawns", "knights", "bishops", "rooks", "queens", "kings",
];
const FIELDS_STATE: &[&str] = &[
    "in_check",
    "double_check",
    "can_castle_short",
    "can_castle_long",
    "en_passant",
    "en_passant_capture_legal",
];
const FIELDS_HISTORY: &[&str] = &[
    "known",
    "halfmove_clock",
    "halfmove_known",
    "repetition_seen",
    "repetition_available",
    "captures_in_last_8",
    "checks_in_last_8",
    "quiet_plies",
    "material_trend",
    "last_move_victim",
    "last_move_mover",
];
const FIELDS_MATERIAL: &[&str] = &[
    "pawns",
    "knights",
    "bishops",
    "rooks",
    "queens",
    "non_pawn_value",
    "value",
    "phase_points",
    "both_queens",
    "pawns_only",
    "insufficient_material",
];
const FIELDS_PAWNS: &[&str] = &[
    "pawns",
    "passed",
    "candidates",
    "doubled",
    "isolated",
    "backward",
    "defended",
    "count_by_file",
    "count_by_rank",
    "open_files",
    "semi_open_files",
    "islands",
    "levers",
    "rams",
    "passer_lead_rank",
    "passer_protected",
    "passers_connected",
    "passer_unstoppable",
    "chain_max_length",
    "chain_base_attacked",
    "queenside_majority",
    "kingside_majority",
    "holes",
    "holes_occupied",
    "fixed_pawns",
    "blocked_passers",
    "passer_distance",
    "passer_own_king_distance",
    "passer_enemy_king_distance",
    "passer_in_square",
    "passer_free_path",
    "half_open_at_enemy_king",
    "backward_on_semi_open",
];
const FIELDS_PIECES: &[&str] = &[
    "bishop_pair",
    "knight_pair",
    "bishops_light",
    "bishops_dark",
    "opposite_coloured_bishops",
    "pawns_on_bishop_colour",
    "rooks_connected_rank",
    "rooks_connected_file",
    "rooks_on_open_file",
    "rooks_on_semi_open_file",
    "rooks_on_seventh_rank",
    "rook_behind_own_passer",
    "rook_behind_enemy_passer",
    "trapped_rook",
    "outposts",
    "minors_on_outpost",
    "outpost_squares_free",
    "knights_on_rim",
    "minors_undeveloped",
    "queen_developed",
    "fixed_pawns_on_bishop_colour",
    "rook_on_seventh_with_king_on_eighth",
    "trapped_pieces",
    "trapped_value",
];
const FIELDS_KING: &[&str] = &[
    "square",
    "on_home_square",
    "castled_queenside",
    "castled_kingside",
    "shield_files",
    "shield",
    "shield_file_open",
    "shield_file_semi_open_for_enemy",
    "storm",
    "ring",
    "ring_attackers",
    "ring_attack_weight",
    "ring_defended",
    "ring_holes",
    "escape_squares",
    "back_rank_risk",
    "distance",
    "tropism",
    "virtual_mobility",
    "ring_defenders",
    "ring_defence_weight",
    "open_rays",
    "luft",
    "castled_side",
    "opposite_side_castling",
];
const FIELDS_MOBILITY: &[&str] = &[
    "pawns",
    "knights",
    "bishops",
    "rooks",
    "queens",
    "safe_pawns",
    "safe_knights",
    "safe_bishops",
    "safe_rooks",
    "safe_queens",
    "total",
    "space",
    "centre_control",
    "extended_centre_control",
    "immobile_pieces",
];
const FIELDS_ATTACKS: &[&str] = &[
    "controlled",
    "by_pawns",
    "by_knights",
    "by_bishops",
    "by_rooks",
    "by_queens",
    "by_kings",
    "attacked",
    "hanging",
    "en_prise",
    "pinned",
    "defended",
    "attacked_value",
    "hanging_value",
    "en_prise_value",
    "en_prise_max_value",
    "pinned_value",
    "skewer_candidates",
];
const FIELDS_EXCHANGE: &[&str] = &[
    "see_best_capture",
    "see_positive_capture_count",
    "see_equal_capture_count",
    "see_positive_total",
];
const FIELDS_THREATS: &[&str] = &[
    "threatened",
    "threatened_value",
    "threat_max_gain",
    "attacked_by_lesser",
    "queen_attacked_by_lesser",
    "overloaded_defenders",
    "removable_defenders",
    "loose",
    "attacker_surplus",
    "xray_through_enemy",
    "battery_count",
    "battery_at_king",
];
const FIELDS_MAPS: &[&str] = &["controlled", "by_pawns", "hanging", "pinned", "threatened"];
const FIELDS_ENDGAME: &[&str] = &[
    "king_centralisation",
    "race_plies",
    "opposition",
    "key_square_occupied",
    "wrong_colour_bishop",
    "drawish_material",
];
const FIELDS_TACTICS: &[&str] = &[
    "available",
    "check_count",
    "check_by_pawn",
    "check_by_knight",
    "check_by_bishop",
    "check_by_rook",
    "check_by_queen",
    "safe_check_count",
    "safe_check_by_pawn",
    "safe_check_by_knight",
    "safe_check_by_bishop",
    "safe_check_by_rook",
    "safe_check_by_queen",
    "double_check_available",
    "discovered_check_available",
    "mate_in_1",
    "stalemate_in_1",
    "promotion_files",
    "promotes_to_queen",
    "promotes_to_rook",
    "promotes_to_bishop",
    "promotes_to_knight",
    "safe_promotion_files",
    "capture_count",
    "winning_capture_available",
    "winning_capture_max_gain",
    "captures_hanging",
    "hanging_victim_max_value",
    "equal_capture_count",
    "losing_capture_count",
    "fork_count",
    "fork_max_value",
    "knight_fork_available",
    "royal_fork_available",
    "pin_creation_count",
    "skewer_creation_available",
    "discovered_attack_available",
    "legal_move_count",
    "safe_check_capturing",
    "discovered_attack_on_queen",
    "back_rank_mate_threat",
    "quiet_threat_available",
    "no_safe_moves",
    "promotion_see_positive",
];
const FIELDS_MOVE: &[&str] = &[
    "victim",
    "mover",
    "promotion",
    "gives_check",
    "gives_safe_check",
    "is_safe",
    "captures_hanging",
    "escapes_attack",
    "to_attacked_by_pawn",
    "is_castling",
    "is_en_passant",
    "see",
    "threat_created_max",
    "moves_attacked_unit",
    "blocks_check",
    "advances_passer",
    "creates_passer",
    "creates_isolated",
    "creates_doubled",
    "creates_backward",
    "opens_file_at_enemy_king",
    "enemy_king_attackers_delta",
    "mover_king_attackers_delta",
    "mover_hanging_delta",
    "enemy_hanging_delta",
    "leaves_unit_hanging",
    "gives_discovered_attack",
];
