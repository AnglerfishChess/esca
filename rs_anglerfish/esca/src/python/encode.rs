//! The schema, and the batch encoders that write `f32` rows.

use numpy::ndarray::Array2;
use numpy::{IntoPyArray, PyArray2, PyArrayMethods, PyUntypedArrayMethods};
use pyo3::IntoPyObject;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use rayon::prelude::*;

use crate::facts::{MoveFacts, RowError, Scratch};
use crate::position::Position;
use crate::schema::{GroupSet, Schema};
use crate::variant::Variant;

use super::board::{PyMove, PyVariant};
use super::convert::value_error;

/// The versioned contract between the extractor and the net.
#[pyclass(frozen, eq, hash, from_py_object, module = "esca", name = "Schema")]
#[derive(Clone, Copy)]
pub struct PySchema {
    pub(crate) inner: &'static Schema,
}

impl PySchema {
    pub(crate) fn new(inner: &'static Schema) -> PySchema {
        PySchema { inner }
    }
}

impl PartialEq for PySchema {
    fn eq(&self, other: &PySchema) -> bool {
        self.inner.id() == other.inner.id()
    }
}

impl Eq for PySchema {}

impl std::hash::Hash for PySchema {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.id().bytes().hash(state);
    }
}

#[pymethods]
impl PySchema {
    /// The 32 hex digits of the schema's id.
    #[getter]
    fn id(&self) -> String {
        self.inner.id().to_string()
    }

    /// The schema's semantic version.
    #[getter]
    fn semver(&self) -> &'static str {
        self.inner.semver()
    }

    /// The width of every group together.
    #[getter]
    fn width(&self) -> usize {
        self.inner.width()
    }

    /// How many features the schema names.
    #[getter]
    fn feature_count(&self) -> usize {
        self.inner.feature_count()
    }

    /// The group names, in the order they are written.
    #[getter]
    fn group_names(&self) -> Vec<&'static str> {
        self.inner.groups().iter().map(|group| group.name).collect()
    }

    /// The groups, each as `{"name", "version", "width", "offset"}`.
    fn groups<'py>(&self, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyDict>>> {
        let mut out = Vec::with_capacity(self.inner.groups().len());
        let mut offset = 0usize;
        for group in self.inner.groups() {
            let entry = PyDict::new(py);
            entry.set_item("name", group.name)?;
            entry.set_item("version", group.version)?;
            entry.set_item("width", group.width)?;
            entry.set_item("offset", offset)?;
            offset += group.width;
            out.push(entry);
        }
        Ok(out)
    }

    /// The width of the named groups; every group when `groups` is `None`.
    #[pyo3(signature = (groups = None))]
    fn width_of(&self, groups: Option<Vec<String>>) -> PyResult<usize> {
        Ok(self.inner.width_of(group_set(self.inner, groups)?))
    }

    /// The features whose definitions hold under `variant`, as group and
    /// feature names.
    fn features_for(&self, variant: &PyVariant) -> Vec<(&'static str, &'static str)> {
        self.inner.features_for(variant.rules()).names().collect()
    }

    /// The canonical text the id hashes.
    fn canonical(&self) -> String {
        self.inner.canonical()
    }

    fn __repr__(&self) -> String {
        format!("<Schema {} {}>", self.inner.semver(), self.inner.id())
    }
}

/// The named groups of `schema`; every group when `names` is `None`.
pub(crate) fn group_set(schema: &Schema, names: Option<Vec<String>>) -> PyResult<GroupSet> {
    let Some(names) = names else {
        return Ok(schema.all());
    };
    let borrowed: Vec<&str> = names.iter().map(String::as_str).collect();
    schema.group_set(&borrowed).ok_or_else(|| {
        PyValueError::new_err(format!(
            "not all of {names:?} are groups of the schema: {:?}",
            schema
                .groups()
                .iter()
                .map(|group| group.name)
                .collect::<Vec<_>>()
        ))
    })
}

