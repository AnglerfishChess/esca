//! The protocol as values, with no I/O.
//!
//! [`Command`] writes what a client says; [`parse`] reads one line of what an
//! engine says into a [`Message`]; [`Session`] says which of them may come
//! next. A line an engine sends that breaks the grammar is kept whole as
//! [`Message::Raw`], and a token inside a line that does is kept in
//! [`Info::unknown`]: reading engine output never fails.

use core::fmt;
use std::time::Duration;

use crate::game::Game;
use crate::moves::Move;
use crate::position::Score;
use crate::variant::{CLASSIC, CastlingOutput, Variant};

/// The name of the option that puts an engine into Chess960.
pub const CHESS960_OPTION: &str = "UCI_Chess960";

// -- Commands ---------------------------------------------------------------

/// One line a client sends to an engine.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Command {
    /// Ask the engine to identify itself and list its options.
    Uci,
    /// Turn the engine's `info string` diagnostics on or off.
    Debug(bool),
    /// Ask for a `readyok`.
    IsReady,
    /// Set one option; the text is the value as the engine will read it.
    SetOption {
        /// The option's name, as the engine declared it.
        name: String,
        /// The value, or `None` for a button.
        value: Option<String>,
    },
    /// Answer a `registration error`.
    Register(Register),
    /// Announce that the next position belongs to a new game.
    NewGame,
    /// Set the position to search from.
    Position(Setup),
    /// Start searching under `Limits`.
    Go(Limits),
    /// Ask the search to finish now.
    Stop,
    /// Tell the engine the move it is pondering on was played.
    PonderHit,
    /// Ask the engine to exit.
    Quit,
}

/// What a `register` command carries.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Register {
    /// Postpone registration.
    Later,
    /// Register with these credentials; each part is omitted when absent.
    Credentials {
        /// The registered name.
        name: Option<String>,
        /// The registration code.
        code: Option<String>,
    },
}

/// The position a `position` command names.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Setup {
    /// The FEN to start from; `None` asks for `startpos`.
    pub fen: Option<String>,
    /// The moves played onto it, in UCI notation.
    pub moves: Vec<String>,
}

impl Setup {
    /// The standard array with no moves played.
    pub fn start() -> Setup {
        Setup::default()
    }

    /// The position `fen` describes, with no moves played.
    pub fn fen(fen: impl Into<String>) -> Setup {
        Setup {
            fen: Some(fen.into()),
            moves: Vec::new(),
        }
    }

    /// The moves played onto the start position, each written where it was
    /// played and with castling spelled as `style` asks.
    pub fn of_game(game: &Game, style: CastlingOutput) -> Setup {
        let start = game.start_position();
        let fen = (*start != CLASSIC.start_position(0)).then(|| start.fen());
        let variant = game.variant();
        let moves = game
            .positions()
            .zip(game.moves())
            .map(|(position, &mv)| variant.move_to_uci(position, mv, style))
            .collect();
        Setup { fen, moves }
    }
}

/// What bounds a search: every limit a `go` may name. Limits combine, and one
/// with nothing set asks the engine to search until stopped.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Limits {
    /// The only moves to consider, in UCI notation; empty allows every legal
    /// move.
    pub search_moves: Vec<String>,
    /// Search on the move the last `position` ends with, until `ponderhit` or
    /// `stop`.
    pub ponder: bool,
    /// White's clock.
    pub white_time: Option<Duration>,
    /// Black's clock.
    pub black_time: Option<Duration>,
    /// White's increment per move.
    pub white_increment: Option<Duration>,
    /// Black's increment per move.
    pub black_increment: Option<Duration>,
    /// Moves left to the next time control.
    pub moves_to_go: Option<u32>,
    /// Plies to search.
    pub depth: Option<u32>,
    /// Nodes to search.
    pub nodes: Option<u64>,
    /// Search for a mate in this many moves.
    pub mate: Option<u32>,
    /// Search for exactly this long.
    pub movetime: Option<Duration>,
    /// Search until stopped, whatever else is set.
    pub infinite: bool,
}

impl Limits {
    /// Search until stopped.
    pub fn infinite() -> Limits {
        Limits {
            infinite: true,
            ..Limits::default()
        }
    }

