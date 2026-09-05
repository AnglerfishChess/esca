//! The evidence behind a rules answer.
//!
//! An enum that carries nothing is its name in `snake_case`; an enum that
//! carries something is one class with a `kind` naming the case and the
//! payload of every case as attributes, empty where the case does not carry
//! it.

use pyo3::prelude::*;

use crate::explain;
use crate::types::{Square, SquareSet};

use super::board::PySquareSet;
use super::convert::{colour_name, square_name, wing_name};

/// Square-and-set pairs, the squares as text.
fn pairs(items: &[(Square, SquareSet)]) -> Vec<(String, PySquareSet)> {
    items
        .iter()
        .map(|&(square, set)| (square_name(square), PySquareSet::new(set)))
        .collect()
}

/// One castling of one colour, and everything standing in its way.
#[pyclass(
    frozen,
    skip_from_py_object,
    module = "esca.explain",
    name = "Castling"
)]
#[derive(Clone)]
pub struct PyCastling {
    /// Whose castling this is.
    #[pyo3(get)]
    colour: String,
    /// Which castling of that colour: `short` or `long`.
    #[pyo3(get)]
    wing: String,
    /// The position still holds this castling right.
    #[pyo3(get)]
    right: bool,
    /// The rook the right names stands on its square.
    #[pyo3(get)]
    rook_present: bool,
    /// The enemy units attacking the king where it stands.
    #[pyo3(get)]
    king_in_check_by: PySquareSet,
    /// Each square the king crosses or lands on that the enemy covers, with
    /// the units covering it.
    #[pyo3(get)]
    path_attacked: Vec<(String, PySquareSet)>,
    /// The units standing on squares the king or the rook must pass.
    #[pyo3(get)]
    path_blocked: PySquareSet,
    /// Nothing above prevents the castling.
    #[pyo3(get)]
    allowed: bool,
    inner: explain::Castling,
}

impl PyCastling {
    pub(crate) fn of(castling: &explain::Castling) -> PyCastling {
        PyCastling {
            colour: colour_name(castling.colour),
            wing: wing_name(castling.wing),
            right: castling.right,
            rook_present: castling.rook_present,
            king_in_check_by: PySquareSet::new(castling.king_in_check_by),
            path_attacked: pairs(&castling.path_attacked),
            path_blocked: PySquareSet::new(castling.path_blocked),
            allowed: castling.allowed,
            inner: castling.clone(),
        }
    }
}

#[pymethods]
impl PyCastling {
    /// One plain sentence: whether this castling is available, and every reason
    /// it is not.
    fn describe(&self) -> String {
        self.inner.describe()
    }

    fn __repr__(&self) -> String {
        format!("<Castling allowed={}>", self.allowed)
    }
}

/// The en-passant capture a position offers the side to move.
#[pyclass(
    frozen,
    skip_from_py_object,
    module = "esca.explain",
    name = "EnPassant"
)]
#[derive(Clone)]
pub struct PyEnPassant {
    /// The square a pawn skipped, if one did.
    #[pyo3(get)]
    target: Option<String>,
    /// Every pawn of the side to move standing beside it.
    #[pyo3(get)]
    captures: Vec<PyEpCapture>,
    inner: explain::EnPassant,
}

impl PyEnPassant {
    pub(crate) fn of(status: &explain::EnPassant) -> PyEnPassant {
        PyEnPassant {
            target: status.target().map(square_name),
            captures: status.captures().iter().map(PyEpCapture::of).collect(),
            inner: status.clone(),
        }
    }
}

#[pymethods]
impl PyEnPassant {
    /// One plain sentence: what the previous ply left on offer.
    fn describe(&self) -> String {
        self.inner.describe()
    }

    fn __repr__(&self) -> String {
        match &self.target {
            Some(target) => format!("<EnPassant {target}>"),
            None => "<EnPassant none>".to_string(),
        }
    }
}

