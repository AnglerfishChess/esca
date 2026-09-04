//! The UCI client on the Python surface.
//!
//! Times are seconds here, scores are `cp`/`mate` pairs, and moves are `Move`
//! objects. Every call that waits releases the GIL.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyOSError, PyStopIteration, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::game::Game;
use crate::uci::{self, Info, Limits, Message, OptionKind, OptionSpec, OptionValue, Progress};

use super::board::{PyGame, PyMove};
use super::convert::castling_output_from;

create_exception!(
    esca.uci,
    UciError,
    PyException,
    "The base of every error talking to an engine raises."
);
create_exception!(
    esca.uci,
    EngineTimeout,
    UciError,
    "The engine did not say what was awaited in time."
);
create_exception!(
    esca.uci,
    EngineDied,
    UciError,
    "The engine closed its output or exited."
);
create_exception!(
    esca.uci,
    ProtocolError,
    UciError,
    "The engine broke the order of the conversation."
);

/// The Python exception for one client error.
fn to_py_error(error: uci::Error) -> PyErr {
    let message = error.to_string();
    match error {
        uci::Error::Timeout { .. } => EngineTimeout::new_err(message),
        uci::Error::Died { .. } => EngineDied::new_err(message),
        uci::Error::Protocol(_) | uci::Error::NotIdentified => ProtocolError::new_err(message),
        uci::Error::NoSuchOption(_) | uci::Error::BadValue { .. } => PyValueError::new_err(message),
        uci::Error::Io(_) => PyOSError::new_err(message),
    }
}

/// Seconds as a duration, refusing what no wait can be.
fn seconds(value: f64, what: &str) -> PyResult<Duration> {
    if !value.is_finite() || value < 0.0 {
        return Err(PyValueError::new_err(format!(
            "{what} is a count of seconds, not {value}"
        )));
    }
    Ok(Duration::from_secs_f64(value))
}

/// The shared engine, so that a search can be driven while the client holds
/// the engine too.
type Shared = Arc<Mutex<Option<uci::Engine>>>;

fn locked(shared: &Shared) -> MutexGuard<'_, Option<uci::Engine>> {
    shared.lock().unwrap_or_else(|held| held.into_inner())
}

/// Runs `work` on the engine with the GIL released.
fn on_engine<T: Send>(
    py: Python<'_>,
    shared: &Shared,
    work: impl FnOnce(&mut uci::Engine) -> Result<T, uci::Error> + Send,
) -> PyResult<T> {
    py.detach(|| {
        let mut guard = locked(shared);
        let engine = guard
            .as_mut()
            .ok_or_else(|| EngineDied::new_err("the engine has been closed"))?;
        work(engine).map_err(to_py_error)
    })
}

// -- Values -----------------------------------------------------------------

/// What bounds a search. Times are seconds; nothing set asks the engine to
/// search until stopped.
#[pyclass(frozen, from_py_object, module = "esca.uci", name = "Limits")]
#[derive(Clone)]
pub struct PyLimits {
    pub(crate) inner: Limits,
}

#[pymethods]
impl PyLimits {
    #[new]
    #[pyo3(signature = (
        *,
        depth = None,
        nodes = None,
        movetime = None,
        mate = None,
        infinite = false,
        ponder = false,
        white_time = None,
        black_time = None,
        white_increment = None,
        black_increment = None,
        moves_to_go = None,
        search_moves = None,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn py_new(
        depth: Option<u32>,
        nodes: Option<u64>,
        movetime: Option<f64>,
        mate: Option<u32>,
        infinite: bool,
        ponder: bool,
        white_time: Option<f64>,
        black_time: Option<f64>,
        white_increment: Option<f64>,
        black_increment: Option<f64>,
        moves_to_go: Option<u32>,
        search_moves: Option<Vec<String>>,
    ) -> PyResult<PyLimits> {
        let time = |value: Option<f64>, what: &str| -> PyResult<Option<Duration>> {
            value.map(|value| seconds(value, what)).transpose()
        };
        Ok(PyLimits {
            inner: Limits {
                search_moves: search_moves.unwrap_or_default(),
                ponder,
                white_time: time(white_time, "white_time")?,
                black_time: time(black_time, "black_time")?,
                white_increment: time(white_increment, "white_increment")?,
                black_increment: time(black_increment, "black_increment")?,
                moves_to_go,
                depth,
                nodes,
                mate,
                movetime: time(movetime, "movetime")?,
                infinite,
            },
        })
    }

    /// Plies to search.
    #[getter]
    fn depth(&self) -> Option<u32> {
        self.inner.depth
    }

    /// Nodes to search.
    #[getter]
    fn nodes(&self) -> Option<u64> {
        self.inner.nodes
    }

    /// Seconds to search for.
    #[getter]
    fn movetime(&self) -> Option<f64> {
        self.inner.movetime.map(|time| time.as_secs_f64())
    }

    /// Moves to a mate to search for.
    #[getter]
    fn mate(&self) -> Option<u32> {
        self.inner.mate
    }

    /// Whether the search runs until stopped.
    #[getter]
    fn infinite(&self) -> bool {
        self.inner.infinite
    }

