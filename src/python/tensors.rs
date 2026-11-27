//! The arrays of `esca.tensors`, filled from each fact's own value.

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyByteArray, PyString};

use crate::facts::Scratch;
use crate::facts::tensors::Batch;
use crate::position::Position;

use super::board::{PyGame, PyPosition, PyVariant};
use super::errors::fen_error;
use super::facts::{PyAnnotatedMove, PyFacts, PyMoveFacts};

/// One array: its name, its little-endian bytes, its dtype and its shape,
/// batch axis included.
type Array<'py> = (&'static str, Bound<'py, PyByteArray>, String, Vec<usize>);

/// The arrays of `batch` named in `names`, packed first where asked, in the
/// order of `names`.
fn arrays<'py>(
    py: Python<'py>,
    batch: Batch,
    names: &[String],
    packed: bool,
) -> PyResult<Vec<Array<'py>>> {
    let batch = if packed { batch.packed() } else { batch };
    let mut out = Vec::with_capacity(names.len());
    for name in names {
        let (entry, buffer) = batch
            .entries()
            .iter()
            .zip(batch.buffers())
            .find(|(entry, _)| entry.name == name)
            .ok_or_else(|| super::convert::value_error(format!("not an array: {name}")))?;
        out.push((
            entry.name,
            PyByteArray::new(py, &buffer.bytes()),
            buffer.dtype().to_string(),
            batch.shape_of(entry),
        ));
    }
    Ok(out)
}

/// The position facts of every item: a FEN, a `Position`, a `Game` or a
/// `Facts`. `variant` is the rules a FEN or a `Position` is read under.
#[pyfunction]
#[pyo3(name = "_facts_tensors", signature = (positions, *, names, packed, variant = None))]
pub(crate) fn facts_tensors<'py>(
    py: Python<'py>,
    positions: &Bound<'py, PyAny>,
    names: Vec<String>,
    packed: bool,
    variant: Option<PyVariant>,
) -> PyResult<Vec<Array<'py>>> {
    let variant = variant.unwrap_or_else(super::default_variant);
    let mut scratch = Scratch::new();
    let mut batch = Batch::of_positions();
    for item in positions.try_iter()? {
        let item = item?;
        if let Ok(facts) = item.cast::<PyFacts>() {
            batch.push_facts(&facts.get().inner);
        } else if let Ok(text) = item.cast::<PyString>() {
            let fen = text.to_str()?;
            let position = Position::from_fen(fen).map_err(|error| fen_error(error, fen))?;
            batch.push_facts(&position.facts_in(variant.rules(), &mut scratch));
        } else if let Ok(position) = item.cast::<PyPosition>() {
            let facts = position
                .borrow()
                .inner
                .facts_in(variant.rules(), &mut scratch);
            batch.push_facts(&facts);
        } else if let Ok(game) = item.cast::<PyGame>() {
            batch.push_facts(&game.borrow().inner.facts());
        } else {
            return Err(PyTypeError::new_err(format!(
                "not a position, game or facts: {}",
                item.repr()?
            )));
        }
    }
    arrays(py, batch, &names, packed)
}

/// The move facts of every item: an `AnnotatedMove` or a `MoveFacts`.
#[pyfunction]
#[pyo3(name = "_move_tensors", signature = (moves, *, names, packed))]
pub(crate) fn move_tensors<'py>(
    py: Python<'py>,
    moves: &Bound<'py, PyAny>,
    names: Vec<String>,
    packed: bool,
) -> PyResult<Vec<Array<'py>>> {
    let mut batch = Batch::of_moves();
    for item in moves.try_iter()? {
        let item = item?;
        if let Ok(annotated) = item.cast::<PyAnnotatedMove>() {
            batch.push_move_facts(&annotated.get().inner.facts);
        } else if let Ok(facts) = item.cast::<PyMoveFacts>() {
            batch.push_move_facts(&facts.get().inner);
        } else {
            return Err(PyTypeError::new_err(format!(
                "not an annotated move or move facts: {}",
                item.repr()?
            )));
        }
    }
    arrays(py, batch, &names, packed)
}