/// One pawn's en-passant capture of the target.
#[pyclass(
    frozen,
    skip_from_py_object,
    module = "esca.explain",
    name = "EpCapture"
)]
#[derive(Clone)]
pub struct PyEpCapture {
    /// Where the capturing pawn stands.
    #[pyo3(get)]
    origin: String,
    /// The capture is a legal move.
    #[pyo3(get)]
    legal: bool,
    /// What forbids it, if anything does.
    #[pyo3(get)]
    forbidden_by: Option<PyEpObstacle>,
    inner: explain::EpCapture,
}

impl PyEpCapture {
    fn of(capture: &explain::EpCapture) -> PyEpCapture {
        PyEpCapture {
            origin: square_name(capture.from),
            legal: capture.legal,
            forbidden_by: capture.forbidden_by.as_ref().map(PyEpObstacle::of),
            inner: capture.clone(),
        }
    }
}

#[pymethods]
impl PyEpCapture {
    /// One plain sentence: whether this pawn may take, and what stops it.
    fn describe(&self) -> String {
        self.inner.describe()
    }

    fn __repr__(&self) -> String {
        format!("<EpCapture {} legal={}>", self.origin, self.legal)
    }
}

/// What keeps an en-passant capture off the board.
#[pyclass(
    frozen,
    skip_from_py_object,
    module = "esca.explain",
    name = "EpObstacle"
)]
#[derive(Clone)]
pub struct PyEpObstacle {
    /// `pinned`, `exposes_king` or `in_check`.
    #[pyo3(get)]
    kind: String,
    /// The ray a pinned pawn may not leave.
    #[pyo3(get)]
    ray: PySquareSet,
    /// The unit doing the pinning.
    #[pyo3(get)]
    pinner: Option<String>,
    /// The unit the two pawns hide.
    #[pyo3(get)]
    attacker: Option<String>,
    /// The units giving check.
    #[pyo3(get)]
    by: PySquareSet,
    inner: explain::EpObstacle,
}

impl PyEpObstacle {
    fn of(obstacle: &explain::EpObstacle) -> PyEpObstacle {
        let mut out = PyEpObstacle {
            kind: String::new(),
            ray: PySquareSet::new(SquareSet::EMPTY),
            pinner: None,
            attacker: None,
            by: PySquareSet::new(SquareSet::EMPTY),
            inner: obstacle.clone(),
        };
        match *obstacle {
            explain::EpObstacle::Pinned { ray, pinner } => {
                out.kind = "pinned".to_string();
                out.ray = PySquareSet::new(ray);
                out.pinner = Some(square_name(pinner));
            }
            explain::EpObstacle::ExposesKing { attacker } => {
                out.kind = "exposes_king".to_string();
                out.attacker = Some(square_name(attacker));
            }
            explain::EpObstacle::InCheck { by } => {
                out.kind = "in_check".to_string();
                out.by = PySquareSet::new(by);
            }
        }
        out
    }
}

#[pymethods]
impl PyEpObstacle {
    /// One plain sentence naming what keeps the capture off the board.
    fn describe(&self) -> String {
        self.inner.describe()
    }

    fn __repr__(&self) -> String {
        format!("<EpObstacle {}>", self.kind)
    }
}

/// A unit that may not move off the line between an enemy slider and its own
/// king.
#[pyclass(frozen, skip_from_py_object, module = "esca.explain", name = "Pin")]
#[derive(Clone)]
pub struct PyPin {
    /// The unit that may not move off the ray.
    #[pyo3(get)]
    pinned: String,
    /// The slider holding it there.
    #[pyo3(get)]
    pinner: String,
    /// The king behind it.
    #[pyo3(get)]
    king: String,
    /// Between pinner and king, exclusive.
    #[pyo3(get)]
    ray: PySquareSet,
    inner: explain::Pin,
}

