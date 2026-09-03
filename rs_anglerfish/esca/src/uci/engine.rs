//! A UCI engine as a subprocess, addressed with blocking calls.
//!
//! Every wait is bounded: a silent engine fails with [`Error::Timeout`] rather
//! than hanging, and one that has exited fails with [`Error::Died`] on every
//! call after. The process is killed when the [`Engine`] is dropped.

use std::ffi::{OsStr, OsString};
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command as Process, Stdio};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::thread;
use std::time::{Duration, Instant};

use log::{debug, warn};

use crate::game::Game;
use crate::moves::Move;
use crate::variant::{CastlingOutput, classic};

use super::protocol::{
    self, CHESS960_OPTION, Command, Info, Limits, Message, OptionSpec, OptionValue, ProtocolError,
    Register, Setup, State, Status,
};

/// How long a wait that is not given its own limit may take.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// How long one blocking read of the engine's output may take, so that a
/// process that dies without closing its pipes is still noticed.
const POLL: Duration = Duration::from_millis(200);

/// The instant `budget` from now, saturating rather than overflowing.
fn deadline(budget: Duration) -> Instant {
    let now = Instant::now();
    now.checked_add(budget)
        .unwrap_or_else(|| now + Duration::from_secs(60 * 60 * 24 * 365))
}