    /// Whether the search runs on the move the position ends with.
    #[getter]
    fn ponder(&self) -> bool {
        self.inner.ponder
    }

    /// White's clock, in seconds.
    #[getter]
    fn white_time(&self) -> Option<f64> {
        self.inner.white_time.map(|time| time.as_secs_f64())
    }

    /// Black's clock, in seconds.
    #[getter]
    fn black_time(&self) -> Option<f64> {
        self.inner.black_time.map(|time| time.as_secs_f64())
    }

    /// White's increment per move, in seconds.
    #[getter]
    fn white_increment(&self) -> Option<f64> {
        self.inner.white_increment.map(|time| time.as_secs_f64())
    }

    /// Black's increment per move, in seconds.
    #[getter]
    fn black_increment(&self) -> Option<f64> {
        self.inner.black_increment.map(|time| time.as_secs_f64())
    }

    /// Moves left to the next time control.
    #[getter]
    fn moves_to_go(&self) -> Option<u32> {
        self.inner.moves_to_go
    }

    /// The only moves to consider, in UCI notation.
    #[getter]
    fn search_moves(&self) -> Vec<String> {
        self.inner.search_moves.clone()
    }

    fn __repr__(&self) -> String {
        format!("<Limits {}>", uci::Command::Go(self.inner.clone()))
    }
}

/// One option an engine offers, and the domain it declared for it.
#[pyclass(frozen, skip_from_py_object, module = "esca.uci", name = "Option")]
#[derive(Clone)]
pub struct PyOption {
    inner: OptionSpec,
}

#[pymethods]
impl PyOption {
    /// The name to set it by, as the engine wrote it.
    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    /// `check`, `spin`, `combo`, `button` or `string`.
    #[getter]
    fn r#type(&self) -> &'static str {
        self.inner.kind.type_name()
    }

    /// The value the engine starts with, of this option's own type.
    #[getter]
    fn default(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        Ok(match &self.inner.kind {
            OptionKind::Check { default } => default.into_pyobject(py)?.into_any().unbind(),
            OptionKind::Spin { default, .. } => default.into_pyobject(py)?.into_any().unbind(),
            OptionKind::Combo { default, .. } | OptionKind::String { default } => {
                default.clone().into_pyobject(py)?.into_any().unbind()
            }
            OptionKind::Button => py.None(),
        })
    }

    /// The smallest value a `spin` accepts.
    #[getter]
    fn min(&self) -> Option<i64> {
        match self.inner.kind {
            OptionKind::Spin { min, .. } => min,
            _ => None,
        }
    }

    /// The largest value a `spin` accepts.
    #[getter]
    fn max(&self) -> Option<i64> {
        match self.inner.kind {
            OptionKind::Spin { max, .. } => max,
            _ => None,
        }
    }

    /// The values a `combo` offers, in the order declared.
    #[getter]
    fn vars(&self) -> Vec<String> {
        match &self.inner.kind {
            OptionKind::Combo { vars, .. } => vars.clone(),
            _ => Vec::new(),
        }
    }

    /// The text a `setoption` carries to set this option to `value`, refusing
    /// a value the declared domain does not accept; `None` for a button.
    #[pyo3(signature = (value = None))]
    fn value_text(&self, value: Option<&Bound<'_, PyAny>>) -> PyResult<Option<String>> {
        let read = read_value(&self.inner.kind, &self.inner.name, value)?;
        self.inner.accepts(&read).map_err(|reason| {
            PyValueError::new_err(format!("option {:?}: {reason}", self.inner.name))
        })?;
        Ok(read.to_text())
    }

    fn __repr__(&self) -> String {
        format!("<Option {} type {}>", self.inner.name, self.r#type())
    }
}

/// One `info` line of a search, with its moves read against the position
/// searched.
#[pyclass(frozen, skip_from_py_object, module = "esca.uci", name = "Info")]
#[derive(Clone)]
pub struct PyInfo {
    depth: Option<u32>,
    seldepth: Option<u32>,
    time: Option<f64>,
    nodes: Option<u64>,
    nps: Option<u64>,
    pv: Vec<PyMove>,
    multipv: Option<u32>,
    cp: Option<i32>,
    mate: Option<i32>,
    bound: Option<&'static str>,
    wdl: Option<(u32, u32, u32)>,
    currmove: Option<PyMove>,
    currmovenumber: Option<u32>,
    hashfull: Option<u32>,
    tbhits: Option<u64>,
    sbhits: Option<u64>,
    cpuload: Option<u32>,
    refutation: Vec<PyMove>,
    currline: Vec<PyMove>,
    currline_cpu: Option<u32>,
    string: Option<String>,
    unknown: Vec<String>,
}