impl PyPin {
    pub(crate) fn of(pin: &explain::Pin) -> PyPin {
        PyPin {
            pinned: square_name(pin.pinned),
            pinner: square_name(pin.pinner),
            king: square_name(pin.king),
            ray: PySquareSet::new(pin.ray),
            inner: *pin,
        }
    }
}

#[pymethods]
impl PyPin {
    /// One plain sentence naming the three squares of the pin.
    fn describe(&self) -> String {
        self.inner.describe()
    }

    fn __repr__(&self) -> String {
        format!("<Pin {} by {}>", self.pinned, self.pinner)
    }
}

/// A unit attacked with a less valuable one of the same colour directly
/// behind it on the slider's line.
#[pyclass(frozen, skip_from_py_object, module = "esca.explain", name = "Skewer")]
#[derive(Clone)]
pub struct PySkewer {
    /// The slider attacking the front unit.
    #[pyo3(get)]
    attacker: String,
    /// The unit in front, the more valuable of the two.
    #[pyo3(get)]
    front: String,
    /// The unit the front one shields.
    #[pyo3(get)]
    behind: String,
    /// Between attacker and the unit behind, exclusive.
    #[pyo3(get)]
    ray: PySquareSet,
    inner: explain::Skewer,
}

impl PySkewer {
    pub(crate) fn of(skewer: &explain::Skewer) -> PySkewer {
        PySkewer {
            attacker: square_name(skewer.attacker),
            front: square_name(skewer.front),
            behind: square_name(skewer.behind),
            ray: PySquareSet::new(skewer.ray),
            inner: *skewer,
        }
    }
}

#[pymethods]
impl PySkewer {
    /// One plain sentence naming the three squares of the skewer.
    fn describe(&self) -> String {
        self.inner.describe()
    }

    fn __repr__(&self) -> String {
        format!("<Skewer {} then {}>", self.front, self.behind)
    }
}

/// How often the current position has stood, and what nearly counted.
#[pyclass(
    frozen,
    skip_from_py_object,
    module = "esca.explain",
    name = "Repetition"
)]
#[derive(Clone)]
pub struct PyRepetition {
    /// How many of the plies are occurrences.
    #[pyo3(get)]
    count: u32,
    /// Every ply the current position occurred at, this one last.
    #[pyo3(get)]
    plies: Vec<u32>,
    /// The earlier plies with the same placement that do not count.
    #[pyo3(get)]
    near_misses: Vec<PyNearMiss>,
    inner: explain::Repetition,
}

impl PyRepetition {
    pub(crate) fn of(repetition: &explain::Repetition) -> PyRepetition {
        PyRepetition {
            count: repetition.count,
            plies: repetition.plies.clone(),
            near_misses: repetition.near_misses.iter().map(PyNearMiss::of).collect(),
            inner: repetition.clone(),
        }
    }
}

#[pymethods]
impl PyRepetition {
    /// One plain sentence: how often the position has stood, and what that is
    /// worth.
    fn describe(&self) -> String {
        self.inner.describe()
    }

    fn __repr__(&self) -> String {
        format!("<Repetition {}>", self.count)
    }
}

/// An earlier ply with the same placement that is not a repetition.
#[pyclass(
    frozen,
    skip_from_py_object,
    module = "esca.explain",
    name = "NearMiss"
)]
#[derive(Clone)]
pub struct PyNearMiss {
    /// The ply it stood at.
    #[pyo3(get)]
    ply: u32,
    /// Everything about it that differs: `castling_rights`, `en_passant`,
    /// `side_to_move`.
    #[pyo3(get)]
    differs: Vec<String>,
    inner: explain::NearMiss,
}

impl PyNearMiss {
    fn of(miss: &explain::NearMiss) -> PyNearMiss {
        PyNearMiss {
            ply: miss.ply,
            differs: miss
                .differs
                .iter()
                .map(|difference| {
                    match difference {
                        explain::Difference::CastlingRights => "castling_rights",
                        explain::Difference::EnPassant => "en_passant",
                        explain::Difference::SideToMove => "side_to_move",
                    }
                    .to_string()
                })
                .collect(),
            inner: miss.clone(),
        }
    }
}

