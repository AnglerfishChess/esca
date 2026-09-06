//! Opening books in the Polyglot format.
//!
//! A key is a plain integer here, and an entry that has not been read against
//! a position keeps its move as UCI text.

use std::path::PathBuf;
use std::sync::Mutex;

use pyo3::prelude::*;

use crate::polyglot::{self, Book, Builder, Entry, Raw};

use super::board::{PyGame, PyMove, PyPosition, PyVariant};

/// One entry as the file holds it.
#[pyclass(
    frozen,
    eq,
    hash,
    from_py_object,
    module = "esca.polyglot",
    name = "PolyglotRaw"
)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PyRaw {
    inner: Raw,
}

#[pymethods]
impl PyRaw {
    /// An entry naming the move `bits` encodes in the position `key`
    /// identifies.
    #[new]
    #[pyo3(signature = (key, bits, weight = 1, learn = 0))]
    fn py_new(key: u64, bits: u16, weight: u16, learn: u32) -> PyRaw {
        PyRaw {
            inner: Raw {
                key,
                mv: bits,
                weight,
                learn,
            },
        }
    }

    /// The Polyglot key of the position the move belongs to.
    #[getter]
    fn key(&self) -> u64 {
        self.inner.key
    }

    /// The move as the format encodes it.
    #[getter]
    fn bits(&self) -> u16 {
        self.inner.mv
    }

    /// The move as UCI text, castling king-to-rook; `None` when the bits name
    /// no move.
    #[getter]
    fn uci(&self) -> Option<String> {
        self.inner.uci()
    }

    /// How good, or how often played, the move is, relative to the other
    /// entries of this key.
    #[getter]
    fn weight(&self) -> u16 {
        self.inner.weight
    }

    /// Four bytes the format reserves for a program's own use.
    #[getter]
    fn learn(&self) -> u32 {
        self.inner.learn
    }

    /// The entry read against `position`; `None` when its move is not legal
    /// there.
    #[pyo3(signature = (position, *, variant = None))]
    fn decode(&self, position: &PyPosition, variant: Option<PyVariant>) -> Option<PyEntry> {
        let variant = variant.unwrap_or_else(super::default_variant);
        self.inner
            .decode(variant.rules(), &position.inner)
            .map(PyEntry::new)
    }

    fn __repr__(&self) -> String {
        format!(
            "<PolyglotRaw {:016x} {} weight {}>",
            self.inner.key,
            self.inner.uci().unwrap_or_else(|| "?".to_string()),
            self.inner.weight
        )
    }
}

/// One entry whose move has been read against a position.
#[pyclass(
    frozen,
    eq,
    hash,
    from_py_object,
    module = "esca.polyglot",
    name = "PolyglotEntry"
)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PyEntry {
    inner: Entry,
}

impl PyEntry {
    fn new(inner: Entry) -> PyEntry {
        PyEntry { inner }
    }
}

#[pymethods]
impl PyEntry {
    /// An entry naming `move` in the position `key` identifies.
    #[new]
    #[pyo3(signature = (key, mv, weight = 1, learn = 0))]
    fn py_new(key: u64, mv: &PyMove, weight: u16, learn: u32) -> PyEntry {
        PyEntry::new(Entry::new(key, mv.inner, weight, learn))
    }

    /// The Polyglot key of the position the move belongs to.
    #[getter]
    fn key(&self) -> u64 {
        self.inner.key
    }

    /// The move.
    #[getter]
    #[pyo3(name = "move")]
    fn get_move(&self) -> PyMove {
        PyMove::new(self.inner.mv)
    }

    /// The move as the format encodes it.
    #[getter]
    fn bits(&self) -> u16 {
        Raw::from(self.inner).mv
    }

    /// How good, or how often played, the move is, relative to the other
    /// entries of this key.
    #[getter]
    fn weight(&self) -> u16 {
        self.inner.weight
    }

    /// Four bytes the format reserves for a program's own use.
    #[getter]
    fn learn(&self) -> u32 {
        self.inner.learn
    }

    fn __repr__(&self) -> String {
        format!(
            "<PolyglotEntry {:016x} {} weight {}>",
            self.inner.key, self.inner.mv, self.inner.weight
        )
    }
}

/// An opening book: entries sorted by key, looked up by position.
#[pyclass(frozen, module = "esca.polyglot", name = "PolyglotBook")]
pub struct PyBook {
    inner: Book,
}

#[pymethods]
impl PyBook {
    /// The book at `path`, memory-mapped.
    #[new]
    fn py_new(path: PathBuf) -> PyResult<PyBook> {
        Ok(PyBook {
            inner: Book::open(&path)?,
        })
    }

    /// The book `data` holds.
    #[staticmethod]
    fn from_bytes(data: Vec<u8>) -> PyResult<PyBook> {
        Ok(PyBook {
            inner: Book::from_bytes(data)?,
        })
    }

    /// Writes `entries` to `path`, sorted and merged.
    #[staticmethod]
    fn write(path: PathBuf, entries: Vec<PyEntry>) -> PyResult<()> {
        let entries: Vec<Entry> = entries.into_iter().map(|entry| entry.inner).collect();
        Book::write(&path, &entries)?;
        Ok(())
    }