impl PyInfo {
    /// One report, with its move text read as moves of `game`.
    fn of(info: &Info, game: &Game) -> PyInfo {
        let moves = |line: &[String]| {
            uci::moves_of_line(game, line)
                .into_iter()
                .map(PyMove::new)
                .collect()
        };
        PyInfo {
            depth: info.depth,
            seldepth: info.seldepth,
            time: info.time.map(|time| time.as_secs_f64()),
            nodes: info.nodes,
            nps: info.nps,
            pv: moves(&info.pv),
            multipv: info.multipv,
            cp: match info.score {
                Some(crate::position::Score::Cp(cp)) => Some(cp),
                _ => None,
            },
            mate: match info.score {
                Some(crate::position::Score::Mate(mate)) => Some(mate),
                _ => None,
            },
            bound: info.bound.map(|bound| match bound {
                uci::Bound::Lower => "lowerbound",
                uci::Bound::Upper => "upperbound",
            }),
            wdl: info.wdl.map(|wdl| (wdl.win, wdl.draw, wdl.loss)),
            currmove: info.current_move(game).map(PyMove::new),
            currmovenumber: info.currmovenumber,
            hashfull: info.hashfull,
            tbhits: info.tbhits,
            sbhits: info.sbhits,
            cpuload: info.cpuload,
            refutation: moves(&info.refutation),
            currline: info
                .currline
                .as_ref()
                .map(|line| moves(&line.moves))
                .unwrap_or_default(),
            currline_cpu: info.currline.as_ref().and_then(|line| line.cpu),
            string: info.string.clone(),
            unknown: info.unknown.clone(),
        }
    }
}

#[pymethods]
impl PyInfo {
    /// Plies searched.
    #[getter]
    fn depth(&self) -> Option<u32> {
        self.depth
    }

    /// Plies searched on the deepest line.
    #[getter]
    fn seldepth(&self) -> Option<u32> {
        self.seldepth
    }

    /// Seconds spent.
    #[getter]
    fn time(&self) -> Option<f64> {
        self.time
    }

    /// Nodes searched.
    #[getter]
    fn nodes(&self) -> Option<u64> {
        self.nodes
    }

    /// Nodes per second.
    #[getter]
    fn nps(&self) -> Option<u64> {
        self.nps
    }

    /// The principal variation, up to the first move that is not legal.
    #[getter]
    fn pv(&self) -> Vec<PyMove> {
        self.pv.clone()
    }

    /// Which of the ranked variations this report is, counting from 1.
    #[getter]
    fn multipv(&self) -> Option<u32> {
        self.multipv
    }

    /// The score in centipawns, from the side to move's point of view.
    #[getter]
    fn cp(&self) -> Option<i32> {
        self.cp
    }

    /// Moves to a forced mate, negative when it is against the side to move.
    #[getter]
    fn mate(&self) -> Option<i32> {
        self.mate
    }