#[pymethods]
impl PyNearMiss {
    /// One plain sentence: why this earlier ply is not a repetition.
    fn describe(&self) -> String {
        self.inner.describe()
    }

    fn __repr__(&self) -> String {
        format!("<NearMiss ply {}>", self.ply)
    }
}

/// The halfmove clock, and how far it is from ending the game.
#[pyclass(
    frozen,
    skip_from_py_object,
    module = "esca.explain",
    name = "FiftyMove"
)]
#[derive(Clone)]
pub struct PyFiftyMove {
    /// Plies since the last capture or pawn move.
    #[pyo3(get)]
    clock: u32,
    /// Plies until a player may claim; 0 once one may.
    #[pyo3(get)]
    plies_to_claim: u32,
    /// Plies until the draw is automatic; 0 once it is.
    #[pyo3(get)]
    plies_to_automatic: u32,
    /// The last move of this game that set the clock to 0.
    #[pyo3(get)]
    last_reset: Option<PyReset>,
    inner: explain::FiftyMove,
}

impl PyFiftyMove {
    pub(crate) fn of(fifty: &explain::FiftyMove) -> PyFiftyMove {
        PyFiftyMove {
            clock: fifty.clock,
            plies_to_claim: fifty.plies_to_claim,
            plies_to_automatic: fifty.plies_to_automatic,
            last_reset: fifty.last_reset.map(|reset| PyReset::of(&reset)),
            inner: fifty.clone(),
        }
    }
}

#[pymethods]
impl PyFiftyMove {
    /// One plain sentence: where the clock stands and what it counts towards.
    fn describe(&self) -> String {
        self.inner.describe()
    }

    fn __repr__(&self) -> String {
        format!("<FiftyMove clock {}>", self.clock)
    }
}

/// The move that last set the halfmove clock to 0.
#[pyclass(frozen, skip_from_py_object, module = "esca.explain", name = "Reset")]
#[derive(Clone)]
pub struct PyReset {
    /// The ply it produced.
    #[pyo3(get)]
    ply: u32,
    /// `capture` or `pawn_move`.
    #[pyo3(get)]
    kind: String,
    inner: explain::Reset,
}

impl PyReset {
    fn of(reset: &explain::Reset) -> PyReset {
        PyReset {
            ply: reset.ply,
            kind: match reset.kind {
                explain::ResetKind::Capture => "capture",
                explain::ResetKind::PawnMove => "pawn_move",
            }
            .to_string(),
            inner: *reset,
        }
    }
}

#[pymethods]
impl PyReset {
    /// One plain sentence naming the move that last cleared the clock.
    fn describe(&self) -> String {
        self.inner.describe()
    }

    fn __repr__(&self) -> String {
        format!("<Reset ply {} {}>", self.ply, self.kind)
    }
}

/// Every draw condition that holds, not the first of them.
#[pyclass(
    frozen,
    skip_from_py_object,
    module = "esca.explain",
    name = "DrawStatus"
)]
#[derive(Clone)]
pub struct PyDrawStatus {
    /// The draws that end the game as they are.
    #[pyo3(get)]
    automatic: Vec<PyAutomaticDraw>,
    /// The draws a player may ask for, the position still playable.
    #[pyo3(get)]
    claimable: Vec<PyClaimableDraw>,
    inner: explain::DrawStatus,
}

impl PyDrawStatus {
    pub(crate) fn of(status: &explain::DrawStatus) -> PyDrawStatus {
        PyDrawStatus {
            automatic: status.automatic.iter().map(PyAutomaticDraw::of).collect(),
            claimable: claims(&status.claimable),
            inner: status.clone(),
        }
    }
}

