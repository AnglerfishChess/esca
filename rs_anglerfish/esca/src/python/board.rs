//! Variants, square sets, moves, positions and games.

use std::sync::Arc;

use pyo3::prelude::*;
use pyo3::types::PyList;

use crate::game::Game;
use crate::moves::Move;
use crate::position::Position;
use crate::types::{Piece, SquareSet};
use crate::variant::Variant as VariantTrait;

use super::convert::{
    castling_output_from, castling_output_name, claim_name, colour_from, colour_name,
    move_kind_from, move_kind_name, outcome_name, role_from, role_name, square_from, square_name,
    value_error, variant_by_name,
};
use super::facts::{PyAnnotatedMove, PyFacts};

/// One set of chess rules.
#[pyclass(frozen, eq, hash, from_py_object, module = "esca", name = "Variant")]
#[derive(Clone)]
pub struct PyVariant {
    pub(crate) inner: Arc<dyn VariantTrait>,
}

impl PyVariant {
    pub(crate) fn new(inner: Arc<dyn VariantTrait>) -> PyVariant {
        PyVariant { inner }
    }

    pub(crate) fn rules(&self) -> &dyn VariantTrait {
        self.inner.as_ref()
    }
}

impl PartialEq for PyVariant {
    fn eq(&self, other: &PyVariant) -> bool {
        self.inner.name() == other.inner.name()
    }
}

impl Eq for PyVariant {}

impl std::hash::Hash for PyVariant {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.name().hash(state);
    }
}

#[pymethods]
impl PyVariant {
    /// The shared variant of that name: `chess` or `chess960`.
    #[staticmethod]
    fn named(name: &str) -> PyResult<PyVariant> {
        variant_by_name(name).map(PyVariant::new)
    }

    /// The identifier PGN and UCI use.
    #[getter]
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    /// The roles a pawn may promote to.
    #[getter]
    fn promotion_roles(&self) -> Vec<String> {
        self.inner
            .promotion_roles()
            .iter()
            .copied()
            .map(role_name)
            .collect()
    }

    /// The position a game of this variant starts from.
    #[pyo3(signature = (seed = 0))]
    fn start_position(&self, seed: u64) -> PyPosition {
        PyPosition::new(self.inner.start_position(seed))
    }

    fn __repr__(&self) -> String {
        format!("<Variant {}>", self.inner.name())
    }

    fn __reduce__<'py>(slf: &Bound<'py, Self>) -> PyResult<(Bound<'py, PyAny>, (&'static str,))> {
        let named = slf.get_type().getattr("named")?;
        Ok((named, (slf.get().inner.name(),)))
    }
}

/// A set of squares.
#[pyclass(frozen, eq, hash, from_py_object, module = "esca", name = "SquareSet")]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PySquareSet {
    pub(crate) inner: SquareSet,
}

impl PySquareSet {
    pub(crate) fn new(inner: SquareSet) -> PySquareSet {
        PySquareSet { inner }
    }

    /// The two sets of a side-paired fact, us first.
    pub(crate) fn pair(sets: [SquareSet; 2]) -> (PySquareSet, PySquareSet) {
        (PySquareSet::new(sets[0]), PySquareSet::new(sets[1]))
    }
}

#[pymethods]
impl PySquareSet {
    #[new]
    #[pyo3(signature = (squares = None))]
    fn py_new(squares: Option<Vec<String>>) -> PyResult<PySquareSet> {
        let mut set = SquareSet::EMPTY;
        for name in squares.unwrap_or_default() {
            set.insert(square_from(&name)?);
        }
        Ok(PySquareSet::new(set))
    }

    /// The membership bits, bit *i* for square *i*.
    #[getter]
    fn bits(&self) -> u64 {
        self.inner.bits()
    }

    /// The members, in ascending square index.
    #[getter]
    fn squares(&self) -> Vec<String> {
        self.inner.into_iter().map(square_name).collect()
    }

    fn __len__(&self) -> usize {
        self.inner.len() as usize
    }

    fn __bool__(&self) -> bool {
        !self.inner.is_empty()
    }