    /// `lowerbound`, `upperbound`, or `None` when the score is exact.
    #[getter]
    fn bound(&self) -> Option<&'static str> {
        self.bound
    }

    /// Win, draw and loss in permille, for the side to move.
    #[getter]
    fn wdl(&self) -> Option<(u32, u32, u32)> {
        self.wdl
    }

    /// The move being searched.
    #[getter]
    fn currmove(&self) -> Option<PyMove> {
        self.currmove
    }

    /// Its ordinal among the root moves, counting from 1.
    #[getter]
    fn currmovenumber(&self) -> Option<u32> {
        self.currmovenumber
    }

    /// How full the hash table is, in permille.
    #[getter]
    fn hashfull(&self) -> Option<u32> {
        self.hashfull
    }

    /// Endgame-table hits.
    #[getter]
    fn tbhits(&self) -> Option<u64> {
        self.tbhits
    }

    /// Shredder-base hits.
    #[getter]
    fn sbhits(&self) -> Option<u64> {
        self.sbhits
    }

    /// CPU load, in permille.
    #[getter]
    fn cpuload(&self) -> Option<u32> {
        self.cpuload
    }

    /// The move refuted followed by the line that refutes it.
    #[getter]
    fn refutation(&self) -> Vec<PyMove> {
        self.refutation.clone()
    }

    /// The line a CPU is searching now.
    #[getter]
    fn currline(&self) -> Vec<PyMove> {
        self.currline.clone()
    }

    /// Which CPU `currline` belongs to, when the engine says.
    #[getter]
    fn currline_cpu(&self) -> Option<u32> {
        self.currline_cpu
    }

    /// The rest of the line after `string`, spacing kept.
    #[getter]
    fn string(&self) -> Option<String> {
        self.string.clone()
    }

    /// Tokens the parser did not understand, in the order they arrived.
    #[getter]
    fn unknown(&self) -> Vec<String> {
        self.unknown.clone()
    }

    fn __repr__(&self) -> String {
        let score = match (self.cp, self.mate) {
            (Some(cp), _) => format!(" cp {cp}"),
            (_, Some(mate)) => format!(" mate {mate}"),
            _ => String::new(),
        };
        format!(
            "<Info depth {}{score} pv {}>",
            self.depth.map_or("-".to_owned(), |depth| depth.to_string()),
            self.pv
                .iter()
                .map(|mv| mv.inner.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

/// An engine's answer to a search.
#[pyclass(frozen, skip_from_py_object, module = "esca.uci", name = "Answer")]
#[derive(Clone, Copy)]
pub struct PyAnswer {
    best: Option<PyMove>,
    ponder: Option<PyMove>,
}

#[pymethods]
impl PyAnswer {
    /// The move chosen, or `None` when the engine reported that it has none.
    #[getter]
    fn best(&self) -> Option<PyMove> {
        self.best
    }

    /// The reply it expects.
    #[getter]
    fn ponder(&self) -> Option<PyMove> {
        self.ponder
    }

    fn __repr__(&self) -> String {
        match self.best {
            None => "<Answer (none)>".to_owned(),
            Some(best) => format!("<Answer {}>", best.inner),
        }
    }
}

impl PyAnswer {
    fn of(answer: uci::Answer) -> PyAnswer {
        PyAnswer {
            best: answer.best.map(PyMove::new),
            ponder: answer.ponder.map(PyMove::new),
        }
    }
}

// -- The protocol as values -------------------------------------------------

/// One line a client sends to an engine.
#[pyclass(
    frozen,
    skip_from_py_object,
    module = "esca.uci.protocol",
    name = "Command"
)]
#[derive(Clone)]
pub struct PyCommand {
    inner: uci::Command,
}

#[pymethods]
impl PyCommand {
    /// Ask the engine to identify itself and list its options.
    #[staticmethod]
    fn uci() -> PyCommand {
        PyCommand {
            inner: uci::Command::Uci,
        }
    }

    /// Turn the engine's `info string` diagnostics on or off.
    #[staticmethod]
    fn debug(on: bool) -> PyCommand {
        PyCommand {
            inner: uci::Command::Debug(on),
        }
    }

    /// Ask for a `readyok`.
    #[staticmethod]
    fn isready() -> PyCommand {
        PyCommand {
            inner: uci::Command::IsReady,
        }
    }

    /// Set one option; the text is the value as the engine will read it, and
    /// `None` is what a button carries.
    #[staticmethod]
    #[pyo3(signature = (name, value = None))]
    fn setoption(name: String, value: Option<String>) -> PyCommand {
        PyCommand {
            inner: uci::Command::SetOption { name, value },
        }
    }

    /// Announce that the next position belongs to a new game.
    #[staticmethod]
    fn ucinewgame() -> PyCommand {
        PyCommand {
            inner: uci::Command::NewGame,
        }
    }

    /// Set the position to `game`, its moves written as `castling` asks, or as
    /// the game's own variant spells them.
    #[staticmethod]
    #[pyo3(signature = (game, castling = None))]
    fn position(game: &PyGame, castling: Option<&str>) -> PyResult<PyCommand> {
        let played = game.played();
        let style = match castling {
            Some(name) => castling_output_from(name)?,
            None => played.castling_output(),
        };
        Ok(PyCommand {
            inner: uci::Command::Position(uci::Setup::of_game(played, style)),
        })
    }

    /// Start searching under `limits`; none asks for a search until stopped.
    #[staticmethod]
    #[pyo3(signature = (limits = None))]
    fn go(limits: Option<PyLimits>) -> PyCommand {
        PyCommand {
            inner: uci::Command::Go(limits.map(|limits| limits.inner).unwrap_or_default()),
        }
    }

    /// Ask the search to finish now.
    #[staticmethod]
    fn stop() -> PyCommand {
        PyCommand {
            inner: uci::Command::Stop,
        }
    }

    /// Tell the engine the move it is pondering on was played.
    #[staticmethod]
    fn ponderhit() -> PyCommand {
        PyCommand {
            inner: uci::Command::PonderHit,
        }
    }

    /// Ask the engine to exit.
    #[staticmethod]
    fn quit() -> PyCommand {
        PyCommand {
            inner: uci::Command::Quit,
        }
    }

    /// The line to write, without its newline.
    fn to_line(&self) -> String {
        self.inner.to_line()
    }

    /// The keyword the command is named by.
    #[getter]
    fn keyword(&self) -> &'static str {
        self.inner.keyword()
    }

    fn __repr__(&self) -> String {
        format!("<Command {}>", self.inner.to_line())
    }
}

/// One line an engine sent, read into what it says.
#[pyclass(
    frozen,
    skip_from_py_object,
    module = "esca.uci.protocol",
    name = "Message"
)]
#[derive(Clone)]
pub struct PyMessage {
    inner: Message,
    line: String,
    option: Option<PyOption>,
    info: Option<PyInfo>,
    answer: Option<PyAnswer>,
}

impl PyMessage {
    /// One line, with its moves read as moves of `game`.
    fn of(message: Message, line: &str, game: &Game) -> PyMessage {
        PyMessage {
            option: match &message {
                Message::Option(spec) => Some(PyOption {
                    inner: spec.clone(),
                }),
                _ => None,
            },
            info: match &message {
                Message::Info(info) => Some(PyInfo::of(info, game)),
                _ => None,
            },
            answer: match &message {
                Message::BestMove(best) => Some(PyAnswer {
                    best: best.best_move(game).map(PyMove::new),
                    ponder: best.ponder_move(game).map(PyMove::new),
                }),
                _ => None,
            },
            line: line.to_owned(),
            inner: message,
        }
    }
}