    /// Search `plies` deep.
    pub fn depth(plies: u32) -> Limits {
        Limits {
            depth: Some(plies),
            ..Limits::default()
        }
    }

    /// Search `nodes` nodes.
    pub fn nodes(nodes: u64) -> Limits {
        Limits {
            nodes: Some(nodes),
            ..Limits::default()
        }
    }

    /// Search for `time`.
    pub fn movetime(time: Duration) -> Limits {
        Limits {
            movetime: Some(time),
            ..Limits::default()
        }
    }

    /// Search for a mate in `moves` moves.
    pub fn mate(moves: u32) -> Limits {
        Limits {
            mate: Some(moves),
            ..Limits::default()
        }
    }

    /// Search under a clock, each side with its own increment.
    pub fn clock(
        white: Duration,
        black: Duration,
        white_increment: Duration,
        black_increment: Duration,
    ) -> Limits {
        Limits {
            white_time: Some(white),
            black_time: Some(black),
            white_increment: Some(white_increment),
            black_increment: Some(black_increment),
            ..Limits::default()
        }
    }

    /// The same limits, restricted to `moves`, in UCI notation.
    pub fn searching(mut self, moves: impl IntoIterator<Item = String>) -> Limits {
        self.search_moves = moves.into_iter().collect();
        self
    }

    /// The same limits, searched on the move the position ends with.
    pub fn pondering(mut self) -> Limits {
        self.ponder = true;
        self
    }
}

impl Command {
    /// The command's line, without its newline.
    pub fn to_line(&self) -> String {
        match self {
            Command::Uci => "uci".to_owned(),
            Command::Debug(on) => format!("debug {}", if *on { "on" } else { "off" }),
            Command::IsReady => "isready".to_owned(),
            Command::SetOption { name, value: None } => format!("setoption name {name}"),
            Command::SetOption {
                name,
                value: Some(value),
            } => format!("setoption name {name} value {value}"),
            Command::Register(Register::Later) => "register later".to_owned(),
            Command::Register(Register::Credentials { name, code }) => {
                let mut line = "register".to_owned();
                if let Some(name) = name {
                    line.push_str(&format!(" name {name}"));
                }
                if let Some(code) = code {
                    line.push_str(&format!(" code {code}"));
                }
                line
            }
            Command::NewGame => "ucinewgame".to_owned(),
            Command::Position(setup) => position_line(setup),
            Command::Go(limits) => go_line(limits),
            Command::Stop => "stop".to_owned(),
            Command::PonderHit => "ponderhit".to_owned(),
            Command::Quit => "quit".to_owned(),
        }
    }

    /// The keyword this command is named by.
    pub fn keyword(&self) -> &'static str {
        match self {
            Command::Uci => "uci",
            Command::Debug(_) => "debug",
            Command::IsReady => "isready",
            Command::SetOption { .. } => "setoption",
            Command::Register(_) => "register",
            Command::NewGame => "ucinewgame",
            Command::Position(_) => "position",
            Command::Go(_) => "go",
            Command::Stop => "stop",
            Command::PonderHit => "ponderhit",
            Command::Quit => "quit",
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_line())
    }
}

fn position_line(setup: &Setup) -> String {
    let mut line = match &setup.fen {
        None => "position startpos".to_owned(),
        Some(fen) => format!("position fen {fen}"),
    };
    if !setup.moves.is_empty() {
        line.push_str(" moves ");
        line.push_str(&setup.moves.join(" "));
    }
    line
}

/// Appends ` <keyword> <milliseconds>` when the time is set.
fn push_millis(line: &mut String, keyword: &str, time: &Option<Duration>) {
    if let Some(time) = time {
        line.push_str(&format!(" {keyword} {}", time.as_millis()));
    }
}

