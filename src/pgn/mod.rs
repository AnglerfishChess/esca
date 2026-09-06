//! Portable Game Notation: the tag pairs, the move tree and the text.
//!
//! ```
//! use esca::pgn;
//!
//! let text = "[Event \"Casual\"]\n\n1. e4 e5 2. Nf3 (2. f4 exf4) 2... Nc6 *\n";
//! let game = pgn::read_str(text).next().expect("a game").expect("it reads");
//! assert_eq!(game.headers.get("Event"), Some("Casual"));
//! assert_eq!(game.mainline().len(), 4);
//! assert_eq!(game.mainline()[2].variations[0][0].san, "f4");
//! assert_eq!(game.to_string(), text);
//! ```

mod read;
mod write;

use core::fmt;
use std::sync::Arc;

use crate::error::{FenError, PositionError};
use crate::game::Game as PlayedGame;
use crate::moves::Move;
use crate::position::Position;
use crate::types::Colour;
use crate::variant::{Outcome, Variant, chess960, classic};

pub use read::{Reader, count_games, read, read_str};
pub use write::EXPORT_WIDTH;

/// The game-termination marker.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum GameResult {
    /// `1-0`.
    White,
    /// `0-1`.
    Black,
    /// `1/2-1/2`.
    Draw,
    /// `*`: unknown, or the game is still in progress.
    #[default]
    Unknown,
}

impl GameResult {
    /// The result `text` names, if it names one.
    pub fn from_text(text: &str) -> Option<GameResult> {
        match text {
            "1-0" => Some(GameResult::White),
            "0-1" => Some(GameResult::Black),
            "1/2-1/2" => Some(GameResult::Draw),
            "*" => Some(GameResult::Unknown),
            _ => None,
        }
    }

    /// The marker.
    pub fn as_str(self) -> &'static str {
        match self {
            GameResult::White => "1-0",
            GameResult::Black => "0-1",
            GameResult::Draw => "1/2-1/2",
            GameResult::Unknown => "*",
        }
    }
}

impl fmt::Display for GameResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The tag pairs of a game, in the order they were set.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Headers {
    pairs: Vec<(String, String)>,
}

impl Headers {
    /// The tags every export-format game carries, in the order it carries
    /// them.
    pub const SEVEN_TAG_ROSTER: [&'static str; 7] =
        ["Event", "Site", "Date", "Round", "White", "Black", "Result"];

    /// No tag pairs.
    pub fn new() -> Headers {
        Headers::default()
    }

    /// The value of the tag `name`.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find(|(tag, _)| tag == name)
            .map(|(_, value)| value.as_str())
    }

    /// Whether the tag `name` is present.
    pub fn contains(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// Sets the tag `name`, keeping its place when it is already present.
    pub fn set(&mut self, name: &str, value: &str) {
        match self.pairs.iter_mut().find(|(tag, _)| tag == name) {
            Some(pair) => pair.1 = value.to_string(),
            None => self.pairs.push((name.to_string(), value.to_string())),
        }
    }

    /// Removes the tag `name`, returning its value.
    pub fn remove(&mut self, name: &str) -> Option<String> {
        let index = self.pairs.iter().position(|(tag, _)| tag == name)?;
        Some(self.pairs.remove(index).1)
    }

    /// The tag pairs, in the order they were set.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.pairs
            .iter()
            .map(|(tag, value)| (tag.as_str(), value.as_str()))
    }

    /// How many tag pairs there are.
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    /// Whether there are no tag pairs.
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    /// The tag pairs in export order: the seven-tag roster first, in roster
    /// order and only where present, then the rest in the order they were set.
    pub fn export_order(&self) -> Vec<(&str, &str)> {
        let mut out: Vec<(&str, &str)> = Headers::SEVEN_TAG_ROSTER
            .iter()
            .filter_map(|&name| self.get(name).map(|value| (name, value)))
            .collect();
        out.extend(
            self.iter()
                .filter(|(name, _)| !Headers::SEVEN_TAG_ROSTER.contains(name)),
        );
        out
    }
}

