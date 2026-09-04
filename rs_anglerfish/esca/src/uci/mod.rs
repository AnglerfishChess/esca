//! Talking to a UCI engine.
//!
//! [`protocol`] is the protocol as values and does no I/O. There are two ways
//! to hold an engine, over the same values and the same [`Error`]: [`Engine`]
//! runs it as a subprocess and speaks it, blocking, and [`tokio::Engine`] does
//! the same on a tokio runtime, behind the `tokio` feature. Both bound every
//! wait, and both kill the process when they are dropped.
//!
//! ```no_run
//! use std::time::Duration;
//! use esca::uci::{Engine, Limits};
//! use esca::{Game, classic};
//!
//! let mut engine = Engine::spawn("stockfish", ["--quiet"])?;
//! engine.handshake()?;
//! engine.new_game()?;
//!
//! let mut game = Game::new(classic());
//! game.play_san("e4").unwrap();
//! let answer = engine.play(&game, &Limits::depth(12), Duration::from_secs(30))?;
//! if let Some(mv) = answer.best {
//!     game.play(mv).unwrap();
//! }
//! engine.quit()?;
//! # Ok::<(), esca::uci::Error>(())
//! ```

pub mod protocol;
#[cfg(feature = "tokio")]
pub mod tokio;

mod engine;
mod lines;

pub use engine::{Answer, DEFAULT_TIMEOUT, Engine, Error, Identity, Launch, Progress, Search};
pub use protocol::{
    BestMove, Bound, CHESS960_OPTION, Command, CurrLine, Info, Limits, Message, OptionKind,
    OptionSpec, OptionValue, ProtocolError, Register, Session, Setup, State, Status, Wdl,
    moves_of_line, parse,
};
