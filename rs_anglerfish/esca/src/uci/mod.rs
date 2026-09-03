//! Talking to a UCI engine.
//!
//! [`protocol`] is the protocol as values and does no I/O; [`Engine`] runs an
//! engine as a subprocess and speaks it, blocking, with every wait bounded.
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

mod engine;

pub use engine::{Answer, DEFAULT_TIMEOUT, Engine, Error, Identity, Launch, Progress, Search};
pub use protocol::{
    BestMove, Bound, CHESS960_OPTION, Command, CurrLine, Info, Limits, Message, OptionKind,
    OptionSpec, OptionValue, ProtocolError, Register, Session, Setup, State, Status, Wdl,
    moves_of_line, parse,
};