/// Why a call on an engine did not do what it asked.
#[derive(Debug)]
pub enum Error {
    /// The process could not be started, or a pipe broke.
    Io(io::Error),
    /// The engine did not say what was awaited in time.
    Timeout {
        /// What the client was waiting for.
        awaited: &'static str,
        /// How long it waited.
        after: Duration,
    },
    /// The engine closed its output or exited.
    Died {
        /// Its exit code, when it is known.
        code: Option<i32>,
    },
    /// The engine broke the order of the conversation.
    Protocol(ProtocolError),
    /// The engine has not answered `uci`, so what it offers is unknown.
    NotIdentified,
    /// The engine declares no option of that name.
    NoSuchOption(String),
    /// The value does not fit the option's declared domain.
    BadValue {
        /// The option's name.
        option: String,
        /// What is wrong with the value.
        reason: String,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(error) => write!(f, "{error}"),
            Error::Timeout { awaited, after } => {
                write!(f, "no {awaited} within {:.3} s", after.as_secs_f64())
            }
            Error::Died { code: Some(code) } => write!(f, "the engine exited with code {code}"),
            Error::Died { code: None } => write!(f, "the engine closed its output"),
            Error::Protocol(error) => write!(f, "{error}"),
            Error::NotIdentified => write!(f, "the engine has not answered uci"),
            Error::NoSuchOption(name) => write!(f, "the engine offers no option {name:?}"),
            Error::BadValue { option, reason } => write!(f, "option {option:?}: {reason}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}

/// What an engine says about itself.
#[derive(Clone, Debug, Default)]
pub struct Identity {
    /// `id name`.
    pub name: Option<String>,
    /// `id author`.
    pub author: Option<String>,
    /// Every other `id` key it sent, in order.
    pub extra: Vec<(String, String)>,
}

/// An engine's answer to a search.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Answer {
    /// The move chosen; `None` when the engine reported that it has none, or
    /// named one that is not legal.
    pub best: Option<Move>,
    /// The reply it expects.
    pub ponder: Option<Move>,
}

/// What a running search produces.
#[derive(Clone, PartialEq, Debug)]
pub enum Progress {
    /// One search report.
    Info(Box<Info>),
    /// The search is over.
    Done(Answer),
}

/// How an engine process is started.
pub struct Launch {
    program: OsString,
    args: Vec<OsString>,
    directory: Option<PathBuf>,
    timeout: Duration,
}

impl Launch {
    /// Starts `program`, with no arguments and the default timeout.
    pub fn new(program: impl AsRef<OsStr>) -> Launch {
        Launch {
            program: program.as_ref().to_owned(),
            args: Vec::new(),
            directory: None,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// Appends one argument.
    pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Launch {
        self.args.push(arg.as_ref().to_owned());
        self
    }

    /// Appends several arguments.
    pub fn args<I: IntoIterator<Item = S>, S: AsRef<OsStr>>(mut self, args: I) -> Launch {
        self.args
            .extend(args.into_iter().map(|arg| arg.as_ref().to_owned()));
        self
    }

    /// Runs the engine in `directory`.
    pub fn current_dir(mut self, directory: impl Into<PathBuf>) -> Launch {
        self.directory = Some(directory.into());
        self
    }

    /// Bounds every wait that is not given its own limit.
    pub fn timeout(mut self, timeout: Duration) -> Launch {
        self.timeout = timeout;
        self
    }

    /// Starts the process.
    pub fn spawn(self) -> Result<Engine, Error> {
        let mut process = Process::new(&self.program);
        process
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(directory) = &self.directory {
            process.current_dir(directory);
        }
        debug!("Starting {:?}", self.program);
        let mut child = process.spawn()?;
        let stdin = child.stdin.take().expect("a piped stdin");
        let stdout = child.stdout.take().expect("a piped stdout");
        let stderr = child.stderr.take().expect("a piped stderr");

        let (sender, lines) = channel();
        thread::Builder::new()
            .name("esca-uci-stdout".to_owned())
            .spawn(move || {
                let mut reader = BufReader::new(stdout);
                let mut raw = Vec::new();
                while let Ok(read) = reader.read_until(b'\n', &mut raw) {
                    if read == 0 {
                        break;
                    }
                    let text = String::from_utf8_lossy(&raw)
                        .trim_end_matches(['\n', '\r'])
                        .to_owned();
                    raw.clear();
                    debug!("<< {text}");
                    if sender.send(text).is_err() {
                        break;
                    }
                }
            })?;
        thread::Builder::new()
            .name("esca-uci-stderr".to_owned())
            .spawn(move || {
                for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                    debug!("!! {line}");
                }
            })?;

        Ok(Engine {
            child,
            stdin: Some(stdin),
            lines,
            session: protocol::Session::new(),
            timeout: self.timeout,
            identity: Identity::default(),
            options: Vec::new(),
            identified: false,
            game: None,
            chess960: false,
            dead: false,
            code: None,
        })
    }
}

/// A UCI engine process.
pub struct Engine {
    child: Child,
    stdin: Option<ChildStdin>,
    lines: Receiver<String>,
    session: protocol::Session,
    timeout: Duration,
    identity: Identity,
    options: Vec<OptionSpec>,
    identified: bool,
    game: Option<Game>,
    chess960: bool,
    dead: bool,
    code: Option<i32>,
}

impl Engine {
    /// Starts `program` with `args`.
    pub fn spawn<I: IntoIterator<Item = S>, S: AsRef<OsStr>>(
        program: impl AsRef<OsStr>,
        args: I,
    ) -> Result<Engine, Error> {
        Launch::new(program).args(args).spawn()
    }

    /// How long a wait that is not given its own limit may take.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Bounds every wait that is not given its own limit.
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// What the engine said about itself.
    pub fn identity(&self) -> &Identity {
        &self.identity
    }

    /// The options it declared, in the order it declared them.
    pub fn options(&self) -> &[OptionSpec] {
        &self.options
    }

    /// The option of that name, matched without regard to case.
    pub fn option(&self, name: &str) -> Option<&OptionSpec> {
        self.options
            .iter()
            .find(|option| option.name.eq_ignore_ascii_case(name))
    }

    /// What the engine is doing.
    pub fn state(&self) -> State {
        self.session.state()
    }

    /// The game its position was last set to.
    pub fn game(&self) -> Option<&Game> {
        self.game.as_ref()
    }

    /// Whether the process is still running.
    pub fn is_alive(&mut self) -> bool {
        !self.dead && matches!(self.child.try_wait(), Ok(None))
    }

    // -- The raw line interface ---------------------------------------------

    /// Writes one line, bypassing the state machine.
    pub fn send_line(&mut self, text: &str) -> Result<(), Error> {
        self.check_alive()?;
        debug!(">> {text}");
        let Some(stdin) = self.stdin.as_mut() else {
            return Err(Error::Died { code: self.code });
        };
        let written = stdin
            .write_all(text.as_bytes())
            .and_then(|()| stdin.write_all(b"\n"))
            .and_then(|()| stdin.flush());
        match written {
            Ok(()) => Ok(()),
            Err(_) => Err(self.die()),
        }
    }

    /// The next line the engine wrote, or `None` if it wrote none in time.
    pub fn next_line(&mut self, timeout: Duration) -> Result<Option<String>, Error> {
        match self.lines.recv_timeout(timeout) {
            Ok(line) => Ok(Some(line)),
            Err(RecvTimeoutError::Timeout) => {
                self.check_alive()?;
                Ok(None)
            }
            Err(RecvTimeoutError::Disconnected) => Err(self.die()),
        }
    }

    // -- The protocol -------------------------------------------------------

    /// Sends `uci` and collects what the engine says about itself, up to
    /// `uciok`.
    pub fn handshake(&mut self) -> Result<&Identity, Error> {
        self.send(Command::Uci)?;
        self.identity = Identity::default();
        self.options.clear();
        let until = deadline(self.timeout);
        loop {
            match self.receive(until, self.timeout, "uciok")? {
                Message::UciOk => break,
                Message::Id { key, value } => match key.as_str() {
                    "name" => self.identity.name = Some(value),
                    "author" => self.identity.author = Some(value),
                    _ => self.identity.extra.push((key, value)),
                },
                Message::Option(spec) => self.options.push(spec),
                Message::CopyProtection(Status::Error) => {
                    warn!("The engine reports a copy-protection error")
                }
                _ => {}
            }
        }
        self.identified = true;
        Ok(&self.identity)
    }

    /// Turns the engine's `info string` diagnostics on or off.
    pub fn set_debug(&mut self, on: bool) -> Result<(), Error> {
        self.send(Command::Debug(on))
    }

    /// Answers a `registration error`.
    pub fn register(&mut self, register: Register) -> Result<(), Error> {
        self.send(Command::Register(register))
    }

    /// Sets one option, refusing a name the engine does not offer and a value
    /// outside the domain it declared.
    pub fn set_option(&mut self, name: &str, value: OptionValue) -> Result<(), Error> {
        if !self.identified {
            return Err(Error::NotIdentified);
        }
        let spec = self
            .option(name)
            .ok_or_else(|| Error::NoSuchOption(name.to_owned()))?;
        let declared = spec.name.clone();
        spec.accepts(&value).map_err(|reason| Error::BadValue {
            option: declared.clone(),
            reason,
        })?;
        self.send(Command::SetOption {
            name: declared,
            value: value.to_text(),
        })
    }

    /// Sends `isready` and waits for `readyok`.
    pub fn is_ready(&mut self) -> Result<(), Error> {
        self.send(Command::IsReady)?;
        let until = deadline(self.timeout);
        loop {
            if let Message::ReadyOk = self.receive(until, self.timeout, "readyok")? {
                return Ok(());
            }
        }
    }

    /// Announces a new game and waits for the engine to be ready again.
    pub fn new_game(&mut self) -> Result<(), Error> {
        self.send(Command::NewGame)?;
        self.game = None;
        self.is_ready()
    }

    /// Sets the position to `game`, putting the engine into the game's variant
    /// first.
    ///
    /// A Chess960 game needs the engine to offer `UCI_Chess960`; one that does
    /// not is an [`Error::NoSuchOption`] rather than a game played by the
    /// wrong rules.
    pub fn set_position(&mut self, game: &Game) -> Result<(), Error> {
        let chess960 = game.variant().name() == "chess960";
        if chess960 != self.chess960 {
            if !self.identified {
                return Err(Error::NotIdentified);
            }
            if chess960 || self.option(CHESS960_OPTION).is_some() {
                self.set_option(CHESS960_OPTION, OptionValue::Check(chess960))?;
                self.is_ready()?;
            }
            self.chess960 = chess960;
        }
        let style = if chess960 {
            CastlingOutput::KingToRook
        } else {
            CastlingOutput::KingTwoSquares
        };
        self.send(Command::Position(Setup::of_game(game, style)))?;
        self.game = Some(game.clone());
        Ok(())
    }

    /// Sends `go`, leaving its reports to be read one at a time.
    pub fn start_search(&mut self, limits: &Limits) -> Result<(), Error> {
        self.send(Command::Go(limits.clone()))
    }

    /// The next report of the search in flight, waiting at most `timeout`.
    pub fn next_progress(&mut self, timeout: Duration) -> Result<Progress, Error> {
        let game = self.searched();
        let until = deadline(timeout);
        loop {
            match self.receive(until, timeout, "bestmove")? {
                Message::Info(info) => return Ok(Progress::Info(info)),
                Message::BestMove(best) => {
                    return Ok(Progress::Done(Answer {
                        best: best.best_move(&game),
                        ponder: best.ponder_move(&game),
                    }));
                }
                _ => {}
            }
        }
    }

    /// Starts a search, which must produce its `bestmove` within `budget`.
    pub fn go(&mut self, limits: &Limits, budget: Duration) -> Result<Search<'_>, Error> {
        self.start_search(limits)?;
        let game = self.searched();
        Ok(Search {
            until: deadline(budget),
            budget,
            engine: self,
            game,
            answer: None,
        })
    }

    /// Sets the position, searches it, and answers.
    pub fn play(
        &mut self,
        game: &Game,
        limits: &Limits,
        budget: Duration,
    ) -> Result<Answer, Error> {
        self.set_position(game)?;
        self.go(limits, budget)?.answer()
    }

    /// Asks the search in flight to finish now.
    pub fn stop(&mut self) -> Result<(), Error> {
        self.send(Command::Stop)
    }

    /// Tells the engine the move it is pondering on was played.
    pub fn ponderhit(&mut self) -> Result<(), Error> {
        self.send(Command::PonderHit)
    }

    /// Asks the engine to exit, and kills it if it will not within the
    /// timeout. Answers with its exit code.
    pub fn quit(&mut self) -> Result<Option<i32>, Error> {
        if self.dead {
            return Ok(self.code);
        }
        let _ = self.send(Command::Quit);
        self.stdin = None;
        let until = deadline(self.timeout);
        loop {
            match self.child.try_wait() {
                Ok(Some(status)) => {
                    self.dead = true;
                    self.code = status.code();
                    return Ok(self.code);
                }
                Ok(None) if Instant::now() < until => thread::sleep(Duration::from_millis(10)),
                Ok(None) => {
                    warn!("The engine did not exit, killing it");
                    self.kill();
                    return Ok(self.code);
                }
                Err(error) => return Err(Error::Io(error)),
            }
        }
    }

    /// Kills the process.
    pub fn kill(&mut self) {
        self.stdin = None;
        let _ = self.child.kill();
        if let Ok(status) = self.child.wait() {
            self.code = status.code();
        }
        self.dead = true;
    }

    // -- Machinery ----------------------------------------------------------

    /// The game a search runs on: the one last set, or the standard array,
    /// which is where an engine that was told nothing stands.
    fn searched(&self) -> Game {
        self.game.clone().unwrap_or_else(|| Game::new(classic()))
    }

    /// Sends one command, after the state machine has allowed it.
    fn send(&mut self, command: Command) -> Result<(), Error> {
        self.session.sent(&command).map_err(Error::Protocol)?;
        self.send_line(&command.to_line())
    }

    /// The next message the state machine accepts, awaiting `awaited` until
    /// `until`. A message it does not accept is logged and dropped, so that
    /// one stray line cannot derail the conversation.
    fn receive(
        &mut self,
        until: Instant,
        budget: Duration,
        awaited: &'static str,
    ) -> Result<Message, Error> {
        loop {
            let left = until.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return Err(Error::Timeout {
                    awaited,
                    after: budget,
                });
            }
            let Some(line) = self.next_line(left.min(POLL))? else {
                continue;
            };
            let message = protocol::parse(&line);
            match self.session.received(&message) {
                Ok(()) => return Ok(message),
                Err(error) => warn!("Ignoring a line out of turn: {error} ({line:?})"),
            }
        }
    }

    fn check_alive(&mut self) -> Result<(), Error> {
        if self.dead {
            return Err(Error::Died { code: self.code });
        }
        if let Ok(Some(status)) = self.child.try_wait() {
            self.code = status.code();
            self.dead = true;
            return Err(Error::Died { code: self.code });
        }
        Ok(())
    }

    /// Marks the engine dead and reports it.
    fn die(&mut self) -> Error {
        self.dead = true;
        if self.code.is_none() {
            self.code = self.child.wait().ok().and_then(|status| status.code());
        }
        Error::Died { code: self.code }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        if !self.dead {
            self.kill();
        }
    }
}

/// A search in flight. Dropping one stops it and waits for the engine's
/// answer, so the engine is left idle.
pub struct Search<'a> {
    engine: &'a mut Engine,
    game: Game,
    until: Instant,
    budget: Duration,
    answer: Option<Answer>,
}

