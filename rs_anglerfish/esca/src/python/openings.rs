//! The bundled ECO catalogue.

use pyo3::prelude::*;

use crate::openings::{self, Opening};

use super::board::PyPosition;

/// An ECO code and the name that goes with it.
#[pyclass(
    frozen,
    eq,
    hash,
    from_py_object,
    module = "esca.openings",
    name = "Opening"
)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PyOpening {
    inner: Opening,
}

impl PyOpening {
    pub(crate) fn new(inner: Opening) -> PyOpening {
        PyOpening { inner }
    }
}

#[pymethods]
impl PyOpening {
    /// The ECO classification: a volume letter A to E and two digits.
    #[getter]
    fn eco(&self) -> &'static str {
        self.inner.eco
    }

    /// The name, in English.
    #[getter]
    fn name(&self) -> &'static str {
        self.inner.name
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!("<Opening {}>", self.inner)
    }
}

/// The opening `position` is named after, if it has a name.
#[pyfunction]
pub(crate) fn openings_lookup(position: &PyPosition) -> Option<PyOpening> {
    openings::lookup(&position.inner).map(PyOpening::new)
}

/// How many named positions the catalogue holds.
#[pyfunction]
pub(crate) fn openings_count() -> usize {
    openings::count()
}

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyOpening>()?;
    module.add_function(wrap_pyfunction!(openings_lookup, module)?)?;
    module.add_function(wrap_pyfunction!(openings_count, module)?)?;
    Ok(())
}
