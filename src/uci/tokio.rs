//! A UCI engine as a subprocess, addressed with async calls on a tokio
//! runtime.
//!
//! The same conversation as [`super::Engine`], with the same values and the
//! same [`Error`]: every wait is bounded, a silent engine fails with
//! [`Error::Timeout`] and one that has exited with [`Error::Died`]. The
//! process is killed when the [`Engine`] is dropped.
//!
//! Every future here is cancellation-safe. Dropping one mid-wait loses no
//! line and leaves the conversation where it stood, so the next call still
//! works; a [`Search`] let go of unanswered asks the engine to stop, and the
//! next call waits that search out before it says anything of its own.
//!
//! ```no_run
//! use std::time::Duration;
//! use esca::uci::tokio::Engine;
//! use esca::uci::Limits;
//! use esca::{Game, classic};
//!
//! # async fn play() -> Result<(), esca::uci::Error> {
//! let mut engine = Engine::spawn("stockfish", ["--quiet"]).await?;
//! engine.handshake().await?;
//! engine.new_game().await?;
//!
//! let game = Game::new(classic());
//! let answer = engine
//!     .play(&game, &Limits::depth(12), Duration::from_secs(30))
//!     .await?;
//! engine.quit().await?;
//! # Ok(())
//! # }
//! ```

use std::ffi::OsStr;
use std::process::Stdio;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use ::tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use ::tokio::process::{Child, Command as Process};
use ::tokio::sync::{Notify, mpsc};
use ::tokio::time::timeout;
use log::{debug, warn};

use crate::game::Game;
use crate::variant::{CastlingOutput, classic};

use super::engine::{Answer, Error, Identity, Launch, POLL, Progress, deadline};
use super::lines::Queue;
use super::protocol::{
    self, CHESS960_OPTION, Command, Info, Limits, Message, OptionSpec, OptionValue, Register,
    Setup, State, Status,
};

/// The queue a reader task fills and the client reads, and the signal that it
/// is no longer empty.
struct Lines {
    queue: Mutex<Queue>,
    ready: Notify,
}

/// The queue, locked; a task that panicked holding it left it usable.
fn held(lines: &Lines) -> MutexGuard<'_, Queue> {
    lines
        .queue
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl Launch {
    /// Starts the process, to be addressed with async calls. Runs on the
    /// current tokio runtime.
    pub async fn spawn_tokio(self) -> Result<Engine, Error> {
        let mut process = Process::new(&self.program);
        process
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        if let Some(directory) = &self.directory {
            process.current_dir(directory);
        }
        debug!("Starting {:?}", self.program);
        let mut child = process.spawn()?;
        let mut stdin = child.stdin.take().expect("a piped stdin");
        let stdout = child.stdout.take().expect("a piped stdout");
        let stderr = child.stderr.take().expect("a piped stderr");

        let lines = Arc::new(Lines {
            queue: Mutex::new(Queue::new()),
            ready: Notify::new(),
        });
        let filled = Arc::clone(&lines);
        ::tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut raw = Vec::new();
            while let Ok(read) = reader.read_until(b'\n', &mut raw).await {
                if read == 0 {
                    break;
                }
                let text = String::from_utf8_lossy(&raw)
                    .trim_end_matches(['\n', '\r'])
                    .to_owned();
                raw.clear();
                debug!("<< {text}");
                held(&filled).push(text);
                filled.ready.notify_waiters();
            }
            held(&filled).close();
            filled.ready.notify_waiters();
        });
        ::tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                debug!("!! {line}");
            }
        });

        // Writing goes through a task of its own, so that sending a command is
        // one non-blocking step that no cancelled future can cut in half.
        let (out, mut outbox) = mpsc::unbounded_channel::<String>();
        ::tokio::spawn(async move {
            while let Some(line) = outbox.recv().await {
                let written = stdin.write_all(line.as_bytes()).await;
                if written.is_err() || stdin.write_all(b"\n").await.is_err() {
                    break;
                }
                if stdin.flush().await.is_err() {
                    break;
                }
            }
        });

        Ok(Engine {
            child,
            out: Some(out),
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
            abandoned: false,
        })
    }
}

/// A UCI engine process.
pub struct Engine {
    child: Child,
    out: Option<mpsc::UnboundedSender<String>>,
    lines: Arc<Lines>,
    session: protocol::Session,
    timeout: Duration,
    identity: Identity,
    options: Vec<OptionSpec>,
    identified: bool,
    game: Option<Game>,
    chess960: bool,
    dead: bool,
    code: Option<i32>,
    abandoned: bool,
}

