//! Reading PGN, one game at a time.

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use crate::error::MoveParseError;
use crate::position::Position;
use crate::variant::Variant;

use super::{ErrorKind, Game, GameResult, Node, PgnError};

/// The numeric glyph a `!`/`?` suffix stands for.
fn suffix_nag(text: &str) -> Option<u16> {
    match text {
        "!" => Some(1),
        "?" => Some(2),
        "!!" => Some(3),
        "??" => Some(4),
        "!?" => Some(5),
        "?!" => Some(6),
        _ => None,
    }
}

/// Whether `c` belongs to a movetext word rather than delimiting one.
fn is_word(c: char) -> bool {
    !c.is_whitespace() && !matches!(c, '{' | '}' | '(' | ')' | '[' | ']' | '<' | '>' | ';' | '$')
}

/// Games read one at a time from a buffered source.
///
/// After a malformed game the reader resynchronises on the next tag section,
/// so one bad game does not lose the rest of the stream.
pub struct Reader<R> {
    scanner: Scanner<R>,
    skip_errors: bool,
}

impl<R: BufRead> Reader<R> {
    /// A reader over `input`, reporting malformed games as errors.
    pub fn new(input: R) -> Reader<R> {
        Reader {
            scanner: Scanner::new(input),
            skip_errors: false,
        }
    }

    /// A reader that drops malformed games instead of reporting them.
    pub fn skipping(mut self) -> Reader<R> {
        self.skip_errors = true;
        self
    }

    /// The next game, or `None` at end of input.
    pub fn read_game(&mut self) -> Option<Result<Game, PgnError>> {
        loop {
            match self.parse_game() {
                None => return None,
                Some(Ok(game)) => return Some(Ok(game)),
                Some(Err(error)) => {
                    self.scanner.skip_to_next_game();
                    if !self.skip_errors {
                        return Some(Err(error));
                    }
                }
            }
        }
    }

    fn parse_game(&mut self) -> Option<Result<Game, PgnError>> {
        self.scanner.skip_whitespace();
        self.scanner.peek()?;
        Some(self.game())
    }

    fn game(&mut self) -> Result<Game, PgnError> {
        let mut game = Game::new();
        let mut setup_at = (0, 0);
        while self.scanner.peek() == Some('[') {
            let at = self.scanner.here();
            let (name, value) = self.scanner.tag_pair()?;
            if (name == "Variant" || name == "FEN") && setup_at == (0, 0) {
                setup_at = at;
            }
            game.headers.set(&name, &value);
        }
        let (variant, start) = game
            .setup()
            .map_err(|error| error.placed(setup_at.0, setup_at.1))?;
        self.movetext(&mut game, variant.as_ref(), start)?;
        Ok(game)
    }

    fn movetext(
        &mut self,
        game: &mut Game,
        variant: &dyn Variant,
        start: Position,
    ) -> Result<(), PgnError> {
        let mut frames = vec![Frame::new(start)];
        let mut terminated = false;
        while let Some(c) = self.scanner.peek() {
            let at = self.scanner.here();
            match c {
                '[' => break,
                '{' | ';' => {
                    let text = if c == '{' {
                        self.scanner.brace_comment()?
                    } else {
                        self.scanner.line_comment()
                    };
                    attach_comment(game, &mut frames, &text);
                }
                '(' => {
                    self.scanner.bump();
                    let parent = frames.last().expect("the root frame is never popped");
                    let prior = parent
                        .prior
                        .clone()
                        .ok_or_else(|| PgnError::at(at.0, at.1, ErrorKind::StrayVariation))?;
                    frames.push(Frame::new(prior));
                }
                ')' => {
                    self.scanner.bump();
                    let frame = frames.pop().expect("the root frame is never popped");
                    if frames.is_empty() {
                        return Err(PgnError::at(at.0, at.1, ErrorKind::UnexpectedVariationEnd));
                    }
                    let parent = frames.last_mut().expect("a parent was just checked for");
                    let host = parent
                        .nodes
                        .last_mut()
                        .expect("a frame is only pushed after a move");
                    host.variations.push(frame.nodes);
                }
                '$' => {
                    self.scanner.bump();
                    let digits = self.scanner.word();
                    let nag: u16 = digits
                        .parse()
                        .map_err(|_| PgnError::at(at.0, at.1, ErrorKind::Syntax(digits.clone())))?;
                    attach_nag(&mut frames, nag);
                }
                '<' | '>' => {
                    self.scanner.bump();
                }
                ']' | '}' => {
                    self.scanner.bump();
                    return Err(PgnError::at(at.0, at.1, ErrorKind::Syntax(c.to_string())));
                }
                c if c.is_whitespace() => self.scanner.bump(),
                _ => {
                    let word = self.scanner.word();
                    if let Some(result) = GameResult::from_text(&word) {
                        game.result = result;
                        terminated = true;
                        break;
                    }
                    self.token(&word, variant, &mut frames, at)?;
                }
            }
        }
        if frames.len() > 1 {
            let at = self.scanner.here();
            return Err(PgnError::at(at.0, at.1, ErrorKind::UnterminatedVariation));
        }
        if !terminated {
            game.result = game
                .headers
                .get("Result")
                .and_then(GameResult::from_text)
                .unwrap_or(GameResult::Unknown);
        }
        game.moves = frames.pop().expect("the root frame is never popped").nodes;
        Ok(())
    }