#[pymethods]
impl PyDrawStatus {
    /// One plain sentence per draw that holds, or one saying none does.
    fn describe(&self) -> String {
        self.inner.describe()
    }

    fn __repr__(&self) -> String {
        format!(
            "<DrawStatus automatic {} claimable {}>",
            self.automatic.len(),
            self.claimable.len()
        )
    }
}

/// The claimable draws of a status, or of a move not yet played.
pub(crate) fn claims(claims: &[explain::ClaimableDraw]) -> Vec<PyClaimableDraw> {
    claims.iter().map(PyClaimableDraw::of).collect()
}

/// A draw that needs no claim.
#[pyclass(
    frozen,
    skip_from_py_object,
    module = "esca.explain",
    name = "AutomaticDraw"
)]
#[derive(Clone)]
pub struct PyAutomaticDraw {
    /// `stalemate`, `insufficient_material`, `fivefold` or
    /// `seventy_five_moves`.
    #[pyo3(get)]
    kind: String,
    /// Why the side to move has no move.
    #[pyo3(get)]
    stalemate: Option<PyStalemateDetail>,
    /// The material configuration, `k_v_k` and its kin.
    #[pyo3(get)]
    material: Option<String>,
    /// The plies the position has stood at.
    #[pyo3(get)]
    repetition: Option<PyRepetition>,
    /// The clock that ran out.
    #[pyo3(get)]
    fifty_move: Option<PyFiftyMove>,
    inner: explain::AutomaticDraw,
}

impl PyAutomaticDraw {
    fn of(draw: &explain::AutomaticDraw) -> PyAutomaticDraw {
        let mut out = PyAutomaticDraw {
            kind: String::new(),
            stalemate: None,
            material: None,
            repetition: None,
            fifty_move: None,
            inner: draw.clone(),
        };
        match draw {
            explain::AutomaticDraw::Stalemate(detail) => {
                out.kind = "stalemate".to_string();
                out.stalemate = Some(PyStalemateDetail::of(detail));
            }
            explain::AutomaticDraw::InsufficientMaterial(config) => {
                out.kind = "insufficient_material".to_string();
                out.material = Some(material_name(*config).to_string());
            }
            explain::AutomaticDraw::Fivefold(repetition) => {
                out.kind = "fivefold".to_string();
                out.repetition = Some(PyRepetition::of(repetition));
            }
            explain::AutomaticDraw::SeventyFiveMoves(fifty) => {
                out.kind = "seventy_five_moves".to_string();
                out.fifty_move = Some(PyFiftyMove::of(fifty));
            }
        }
        out
    }
}

#[pymethods]
impl PyAutomaticDraw {
    /// One plain sentence naming the draw and why it needs no claim.
    fn describe(&self) -> String {
        self.inner.describe()
    }

    fn __repr__(&self) -> String {
        format!("<AutomaticDraw {}>", self.kind)
    }
}

/// The material a variant calls insufficient, named.
fn material_name(config: explain::MaterialConfig) -> &'static str {
    match config {
        explain::MaterialConfig::KvK => "k_v_k",
        explain::MaterialConfig::KNvK => "kn_v_k",
        explain::MaterialConfig::KBvK => "kb_v_k",
        explain::MaterialConfig::KBvKBSameColour => "kb_v_kb_same_colour",
    }
}

/// A draw a player may ask for.
#[pyclass(
    frozen,
    skip_from_py_object,
    module = "esca.explain",
    name = "ClaimableDraw"
)]
#[derive(Clone)]
pub struct PyClaimableDraw {
    /// `threefold` or `fifty_moves`.
    #[pyo3(get)]
    kind: String,
    /// The plies the position has stood at.
    #[pyo3(get)]
    repetition: Option<PyRepetition>,
    /// The clock that earns the claim.
    #[pyo3(get)]
    fifty_move: Option<PyFiftyMove>,
    inner: explain::ClaimableDraw,
}