/// Writes one row per FEN, row-major, into `out`; the first row that is not a
/// position stops nothing but is reported.
pub(crate) fn encode_rows(
    variant: &dyn Variant,
    fens: &[String],
    schema: &'static Schema,
    groups: GroupSet,
    out: &mut [f32],
) -> Option<RowError> {
    let width = schema.width_of(groups);
    if width == 0 || fens.is_empty() {
        return None;
    }
    out.par_chunks_mut(width)
        .enumerate()
        .map_init(Scratch::new, |scratch, (row, chunk)| {
            let position =
                Position::from_fen(&fens[row]).map_err(|source| RowError { row, source })?;
            let facts = position.facts_in(variant, scratch);
            facts.encode_into(schema, groups, chunk);
            Ok(())
        })
        .filter_map(|row: Result<(), RowError>| row.err())
        .min_by_key(|error| error.row)
}

/// Writes one row per position, row-major, into `out`.
pub(crate) fn encode_position_rows(
    variant: &dyn Variant,
    positions: &[Position],
    schema: &'static Schema,
    groups: GroupSet,
    out: &mut [f32],
) {
    let width = schema.width_of(groups);
    if width == 0 || positions.is_empty() {
        return;
    }
    out.par_chunks_mut(width)
        .enumerate()
        .for_each_init(Scratch::new, |scratch, (row, chunk)| {
            let facts = positions[row].facts_in(variant, scratch);
            facts.encode_into(schema, groups, chunk);
        });
}

/// The feature rows of `fens`, as an `(n, width)` float32 array.
#[pyfunction]
#[pyo3(signature = (fens, *, variant = None, schema = None, groups = None))]
pub(crate) fn encode(
    py: Python<'_>,
    fens: Vec<String>,
    variant: Option<PyVariant>,
    schema: Option<PySchema>,
    groups: Option<Vec<String>>,
) -> PyResult<Bound<'_, PyArray2<f32>>> {
    let variant = variant.unwrap_or_else(super::default_variant);
    let schema = schema.unwrap_or_else(super::default_schema).inner;
    let set = group_set(schema, groups)?;
    let width = schema.width_of(set);
    let rows = fens.len();
    let mut data = vec![0.0f32; rows * width];
    let rules = variant.rules();
    if let Some(error) = py.detach(|| encode_rows(rules, &fens, schema, set, &mut data)) {
        return Err(value_error(error));
    }
    let array = Array2::from_shape_vec((rows, width), data).expect("the buffer is rows by width");
    Ok(array.into_pyarray(py))
}

/// The same, into the caller's C-contiguous `(n, width)` float32 array.
#[pyfunction]
#[pyo3(signature = (fens, out, *, variant = None, schema = None, groups = None))]
pub(crate) fn encode_into(
    py: Python<'_>,
    fens: Vec<String>,
    out: &Bound<'_, PyArray2<f32>>,
    variant: Option<PyVariant>,
    schema: Option<PySchema>,
    groups: Option<Vec<String>>,
) -> PyResult<()> {
    let variant = variant.unwrap_or_else(super::default_variant);
    let schema = schema.unwrap_or_else(super::default_schema).inner;
    let set = group_set(schema, groups)?;
    let width = schema.width_of(set);
    let shape = out.shape();
    if shape != [fens.len(), width] {
        return Err(PyValueError::new_err(format!(
            "the output is {shape:?}, not {:?}",
            [fens.len(), width]
        )));
    }
    let mut array = out.readwrite();
    let slice = array
        .as_slice_mut()
        .map_err(|_| PyValueError::new_err("the output is not C-contiguous"))?;
    let rules = variant.rules();
    match py.detach(|| encode_rows(rules, &fens, schema, set, slice)) {
        Some(error) => Err(value_error(error)),
        None => Ok(()),
    }
}