#[pymethods]
impl PyMessage {
    /// What the line says: `id`, `uciok`, `readyok`, `option`, `info`,
    /// `bestmove`, `registration`, `copyprotection`, or `raw` for a line the
    /// grammar has no reading for.
    #[getter]
    fn kind(&self) -> &'static str {
        match self.inner {
            Message::Id { .. } => "id",
            Message::UciOk => "uciok",
            Message::ReadyOk => "readyok",
            Message::Option(_) => "option",
            Message::Info(_) => "info",
            Message::BestMove(_) => "bestmove",
            Message::Registration(_) => "registration",
            Message::CopyProtection(_) => "copyprotection",
            Message::Raw(_) => "raw",
        }
    }

    /// The line as it arrived.
    #[getter]
    fn line(&self) -> String {
        self.line.clone()
    }

    /// What an `id` names: `name`, `author`, or the key the engine chose.
    #[getter]
    fn key(&self) -> Option<String> {
        match &self.inner {
            Message::Id { key, .. } => Some(key.clone()),
            _ => None,
        }
    }

    /// The rest of an `id` line.
    #[getter]
    fn value(&self) -> Option<String> {
        match &self.inner {
            Message::Id { value, .. } => Some(value.clone()),
            _ => None,
        }
    }

    /// How a `registration` or `copyprotection` check went: `checking`, `ok`
    /// or `error`.
    #[getter]
    fn status(&self) -> Option<&'static str> {
        let status = match self.inner {
            Message::Registration(status) | Message::CopyProtection(status) => status,
            _ => return None,
        };
        Some(match status {
            uci::Status::Checking => "checking",
            uci::Status::Ok => "ok",
            uci::Status::Error => "error",
        })
    }

    /// The option an `option` line declares.
    #[getter]
    fn option(&self) -> Option<PyOption> {
        self.option.clone()
    }

    /// The report an `info` line carries.
    #[getter]
    fn info(&self) -> Option<PyInfo> {
        self.info.clone()
    }

    /// The answer a `bestmove` line carries.
    #[getter]
    fn answer(&self) -> Option<PyAnswer> {
        self.answer
    }

    fn __repr__(&self) -> String {
        format!("<Message {} {:?}>", self.kind(), self.line)
    }
}

/// Reads one line of engine output. Moves are read against `game`, or against
/// the start of a classic game. Reading never fails: a line the grammar has no
/// reading for is a `raw` message.
#[pyfunction]
#[pyo3(signature = (line, game = None))]
fn uci_parse(line: &str, game: Option<&PyGame>) -> PyMessage {
    let played = game
        .map(|game| game.played().clone())
        .unwrap_or_else(|| Game::new(crate::variant::classic()));
    PyMessage::of(uci::parse(line), line, &played)
}

/// Which commands may go out and which messages may come in, tracked over one
/// conversation.
#[pyclass(module = "esca.uci.protocol", name = "Session")]
pub struct PySession {
    inner: uci::Session,
}

#[pymethods]
impl PySession {
    /// A session with nothing asked yet.
    #[new]
    fn py_new() -> PySession {
        PySession {
            inner: uci::Session::new(),
        }
    }

    /// What the engine is doing: `started`, `identifying`, `idle`,
    /// `searching`, `pondering` or `quitting`.
    #[getter]
    fn state(&self) -> &'static str {
        self.inner.state().name()
    }

    /// How many `isready` commands are still unanswered.
    #[getter]
    fn pending_ready(&self) -> u32 {
        self.inner.pending_ready()
    }

    /// Records a command as sent, raising `ProtocolError` for one this state
    /// has no room for.
    fn sent(&mut self, command: &PyCommand) -> PyResult<()> {
        self.inner
            .sent(&command.inner)
            .map_err(|error| ProtocolError::new_err(error.to_string()))
    }

    /// Records a message as received, raising `ProtocolError` for one this
    /// state has no room for.
    fn received(&mut self, message: &PyMessage) -> PyResult<()> {
        self.inner
            .received(&message.inner)
            .map_err(|error| ProtocolError::new_err(error.to_string()))
    }

    fn __repr__(&self) -> String {
        format!("<Session {}>", self.state())
    }
}

// -- The engine -------------------------------------------------------------

/// A UCI engine process, addressed with blocking calls.
///
/// Every wait is bounded by `timeout`, or by the `timeout` of the call that
/// takes one; an engine that has exited raises `EngineDied` on every call
/// after. Use it as a context manager, or call `quit`.
#[pyclass(frozen, module = "esca.uci", name = "Engine")]
pub struct PyEngine {
    shared: Shared,
}

#[pymethods]
impl PyEngine {
    #[new]
    #[pyo3(signature = (command, args = Vec::new(), *, cwd = None, timeout = 10.0))]
    fn py_new(
        command: PathBuf,
        args: Vec<String>,
        cwd: Option<PathBuf>,
        timeout: f64,
    ) -> PyResult<PyEngine> {
        let mut launch = uci::Launch::new(command)
            .args(args)
            .timeout(seconds(timeout, "timeout")?);
        if let Some(cwd) = cwd {
            launch = launch.current_dir(cwd);
        }
        let engine = launch.spawn().map_err(to_py_error)?;
        Ok(PyEngine {
            shared: Arc::new(Mutex::new(Some(engine))),
        })
    }

