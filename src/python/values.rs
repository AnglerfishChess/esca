//! The Python spellings of the values a fact can hold: a pair by colour, a set
//! of files, and the plain-data form every one of them takes in `to_dict`.

use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};

use crate::types::{ByColour, Colour, FileSet};

use super::board::PySquareSet;
use super::convert::{colour_from, colour_name, file_from, file_name};

/// One value for White and one for Black.
///
/// Read a side by name — `.white`, `.black` — or by colour, with `of("white")`
/// or `["w"]`.
#[pyclass(frozen, module = "esca", name = "ByColour")]
pub struct PyByColour {
    /// White's value.
    #[pyo3(get)]
    white: Py<PyAny>,
    /// Black's value.
    #[pyo3(get)]
    black: Py<PyAny>,
}

impl PyByColour {
    /// The pair of `values`, each converted to its Python form.
    pub(crate) fn pair<'py, T>(py: Python<'py>, values: ByColour<T>) -> PyResult<PyByColour>
    where
        T: IntoPyObject<'py>,
    {
        Ok(PyByColour {
            white: values.white.into_py_any(py)?,
            black: values.black.into_py_any(py)?,
        })
    }
}

#[pymethods]
impl PyByColour {
    #[new]
    fn py_new(white: Py<PyAny>, black: Py<PyAny>) -> PyByColour {
        PyByColour { white, black }
    }

    /// The value of `colour`, named `"white"`, `"black"`, `"w"` or `"b"`.
    fn of(&self, py: Python<'_>, colour: &str) -> PyResult<Py<PyAny>> {
        let colour = colour_from(colour)?;
        Ok(match colour {
            Colour::White => self.white.clone_ref(py),
            Colour::Black => self.black.clone_ref(py),
        })
    }

    fn __getitem__(&self, py: Python<'_>, colour: &str) -> PyResult<Py<PyAny>> {
        self.of(py, colour)
    }

    fn __eq__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        let Ok(other) = other.cast::<PyByColour>() else {
            return Ok(false);
        };
        let other = other.get();
        Ok(self.white.bind(py).eq(other.white.bind(py))?
            && self.black.bind(py).eq(other.black.bind(py))?)
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        Ok(format!(
            "ByColour(white={}, black={})",
            self.white.bind(py).repr()?,
            self.black.bind(py).repr()?
        ))
    }

    fn __getnewargs__(&self, py: Python<'_>) -> (Py<PyAny>, Py<PyAny>) {
        (self.white.clone_ref(py), self.black.clone_ref(py))
    }
}

/// A set of files.
#[pyclass(frozen, eq, hash, from_py_object, module = "esca", name = "FileSet")]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PyFileSet {
    pub(crate) inner: FileSet,
}

impl PyFileSet {
    pub(crate) fn new(inner: FileSet) -> PyFileSet {
        PyFileSet { inner }
    }
}

#[pymethods]
impl PyFileSet {
    #[new]
    #[pyo3(signature = (files = None))]
    fn py_new(files: Option<Vec<String>>) -> PyResult<PyFileSet> {
        let mut set = FileSet::EMPTY;
        for name in files.unwrap_or_default() {
            set.insert(file_from(&name)?);
        }
        Ok(PyFileSet::new(set))
    }

    /// The members, from file a upwards.
    #[getter]
    fn files(&self) -> Vec<String> {
        self.inner.into_iter().map(file_name).collect()
    }

    fn __len__(&self) -> usize {
        self.inner.len() as usize
    }

    fn __bool__(&self) -> bool {
        !self.inner.is_empty()
    }

    fn __contains__(&self, file: &str) -> PyResult<bool> {
        Ok(self.inner.contains(file_from(file)?))
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<PyAny>> {
        let py = slf.py();
        Ok(PyList::new(py, slf.files())?
            .try_iter()?
            .into_any()
            .unbind())
    }

    fn __and__(&self, other: &PyFileSet) -> PyFileSet {
        PyFileSet::new(self.inner & other.inner)
    }

    fn __or__(&self, other: &PyFileSet) -> PyFileSet {
        PyFileSet::new(self.inner | other.inner)
    }

    fn __sub__(&self, other: &PyFileSet) -> PyFileSet {
        PyFileSet::new(self.inner - other.inner)
    }

    fn __invert__(&self) -> PyFileSet {
        PyFileSet::new(!self.inner)
    }

    /// Whether every member is a member of `other`.
    fn is_subset(&self, other: &PyFileSet) -> bool {
        self.inner.is_subset(other.inner)
    }

    fn __repr__(&self) -> String {
        format!("FileSet({:?})", self.files())
    }

    fn __getnewargs__(&self) -> (Vec<String>,) {
        (self.files(),)
    }
}

/// The plain-data form of a fact: a square set becomes the list of its squares,
/// a file set the list of its files, a pair by colour an object keyed by
/// colour, and everything else is already plain.
pub(crate) fn jsonable<'py>(value: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    let py = value.py();
    if let Ok(set) = value.cast::<PySquareSet>() {
        return PyList::new(
            py,
            set.get().inner.into_iter().map(|square| square.to_string()),
        )?
        .into_bound_py_any(py);
    }
    if let Ok(set) = value.cast::<PyFileSet>() {
        return PyList::new(py, set.get().inner.into_iter().map(file_name))?.into_bound_py_any(py);
    }
    if let Ok(pair) = value.cast::<PyByColour>() {
        let pair = pair.get();
        let out = PyDict::new(py);
        out.set_item(colour_name(Colour::White), jsonable(pair.white.bind(py))?)?;
        out.set_item(colour_name(Colour::Black), jsonable(pair.black.bind(py))?)?;
        return out.into_bound_py_any(py);
    }
    if let Ok(items) = value.cast::<PyList>() {
        let out = PyList::empty(py);
        for item in items.iter() {
            out.append(jsonable(&item)?)?;
        }
        return out.into_bound_py_any(py);
    }
    if let Ok(items) = value.cast::<PyTuple>() {
        let out = PyList::empty(py);
        for item in items.iter() {
            out.append(jsonable(&item)?)?;
        }
        return out.into_bound_py_any(py);
    }
    Ok(value.clone())
}

/// The named attributes of `owner`, in the order `names` gives them.
pub(crate) fn fields_of<'py>(
    owner: &Bound<'py, PyAny>,
    names: &[&'static str],
) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(owner.py());
    for name in names {
        out.set_item(name, owner.getattr(*name)?)?;
    }
    Ok(out)
}

/// The same, with every value in its plain-data form.
pub(crate) fn dict_of<'py>(
    owner: &Bound<'py, PyAny>,
    names: &[&'static str],
) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(owner.py());
    for name in names {
        out.set_item(name, jsonable(&owner.getattr(*name)?)?)?;
    }
    Ok(out)
}