    fn __contains__(&self, square: &str) -> PyResult<bool> {
        Ok(self.inner.contains(square_from(square)?))
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<PyAny>> {
        let py = slf.py();
        Ok(PyList::new(py, slf.squares())?
            .try_iter()?
            .into_any()
            .unbind())
    }

    fn __and__(&self, other: &PySquareSet) -> PySquareSet {
        PySquareSet::new(self.inner & other.inner)
    }

    fn __or__(&self, other: &PySquareSet) -> PySquareSet {
        PySquareSet::new(self.inner | other.inner)
    }

    fn __xor__(&self, other: &PySquareSet) -> PySquareSet {
        PySquareSet::new(self.inner ^ other.inner)
    }

    fn __sub__(&self, other: &PySquareSet) -> PySquareSet {
        PySquareSet::new(self.inner - other.inner)
    }

    fn __invert__(&self) -> PySquareSet {
        PySquareSet::new(!self.inner)
    }

    /// Whether every member is a member of `other`.
    fn is_subset(&self, other: &PySquareSet) -> bool {
        self.inner.is_subset(other.inner)
    }

    fn __repr__(&self) -> String {
        format!("SquareSet({:?})", self.squares())
    }

    fn __getnewargs__(&self) -> (Vec<String>,) {
        (self.squares(),)
    }
}

/// One action: origin, destination, promotion role and kind.
#[pyclass(frozen, eq, hash, from_py_object, module = "esca", name = "Move")]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PyMove {
    pub(crate) inner: Move,
}

impl PyMove {
    pub(crate) fn new(inner: Move) -> PyMove {
        PyMove { inner }
    }
}

#[pymethods]
impl PyMove {
    #[new]
    #[pyo3(signature = (origin, destination, promotion = None, kind = "quiet", is_capture = false))]
    fn py_new(
        origin: &str,
        destination: &str,
        promotion: Option<&str>,
        kind: &str,
        is_capture: bool,
    ) -> PyResult<PyMove> {
        let promotion = promotion.map(role_from).transpose()?;
        let mv = Move::new(
            square_from(origin)?,
            square_from(destination)?,
            promotion,
            move_kind_from(kind)?,
        );
        Ok(PyMove::new(mv.with_capture(is_capture)))
    }

    /// The square the moving unit starts on; for castling, the king's.
    #[getter]
    fn origin(&self) -> String {
        square_name(self.inner.from())
    }

    /// The square it ends on; for castling, the rook's own square.
    #[getter]
    fn destination(&self) -> String {
        square_name(self.inner.to())
    }

    /// The role a promoting pawn becomes.
    #[getter]
    fn promotion(&self) -> Option<String> {
        self.inner.promotion().map(role_name)
    }

    /// `quiet`, `capture`, `en_passant`, `castling` or `promotion`.
    #[getter]
    fn kind(&self) -> &'static str {
        move_kind_name(self.inner.kind())
    }

    #[getter]
    fn is_capture(&self) -> bool {
        self.inner.is_capture()
    }

    #[getter]
    fn is_castling(&self) -> bool {
        self.inner.is_castling()
    }

    #[getter]
    fn is_en_passant(&self) -> bool {
        self.inner.is_en_passant()
    }

    /// Origin, destination and promotion role, castling king-to-rook. The
    /// spelling a variant asks for comes from `Game.move_to_uci`.
    #[getter]
    fn uci(&self) -> String {
        self.inner.to_string()
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!("<Move {}>", self.inner)
    }

    fn __getnewargs__(&self) -> (String, String, Option<String>, &'static str, bool) {
        (
            self.origin(),
            self.destination(),
            self.promotion(),
            self.kind(),
            self.inner.is_capture(),
        )
    }
}

/// Placement and state, with no rules attached.
#[pyclass(frozen, eq, hash, from_py_object, module = "esca", name = "Position")]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PyPosition {
    pub(crate) inner: Position,
}

impl PyPosition {
    pub(crate) fn new(inner: Position) -> PyPosition {
        PyPosition { inner }
    }

    /// The text a round trip has to preserve: four fields when the clocks are
    /// unknown, six otherwise.
    fn text(&self) -> String {
        if self.inner.clocks_known() {
            self.inner.fen()
        } else {
            self.inner.epd()
        }
    }
}

#[pymethods]
impl PyPosition {
    /// Reads a six-field FEN, or a four-field one, which leaves the clocks
    /// unknown.
    #[staticmethod]
    fn from_fen(fen: &str) -> PyResult<PyPosition> {
        Position::from_fen(fen)
            .map(PyPosition::new)
            .map_err(value_error)
    }

    /// The six-field FEN.
    #[getter]
    fn fen(&self) -> String {
        self.inner.fen()
    }

    /// The first four FEN fields.
    #[getter]
    fn epd(&self) -> String {
        self.inner.epd()
    }

    /// `w` or `b`.
    #[getter]
    fn side_to_move(&self) -> String {
        colour_name(self.inner.side_to_move())
    }

    /// The FEN castling field.
    #[getter]
    fn castling_rights(&self) -> String {
        self.inner.castling_rights().to_fen_field()
    }

    /// The square a pawn skipped on the previous ply.
    #[getter]
    fn en_passant(&self) -> Option<String> {
        self.inner.en_passant().map(square_name)
    }