    /// How long a wait that is not given its own limit may take, in seconds.
    #[getter]
    fn timeout(&self) -> f64 {
        locked(&self.shared)
            .as_ref()
            .map_or(0.0, |engine| engine.timeout().as_secs_f64())
    }

    #[setter]
    fn set_timeout(&self, timeout: f64) -> PyResult<()> {
        let timeout = seconds(timeout, "timeout")?;
        if let Some(engine) = locked(&self.shared).as_mut() {
            engine.set_timeout(timeout);
        }
        Ok(())
    }

    /// Sends `uci` and collects what the engine says about itself.
    fn handshake(&self, py: Python<'_>) -> PyResult<()> {
        on_engine(py, &self.shared, |engine| engine.handshake().map(|_| ()))
    }

    /// The engine's `id name`.
    #[getter]
    fn name(&self) -> Option<String> {
        locked(&self.shared)
            .as_ref()
            .and_then(|engine| engine.identity().name.clone())
    }

    /// The engine's `id author`.
    #[getter]
    fn author(&self) -> Option<String> {
        locked(&self.shared)
            .as_ref()
            .and_then(|engine| engine.identity().author.clone())
    }

    /// Every option the engine offers, by name, in the order it declared them.
    #[getter]
    fn options<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let options = PyDict::new(py);
        if let Some(engine) = locked(&self.shared).as_ref() {
            for spec in engine.options() {
                options.set_item(
                    spec.name.clone(),
                    PyOption {
                        inner: spec.clone(),
                    },
                )?;
            }
        }
        Ok(options)
    }

    /// The option of that name, matched without regard to case.
    fn option(&self, name: &str) -> Option<PyOption> {
        locked(&self.shared)
            .as_ref()
            .and_then(|engine| engine.option(name))
            .map(|spec| PyOption {
                inner: spec.clone(),
            })
    }

    /// Sets one option to a value of the option's own type: a bool for a
    /// `check`, an int for a `spin`, text for a `combo` or a `string`, and
    /// `None` for a `button`.
    #[pyo3(signature = (name, value = None))]
    fn set_option(
        &self,
        py: Python<'_>,
        name: &str,
        value: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        let kind = self
            .option(name)
            .ok_or_else(|| PyValueError::new_err(format!("the engine offers no option {name:?}")))?
            .inner
            .kind;
        let value = read_value(&kind, name, value)?;
        on_engine(py, &self.shared, move |engine| {
            engine.set_option(name, value)
        })
    }

    /// Turns the engine's `info string` diagnostics on or off.
    fn debug(&self, py: Python<'_>, on: bool) -> PyResult<()> {
        on_engine(py, &self.shared, |engine| engine.set_debug(on))
    }

    /// Announces a new game and waits for the engine to be ready again.
    fn new_game(&self, py: Python<'_>) -> PyResult<()> {
        on_engine(py, &self.shared, |engine| engine.new_game())
    }

    /// Sends `isready` and waits for `readyok`.
    fn is_ready(&self, py: Python<'_>) -> PyResult<()> {
        on_engine(py, &self.shared, |engine| engine.is_ready())
    }

    /// Sets the position to `game`, putting the engine into its variant first.
    ///
    /// A Chess960 game needs the engine to offer `UCI_Chess960`; one that does
    /// not raises `ValueError` rather than playing by the wrong rules.
    fn set_position(&self, py: Python<'_>, game: &PyGame) -> PyResult<()> {
        let game = game.played().clone();
        on_engine(py, &self.shared, move |engine| engine.set_position(&game))
    }

    /// Starts a search on the position last set, to be read report by report.
    #[pyo3(signature = (limits = None, *, timeout = None))]
    fn go(
        &self,
        py: Python<'_>,
        limits: Option<PyLimits>,
        timeout: Option<f64>,
    ) -> PyResult<PySearch> {
        let limits = limits.map(|limits| limits.inner).unwrap_or_default();
        let budget = self.budget(timeout)?;
        let game = on_engine(py, &self.shared, move |engine| {
            engine.start_search(&limits)?;
            Ok(engine.game().cloned())
        })?;
        let now = Instant::now();
        Ok(PySearch {
            shared: Arc::clone(&self.shared),
            game: game.unwrap_or_else(|| Game::new(crate::variant::classic())),
            until: now
                .checked_add(budget)
                .unwrap_or_else(|| now + Duration::from_secs(60 * 60 * 24 * 365)),
            answer: Mutex::new(None),
        })
    }

    /// Sets the position, searches it, and answers.
    #[pyo3(signature = (game, limits = None, *, timeout = None))]
    fn play(
        &self,
        py: Python<'_>,
        game: &PyGame,
        limits: Option<PyLimits>,
        timeout: Option<f64>,
    ) -> PyResult<PyAnswer> {
        let limits = limits.map(|limits| limits.inner).unwrap_or_default();
        let budget = self.budget(timeout)?;
        let game = game.played().clone();
        let answer = on_engine(py, &self.shared, move |engine| {
            engine.play(&game, &limits, budget)
        })?;
        Ok(PyAnswer::of(answer))
    }

    /// Searches `game` and answers with the deepest report of each variation,
    /// ranked as the engine ranked them.
    #[pyo3(signature = (game, limits = None, *, multipv = None, timeout = None))]
    fn analyse(
        &self,
        py: Python<'_>,
        game: &PyGame,
        limits: Option<PyLimits>,
        multipv: Option<i64>,
        timeout: Option<f64>,
    ) -> PyResult<Vec<PyInfo>> {
        if let Some(lines) = multipv {
            self.set_option(
                py,
                "MultiPV",
                Some(&pyo3::types::PyInt::new(py, lines).into_any()),
            )?;
        }
        let limits = limits.map(|limits| limits.inner).unwrap_or_default();
        let budget = self.budget(timeout)?;
        let searched = game.played().clone();
        let reports = on_engine(py, &self.shared, move |engine| {
            engine.set_position(&searched)?;
            let mut search = engine.go(&limits, budget)?;
            let mut reports: Vec<Info> = Vec::new();
            while let Some(info) = search.next_info()? {
                if info.score.is_none() {
                    continue;
                }
                let line = info.multipv.unwrap_or(1);
                match reports
                    .iter_mut()
                    .find(|kept| kept.multipv.unwrap_or(1) == line)
                {
                    Some(kept) => *kept = info,
                    None => reports.push(info),
                }
            }
            Ok(reports)
        })?;
        let game = game.played();
        let mut reports: Vec<PyInfo> = reports.iter().map(|info| PyInfo::of(info, game)).collect();
        reports.sort_by_key(|info| info.multipv.unwrap_or(1));
        Ok(reports)
    }

    /// Asks the search in flight to finish now.
    fn stop(&self, py: Python<'_>) -> PyResult<()> {
        on_engine(py, &self.shared, |engine| engine.stop())
    }

    /// Tells the engine the move it is pondering on was played.
    fn ponderhit(&self, py: Python<'_>) -> PyResult<()> {
        on_engine(py, &self.shared, |engine| engine.ponderhit())
    }

    /// Writes one line, bypassing the order of the conversation.
    fn send_line(&self, py: Python<'_>, text: &str) -> PyResult<()> {
        on_engine(py, &self.shared, |engine| engine.send_line(text))
    }

    /// The next line the engine wrote, or `None` if it wrote none in time.
    #[pyo3(signature = (timeout = None))]
    fn next_line(&self, py: Python<'_>, timeout: Option<f64>) -> PyResult<Option<String>> {
        let budget = self.budget(timeout)?;
        on_engine(py, &self.shared, move |engine| engine.next_line(budget))
    }

    /// What the engine is doing: `started`, `identifying`, `idle`,
    /// `searching`, `pondering` or `quitting`.
    #[getter]
    fn state(&self) -> &'static str {
        locked(&self.shared)
            .as_ref()
            .map_or("quitting", |engine| engine.state().name())
    }

    /// Whether the process is still running.
    #[getter]
    fn is_alive(&self) -> bool {
        locked(&self.shared)
            .as_mut()
            .is_some_and(|engine| engine.is_alive())
    }

    /// Asks the engine to exit, killing it if it will not, and answers with
    /// its exit code.
    fn quit(&self, py: Python<'_>) -> PyResult<Option<i32>> {
        py.detach(|| {
            let mut guard = locked(&self.shared);
            match guard.take() {
                None => Ok(None),
                Some(mut engine) => engine.quit().map_err(to_py_error),
            }
        })
    }

    /// Kills the process.
    fn kill(&self, py: Python<'_>) {
        py.detach(|| {
            if let Some(mut engine) = locked(&self.shared).take() {
                engine.kill();
            }
        });
    }

    fn __enter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    #[pyo3(signature = (*_args))]
    fn __exit__(&self, py: Python<'_>, _args: &Bound<'_, PyAny>) -> PyResult<()> {
        self.quit(py).map(|_| ())
    }

    fn __repr__(&self) -> String {
        match self.name() {
            Some(name) => format!("<Engine {name} {}>", self.state()),
            None => format!("<Engine {}>", self.state()),
        }
    }
}