/// One move of a game tree: the move, its text, its annotations and the
/// alternatives written beside it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Node {
    /// The move played.
    pub mv: Move,
    /// The move's own text, as written, less any `!`/`?` suffix.
    pub san: String,
    /// The numeric annotation glyphs, in the order written.
    pub nags: Vec<u16>,
    /// The comment written before the move.
    pub comment_before: String,
    /// The comment written after the move.
    pub comment_after: String,
    /// Alternatives to this move, each a line starting from the position this
    /// move was played in.
    pub variations: Vec<Vec<Node>>,
}

impl Node {
    /// A move with its text and nothing else.
    pub fn new(mv: Move, san: &str) -> Node {
        Node {
            mv,
            san: san.to_string(),
            nags: Vec::new(),
            comment_before: String::new(),
            comment_after: String::new(),
            variations: Vec::new(),
        }
    }
}

/// A game as PGN describes one: tag pairs, a move tree, and a result.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Game {
    /// The tag pairs.
    pub headers: Headers,
    /// The comment written before the first move.
    pub comment: String,
    /// The mainline.
    pub moves: Vec<Node>,
    /// The game-termination marker.
    pub result: GameResult,
}

impl Game {
    /// A game with no tags and no moves.
    pub fn new() -> Game {
        Game::default()
    }

    /// The moves of the mainline.
    pub fn mainline(&self) -> &[Node] {
        &self.moves
    }

    /// The rules and the start position the `Variant` and `FEN` tags name.
    ///
    /// `Variant` selects the rules: absent, `Chess`, `Standard` or `Normal`
    /// means classic chess, and `Chess960`, `Chess 960` or `Fischerandom`
    /// means Chess960; any other value is an error. A `FEN` tag is read in
    /// either castling dialect and is used whatever `SetUp` says.
    pub fn setup(&self) -> Result<(Arc<dyn Variant>, Position), PgnError> {
        let variant = match self.headers.get("Variant") {
            None => classic(),
            Some(name) => match name
                .to_ascii_lowercase()
                .replace(['-', ' ', '_'], "")
                .as_str()
            {
                "" | "chess" | "standard" | "normal" | "fromposition" => classic(),
                "chess960" | "fischerandom" | "fischerrandom" => chess960(),
                _ => {
                    return Err(PgnError::detached(ErrorKind::UnknownVariant(
                        name.to_string(),
                    )));
                }
            },
        };
        let position = match self.headers.get("FEN") {
            None => variant.start_position(0),
            Some(fen) => {
                Position::from_fen(fen).map_err(|e| PgnError::detached(ErrorKind::BadFen(e)))?
            }
        };
        variant
            .validate(&position)
            .map_err(|e| PgnError::detached(ErrorKind::BadSetup(e)))?;
        Ok((variant, position))
    }

    /// The mainline as a played game.
    pub fn mainline_game(&self) -> Result<PlayedGame, PgnError> {
        let (variant, start) = self.setup()?;
        let mut game = PlayedGame::from_position(variant, start)
            .map_err(|e| PgnError::detached(ErrorKind::BadSetup(e)))?;
        for node in &self.moves {
            game.play(node.mv)
                .map_err(|_| PgnError::detached(ErrorKind::IllegalMove(node.san.clone())))?;
        }
        Ok(game)
    }

    /// The PGN of a played game: the seven-tag roster, the mainline, and the
    /// `Variant`, `SetUp` and `FEN` tags the start needs.
    pub fn from_game(game: &PlayedGame) -> Game {
        let variant = game.variant();
        let result = match game.outcome() {
            Some(Outcome::Checkmate {
                winner: Colour::White,
            }) => GameResult::White,
            Some(Outcome::Checkmate {
                winner: Colour::Black,
            }) => GameResult::Black,
            Some(_) => GameResult::Draw,
            None => GameResult::Unknown,
        };

        let mut headers = Headers::new();
        headers.set("Event", "?");
        headers.set("Site", "?");
        headers.set("Date", "????.??.??");
        headers.set("Round", "?");
        headers.set("White", "?");
        headers.set("Black", "?");
        headers.set("Result", result.as_str());
        if variant.name() != "chess" {
            headers.set("Variant", "Chess960");
        }
        let start = game.start_position();
        if start.fen() != variant.start_position(0).fen() {
            headers.set("SetUp", "1");
            headers.set("FEN", &start.fen());
        }

        let mut position = start.clone();
        let mut moves = Vec::with_capacity(game.moves().len());
        for &mv in game.moves() {
            moves.push(Node::new(mv, &variant.move_to_san(&position, mv)));
            position = variant.play(&position, mv);
        }
        Game {
            headers,
            comment: String::new(),
            moves,
            result,
        }
    }