fn go_line(limits: &Limits) -> String {
    let mut line = "go".to_owned();
    if limits.ponder {
        line.push_str(" ponder");
    }
    push_millis(&mut line, "wtime", &limits.white_time);
    push_millis(&mut line, "btime", &limits.black_time);
    push_millis(&mut line, "winc", &limits.white_increment);
    push_millis(&mut line, "binc", &limits.black_increment);
    if let Some(moves) = limits.moves_to_go {
        line.push_str(&format!(" movestogo {moves}"));
    }
    if let Some(depth) = limits.depth {
        line.push_str(&format!(" depth {depth}"));
    }
    if let Some(nodes) = limits.nodes {
        line.push_str(&format!(" nodes {nodes}"));
    }
    if let Some(mate) = limits.mate {
        line.push_str(&format!(" mate {mate}"));
    }
    push_millis(&mut line, "movetime", &limits.movetime);
    if limits.infinite {
        line.push_str(" infinite");
    }
    // Last: the move list runs to the end of the line.
    if !limits.search_moves.is_empty() {
        line.push_str(" searchmoves ");
        line.push_str(&limits.search_moves.join(" "));
    }
    line
}

// -- Options ----------------------------------------------------------------

/// One option an engine declares, and the domain it declares for it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OptionSpec {
    /// The name to set it by, as written.
    pub name: String,
    /// Its type and the constraints that come with it.
    pub kind: OptionKind,
}

/// The five option types and what each declares.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum OptionKind {
    /// A boolean.
    Check {
        /// The value the engine starts with.
        default: Option<bool>,
    },
    /// An integer in a range.
    Spin {
        /// The value the engine starts with.
        default: Option<i64>,
        /// The smallest value accepted.
        min: Option<i64>,
        /// The largest value accepted.
        max: Option<i64>,
    },
    /// One of a list of names.
    Combo {
        /// The value the engine starts with.
        default: Option<String>,
        /// The values accepted, in the order declared.
        vars: Vec<String>,
    },
    /// An action, carrying no value.
    Button,
    /// Free text.
    String {
        /// The value the engine starts with; `<empty>` reads as empty.
        default: Option<String>,
    },
}

/// A value to set an option to.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum OptionValue {
    /// For a `check`.
    Check(bool),
    /// For a `spin`.
    Spin(i64),
    /// For a `combo`.
    Combo(String),
    /// For a `button`.
    Button,
    /// For a `string`.
    String(String),
}

impl OptionValue {
    /// The text a `setoption` carries; `None` for a button, which carries
    /// none. Empty text is written `<empty>`, as the protocol asks.
    pub fn to_text(&self) -> Option<String> {
        match self {
            OptionValue::Check(value) => Some(value.to_string()),
            OptionValue::Spin(value) => Some(value.to_string()),
            OptionValue::Combo(value) | OptionValue::String(value) => Some(if value.is_empty() {
                "<empty>".to_owned()
            } else {
                value.clone()
            }),
            OptionValue::Button => None,
        }
    }

    /// The name of this value's type: `check`, `spin`, `combo`, `button` or
    /// `string`.
    pub fn type_name(&self) -> &'static str {
        match self {
            OptionValue::Check(_) => "check",
            OptionValue::Spin(_) => "spin",
            OptionValue::Combo(_) => "combo",
            OptionValue::Button => "button",
            OptionValue::String(_) => "string",
        }
    }
}

impl OptionKind {
    /// The name of this type: `check`, `spin`, `combo`, `button` or `string`.
    pub fn type_name(&self) -> &'static str {
        match self {
            OptionKind::Check { .. } => "check",
            OptionKind::Spin { .. } => "spin",
            OptionKind::Combo { .. } => "combo",
            OptionKind::Button => "button",
            OptionKind::String { .. } => "string",
        }
    }
}

impl OptionSpec {
    /// The value the engine starts with, as far as it declared one.
    pub fn default_value(&self) -> Option<OptionValue> {
        match &self.kind {
            OptionKind::Check { default } => default.map(OptionValue::Check),
            OptionKind::Spin { default, .. } => default.map(OptionValue::Spin),
            OptionKind::Combo { default, .. } => default.clone().map(OptionValue::Combo),
            OptionKind::Button => Some(OptionValue::Button),
            OptionKind::String { default } => default.clone().map(OptionValue::String),
        }
    }

