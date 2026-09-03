//! PGN games, read one at a time.
//!
//! A result is text here — `1-0`, `0-1`, `1/2-1/2` or `*` — and a comment is
//! the empty string when there is none.

use std::io::{BufReader, Cursor};
use std::path::PathBuf;
use std::sync::Mutex;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::pgn::{self, GameResult, PgnError};

use super::board::{PyGame, PyMove, PyPosition, PyVariant};
use super::convert::value_error;

/// One move of a game tree.
#[pyclass(frozen, from_py_object, module = "esca.pgn", name = "PgnNode")]
#[derive(Clone)]
pub struct PyPgnNode {
    inner: pgn::Node,
}

impl PyPgnNode {
    fn line(nodes: &[pgn::Node]) -> Vec<PyPgnNode> {
        nodes
            .iter()
            .map(|node| PyPgnNode {
                inner: node.clone(),
            })
            .collect()
    }
}

#[pymethods]
impl PyPgnNode {
    /// The move played.
    #[getter]
    #[pyo3(name = "move")]
    fn get_move(&self) -> PyMove {
        PyMove::new(self.inner.mv)
    }

    /// The move's own text, as written, less any `!`/`?` suffix.
    #[getter]
    fn san(&self) -> &str {
        &self.inner.san
    }

    /// The numeric annotation glyphs, in the order written.
    #[getter]
    fn nags(&self) -> Vec<u16> {
        self.inner.nags.clone()
    }

    /// The comment written before the move.
    #[getter]
    fn comment_before(&self) -> &str {
        &self.inner.comment_before
    }

    /// The comment written after the move.
    #[getter]
    fn comment_after(&self) -> &str {
        &self.inner.comment_after
    }

    /// Alternatives to this move, each a line from the position it was played
    /// in.
    #[getter]
    fn variations(&self) -> Vec<Vec<PyPgnNode>> {
        self.inner
            .variations
            .iter()
            .map(|line| PyPgnNode::line(line))
            .collect()
    }

    fn __repr__(&self) -> String {
        format!("<PgnNode {}>", self.inner.san)
    }
}

/// A game as PGN describes one: tag pairs, a move tree, and a result.
#[pyclass(eq, module = "esca.pgn", name = "PgnGame")]
#[derive(PartialEq)]
pub struct PyPgnGame {
    pub(crate) inner: pgn::Game,
}

impl PyPgnGame {
    pub(crate) fn new(inner: pgn::Game) -> PyPgnGame {
        PyPgnGame { inner }
    }
}

#[pymethods]
impl PyPgnGame {
    /// A game with no tags and no moves.
    #[new]
    fn py_new() -> PyPgnGame {
        PyPgnGame::new(pgn::Game::new())
    }

    /// The PGN of a played game, with a seven-tag roster of placeholders.
    #[staticmethod]
    fn from_game(game: &PyGame) -> PyPgnGame {
        PyPgnGame::new(game.played().to_pgn())
    }

    /// The tag pairs, in the order they were set.
    #[getter]
    fn headers<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let out = PyDict::new(py);
        for (name, value) in self.inner.headers.iter() {
            out.set_item(name, value)?;
        }
        Ok(out)
    }

    /// Sets the tag `name`, keeping its place when it is already present.
    fn set_header(&mut self, name: &str, value: &str) {
        self.inner.headers.set(name, value);
    }

    /// The comment written before the first move.
    #[getter]
    fn comment(&self) -> &str {
        &self.inner.comment
    }

    #[setter]
    fn set_comment(&mut self, text: &str) {
        self.inner.comment = text.to_string();
    }

    /// The game-termination marker.
    #[getter]
    fn result(&self) -> &'static str {
        self.inner.result.as_str()
    }

    #[setter]
    fn set_result(&mut self, marker: &str) -> PyResult<()> {
        self.inner.result = GameResult::from_text(marker)
            .ok_or_else(|| value_error(format!("not a result: {marker:?}")))?;
        Ok(())
    }

    /// The rules the `Variant` tag names.
    #[getter]
    fn variant(&self) -> PyResult<PyVariant> {
        let (variant, _) = self.inner.setup().map_err(value_error)?;
        Ok(PyVariant::new(variant))
    }

    /// The position the `FEN` tag names, or the variant's own start.
    #[getter]
    fn start_position(&self) -> PyResult<PyPosition> {
        let (_, start) = self.inner.setup().map_err(value_error)?;
        Ok(PyPosition::new(start))
    }

    /// The moves of the mainline.
    fn mainline(&self) -> Vec<PyPgnNode> {
        PyPgnNode::line(self.inner.mainline())
    }

    /// The mainline as a played game.
    fn game(&self) -> PyResult<PyGame> {
        let played = self.inner.mainline_game().map_err(value_error)?;
        let variant = PyVariant::new(self.inner.setup().map_err(value_error)?.0);
        Ok(PyGame::seeded(played, variant))
    }

    /// The export-format text.
    #[pyo3(name = "to_string")]
    fn text(&self) -> String {
        self.inner.to_string()
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "<PgnGame {} ply {} {}>",
            self.inner.headers.get("Event").unwrap_or("?"),
            self.inner.mainline().len(),
            self.inner.result
        )
    }
}

/// An iterator over the games of a PGN source.
#[pyclass(module = "esca.pgn", name = "PgnReader")]
pub struct PyPgnReader {
    games: Mutex<Box<dyn Iterator<Item = Result<pgn::Game, PgnError>> + Send>>,
}

#[pymethods]
impl PyPgnReader {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&self) -> PyResult<Option<PyPgnGame>> {
        let mut games = self.games.lock().expect("the reader is not shared");
        match games.next() {
            None => Ok(None),
            Some(Ok(game)) => Ok(Some(PyPgnGame::new(game))),
            Some(Err(error)) => Err(value_error(error)),
        }
    }

    fn __repr__(&self) -> String {
        "<PgnReader>".to_string()
    }
}

/// Streams the games of the PGN file at `path`.
///
/// With `skip_errors`, a malformed game is dropped; without it, one raises
/// `ValueError` naming the line and column, and the stream goes on with the
/// next game.
#[pyfunction]
#[pyo3(signature = (path, *, skip_errors = false))]
pub(crate) fn pgn_read(path: PathBuf, skip_errors: bool) -> PyResult<PyPgnReader> {
    let file = std::fs::File::open(&path)?;
    let reader = pgn::Reader::new(BufReader::new(file));
    let reader = if skip_errors {
        reader.skipping()
    } else {
        reader
    };
    Ok(PyPgnReader {
        games: Mutex::new(Box::new(reader)),
    })
}

/// Streams the games PGN `text` holds.
#[pyfunction]
#[pyo3(signature = (text, *, skip_errors = false))]
pub(crate) fn pgn_read_string(text: &str, skip_errors: bool) -> PyPgnReader {
    let reader = pgn::Reader::new(Cursor::new(text.as_bytes().to_vec()));
    let reader = if skip_errors {
        reader.skipping()
    } else {
        reader
    };
    PyPgnReader {
        games: Mutex::new(Box::new(reader)),
    }
}

/// How many games the PGN file at `path` holds that read without error.
#[pyfunction]
pub(crate) fn pgn_count(path: PathBuf) -> PyResult<usize> {
    let file = std::fs::File::open(&path)?;
    Ok(pgn::count_games(BufReader::new(file)))
}