impl PyEngine {
    /// How long the whole of one waiting call may take.
    fn budget(&self, timeout: Option<f64>) -> PyResult<Duration> {
        match timeout {
            Some(timeout) => seconds(timeout, "timeout"),
            None => Ok(locked(&self.shared)
                .as_ref()
                .map_or(uci::DEFAULT_TIMEOUT, |engine| engine.timeout())),
        }
    }
}

/// The value an option of that kind takes, read from Python. A missing value
/// is `None`, which only a button takes.
fn read_value(
    kind: &OptionKind,
    name: &str,
    value: Option<&Bound<'_, PyAny>>,
) -> PyResult<OptionValue> {
    let wrong = |wanted: &str| {
        PyValueError::new_err(format!(
            "option {name:?} is a {} option, which takes {wanted}",
            kind.type_name()
        ))
    };
    let Some(value) = value.filter(|value| !value.is_none()) else {
        return match kind {
            OptionKind::Button => Ok(OptionValue::Button),
            _ => Err(wrong("a value")),
        };
    };
    match kind {
        OptionKind::Check { .. } => value
            .extract::<bool>()
            .map(OptionValue::Check)
            .map_err(|_| wrong("True or False")),
        // True is an int in Python, but never a count of anything here.
        OptionKind::Spin { .. } if value.is_instance_of::<pyo3::types::PyBool>() => {
            Err(wrong("an integer"))
        }
        OptionKind::Spin { .. } => value
            .extract::<i64>()
            .map(OptionValue::Spin)
            .map_err(|_| wrong("an integer")),
        OptionKind::Combo { .. } => value
            .extract::<String>()
            .map(OptionValue::Combo)
            .map_err(|_| wrong("one of its values")),
        OptionKind::String { .. } => value
            .extract::<String>()
            .map(OptionValue::String)
            .map_err(|_| wrong("text")),
        OptionKind::Button => Err(wrong("no value")),
    }
}