    /// Whether `value` fits this option's type and declared domain, naming the
    /// mismatch when it does not.
    pub fn accepts(&self, value: &OptionValue) -> Result<(), String> {
        match (&self.kind, value) {
            (OptionKind::Check { .. }, OptionValue::Check(_))
            | (OptionKind::Button, OptionValue::Button)
            | (OptionKind::String { .. }, OptionValue::String(_)) => Ok(()),
            (OptionKind::Spin { min, max, .. }, OptionValue::Spin(number)) => {
                if min.is_some_and(|min| *number < min) || max.is_some_and(|max| *number > max) {
                    let min = min.map_or("-".to_owned(), |min| min.to_string());
                    let max = max.map_or("-".to_owned(), |max| max.to_string());
                    Err(format!("{number} is outside [{min}, {max}]"))
                } else {
                    Ok(())
                }
            }
            (OptionKind::Combo { vars, .. }, OptionValue::Combo(name)) => {
                if vars.is_empty() || vars.iter().any(|var| var == name) {
                    Ok(())
                } else {
                    Err(format!("{name:?} is none of {}", vars.join(", ")))
                }
            }
            (kind, value) => Err(format!(
                "a {} option takes no {} value",
                kind.type_name(),
                value.type_name()
            )),
        }
    }
}

// -- Engine messages --------------------------------------------------------

/// One line an engine sends.
#[derive(Clone, PartialEq, Debug)]
pub enum Message {
    /// `id name`, `id author`, or another key the engine chose.
    Id {
        /// The key, `name` or `author` for the two the protocol defines.
        key: String,
        /// The rest of the line.
        value: String,
    },
    /// The end of the identification.
    UciOk,
    /// The answer to an `isready`.
    ReadyOk,
    /// One option the engine offers.
    Option(OptionSpec),
    /// A search report.
    Info(Box<Info>),
    /// The end of a search.
    BestMove(BestMove),
    /// How registration went.
    Registration(Status),
    /// How the copy-protection check went.
    CopyProtection(Status),
    /// A line carrying no engine keyword, or one whose grammar is broken,
    /// kept as it arrived.
    Raw(String),
}

/// The three states a `registration` or `copyprotection` line reports.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    /// The check is under way; the engine will report again.
    Checking,
    /// The check passed.
    Ok,
    /// The check failed; the engine will not play.
    Error,
}

/// The move an engine chose, as written.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct BestMove {
    /// The move, in UCI notation; `None` when the engine reported that it has
    /// none.
    pub best: Option<String>,
    /// The reply it expects, in UCI notation.
    pub ponder: Option<String>,
}

/// Which side of the true score a bounded score stands on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bound {
    /// The true score is at least this.
    Lower,
    /// The true score is at most this.
    Upper,
}

/// The engine's estimate of the outcome, in permille.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Wdl {
    /// Wins for the side to move.
    pub win: u32,
    /// Draws.
    pub draw: u32,
    /// Losses for the side to move.
    pub loss: u32,
}

/// A line one CPU is searching, as `currline` reports it.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct CurrLine {
    /// Which CPU, when the engine says.
    pub cpu: Option<u32>,
    /// The moves, in UCI notation.
    pub moves: Vec<String>,
}

/// One `info` line: every field the engine wrote, moves left as text.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Info {
    /// Plies searched.
    pub depth: Option<u32>,
    /// Plies searched on the deepest line.
    pub seldepth: Option<u32>,
    /// Time spent.
    pub time: Option<Duration>,
    /// Nodes searched.
    pub nodes: Option<u64>,
    /// Nodes per second.
    pub nps: Option<u64>,
    /// The principal variation, in UCI notation.
    pub pv: Vec<String>,
    /// Which of the ranked variations this line reports, counting from 1.
    pub multipv: Option<u32>,
    /// The score, from the side to move's point of view.
    pub score: Option<Score>,
    /// Which side of the true score `score` stands on, when it is not exact.
    pub bound: Option<Bound>,
    /// The win/draw/loss estimate.
    pub wdl: Option<Wdl>,
    /// The move being searched, in UCI notation.
    pub currmove: Option<String>,
    /// Its ordinal among the root moves, counting from 1.
    pub currmovenumber: Option<u32>,
    /// How full the hash table is, in permille.
    pub hashfull: Option<u32>,
    /// Endgame-table hits.
    pub tbhits: Option<u64>,
    /// Shredder-base hits.
    pub sbhits: Option<u64>,
    /// CPU load, in permille.
    pub cpuload: Option<u32>,
    /// The move refuted followed by the line that refutes it, in UCI notation.
    pub refutation: Vec<String>,
    /// The line a CPU is searching now.
    pub currline: Option<CurrLine>,
    /// The rest of the line after `string`, spacing kept.
    pub string: Option<String>,
    /// Tokens the parser did not understand, in the order they arrived.
    pub unknown: Vec<String>,
}

