//! The Python spellings of the values a fact can hold: a pair by colour, a set
//! of files, a fixed run as a tuple, and the plain data `to_dict` answers in.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple, PyType};

use crate::types::{ByColour, Colour, FileSet};

use super::convert::{colour_from, file_from, file_name};

/// One value for White and one for Black.
///
/// Read a side by name — `.white`, `.black` — or by colour, with `of("white")`
/// or `["w"]`. Iterating gives the two values, White first, so `white, black =
/// pair` unpacks; `items()` gives them with the colour each belongs to. A pair
/// hashes when both of its values do.
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
    /// The attributes a positional pattern reads, White first.
    #[classattr]
    #[pyo3(name = "__match_args__")]
    const MATCH_ARGS: (&'static str, &'static str) = ("white", "black");

    /// `ByColour[T]`, so that the pair can be written as the type it holds.
    #[classmethod]
    fn __class_getitem__<'py>(
        cls: &Bound<'py, PyType>,
        item: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        cls.py()
            .import("types")?
            .getattr("GenericAlias")?
            .call1((cls, item))
    }

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

    /// The two values, White first.
    fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let values = PyTuple::new(py, [self.white.bind(py), self.black.bind(py)])?;
        values.try_iter()?.into_bound_py_any(py)
    }

    /// Two: a pair is always both colours.
    fn __len__(&self) -> usize {
        2
    }

    /// Whether `colour` names a side of the pair, which both spellings of
    /// both colours do.
    fn __contains__(&self, colour: &str) -> bool {
        colour_from(colour).is_ok()
    }

    /// Each colour with its value, White first.
    fn items<'py>(&self, py: Python<'py>) -> PyResult<Vec<(&'static str, Bound<'py, PyAny>)>> {
        Ok(vec![
            ("white", self.white.bind(py).clone()),
            ("black", self.black.bind(py).clone()),
        ])
    }

    fn __hash__(&self, py: Python<'_>) -> PyResult<isize> {
        let values = PyTuple::new(py, [self.white.bind(py), self.black.bind(py)])?;
        values.hash()
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
/// A fixed run of values, handed over as a tuple so that its length is part of
/// what a reader sees. The order is the one the fact's type or its own meaning
/// states.
pub(crate) struct Fixed<T>(pub Vec<T>);

impl<'py, T> IntoPyObject<'py> for Fixed<T>
where
    T: IntoPyObject<'py>,
{
    type Target = PyTuple;
    type Output = Bound<'py, PyTuple>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyTuple::new(py, self.0)
    }
}

/// The plain data of `value`, in the one JSON form the facts are written in.
///
/// The crate serialises it and Python parses that text back, so the two
/// surfaces cannot answer differently.
pub(crate) fn plain<'py, T: serde::Serialize>(
    py: Python<'py>,
    value: &T,
) -> PyResult<Bound<'py, PyDict>> {
    let text = serde_json::to_string(value).map_err(super::convert::value_error)?;
    Ok(py
        .import("json")?
        .call_method1("loads", (text,))?
        .cast_into::<PyDict>()?)
}

/// `Class(name=value, ...)`, every fact of the group with the value it holds.
pub(crate) fn repr_of(
    owner: &Bound<'_, PyAny>,
    class: &str,
    names: &[&'static str],
) -> PyResult<String> {
    let mut out = String::from(class);
    out.push('(');
    for (index, name) in names.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(name);
        out.push('=');
        out.push_str(&owner.getattr(*name)?.repr()?.to_string());
    }
    out.push(')');
    Ok(out)
}

/// A hash of `value`, read off its JSON form so that two groups holding the
/// same facts hash alike.
pub(crate) fn hash_of<T: serde::Serialize>(value: &T) -> u64 {
    let mut hash = DefaultHasher::new();
    serde_json::to_string(value)
        .unwrap_or_default()
        .hash(&mut hash);
    hash.finish()
}
