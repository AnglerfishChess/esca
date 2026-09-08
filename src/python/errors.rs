//! The exceptions the Python surface raises, one per error the crate returns.
//!
//! Every one of them is both an `esca.Error` and a `ValueError`, so a caller
//! may catch the whole family, one member of it, or keep the `except
//! ValueError` it already had.

use pyo3::prelude::*;
use pyo3::sync::PyOnceLock;
use pyo3::types::{PyDict, PyTuple, PyType};

/// The five classes, built once and kept for the life of the interpreter.
struct Errors {
    base: Py<PyType>,
    fen: Py<PyType>,
    illegal_move: Py<PyType>,
    move_parse: Py<PyType>,
    position: Py<PyType>,
    parse: Py<PyType>,
}

static ERRORS: PyOnceLock<Errors> = PyOnceLock::new();

/// The class `name`, deriving from `bases`, documented by `doc`.
fn class<'py>(
    py: Python<'py>,
    name: &str,
    bases: &[&Bound<'py, PyType>],
    doc: &str,
) -> PyResult<Py<PyType>> {
    let dict = PyDict::new(py);
    dict.set_item("__doc__", doc)?;
    dict.set_item("__module__", "esca")?;
    let bases = PyTuple::new(py, bases)?;
    let built = py.get_type::<PyType>().call1((name, bases, dict))?;
    Ok(built.cast::<PyType>()?.clone().unbind())
}

fn built(py: Python<'_>) -> &Errors {
    ERRORS.get_or_init(py, || {
        let exception = py.get_type::<pyo3::exceptions::PyException>();
        let value_error = py.get_type::<pyo3::exceptions::PyValueError>();
        let base = class(
            py,
            "Error",
            &[&exception],
            "Every error esca raises. Its members are `ValueError`s too.",
        )
        .expect("the base class is buildable");
        let derived = |name: &str, doc: &str| {
            let base = base.bind(py);
            class(py, name, &[base, &value_error], doc).expect("an error class is buildable")
        };
        Errors {
            fen: derived("FenError", "The text is not a FEN this crate can read."),
            illegal_move: derived("IllegalMove", "The move is not legal in the position."),
            move_parse: derived(
                "MoveParseError",
                "The text does not name one legal move of the position.",
            ),
            position: derived(
                "PositionError",
                "The position is not one this variant can play on.",
            ),
            parse: derived(
                "ParseError",
                "The text does not name a square, file, rank, role or colour.",
            ),
            base,
        }
    })
}

/// Adds the five classes to the module.
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = module.py();
    let errors = built(py);
    module.add("Error", errors.base.clone_ref(py))?;
    module.add("FenError", errors.fen.clone_ref(py))?;
    module.add("IllegalMove", errors.illegal_move.clone_ref(py))?;
    module.add("MoveParseError", errors.move_parse.clone_ref(py))?;
    module.add("PositionError", errors.position.clone_ref(py))?;
    module.add("ParseError", errors.parse.clone_ref(py))?;
    Ok(())
}

fn raised(class: impl Fn(&Errors) -> &Py<PyType>, message: String) -> PyErr {
    Python::attach(|py| PyErr::from_type(class(built(py)).bind(py).clone(), message))
}

/// `esca.FenError`, naming the text that is not a FEN.
pub(crate) fn fen_error(error: impl std::fmt::Display, fen: &str) -> PyErr {
    raised(|errors| &errors.fen, format!("{error}: {fen:?}"))
}

/// `esca.IllegalMove`, naming the move and the position that refuses it.
pub(crate) fn illegal_move(mv: &str, fen: &str) -> PyErr {
    raised(
        |errors| &errors.illegal_move,
        format!("{mv} is not legal in {fen}"),
    )
}

/// `esca.MoveParseError`, naming the text and the position it was read in.
pub(crate) fn move_parse_error(error: impl std::fmt::Display, text: &str, fen: &str) -> PyErr {
    raised(
        |errors| &errors.move_parse,
        format!("{error}: {text:?} in {fen}"),
    )
}

/// `esca.PositionError`, naming the position the variant refuses.
pub(crate) fn position_error(error: impl std::fmt::Display, fen: &str) -> PyErr {
    raised(|errors| &errors.position, format!("{error}: {fen}"))
}

/// `esca.ParseError`, naming what was expected and what was given.
pub(crate) fn parse_error(expected: &str, text: &str) -> PyErr {
    raised(
        |errors| &errors.parse,
        format!("not a {expected}: {text:?}"),
    )
}