impl Info {
    /// The principal variation as moves of `game`, up to the first move that
    /// is not legal there.
    pub fn pv_moves(&self, game: &Game) -> Vec<Move> {
        moves_of_line(game, &self.pv)
    }

    /// The move `currmove` names, if it is legal in `game`.
    pub fn current_move(&self, game: &Game) -> Option<Move> {
        let text = self.currmove.as_deref()?;
        game.variant().move_from_uci(game.position(), text).ok()
    }
}

impl BestMove {
    /// The move chosen, if it is legal in `game`.
    pub fn best_move(&self, game: &Game) -> Option<Move> {
        let text = self.best.as_deref()?;
        game.variant().move_from_uci(game.position(), text).ok()
    }

    /// The reply expected, played after the move chosen.
    pub fn ponder_move(&self, game: &Game) -> Option<Move> {
        let best = self.best.as_deref()?;
        let ponder = self.ponder.as_deref()?;
        let mut game = game.clone();
        game.play_uci(best).ok()?;
        game.variant().move_from_uci(game.position(), ponder).ok()
    }
}

/// The moves `line` names, played in order from `game`'s current position, up
/// to the first one that is not legal there.
pub fn moves_of_line(game: &Game, line: &[String]) -> Vec<Move> {
    let mut game = game.clone();
    let mut moves = Vec::with_capacity(line.len());
    for text in line {
        let Ok(mv) = game.variant().move_from_uci(game.position(), text) else {
            break;
        };
        moves.push(mv);
        if game.play(mv).is_err() {
            break;
        }
    }
    moves
}

// -- Reading engine lines ---------------------------------------------------

/// The keywords an engine names its lines by.
const ENGINE_KEYWORDS: [&str; 8] = [
    "id",
    "uciok",
    "readyok",
    "bestmove",
    "copyprotection",
    "registration",
    "info",
    "option",
];

/// Move tokens that stand for "no move".
const NULL_MOVES: [&str; 4] = ["0000", "(none)", "none", "null"];

/// The tokens of `line`, each with the byte offset it starts at.
fn tokens(line: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    for (at, character) in line.char_indices() {
        if character.is_whitespace() {
            if let Some(from) = start.take() {
                out.push((from, &line[from..at]));
            }
        } else if start.is_none() {
            start = Some(at);
        }
    }
    if let Some(from) = start {
        out.push((from, &line[from..]));
    }
    out
}

/// Reads one line of engine output. Leading tokens that are not an engine
/// keyword are skipped, as the protocol asks.
pub fn parse(line: &str) -> Message {
    let tokens = tokens(line);
    let Some(at) = tokens
        .iter()
        .position(|(_, token)| ENGINE_KEYWORDS.contains(token))
    else {
        return Message::Raw(line.to_owned());
    };
    let keyword = tokens[at].1;
    let rest = &tokens[at + 1..];
    let parsed = match keyword {
        "uciok" => Some(Message::UciOk),
        "readyok" => Some(Message::ReadyOk),
        "id" => parse_id(line, rest),
        "option" => parse_option(rest),
        "info" => Some(Message::Info(Box::new(parse_info(line, rest)))),
        "bestmove" => parse_bestmove(rest),
        "registration" => status(rest).map(Message::Registration),
        "copyprotection" => status(rest).map(Message::CopyProtection),
        _ => None,
    };
    parsed.unwrap_or_else(|| Message::Raw(line.to_owned()))
}

fn status(rest: &[(usize, &str)]) -> Option<Status> {
    match rest.first()?.1 {
        "checking" => Some(Status::Checking),
        "ok" => Some(Status::Ok),
        "error" => Some(Status::Error),
        _ => None,
    }
}