/// One position's legal moves and the values of their rows, row-major.
fn move_rows(
    position: &Position,
    variant: &dyn Variant,
    scratch: &mut Scratch,
) -> (Vec<PyMove>, Vec<f32>) {
    let facts = position.facts_in(variant, scratch);
    let mut data = vec![0.0f32; facts.moves.len() * MoveFacts::WIDTH];
    let mut moves = Vec::with_capacity(facts.moves.len());
    for (row, annotated) in facts.moves.iter().enumerate() {
        moves.push(PyMove::new(annotated.mv));
        annotated
            .facts
            .encode_into(&mut data[row * MoveFacts::WIDTH..(row + 1) * MoveFacts::WIDTH]);
    }
    (moves, data)
}

/// One position's moves and their row values, or why its FEN was not read.
type MoveRows = Result<(Vec<PyMove>, Vec<f32>), RowError>;

/// The legal moves of one FEN, or of a sequence of them, and their `(m, 24)`
/// float32 rows.
///
/// For one FEN the result is `(moves, rows)`. For a sequence it is
/// `(moves, rows, offsets)`: a move list per FEN, every position's rows stacked
/// into one `(total, 24)` array, and the `(n + 1,)` int64 bounds that cut it,
/// so FEN `i` owns `rows[offsets[i]:offsets[i + 1]]`.
#[pyfunction]
#[pyo3(signature = (fens, *, variant = None))]
pub(crate) fn encode_moves<'py>(
    py: Python<'py>,
    fens: &Bound<'py, PyAny>,
    variant: Option<PyVariant>,
) -> PyResult<Bound<'py, PyAny>> {
    let variant = variant.unwrap_or_else(super::default_variant);
    let rules = variant.rules();

    if let Ok(fen) = fens.extract::<String>() {
        let position = Position::from_fen(&fen).map_err(value_error)?;
        let (moves, data) = py.detach(|| move_rows(&position, rules, &mut Scratch::new()));
        let rows = moves.len();
        let array = Array2::from_shape_vec((rows, MoveFacts::WIDTH), data)
            .expect("the buffer is rows by width");
        let out = (moves, array.into_pyarray(py)).into_pyobject(py)?;
        return Ok(out.into_any());
    }

    let fens: Vec<String> = fens.extract()?;
    let encoded: Vec<MoveRows> = py.detach(|| {
        fens.par_iter()
            .enumerate()
            .map_init(Scratch::new, |scratch, (row, fen)| {
                let position =
                    Position::from_fen(fen).map_err(|source| RowError { row, source })?;
                Ok(move_rows(&position, rules, scratch))
            })
            .collect()
    });

    let mut moves = Vec::with_capacity(fens.len());
    let mut offsets = Vec::with_capacity(fens.len() + 1);
    let mut data: Vec<f32> = Vec::new();
    let mut bound = 0i64;
    offsets.push(bound);
    for row in encoded {
        let (row_moves, row_data) = row.map_err(value_error)?;
        bound += row_moves.len() as i64;
        offsets.push(bound);
        data.extend_from_slice(&row_data);
        moves.push(row_moves);
    }
    let total = bound as usize;
    let array = Array2::from_shape_vec((total, MoveFacts::WIDTH), data)
        .expect("the buffer is rows by width");
    let out = (moves, array.into_pyarray(py), offsets.into_pyarray(py)).into_pyobject(py)?;
    Ok(out.into_any())
}

/// The features whose definitions hold under `variant`.
#[pyfunction]
#[pyo3(signature = (variant, *, schema = None))]
pub(crate) fn features_for(
    variant: &PyVariant,
    schema: Option<PySchema>,
) -> Vec<(&'static str, &'static str)> {
    let schema = schema.unwrap_or_else(super::default_schema);
    schema.inner.features_for(variant.rules()).names().collect()
}

/// The groups of the v0 schema, each as `{"name", "version", "width",
/// "offset"}`.
#[pyfunction]
pub(crate) fn schema(py: Python<'_>) -> PyResult<Vec<Bound<'_, PyDict>>> {
    super::default_schema().groups(py)
}