    #[getter]
    fn halfmove_clock(&self) -> u32 {
        self.inner.halfmove_clock()
    }

    #[getter]
    fn fullmove_number(&self) -> u32 {
        self.inner.fullmove_number()
    }

    /// False when the position came from a four-field FEN.
    #[getter]
    fn clocks_known(&self) -> bool {
        self.inner.clocks_known()
    }

    #[getter]
    fn in_check(&self) -> bool {
        self.inner.in_check()
    }

    /// Every square holding a unit.
    #[getter]
    fn occupied(&self) -> PySquareSet {
        PySquareSet::new(self.inner.occupied())
    }

    /// The Zobrist key. An identity within one process run only.
    #[getter]
    fn key(&self) -> u64 {
        self.inner.key().get()
    }

    /// The FEN letter of the unit on `square`, if any.
    fn piece_at(&self, square: &str) -> PyResult<Option<String>> {
        Ok(self
            .inner
            .piece_at(square_from(square)?)
            .map(|piece| piece.to_char().to_string()))
    }

    /// Every square holding a unit of `role`, of either colour.
    fn by_role(&self, role: &str) -> PyResult<PySquareSet> {
        Ok(PySquareSet::new(self.inner.by_role(role_from(role)?)))
    }

    /// Every square holding a unit of `colour`.
    fn by_colour(&self, colour: &str) -> PyResult<PySquareSet> {
        Ok(PySquareSet::new(self.inner.by_colour(colour_from(colour)?)))
    }

    /// Every square holding a unit of `role` and `colour`.
    fn by_piece(&self, role: &str, colour: &str) -> PyResult<PySquareSet> {
        let piece = Piece::new(role_from(role)?, colour_from(colour)?);
        Ok(PySquareSet::new(self.inner.by_piece(piece)))
    }

    /// Where `colour`'s king stands.
    fn king_of(&self, colour: &str) -> PyResult<String> {
        Ok(square_name(self.inner.king_of(colour_from(colour)?)))
    }

    /// The static exchange evaluation of the unit on `square`.
    fn see(&self, square: &str) -> PyResult<i32> {
        Ok(self.inner.see(square_from(square)?))
    }

    /// The static exchange evaluation of `mv`, which the caller has checked is
    /// a move of this position.
    fn see_capture(&self, mv: &PyMove) -> i32 {
        self.inner.see_capture(mv.inner)
    }

    /// The facts of this position under `variant`.
    #[pyo3(signature = (variant = None))]
    fn facts(&self, variant: Option<PyVariant>) -> PyFacts {
        let variant = variant.unwrap_or_else(super::default_variant);
        PyFacts::of_position(&self.inner, variant)
    }

    /// The position with the colours swapped and the ranks flipped.
    fn mirrored(&self) -> PyPosition {
        PyPosition::new(self.inner.mirrored())
    }

    /// Board, side to move and state, for a human reader. Not a stable format.
    fn summary(&self) -> String {
        self.inner.summary()
    }

    fn __str__(&self) -> String {
        self.inner.fen()
    }

    fn __repr__(&self) -> String {
        format!("<Position {}>", self.text())
    }

    fn __reduce__<'py>(slf: &Bound<'py, Self>) -> PyResult<(Bound<'py, PyAny>, (String,))> {
        let from_fen = slf.get_type().getattr("from_fen")?;
        Ok((from_fen, (slf.get().text(),)))
    }
}

/// A variant, a start position and the moves played from it.
#[pyclass(module = "esca", name = "Game")]
pub struct PyGame {
    inner: Game,
    variant: PyVariant,
}

impl PyGame {
    pub(crate) fn seeded(inner: Game, variant: PyVariant) -> PyGame {
        PyGame { inner, variant }
    }

    #[cfg(any(feature = "pgn", feature = "uci"))]
    pub(crate) fn played(&self) -> &Game {
        &self.inner
    }
}

#[pymethods]
impl PyGame {
    #[new]
    #[pyo3(signature = (*, variant = None, seed = 0))]
    fn py_new(variant: Option<PyVariant>, seed: u64) -> PyGame {
        let variant = variant.unwrap_or_else(super::default_variant);
        let game = Game::with_seed(variant.inner.clone(), seed);
        PyGame::seeded(game, variant)
    }

    /// A game starting from the position `fen` describes.
    #[staticmethod]
    #[pyo3(signature = (fen, *, variant = None))]
    fn from_fen(fen: &str, variant: Option<PyVariant>) -> PyResult<PyGame> {
        let variant = variant.unwrap_or_else(super::default_variant);
        let game = Game::from_fen(variant.inner.clone(), fen).map_err(value_error)?;
        Ok(PyGame::seeded(game, variant))
    }