fn parse_id(line: &str, rest: &[(usize, &str)]) -> Option<Message> {
    let (_, key) = *rest.first()?;
    let (at, _) = *rest.get(1)?;
    Some(Message::Id {
        key: key.to_owned(),
        value: line[at..].trim_end().to_owned(),
    })
}

fn parse_bestmove(rest: &[(usize, &str)]) -> Option<Message> {
    let (_, first) = *rest.first()?;
    let best = (!NULL_MOVES.contains(&first)).then(|| first.to_owned());
    let ponder = match rest.get(1) {
        None => None,
        Some((_, "ponder")) => match rest.get(2) {
            Some((_, token)) if !NULL_MOVES.contains(token) => Some((*token).to_owned()),
            Some(_) => None,
            None => return None,
        },
        Some(_) => return None,
    };
    Some(Message::BestMove(BestMove { best, ponder }))
}

/// The keywords that end the value of the option keyword before them.
const OPTION_KEYWORDS: [&str; 4] = ["default", "min", "max", "var"];

fn parse_option(rest: &[(usize, &str)]) -> Option<Message> {
    if rest.first()?.1 != "name" {
        return None;
    }
    let type_at = rest.iter().position(|(_, token)| *token == "type")?;
    let name = join(&rest[1..type_at]);
    if name.is_empty() {
        return None;
    }
    let type_name = rest.get(type_at + 1)?.1;

    // Each of `default`, `min`, `max` and `var` takes everything up to the
    // next of them, so a value may be several words.
    let mut values: Vec<(&str, String)> = Vec::new();
    for (_, token) in &rest[type_at + 2..] {
        if OPTION_KEYWORDS.contains(token) {
            values.push((token, String::new()));
        } else if let Some((_, value)) = values.last_mut() {
            if !value.is_empty() {
                value.push(' ');
            }
            value.push_str(token);
        }
    }
    let named = |wanted: &str| {
        values
            .iter()
            .find(|(keyword, _)| *keyword == wanted)
            .map(|(_, value)| value.clone())
    };
    let default = named("default");
    let min = named("min");
    let max = named("max");
    let vars: Vec<String> = values
        .iter()
        .filter(|(keyword, _)| *keyword == "var")
        .map(|(_, value)| value.clone())
        .collect();

    let kind = match type_name {
        "check" => OptionKind::Check {
            default: default.as_deref().and_then(|text| text.parse().ok()),
        },
        "spin" => OptionKind::Spin {
            default: default.as_deref().and_then(number),
            min: min.as_deref().and_then(number),
            max: max.as_deref().and_then(number),
        },
        "combo" => OptionKind::Combo { default, vars },
        "button" => OptionKind::Button,
        "string" => OptionKind::String {
            default: default.map(|text| {
                if text == "<empty>" {
                    String::new()
                } else {
                    text
                }
            }),
        },
        _ => return None,
    };
    Some(Message::Option(OptionSpec { name, kind }))
}

fn join(tokens: &[(usize, &str)]) -> String {
    tokens
        .iter()
        .map(|(_, token)| *token)
        .collect::<Vec<_>>()
        .join(" ")
}

fn number<T: std::str::FromStr>(text: &str) -> Option<T> {
    text.parse().ok()
}

/// The keywords of an `info` line, each ending the value list of the one
/// before it.
const INFO_KEYWORDS: [&str; 21] = [
    "depth",
    "seldepth",
    "time",
    "nodes",
    "pv",
    "multipv",
    "score",
    "cp",
    "mate",
    "lowerbound",
    "upperbound",
    "wdl",
    "currmove",
    "currmovenumber",
    "hashfull",
    "nps",
    "tbhits",
    "sbhits",
    "cpuload",
    "refutation",
    "currline",
];