    /// The entry at `index`, counting from the start of the file.
    fn get(&self, index: usize) -> Option<PyRaw> {
        self.inner.get(index).map(|inner| PyRaw { inner })
    }

    /// The entries at `key`, in the order the file gives them.
    fn raw_entries(&self, key: u64) -> Vec<PyRaw> {
        self.inner
            .raw_entries(key)
            .into_iter()
            .map(|inner| PyRaw { inner })
            .collect()
    }

    /// The entries at this position's key that name a move legal in it.
    #[pyo3(signature = (position, *, variant = None))]
    fn entries(&self, position: &PyPosition, variant: Option<PyVariant>) -> Vec<PyEntry> {
        let variant = variant.unwrap_or_else(super::default_variant);
        self.inner
            .entries(variant.rules(), &position.inner)
            .into_iter()
            .map(PyEntry::new)
            .collect()
    }

    /// The heaviest of them; ties go to the earlier entry.
    #[pyo3(signature = (position, *, variant = None))]
    fn best(&self, position: &PyPosition, variant: Option<PyVariant>) -> Option<PyEntry> {
        let variant = variant.unwrap_or_else(super::default_variant);
        self.inner
            .best(variant.rules(), &position.inner)
            .map(PyEntry::new)
    }

    /// One of them, drawn by weight: the entry whose running total first
    /// exceeds `seed` reduced modulo the total.
    #[pyo3(signature = (position, seed, *, variant = None))]
    fn pick(
        &self,
        position: &PyPosition,
        seed: u64,
        variant: Option<PyVariant>,
    ) -> Option<PyEntry> {
        let variant = variant.unwrap_or_else(super::default_variant);
        self.inner
            .pick(variant.rules(), &position.inner, seed)
            .map(PyEntry::new)
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn __iter__(&self) -> PyBookIter {
        PyBookIter {
            entries: Mutex::new(self.inner.iter().collect::<Vec<Raw>>().into_iter()),
        }
    }

    fn __repr__(&self) -> String {
        format!("<PolyglotBook {} entries>", self.inner.len())
    }
}

/// An iterator over a book's entries, in file order.
#[pyclass(module = "esca.polyglot", name = "PolyglotBookIter")]
pub struct PyBookIter {
    entries: Mutex<std::vec::IntoIter<Raw>>,
}

#[pymethods]
impl PyBookIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&self) -> Option<PyRaw> {
        let mut entries = self.entries.lock().expect("the iterator is not shared");
        entries.next().map(|inner| PyRaw { inner })
    }

    fn __repr__(&self) -> String {
        "<PolyglotBookIter>".to_string()
    }
}

/// Counts the moves of the games it is given and writes them as a book.
#[pyclass(module = "esca.polyglot", name = "PolyglotBuilder")]
pub struct PyBuilder {
    inner: Builder,
}

#[pymethods]
impl PyBuilder {
    /// A builder that counts moves up to `max_ply` and writes those played at
    /// least `min_count` times.
    #[new]
    #[pyo3(signature = (*, max_ply = None, min_count = 1))]
    fn py_new(max_ply: Option<u32>, min_count: u32) -> PyBuilder {
        let mut inner = Builder::new().min_count(min_count);
        if let Some(plies) = max_ply {
            inner = inner.max_ply(plies);
        }
        PyBuilder { inner }
    }

    /// Counts every move of `game`, each in the position it was played in.
    fn add_game(&mut self, game: &PyGame) {
        self.inner.add_game(game.played());
    }

    /// Counts every game of the PGN file at `path`, skipping the ones that do
    /// not read; returns how many were counted.
    #[cfg(feature = "pgn")]
    fn add_pgn(&mut self, path: PathBuf) -> PyResult<usize> {
        let file = std::fs::File::open(&path)?;
        Ok(self.inner.add_pgn(std::io::BufReader::new(file)))
    }

    /// The same for PGN held as text.
    #[cfg(feature = "pgn")]
    fn add_pgn_string(&mut self, text: &str) -> usize {
        self.inner.add_pgn(std::io::Cursor::new(text.as_bytes()))
    }

    /// The book rows: sorted, merged, and filtered by `min_count`.
    fn entries(&self) -> Vec<PyRaw> {
        self.inner
            .entries()
            .into_iter()
            .map(|inner| PyRaw { inner })
            .collect()
    }

    /// Writes those rows to `path`.
    fn write(&self, path: PathBuf) -> PyResult<()> {
        self.inner.write(&path)?;
        Ok(())
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn __repr__(&self) -> String {
        format!("<PolyglotBuilder {} moves>", self.inner.len())
    }
}

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyRaw>()?;
    module.add_class::<PyEntry>()?;
    module.add_class::<PyBook>()?;
    module.add_class::<PyBookIter>()?;
    module.add_class::<PyBuilder>()?;
    module.add("POLYGLOT_ENTRY_SIZE", polyglot::ENTRY_SIZE)?;
    Ok(())
}