impl PyClaimableDraw {
    fn of(claim: &explain::ClaimableDraw) -> PyClaimableDraw {
        match claim {
            explain::ClaimableDraw::Threefold(repetition) => PyClaimableDraw {
                kind: "threefold".to_string(),
                repetition: Some(PyRepetition::of(repetition)),
                fifty_move: None,
                inner: claim.clone(),
            },
            explain::ClaimableDraw::FiftyMoves(fifty) => PyClaimableDraw {
                kind: "fifty_moves".to_string(),
                repetition: None,
                fifty_move: Some(PyFiftyMove::of(fifty)),
                inner: claim.clone(),
            },
        }
    }
}

#[pymethods]
impl PyClaimableDraw {
    /// One plain sentence naming the draw a player may ask for.
    fn describe(&self) -> String {
        self.inner.describe()
    }

    fn __repr__(&self) -> String {
        format!("<ClaimableDraw {}>", self.kind)
    }
}

/// Why the side to move has no move.
#[pyclass(
    frozen,
    skip_from_py_object,
    module = "esca.explain",
    name = "StalemateDetail"
)]
#[derive(Clone)]
pub struct PyStalemateDetail {
    /// The king with nowhere to go.
    #[pyo3(get)]
    king: String,
    /// Each square beside the king that none of its own units holds, with the
    /// enemy units covering it.
    #[pyo3(get)]
    escape_squares: Vec<(String, PySquareSet)>,
    /// Every other unit of the side to move, and what holds it.
    #[pyo3(get)]
    stuck_units: Vec<(String, PyStuck)>,
    inner: explain::StalemateDetail,
}

impl PyStalemateDetail {
    fn of(detail: &explain::StalemateDetail) -> PyStalemateDetail {
        PyStalemateDetail {
            king: square_name(detail.king),
            escape_squares: pairs(&detail.escape_squares),
            stuck_units: detail
                .stuck_units
                .iter()
                .map(|&(square, stuck)| (square_name(square), PyStuck::of(stuck)))
                .collect(),
            inner: detail.clone(),
        }
    }
}

#[pymethods]
impl PyStalemateDetail {
    /// One plain sentence: the king with nowhere to go, and how much else is
    /// stuck with it.
    fn describe(&self) -> String {
        self.inner.describe()
    }

    fn __repr__(&self) -> String {
        format!("<StalemateDetail {}>", self.king)
    }
}

/// What holds a unit that has no legal move.
#[pyclass(frozen, skip_from_py_object, module = "esca.explain", name = "Stuck")]
#[derive(Clone)]
pub struct PyStuck {
    /// `pinned`, `blocked` or `no_moves`.
    #[pyo3(get)]
    kind: String,
    /// The ray a pinned unit may not leave.
    #[pyo3(get)]
    ray: PySquareSet,
    /// The unit doing the pinning.
    #[pyo3(get)]
    pinner: Option<String>,
    inner: explain::Stuck,
}

impl PyStuck {
    fn of(stuck: explain::Stuck) -> PyStuck {
        match stuck {
            explain::Stuck::Pinned { ray, pinner } => PyStuck {
                kind: "pinned".to_string(),
                ray: PySquareSet::new(ray),
                pinner: Some(square_name(pinner)),
                inner: stuck,
            },
            explain::Stuck::Blocked => PyStuck {
                kind: "blocked".to_string(),
                ray: PySquareSet::new(SquareSet::EMPTY),
                pinner: None,
                inner: stuck,
            },
            explain::Stuck::NoMoves => PyStuck {
                kind: "no_moves".to_string(),
                ray: PySquareSet::new(SquareSet::EMPTY),
                pinner: None,
                inner: stuck,
            },
        }
    }
}

#[pymethods]
impl PyStuck {
    /// One plain sentence naming what holds the unit.
    fn describe(&self) -> String {
        self.inner.describe()
    }

    fn __repr__(&self) -> String {
        format!("<Stuck {}>", self.kind)
    }
}