    /// A game starting from `position`.
    #[staticmethod]
    #[pyo3(signature = (position, *, variant = None))]
    fn from_position(position: &PyPosition, variant: Option<PyVariant>) -> PyResult<PyGame> {
        let variant = variant.unwrap_or_else(super::default_variant);
        let game = Game::from_position(variant.inner.clone(), position.inner.clone())
            .map_err(value_error)?;
        Ok(PyGame::seeded(game, variant))
    }

    /// The rules this game is played under.
    #[getter]
    fn variant(&self) -> PyVariant {
        self.variant.clone()
    }

    /// The position now.
    #[getter]
    fn position(&self) -> PyPosition {
        PyPosition::new(self.inner.position().clone())
    }

    /// The position the game started from.
    #[getter]
    fn start_position(&self) -> PyPosition {
        PyPosition::new(self.inner.start_position().clone())
    }

    /// The moves played, in order.
    #[getter]
    fn moves(&self) -> Vec<PyMove> {
        self.inner
            .moves()
            .iter()
            .copied()
            .map(PyMove::new)
            .collect()
    }

    /// Every position from the start to the current one.
    #[getter]
    fn positions(&self) -> Vec<PyPosition> {
        self.inner
            .positions()
            .cloned()
            .map(PyPosition::new)
            .collect()
    }

    /// How many moves have been played.
    #[getter]
    fn ply(&self) -> u32 {
        self.inner.ply()
    }

    /// The castling spelling of this game's UCI output.
    #[getter]
    fn castling_output(&self) -> &'static str {
        castling_output_name(self.inner.castling_output())
    }

    #[setter]
    fn set_castling_output(&mut self, style: &str) -> PyResult<()> {
        self.inner.set_castling_output(castling_output_from(style)?);
        Ok(())
    }

    /// The legal moves in the current position.
    fn legal_moves(&self) -> Vec<PyMove> {
        self.inner
            .legal_moves()
            .iter()
            .copied()
            .map(PyMove::new)
            .collect()
    }

    /// Every legal move in the current position, annotated.
    fn annotated_moves(&self) -> Vec<PyAnnotatedMove> {
        self.inner
            .annotated_moves()
            .iter()
            .copied()
            .map(PyAnnotatedMove::new)
            .collect()
    }

    /// Plays a move, given as a `Move` or as UCI text.
    fn play(&mut self, mv: &Bound<'_, PyAny>) -> PyResult<()> {
        if let Ok(mv) = mv.extract::<PyMove>() {
            return self.inner.play(mv.inner).map_err(value_error);
        }
        let text: String = mv.extract()?;
        self.inner.play_uci(&text).map_err(value_error)
    }

    /// Plays the move `text` names in SAN.
    fn play_san(&mut self, text: &str) -> PyResult<()> {
        self.inner.play_san(text).map_err(value_error)
    }

    /// Takes back the last move, returning it.
    fn undo(&mut self) -> Option<PyMove> {
        self.inner.undo().map(PyMove::new)
    }

    /// The UCI text of `mv` in the current position.
    fn move_to_uci(&self, mv: &PyMove) -> String {
        self.inner.move_to_uci(mv.inner)
    }

    /// The SAN text of `mv` in the current position.
    fn move_to_san(&self, mv: &PyMove) -> String {
        self.inner.move_to_san(mv.inner)
    }

    /// The automatic terminal condition, if any. The winner of a `checkmate`
    /// is the side that is not to move.
    fn outcome(&self) -> Option<&'static str> {
        self.inner.outcome().map(outcome_name)
    }

    /// The draws a player could claim now.
    fn claims(&self) -> Vec<&'static str> {
        self.inner
            .claims()
            .iter()
            .copied()
            .map(claim_name)
            .collect()
    }

    /// How often the current position has occurred in this game.
    fn repetitions(&self) -> u32 {
        self.inner.repetitions()
    }

    /// The facts of the current position, repetition and history included.
    fn facts(&self) -> PyFacts {
        PyFacts::of_game(&self.inner, self.variant.clone())
    }

    /// This game as PGN, with a seven-tag roster of placeholders.
    #[cfg(feature = "pgn")]
    fn to_pgn(&self) -> super::pgn::PyPgnGame {
        super::pgn::PyPgnGame::new(self.inner.to_pgn())
    }

    fn __repr__(&self) -> String {
        format!(
            "<Game {} ply {} {}>",
            self.inner.variant().name(),
            self.inner.ply(),
            self.inner.position().fen()
        )
    }
}