fn parse_info(line: &str, rest: &[(usize, &str)]) -> Info {
    let mut info = Info::default();
    let mut at = 0;
    while at < rest.len() {
        let (offset, keyword) = rest[at];
        at += 1;
        match keyword {
            "string" => {
                info.string = Some(match rest.get(at) {
                    Some((from, _)) => line[*from..].trim_end().to_owned(),
                    None => String::new(),
                });
                break;
            }
            "pv" | "refutation" => {
                let (moves, next) = move_list(rest, at);
                if moves.is_empty() {
                    info.unknown.push(keyword.to_owned());
                } else if keyword == "pv" {
                    info.pv = moves;
                } else {
                    info.refutation = moves;
                }
                at = next;
            }
            "currline" => {
                let cpu = rest.get(at).and_then(|(_, token)| number::<u32>(token));
                if cpu.is_some() {
                    at += 1;
                }
                let (moves, next) = move_list(rest, at);
                at = next;
                info.currline = Some(CurrLine { cpu, moves });
            }
            "score" => at = parse_score(&mut info, rest, at),
            "wdl" => {
                let values: Vec<u32> = rest[at..]
                    .iter()
                    .take(3)
                    .map_while(|(_, token)| number::<u32>(token))
                    .collect();
                if let [win, draw, loss] = values[..] {
                    info.wdl = Some(Wdl { win, draw, loss });
                    at += 3;
                } else {
                    info.unknown.push(keyword.to_owned());
                }
            }
            "currmove" => match rest.get(at) {
                Some((_, token)) => {
                    info.currmove = Some((*token).to_owned());
                    at += 1;
                }
                None => info.unknown.push(keyword.to_owned()),
            },
            "depth" | "seldepth" | "multipv" | "currmovenumber" | "hashfull" | "cpuload"
            | "time" | "nodes" | "nps" | "tbhits" | "sbhits" => {
                let value = rest.get(at).and_then(|(_, token)| number::<u64>(token));
                match value {
                    Some(value) => {
                        at += 1;
                        match keyword {
                            "depth" => info.depth = Some(value as u32),
                            "seldepth" => info.seldepth = Some(value as u32),
                            "multipv" => info.multipv = Some(value as u32),
                            "currmovenumber" => info.currmovenumber = Some(value as u32),
                            "hashfull" => info.hashfull = Some(value as u32),
                            "cpuload" => info.cpuload = Some(value as u32),
                            "time" => info.time = Some(Duration::from_millis(value)),
                            "nodes" => info.nodes = Some(value),
                            "nps" => info.nps = Some(value),
                            "tbhits" => info.tbhits = Some(value),
                            _ => info.sbhits = Some(value),
                        }
                    }
                    None => info.unknown.push(keyword.to_owned()),
                }
            }
            _ => {
                let _ = offset;
                info.unknown.push(keyword.to_owned());
            }
        }
    }
    info
}

/// The moves from `at` up to the next `info` keyword, and the index after
/// them.
fn move_list(rest: &[(usize, &str)], at: usize) -> (Vec<String>, usize) {
    let mut moves = Vec::new();
    let mut next = at;
    while let Some((_, token)) = rest.get(next) {
        if INFO_KEYWORDS.contains(token) || *token == "string" {
            break;
        }
        moves.push((*token).to_owned());
        next += 1;
    }
    (moves, next)
}

fn parse_score(info: &mut Info, rest: &[(usize, &str)], at: usize) -> usize {
    let mut at = at;
    let mut seen = false;
    while let Some((_, token)) = rest.get(at) {
        match *token {
            "cp" | "mate" => {
                let Some(value) = rest.get(at + 1).and_then(|(_, text)| number::<i32>(text)) else {
                    break;
                };
                info.score = Some(if *token == "cp" {
                    Score::Cp(value)
                } else {
                    Score::Mate(value)
                });
                seen = true;
                at += 2;
            }
            "lowerbound" | "upperbound" => {
                info.bound = Some(if *token == "lowerbound" {
                    Bound::Lower
                } else {
                    Bound::Upper
                });
                seen = true;
                at += 1;
            }
            _ => break,
        }
    }
    if !seen {
        info.unknown.push("score".to_owned());
    }
    at
}

// -- The state machine ------------------------------------------------------

/// What the engine is doing, as far as the conversation says.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    /// Nothing has been asked of it yet.
    Started,
    /// `uci` has gone out and `uciok` has not come back.
    Identifying,
    /// Identified, and not searching.
    Idle,
    /// Searching.
    Searching,
    /// Searching on the move it expects, until `ponderhit` or `stop`.
    Pondering,
    /// `quit` has gone out.
    Quitting,
}