impl Engine {
    /// Starts `program` with `args`.
    pub async fn spawn<I: IntoIterator<Item = S>, S: AsRef<OsStr>>(
        program: impl AsRef<OsStr>,
        args: I,
    ) -> Result<Engine, Error> {
        Launch::new(program).args(args).spawn_tokio().await
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

    /// How many lines the engine wrote that the client never read, because it
    /// wrote them faster than they were read.
    pub fn dropped_lines(&self) -> u64 {
        held(&self.lines).dropped()
    }

    // -- The raw line interface ---------------------------------------------

    /// Writes one line, bypassing the state machine.
    pub async fn send_line(&mut self, text: &str) -> Result<(), Error> {
        self.write(text)
    }

    /// The next line the engine wrote, or `None` if it wrote none in time.
    pub async fn next_line(&mut self, budget: Duration) -> Result<Option<String>, Error> {
        let until = deadline(budget);
        loop {
            let (line, over) = {
                let mut queue = held(&self.lines);
                let line = queue.pop();
                let over = line.is_none() && queue.is_done();
                (line, over)
            };
            if let Some(line) = line {
                return Ok(Some(line));
            }
            if over {
                return Err(self.die().await);
            }
            let left = until.saturating_duration_since(Instant::now());
            if left.is_zero() {
                self.check_alive()?;
                return Ok(None);
            }
            // Registering before the last look at the queue is what makes a
            // line written in between wake this wait rather than be missed.
            let lines = Arc::clone(&self.lines);
            let woken = lines.ready.notified();
            ::tokio::pin!(woken);
            if !held(&lines).is_empty() {
                continue;
            }
            let _ = timeout(left.min(POLL), woken).await;
        }
    }

    // -- The protocol -------------------------------------------------------

    /// Sends `uci` and collects what the engine says about itself, up to
    /// `uciok`.
    pub async fn handshake(&mut self) -> Result<&Identity, Error> {
        self.issue(Command::Uci).await?;
        self.identity = Identity::default();
        self.options.clear();
        let until = deadline(self.timeout);
        loop {
            match self.receive(until, self.timeout, "uciok").await? {
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
    pub async fn set_debug(&mut self, on: bool) -> Result<(), Error> {
        self.issue(Command::Debug(on)).await
    }

    /// Answers a `registration error`.
    pub async fn register(&mut self, register: Register) -> Result<(), Error> {
        self.issue(Command::Register(register)).await
    }

    /// Sets one option, refusing a name the engine does not offer and a value
    /// outside the domain it declared.
    pub async fn set_option(&mut self, name: &str, value: OptionValue) -> Result<(), Error> {
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
        self.issue(Command::SetOption {
            name: declared,
            value: value.to_text(),
        })
        .await
    }

    /// Sends `isready` and waits for `readyok`.
    pub async fn is_ready(&mut self) -> Result<(), Error> {
        self.issue(Command::IsReady).await?;
        let until = deadline(self.timeout);
        loop {
            if let Message::ReadyOk = self.receive(until, self.timeout, "readyok").await? {
                return Ok(());
            }
        }
    }

    /// Announces a new game and waits for the engine to be ready again.
    pub async fn new_game(&mut self) -> Result<(), Error> {
        self.issue(Command::NewGame).await?;
        self.game = None;
        self.is_ready().await
    }

    /// Sets the position to `game`, putting the engine into the game's variant
    /// first.
    ///
    /// A Chess960 game needs the engine to offer `UCI_Chess960`; one that does
    /// not is an [`Error::NoSuchOption`] rather than a game played by the
    /// wrong rules.
    pub async fn set_position(&mut self, game: &Game) -> Result<(), Error> {
        let chess960 = game.variant().name() == "chess960";
        if chess960 != self.chess960 {
            if !self.identified {
                return Err(Error::NotIdentified);
            }
            if chess960 || self.option(CHESS960_OPTION).is_some() {
                self.set_option(CHESS960_OPTION, OptionValue::Check(chess960))
                    .await?;
                self.is_ready().await?;
            }
            self.chess960 = chess960;
        }
        let style = if chess960 {
            CastlingOutput::KingToRook
        } else {
            CastlingOutput::KingTwoSquares
        };
        self.issue(Command::Position(Setup::of_game(game, style)))
            .await?;
        self.game = Some(game.clone());
        Ok(())
    }

    /// Sends `go`, leaving its reports to be read one at a time.
    pub async fn start_search(&mut self, limits: &Limits) -> Result<(), Error> {
        self.issue(Command::Go(limits.clone())).await
    }

    /// The next report of the search in flight, waiting at most `budget`.
    pub async fn next_progress(&mut self, budget: Duration) -> Result<Progress, Error> {
        let game = self.searched();
        let until = deadline(budget);
        loop {
            match self.receive(until, budget, "bestmove").await? {
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
    pub async fn go(&mut self, limits: &Limits, budget: Duration) -> Result<Search<'_>, Error> {
        self.start_search(limits).await?;
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
    pub async fn play(
        &mut self,
        game: &Game,
        limits: &Limits,
        budget: Duration,
    ) -> Result<Answer, Error> {
        self.set_position(game).await?;
        self.go(limits, budget).await?.answer().await
    }

    /// Asks the search in flight to finish now.
    pub async fn stop(&mut self) -> Result<(), Error> {
        self.issue(Command::Stop).await
    }

    /// Tells the engine the move it is pondering on was played.
    pub async fn ponderhit(&mut self) -> Result<(), Error> {
        self.issue(Command::PonderHit).await
    }

    /// Asks the engine to exit, and kills it if it will not within the
    /// timeout. Answers with its exit code.
    pub async fn quit(&mut self) -> Result<Option<i32>, Error> {
        if self.dead {
            return Ok(self.code);
        }
        let _ = self.send(Command::Quit);
        // The engine reads an end of input even if it ignored the command.
        self.out = None;
        match timeout(self.timeout, self.child.wait()).await {
            Ok(Ok(status)) => {
                self.dead = true;
                self.code = status.code();
                Ok(self.code)
            }
            Ok(Err(error)) => Err(Error::Io(error)),
            Err(_) => {
                warn!("The engine did not exit, killing it");
                self.kill().await;
                Ok(self.code)
            }
        }
    }

    /// Kills the process.
    pub async fn kill(&mut self) {
        self.out = None;
        let _ = self.child.start_kill();
        if let Ok(status) = self.child.wait().await {
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

    /// Waits out a search that was let go of unanswered, so that the engine is
    /// idle before anything else is asked of it.
    async fn settle(&mut self) -> Result<(), Error> {
        if !self.abandoned {
            return Ok(());
        }
        self.abandoned = false;
        let until = deadline(self.timeout);
        while matches!(self.state(), State::Searching | State::Pondering) {
            self.receive(until, self.timeout, "bestmove").await?;
        }
        Ok(())
    }

    /// Sends one command, once an abandoned search is over and the state
    /// machine has allowed it.
    async fn issue(&mut self, command: Command) -> Result<(), Error> {
        self.settle().await?;
        self.send(command)
    }

    /// Sends one command, after the state machine has allowed it. Writing is
    /// one step and never waits, so a cancelled future cannot leave half a
    /// command on the wire.
    fn send(&mut self, command: Command) -> Result<(), Error> {
        self.session.sent(&command).map_err(Error::Protocol)?;
        self.write(&command.to_line())
    }

    /// Hands one line to the writer task.
    fn write(&mut self, text: &str) -> Result<(), Error> {
        self.check_alive()?;
        debug!(">> {text}");
        let out = self.out.as_ref().ok_or(Error::Died { code: self.code })?;
        out.send(text.to_owned())
            .map_err(|_| Error::Died { code: self.code })
    }

    /// The next message the state machine accepts, awaiting `awaited` until
    /// `until`. A message it does not accept is logged and dropped, so that
    /// one stray line cannot derail the conversation.
    async fn receive(
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
            let Some(line) = self.next_line(left.min(POLL)).await? else {
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
    async fn die(&mut self) -> Error {
        self.dead = true;
        if self.code.is_none() {
            let _ = self.child.start_kill();
            self.code = self
                .child
                .wait()
                .await
                .ok()
                .and_then(|status| status.code());
        }
        Error::Died { code: self.code }
    }
}

/// A search in flight. Letting one go unanswered asks the engine to stop; the
/// engine's next call waits that search out, so it is idle again from there
/// on.
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
    pub async fn next_info(&mut self) -> Result<Option<Info>, Error> {
        if self.is_done() {
            return Ok(None);
        }
        match self.progress().await? {
            Progress::Info(info) => Ok(Some(*info)),
            Progress::Done(_) => Ok(None),
        }
    }

    /// The engine's answer, with the reports still to come dropped. Answering
    /// again gives the answer already had.
    pub async fn answer(&mut self) -> Result<Answer, Error> {
        loop {
            if let Some(answer) = self.answer {
                return Ok(answer);
            }
            self.progress().await?;
        }
    }

    /// Asks the engine to finish the search now.
    pub async fn stop(&mut self) -> Result<(), Error> {
        self.engine.stop().await
    }

    /// Tells the engine the move it is pondering on was played.
    pub async fn ponderhit(&mut self) -> Result<(), Error> {
        self.engine.ponderhit().await
    }

    /// The next report, or the answer that ends the search.
    ///
    /// # Panics
    /// If the engine has already answered.
    pub async fn progress(&mut self) -> Result<Progress, Error> {
        assert!(!self.is_done(), "the search is over");
        let left = self.until.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(Error::Timeout {
                awaited: "bestmove",
                after: self.budget,
            });
        }
        let progress = self.engine.next_progress(left).await?;
        if let Progress::Done(answer) = progress {
            self.answer = Some(answer);
        }
        Ok(progress)
    }
}

impl Drop for Search<'_> {
    fn drop(&mut self) {
        if self.is_done() || self.engine.dead {
            return;
        }
        if self.engine.send(Command::Stop).is_ok() {
            self.engine.abandoned = true;
        }
    }
}