    /// The full-move number and side to move the movetext starts at.
    fn numbering(&self) -> (u32, bool) {
        match self
            .headers
            .get("FEN")
            .and_then(|fen| Position::from_fen(fen).ok())
        {
            Some(position) => (
                position.fullmove_number(),
                position.side_to_move() == Colour::White,
            ),
            None => (1, true),
        }
    }
}

impl fmt::Display for Game {
    /// Export format: the tag pairs one per line with the seven-tag roster
    /// first, a blank line, then the movetext wrapped at [`EXPORT_WIDTH`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&write::game(self))
    }
}

impl PlayedGame {
    /// This game as PGN, with a seven-tag roster of placeholders.
    #[cfg_attr(docsrs, doc(cfg(feature = "pgn")))]
    pub fn to_pgn(&self) -> Game {
        Game::from_game(self)
    }
}

/// Why PGN text could not be read.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PgnError {
    /// The 1-based line of the offending text; 0 when the error did not come
    /// from text.
    pub line: usize,
    /// The 1-based column of the offending text; 0 when the error did not
    /// come from text.
    pub column: usize,
    /// What was wrong.
    pub kind: ErrorKind,
}

impl PgnError {
    pub(crate) fn at(line: usize, column: usize, kind: ErrorKind) -> PgnError {
        PgnError { line, column, kind }
    }

    /// An error that did not come from text.
    pub(crate) fn detached(kind: ErrorKind) -> PgnError {
        PgnError::at(0, 0, kind)
    }

    /// The same error, placed at `line` and `column` unless it is already
    /// placed.
    pub(crate) fn placed(mut self, line: usize, column: usize) -> PgnError {
        if self.line == 0 {
            self.line = line;
            self.column = column;
        }
        self
    }
}

impl fmt::Display for PgnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(f, "{}", self.kind)
        } else {
            write!(
                f,
                "line {}, column {}: {}",
                self.line, self.column, self.kind
            )
        }
    }
}

impl std::error::Error for PgnError {}

/// What was wrong with PGN text.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ErrorKind {
    /// A `{` comment with no `}`.
    UnterminatedComment,
    /// A tag-pair value with no closing quote.
    UnterminatedString,
    /// A tag pair that is not `[Name "value"]`.
    MalformedTag,
    /// A `(` variation with no `)`.
    UnterminatedVariation,
    /// A `)` with no variation open.
    UnexpectedVariationEnd,
    /// A `(` with no preceding move to give an alternative to.
    StrayVariation,
    /// Move text naming no legal move, as written.
    IllegalMove(String),
    /// Move text naming more than one legal move, as written.
    AmbiguousMove(String),
    /// Text that is not movetext, as written.
    Syntax(String),
    /// A `Variant` tag naming rules esca does not have.
    UnknownVariant(String),
    /// A `FEN` tag that is not a position.
    BadFen(FenError),
    /// A `FEN` tag naming a position the variant cannot play on.
    BadSetup(PositionError),
    /// The input could not be read.
    Io(String),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::UnterminatedComment => f.write_str("unterminated comment"),
            ErrorKind::UnterminatedString => f.write_str("unterminated tag value"),
            ErrorKind::MalformedTag => f.write_str("malformed tag pair"),
            ErrorKind::UnterminatedVariation => f.write_str("unterminated variation"),
            ErrorKind::UnexpectedVariationEnd => f.write_str("no variation to close"),
            ErrorKind::StrayVariation => f.write_str("a variation of no move"),
            ErrorKind::IllegalMove(text) => write!(f, "no such legal move: {text}"),
            ErrorKind::AmbiguousMove(text) => write!(f, "ambiguous move text: {text}"),
            ErrorKind::Syntax(text) => write!(f, "malformed movetext: {text}"),
            ErrorKind::UnknownVariant(name) => write!(f, "unknown variant: {name}"),
            ErrorKind::BadFen(error) => write!(f, "the FEN tag is unreadable: {error}"),
            ErrorKind::BadSetup(error) => write!(f, "the FEN tag is unplayable: {error}"),
            ErrorKind::Io(message) => write!(f, "the input could not be read: {message}"),
        }
    }
}