// -- A search ---------------------------------------------------------------

/// A search in flight: an iterator over its reports, ending with the engine's
/// answer.
///
/// A search that is neither finished nor stopped leaves the engine searching.
/// Use it as a context manager, or call `stop` and `answer`.
#[pyclass(frozen, module = "esca.uci", name = "Search")]
pub struct PySearch {
    shared: Shared,
    game: Game,
    until: Instant,
    answer: Mutex<Option<PyAnswer>>,
}

#[pymethods]
impl PySearch {
    /// Whether the engine has answered.
    #[getter]
    fn done(&self) -> bool {
        self.answered().is_some()
    }

    /// The engine's answer, with the reports still to come dropped.
    fn answer(&self, py: Python<'_>) -> PyResult<PyAnswer> {
        loop {
            if let Some(answer) = self.answered() {
                return Ok(answer);
            }
            self.progress(py)?;
        }
    }

    /// Asks the engine to finish the search now.
    fn stop(&self, py: Python<'_>) -> PyResult<()> {
        on_engine(py, &self.shared, |engine| engine.stop())
    }

    /// Tells the engine the move it is pondering on was played.
    fn ponderhit(&self, py: Python<'_>) -> PyResult<()> {
        on_engine(py, &self.shared, |engine| engine.ponderhit())
    }

    fn __iter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __next__(&self, py: Python<'_>) -> PyResult<PyInfo> {
        if self.done() {
            return Err(PyStopIteration::new_err(()));
        }
        match self.progress(py)? {
            Some(info) => Ok(info),
            None => Err(PyStopIteration::new_err(())),
        }
    }

    fn __enter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    /// Stops the search and waits for the answer, so the engine is left idle.
    #[pyo3(signature = (*_args))]
    fn __exit__(&self, py: Python<'_>, _args: &Bound<'_, PyAny>) -> PyResult<()> {
        if self.done() {
            return Ok(());
        }
        self.stop(py)?;
        self.answer(py).map(|_| ())
    }

    fn __repr__(&self) -> String {
        format!(
            "<Search {}>",
            if self.done() { "answered" } else { "running" }
        )
    }
}

impl PySearch {
    /// The answer, if the engine has given one. Never held across a wait, so
    /// that a caller asking while a search runs is not made to wait for it.
    fn answered(&self) -> Option<PyAnswer> {
        *self.answer.lock().unwrap_or_else(|held| held.into_inner())
    }

    /// The next report, or `None` when the answer arrived instead.
    fn progress(&self, py: Python<'_>) -> PyResult<Option<PyInfo>> {
        let left = self.until.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(EngineTimeout::new_err("no bestmove within the timeout"));
        }
        let progress = on_engine(py, &self.shared, move |engine| engine.next_progress(left))?;
        match progress {
            Progress::Info(info) => Ok(Some(PyInfo::of(&info, &self.game))),
            Progress::Done(answer) => {
                *self.answer.lock().unwrap_or_else(|held| held.into_inner()) =
                    Some(PyAnswer::of(answer));
                Ok(None)
            }
        }
    }
}

/// Adds the client's classes and exceptions to the extension module.
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = module.py();
    module.add_class::<PyLimits>()?;
    module.add_class::<PyOption>()?;
    module.add_class::<PyInfo>()?;
    module.add_class::<PyAnswer>()?;
    module.add_class::<PyEngine>()?;
    module.add_class::<PySearch>()?;
    module.add_class::<PyCommand>()?;
    module.add_class::<PyMessage>()?;
    module.add_class::<PySession>()?;
    module.add_function(wrap_pyfunction!(uci_parse, module)?)?;
    module.add("UciError", py.get_type::<UciError>())?;
    module.add("EngineTimeout", py.get_type::<EngineTimeout>())?;
    module.add("EngineDied", py.get_type::<EngineDied>())?;
    module.add("ProtocolError", py.get_type::<ProtocolError>())?;
    Ok(())
}