impl Search<'_> {
    /// The game the search runs on.
    pub fn game(&self) -> &Game {
        &self.game
    }

    /// What the engine is doing.
    pub fn state(&self) -> State {
        self.engine.state()
    }

    /// Whether the engine has answered.
    pub fn is_done(&self) -> bool {
        self.answer.is_some()
    }

    /// The next report, or `None` once the engine has answered.
    pub fn next_info(&mut self) -> Result<Option<Info>, Error> {
        if self.is_done() {
            return Ok(None);
        }
        match self.progress()? {
            Progress::Info(info) => Ok(Some(*info)),
            Progress::Done(_) => Ok(None),
        }
    }

    /// The engine's answer, with the reports still to come dropped. Answering
    /// again gives the answer already had.
    pub fn answer(&mut self) -> Result<Answer, Error> {
        loop {
            if let Some(answer) = self.answer {
                return Ok(answer);
            }
            self.progress()?;
        }
    }

    /// Asks the engine to finish the search now.
    pub fn stop(&mut self) -> Result<(), Error> {
        self.engine.stop()
    }

    /// Tells the engine the move it is pondering on was played.
    pub fn ponderhit(&mut self) -> Result<(), Error> {
        self.engine.ponderhit()
    }

    /// The next report, or the answer that ends the search.
    ///
    /// # Panics
    /// If the engine has already answered.
    pub fn progress(&mut self) -> Result<Progress, Error> {
        assert!(!self.is_done(), "the search is over");
        let left = self.until.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(Error::Timeout {
                awaited: "bestmove",
                after: self.budget,
            });
        }
        let progress = self.engine.next_progress(left)?;
        if let Progress::Done(answer) = progress {
            self.answer = Some(answer);
        }
        Ok(progress)
    }
}

impl Iterator for Search<'_> {
    type Item = Result<Info, Error>;

    /// Every report of the search, up to the engine's answer.
    fn next(&mut self) -> Option<Result<Info, Error>> {
        self.next_info().transpose()
    }
}

impl Drop for Search<'_> {
    fn drop(&mut self) {
        if self.is_done() || self.engine.dead {
            return;
        }
        if self.engine.stop().is_ok() {
            self.until = deadline(self.engine.timeout);
            self.budget = self.engine.timeout;
            if let Err(error) = self.answer() {
                debug!("Abandoning the search: {error}");
            }
        }
    }
}