impl State {
    /// The state's name, as it reads in an error.
    pub fn name(self) -> &'static str {
        match self {
            State::Started => "started",
            State::Identifying => "identifying",
            State::Idle => "idle",
            State::Searching => "searching",
            State::Pondering => "pondering",
            State::Quitting => "quitting",
        }
    }
}

/// A command sent, or a message received, that the conversation had no room
/// for.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ProtocolError {
    /// The command may not be sent in this state.
    Command {
        /// The command's keyword.
        keyword: &'static str,
        /// The state it was sent in.
        state: State,
    },
    /// The message may not arrive in this state.
    Message {
        /// The message's keyword.
        keyword: &'static str,
        /// The state it arrived in.
        state: State,
    },
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolError::Command { keyword, state } => {
                write!(f, "{keyword} cannot be sent to a {} engine", state.name())
            }
            ProtocolError::Message { keyword, state } => {
                write!(f, "a {} engine cannot send {keyword}", state.name())
            }
        }
    }
}

impl std::error::Error for ProtocolError {}

/// Which commands may go out and which messages may come in, tracked over one
/// conversation.
#[derive(Clone, Debug)]
pub struct Session {
    state: State,
    pending_ready: u32,
}

impl Session {
    /// A session with nothing asked yet.
    pub fn new() -> Session {
        Session {
            state: State::Started,
            pending_ready: 0,
        }
    }

    /// What the engine is doing.
    pub fn state(&self) -> State {
        self.state
    }

    /// How many `isready` commands are still unanswered.
    pub fn pending_ready(&self) -> u32 {
        self.pending_ready
    }

    /// Records a command as sent.
    pub fn sent(&mut self, command: &Command) -> Result<(), ProtocolError> {
        let refuse = || {
            Err(ProtocolError::Command {
                keyword: command.keyword(),
                state: self.state,
            })
        };
        match command {
            Command::Uci => match self.state {
                State::Started | State::Idle => self.state = State::Identifying,
                _ => return refuse(),
            },
            Command::IsReady => {
                if self.state == State::Quitting {
                    return refuse();
                }
                self.pending_ready += 1;
            }
            Command::Debug(_) | Command::Register(_) => {
                if self.state == State::Quitting {
                    return refuse();
                }
            }
            Command::SetOption { .. } | Command::NewGame | Command::Position(_) => {
                if self.state != State::Idle {
                    return refuse();
                }
            }
            Command::Go(limits) => match self.state {
                State::Idle if limits.ponder => self.state = State::Pondering,
                State::Idle => self.state = State::Searching,
                _ => return refuse(),
            },
            Command::Stop => match self.state {
                State::Searching | State::Pondering => {}
                _ => return refuse(),
            },
            Command::PonderHit => match self.state {
                State::Pondering => self.state = State::Searching,
                _ => return refuse(),
            },
            Command::Quit => self.state = State::Quitting,
        }
        Ok(())
    }

    /// Records a message as received.
    pub fn received(&mut self, message: &Message) -> Result<(), ProtocolError> {
        let refuse = |keyword| {
            Err(ProtocolError::Message {
                keyword,
                state: self.state,
            })
        };
        match message {
            Message::Id { .. } | Message::Option(_) => {
                if self.state != State::Identifying {
                    return refuse(if matches!(message, Message::Option(_)) {
                        "option"
                    } else {
                        "id"
                    });
                }
            }
            Message::UciOk => match self.state {
                State::Identifying => self.state = State::Idle,
                _ => return refuse("uciok"),
            },
            Message::ReadyOk => match self.pending_ready.checked_sub(1) {
                Some(left) => self.pending_ready = left,
                None => return refuse("readyok"),
            },
            Message::BestMove(_) => match self.state {
                State::Searching | State::Pondering => self.state = State::Idle,
                _ => return refuse("bestmove"),
            },
            Message::Info(_)
            | Message::Registration(_)
            | Message::CopyProtection(_)
            | Message::Raw(_) => {}
        }
        Ok(())
    }
}

impl Default for Session {
    fn default() -> Session {
        Session::new()
    }
}