    /// One movetext word: a move number, a run of `!`/`?` glyphs, or a move.
    fn token(
        &mut self,
        word: &str,
        variant: &dyn Variant,
        frames: &mut [Frame],
        at: (usize, usize),
    ) -> Result<(), PgnError> {
        let core = word.trim_start_matches(|c: char| c.is_ascii_digit() || c == '.');
        if core.is_empty() {
            return Ok(());
        }
        let san = core.trim_end_matches(['!', '?']);
        let nags = glyphs(&core[san.len()..]);
        if san.is_empty() {
            for nag in nags {
                attach_nag(frames, nag);
            }
            return Ok(());
        }

        let frame = frames.last_mut().expect("the root frame is never popped");
        let mv = variant
            .move_from_san(&frame.position, san)
            .map_err(|error| {
                let kind = match error {
                    MoveParseError::Syntax => ErrorKind::Syntax(word.to_string()),
                    MoveParseError::Illegal => ErrorKind::IllegalMove(word.to_string()),
                    MoveParseError::Ambiguous => ErrorKind::AmbiguousMove(word.to_string()),
                };
                PgnError::at(at.0, at.1, kind)
            })?;
        let before = frame.position.clone();
        frame.position = variant.play(&before, mv);
        frame.prior = Some(before);
        let mut node = Node::new(mv, san);
        node.nags = nags;
        node.comment_before = std::mem::take(&mut frame.pending);
        frame.nodes.push(node);
        Ok(())
    }
}

impl<R: BufRead> Iterator for Reader<R> {
    type Item = Result<Game, PgnError>;

    fn next(&mut self) -> Option<Result<Game, PgnError>> {
        self.read_game()
    }
}

/// One line of the tree being read, and the position its next move is played
/// in.
struct Frame {
    /// The position the next move is played in.
    position: Position,
    /// The position the last move was played in, which a variation branches
    /// from.
    prior: Option<Position>,
    /// A comment read before this line's first move.
    pending: String,
    nodes: Vec<Node>,
}

impl Frame {
    fn new(position: Position) -> Frame {
        Frame {
            position,
            prior: None,
            pending: String::new(),
            nodes: Vec::new(),
        }
    }
}

/// The glyphs `text` spells, longest suffix form first.
fn glyphs(text: &str) -> Vec<u16> {
    let mut out = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        let take = rest.len().min(2);
        if let Some(nag) = suffix_nag(&rest[..take]) {
            out.push(nag);
            rest = &rest[take..];
        } else if let Some(nag) = suffix_nag(&rest[..1]) {
            out.push(nag);
            rest = &rest[1..];
        } else {
            rest = &rest[1..];
        }
    }
    out
}

/// Appends `text` where a comment written here belongs: to the last move of
/// the current line, else before its first move, else to the game.
fn attach_comment(game: &mut Game, frames: &mut [Frame], text: &str) {
    let root = frames.len() == 1;
    let frame = frames.last_mut().expect("the root frame is never popped");
    let target = match frame.nodes.last_mut() {
        Some(node) => &mut node.comment_after,
        None if root => &mut game.comment,
        None => &mut frame.pending,
    };
    if !target.is_empty() {
        target.push(' ');
    }
    target.push_str(text);
}

/// Appends `nag` to the last move of the current line, if there is one.
fn attach_nag(frames: &mut [Frame], nag: u16) {
    if let Some(node) = frames
        .last_mut()
        .expect("the root frame is never popped")
        .nodes
        .last_mut()
    {
        node.nags.push(nag);
    }
}

/// A character cursor over lines, which is where PGN's line and column come
/// from. Escape lines are dropped as they are read.
struct Scanner<R> {
    input: R,
    line: Vec<char>,
    line_no: usize,
    col: usize,
    eof: bool,
}

impl<R: BufRead> Scanner<R> {
    fn new(input: R) -> Scanner<R> {
        Scanner {
            input,
            line: Vec::new(),
            line_no: 0,
            col: 0,
            eof: false,
        }
    }

