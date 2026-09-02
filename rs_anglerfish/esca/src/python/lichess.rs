//! Batches of encoded positions, streamed from the evaluation dump.

use std::io;
use std::path::PathBuf;
use std::sync::Mutex;

use numpy::ndarray::Array2;
use numpy::{IntoPyArray, PyArray1, PyArray2};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::lichess::{self, Record};
use crate::moves::Move;
use crate::position::{Position, Score};
use crate::schema::{GroupSet, Schema};

use super::board::{PyMove, PyVariant};
use super::encode::{PySchema, encode_position_rows, group_set};

/// One batch of positions: their text, their features and their targets.
#[pyclass(frozen, module = "esca", name = "Batch")]
pub struct PyBatch {
    /// The four-field FEN of each row.
    #[pyo3(get)]
    fens: Vec<String>,
    /// The feature rows, `(n, width)` float32.
    #[pyo3(get)]
    features: Py<PyArray2<f32>>,
    /// The centipawn score of each row, side-relative; 0.0 where `mate` is not.
    #[pyo3(get)]
    cp: Py<PyArray1<f32>>,
    /// Moves to a forced mate, side-relative; 0.0 where the row is a `cp` row.
    #[pyo3(get)]
    mate: Py<PyArray1<f32>>,
    /// The first move of each row's best line.
    #[pyo3(get)]
    best_moves: Vec<PyMove>,
}

#[pymethods]
impl PyBatch {
    fn __len__(&self) -> usize {
        self.fens.len()
    }

    fn __repr__(&self) -> String {
        format!("<Batch of {}>", self.fens.len())
    }
}

/// What one gathered row carries before it reaches Python.
struct Row {
    fen: String,
    cp: f32,
    mate: f32,
    best: Move,
}

/// A gathered batch, before it becomes Python objects.
struct Gathered {
    rows: Vec<Row>,
    features: Vec<f32>,
    width: usize,
}

/// The iterator `batches` returns.
#[pyclass(module = "esca", name = "Batches")]
pub struct PyBatches {
    // A `Mutex` for the `Sync` a pyclass needs; the reader itself is only
    // touched through `&mut self`.
    records: Mutex<Box<dyn Iterator<Item = io::Result<Record>> + Send>>,
    batch_size: usize,
    min_depth: u32,
    variant: PyVariant,
    schema: &'static Schema,
    groups: GroupSet,
}

impl PyBatches {
    /// The next batch's rows and their encoded features, or `None` at the end
    /// of the dump.
    fn gather(&mut self) -> io::Result<Option<Gathered>> {
        let mut rows = Vec::with_capacity(self.batch_size);
        let mut positions = Vec::with_capacity(self.batch_size);
        while rows.len() < self.batch_size {
            let Some(record) = self
                .records
                .get_mut()
                .expect("the reader is not poisoned")
                .next()
            else {
                break;
            };
            let record = record?;
            if let Some((row, position)) = row_of(record, self.min_depth, &self.variant) {
                rows.push(row);
                positions.push(position);
            }
        }
        if rows.is_empty() {
            return Ok(None);
        }
        let width = self.schema.width_of(self.groups);
        let mut features = vec![0.0f32; rows.len() * width];
        encode_position_rows(
            self.variant.rules(),
            &positions,
            self.schema,
            self.groups,
            &mut features,
        );
        Ok(Some(Gathered {
            rows,
            features,
            width,
        }))
    }
}

/// The deepest evaluation of `record` that reaches `min_depth`, as a row.
fn row_of(record: Record, min_depth: u32, variant: &PyVariant) -> Option<(Row, Position)> {
    let eval = record
        .evals
        .iter()
        .filter(|eval| eval.depth >= min_depth)
        .max_by_key(|eval| eval.depth)?;
    let pv = eval.pvs.first()?;
    let position = record.position().ok()?;
    let best = pv.best_move(variant.rules(), &position).ok()?;
    let (cp, mate) = match pv.score {
        Score::Cp(value) => (value as f32, 0.0),
        Score::Mate(moves) => (0.0, moves as f32),
    };
    Some((
        Row {
            fen: record.epd,
            cp,
            mate,
            best,
        },
        position,
    ))
}

#[pymethods]
impl PyBatches {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<Option<PyBatch>> {
        let py = slf.py();
        let state = &mut *slf;
        let gathered = py
            .detach(|| state.gather())
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let Some(Gathered {
            rows,
            features,
            width,
        }) = gathered
        else {
            return Ok(None);
        };
        let features = Array2::from_shape_vec((rows.len(), width), features)
            .expect("the buffer is rows by width")
            .into_pyarray(py)
            .unbind();
        let cp: Vec<f32> = rows.iter().map(|row| row.cp).collect();
        let mate: Vec<f32> = rows.iter().map(|row| row.mate).collect();
        let best_moves = rows.iter().map(|row| PyMove::new(row.best)).collect();
        let fens = rows.into_iter().map(|row| row.fen).collect();
        Ok(Some(PyBatch {
            fens,
            features,
            cp: cp.into_pyarray(py).unbind(),
            mate: mate.into_pyarray(py).unbind(),
            best_moves,
        }))
    }

    fn __repr__(&self) -> String {
        format!("<Batches of {}>", self.batch_size)
    }
}

/// Streams the Zstandard-compressed dump at `path` as batches of encoded
/// positions.
///
/// A record is skipped when no evaluation reaches `min_depth`, when its
/// placement is not one a game can reach, or when its best line names no legal
/// move.
#[pyfunction]
#[pyo3(signature = (path, *, batch_size = 8192, min_depth = 0, variant = None, schema = None, groups = None))]
pub(crate) fn batches(
    path: PathBuf,
    batch_size: usize,
    min_depth: u32,
    variant: Option<PyVariant>,
    schema: Option<PySchema>,
    groups: Option<Vec<String>>,
) -> PyResult<PyBatches> {
    if batch_size == 0 {
        return Err(PyValueError::new_err("a batch holds at least one row"));
    }
    let variant = variant.unwrap_or_else(super::default_variant);
    let schema = schema.unwrap_or_else(super::default_schema).inner;
    let groups = group_set(schema, groups)?;
    let records = lichess::read(&path)?;
    Ok(PyBatches {
        records: Mutex::new(Box::new(records)),
        batch_size,
        min_depth,
        variant,
        schema,
        groups,
    })
}
