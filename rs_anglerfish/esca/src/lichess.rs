//! Reader for the Lichess evaluation dump: Zstandard-compressed JSON lines,
//! one record per position.
//!
//! Scores are reported side-relative, as [`Score`] is defined: the dump writes
//! them from White's point of view, and the reader negates those of a record
//! whose position has Black to move.

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use serde::Deserialize;

use crate::error::{FenError, MoveParseError};
use crate::moves::Move;
use crate::position::{Position, Score};
use crate::variant::Variant;

/// One position and every evaluation the dump carries for it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Record {
    /// The four-field FEN the dump identifies the position by.
    pub epd: String,
    /// The evaluations, in the order the dump lists them.
    pub evals: Vec<Eval>,
}

/// One engine run over a position.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Eval {
    /// The depth the run reached.
    pub depth: u32,
    /// Thousands of nodes searched.
    pub knodes: u64,
    /// The principal variations, best first.
    pub pvs: Vec<Pv>,
}

/// One principal variation and its score.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Pv {
    /// The score of the line, positive when it favours the side to move.
    pub score: Score,
    /// The moves, in UCI notation, separated by single spaces.
    pub line: String,
}

impl Record {
    /// The position, read from the four-field FEN: `clocks_known()` is false.
    ///
    /// A small share of the dump's rows describe placements no game can
    /// reach, and are a [`FenError`].
    pub fn position(&self) -> Result<Position, FenError> {
        Position::from_fen(&self.epd)
    }
}

impl Pv {
    /// The first move of the line, read in `position` under `variant`.
    pub fn best_move(
        &self,
        variant: &dyn Variant,
        position: &Position,
    ) -> Result<Move, MoveParseError> {
        let text = self
            .line
            .split_ascii_whitespace()
            .next()
            .ok_or(MoveParseError::Syntax)?;
        variant.move_from_uci(position, text)
    }
}

/// Streams the Zstandard-compressed dump at `path`.
///
/// The file is read as it is iterated, never held in memory.
pub fn read(path: &Path) -> io::Result<impl Iterator<Item = io::Result<Record>> + use<>> {
    let decoder = zstd::Decoder::new(File::open(path)?)?;
    Ok(read_from(BufReader::new(decoder)))
}

/// Streams decompressed JSON lines. Blank lines are skipped.
pub fn read_from<R: BufRead>(reader: R) -> impl Iterator<Item = io::Result<Record>> {
    reader.lines().filter_map(|line| match line {
        Ok(text) if text.trim().is_empty() => None,
        Ok(text) => Some(parse(&text)),
        Err(error) => Some(Err(error)),
    })
}

/// One JSON line.
fn parse(line: &str) -> io::Result<Record> {
    let raw: RawRecord = serde_json::from_str(line)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let black_to_move = raw.fen.split_ascii_whitespace().nth(1) == Some("b");
    let sign = if black_to_move { -1 } else { 1 };
    let evals = raw
        .evals
        .into_iter()
        .map(|eval| Eval {
            depth: eval.depth,
            knodes: eval.knodes,
            pvs: eval
                .pvs
                .into_iter()
                .map(|pv| Pv {
                    score: match (pv.cp, pv.mate) {
                        (_, Some(mate)) => Score::Mate(sign * mate),
                        (Some(cp), None) => Score::Cp(sign * cp),
                        (None, None) => Score::Cp(0),
                    },
                    line: pv.line,
                })
                .collect(),
        })
        .collect();
    Ok(Record {
        epd: raw.fen,
        evals,
    })
}

#[derive(Deserialize)]
struct RawRecord {
    fen: String,
    evals: Vec<RawEval>,
}

#[derive(Deserialize)]
struct RawEval {
    depth: u32,
    knodes: u64,
    pvs: Vec<RawPv>,
}

#[derive(Deserialize)]
struct RawPv {
    #[serde(default)]
    cp: Option<i32>,
    #[serde(default)]
    mate: Option<i32>,
    #[serde(default)]
    line: String,
}