    /// Loads the next line. False at end of input, and on an unreadable one.
    fn next_line(&mut self) -> bool {
        loop {
            let mut buffer = String::new();
            match self.input.read_line(&mut buffer) {
                Ok(0) | Err(_) => {
                    self.eof = true;
                    self.line.clear();
                    self.col = 0;
                    return false;
                }
                Ok(_) => {
                    self.line_no += 1;
                    self.col = 0;
                    while buffer.ends_with('\n') || buffer.ends_with('\r') {
                        buffer.pop();
                    }
                    if buffer.starts_with('%') {
                        continue;
                    }
                    self.line = buffer.chars().collect();
                    return true;
                }
            }
        }
    }

    /// The character at the cursor, crossing line ends. `None` at end of
    /// input.
    fn peek(&mut self) -> Option<char> {
        loop {
            if self.col < self.line.len() {
                return Some(self.line[self.col]);
            }
            if !self.next_line() {
                return None;
            }
        }
    }

    /// The 1-based line and column of the cursor.
    fn here(&self) -> (usize, usize) {
        (self.line_no, self.col + 1)
    }

    fn bump(&mut self) {
        self.col += 1;
    }

    fn at_line_end(&self) -> bool {
        self.col >= self.line.len()
    }

    /// The run of word characters at the cursor. A word never spans lines.
    fn word(&mut self) -> String {
        let mut out = String::new();
        while !self.at_line_end() && is_word(self.line[self.col]) {
            out.push(self.line[self.col]);
            self.bump();
        }
        out
    }

    /// Moves the cursor to the next character that is not whitespace.
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if !c.is_whitespace() {
                return;
            }
            self.bump();
        }
    }

    /// A `{ … }` comment, which may span lines. Its whitespace is collapsed.
    fn brace_comment(&mut self) -> Result<String, PgnError> {
        let (line, column) = self.here();
        self.bump();
        let mut out = String::new();
        loop {
            if self.at_line_end() {
                if !self.next_line() {
                    return Err(PgnError::at(line, column, ErrorKind::UnterminatedComment));
                }
                out.push(' ');
                continue;
            }
            let c = self.line[self.col];
            self.bump();
            if c == '}' {
                break;
            }
            out.push(c);
        }
        Ok(collapse(&out))
    }

    /// A `;` comment: the rest of the line. Its whitespace is collapsed.
    fn line_comment(&mut self) -> String {
        self.bump();
        let out: String = self.line[self.col..].iter().collect();
        self.col = self.line.len();
        collapse(&out)
    }

    /// One `[Name "value"]` tag pair.
    fn tag_pair(&mut self) -> Result<(String, String), PgnError> {
        let (line, column) = self.here();
        let malformed = PgnError::at(line, column, ErrorKind::MalformedTag);
        self.bump();
        self.skip_spaces();
        let mut name = String::new();
        while !self.at_line_end() {
            let c = self.line[self.col];
            if c.is_alphanumeric() || c == '_' {
                name.push(c);
                self.bump();
            } else {
                break;
            }
        }
        self.skip_spaces();
        if name.is_empty() || self.at_line_end() || self.line[self.col] != '"' {
            return Err(malformed);
        }
        self.bump();

        let mut value = String::new();
        loop {
            if self.at_line_end() {
                return Err(PgnError::at(line, column, ErrorKind::UnterminatedString));
            }
            let c = self.line[self.col];
            self.bump();
            match c {
                '"' => break,
                '\\' if !self.at_line_end() => {
                    value.push(self.line[self.col]);
                    self.bump();
                }
                _ => value.push(c),
            }
        }
        self.skip_spaces();
        if self.at_line_end() || self.line[self.col] != ']' {
            return Err(malformed);
        }
        self.bump();
        Ok((name, value))
    }

    fn skip_spaces(&mut self) {
        while !self.at_line_end() && self.line[self.col].is_whitespace() {
            self.bump();
        }
    }

    /// Discards text up to the tag section of the next game.
    fn skip_to_next_game(&mut self) {
        self.col = self.line.len();
        let mut blank = false;
        while self.next_line() {
            if blank && self.line.first() == Some(&'[') {
                return;
            }
            blank = self.line.iter().all(|c| c.is_whitespace());
        }
    }
}

/// One space between words, and none at the ends.
fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Games read from `path`.
pub fn read(path: &Path) -> io::Result<Reader<BufReader<File>>> {
    Ok(Reader::new(BufReader::new(File::open(path)?)))
}

/// Games read from PGN text.
pub fn read_str(text: &str) -> Reader<&[u8]> {
    Reader::new(text.as_bytes())
}

/// How many games `input` holds that read without error.
pub fn count_games<R: BufRead>(input: R) -> usize {
    Reader::new(input).skipping().count()
}
