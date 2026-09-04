//! Counting the moves of played games into a book.

use std::collections::BTreeMap;
use std::io;
use std::path::Path;

use crate::game::Game;

use super::{Raw, write_raw};

/// Counts the moves of the games it is given and writes them as a book.
#[derive(Clone, Debug, Default)]
pub struct Builder {
    max_ply: Option<u32>,
    min_count: u32,
    counts: BTreeMap<(u64, u16), u32>,
}

impl Builder {
    /// A builder that counts every move of every game and writes them all.
    pub fn new() -> Builder {
        Builder {
            max_ply: None,
            min_count: 1,
            counts: BTreeMap::new(),
        }
    }

    /// Moves played later than `plies` into a game are not counted.
    pub fn max_ply(mut self, plies: u32) -> Builder {
        self.max_ply = Some(plies);
        self
    }

    /// A move played in fewer than `count` games is not written.
    pub fn min_count(mut self, count: u32) -> Builder {
        self.min_count = count;
        self
    }

    /// Counts every move of `game`, each in the position it was played in.
    pub fn add_game(&mut self, game: &Game) {
        let limit = self.max_ply.unwrap_or(u32::MAX) as usize;
        for (position, &mv) in game.positions().zip(game.moves()).take(limit) {
            *self
                .counts
                .entry((position.polyglot_key(), Raw::pack(mv)))
                .or_default() += 1;
        }
    }

    /// Counts every game of a PGN source, skipping the ones that do not read;
    /// returns how many were counted.
    #[cfg(feature = "pgn")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pgn")))]
    pub fn add_pgn<R: io::BufRead>(&mut self, input: R) -> usize {
        let mut added = 0;
        for game in crate::pgn::Reader::new(input).skipping() {
            let Ok(game) = game.and_then(|game| game.mainline_game()) else {
                continue;
            };
            self.add_game(&game);
            added += 1;
        }
        added
    }

    /// How many distinct position-and-move pairs have been counted, before
    /// `min_count` drops any.
    pub fn len(&self) -> usize {
        self.counts.len()
    }

    /// Whether nothing has been counted.
    pub fn is_empty(&self) -> bool {
        self.counts.is_empty()
    }

    /// The book rows: one per move counted at least `min_count` times,
    /// weighed by that count and saturated at `u16::MAX`, sorted by key and
    /// then by descending weight.
    pub fn entries(&self) -> Vec<Raw> {
        let mut entries: Vec<Raw> = self
            .counts
            .iter()
            .filter(|&(_, &count)| count >= self.min_count)
            .map(|(&(key, mv), &count)| Raw {
                key,
                mv,
                weight: u16::try_from(count).unwrap_or(u16::MAX),
                learn: 0,
            })
            .collect();
        entries.sort_by_key(|entry| (entry.key, std::cmp::Reverse(entry.weight), entry.mv));
        entries
    }

    /// Writes those rows to `path`.
    pub fn write(&self, path: &Path) -> io::Result<()> {
        write_raw(path, self.entries())
    }
}
