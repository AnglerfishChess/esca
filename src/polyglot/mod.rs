//! Opening books in the Polyglot format: a file of 16-byte entries sorted by
//! [`Position::polyglot_key`].
//!
//! ```
//! use esca::polyglot::{Book, Entry};
//! use esca::{Game, classic};
//!
//! let mut game = Game::new(classic());
//! let e4 = game.legal_moves().iter().copied().find(|mv| mv.to_string() == "e2e4").unwrap();
//! let key = game.position().polyglot_key();
//!
//! let path = std::env::temp_dir().join("esca-doctest-book.bin");
//! Book::write(&path, &[Entry::new(key, e4, 100, 0)]).unwrap();
//!
//! let book = Book::open(&path).unwrap();
//! assert_eq!(book.len(), 1);
//! assert_eq!(book.best(game.variant(), game.position()).unwrap().mv, e4);
//! # std::fs::remove_file(&path).unwrap();
//! ```

mod build;

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use memmap2::Mmap;

use crate::moves::{Move, MoveList};
use crate::position::Position;
use crate::types::{File, Rank, Role, Square};
use crate::variant::Variant;

pub use build::Builder;

/// The bytes one entry occupies.
pub const ENTRY_SIZE: usize = 16;

/// One entry as the file holds it, with its move still the format's 16 bits.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Raw {
    /// The Polyglot key of the position the move belongs to.
    pub key: u64,
    /// The move, encoded as the format encodes it.
    pub mv: u16,
    /// How good, or how often played, the move is, relative to the other
    /// entries of this key.
    pub weight: u16,
    /// Four bytes the format reserves for a program's own use.
    pub learn: u32,
}

impl Raw {
    /// The bits the format encodes `mv` as.
    pub fn pack(mv: Move) -> u16 {
        let promotion: u16 = match mv.promotion() {
            Some(Role::Knight) => 1,
            Some(Role::Bishop) => 2,
            Some(Role::Rook) => 3,
            Some(Role::Queen) => 4,
            _ => 0,
        };
        (promotion << 12)
            | ((mv.from().rank().index() as u16) << 9)
            | ((mv.from().file().index() as u16) << 6)
            | ((mv.to().rank().index() as u16) << 3)
            | (mv.to().file().index() as u16)
    }

    /// The origin, destination and promotion role the bits name, castling
    /// king-to-rook; `None` when they name no move.
    pub fn uci(&self) -> Option<String> {
        let (from, to, promotion) = unpack(self.mv)?;
        let mut text = format!("{from}{to}");
        if let Some(role) = promotion {
            text.push(role.to_char());
        }
        Some(text)
    }

    /// The entry with its move read against `position`; `None` when the bits
    /// name no move, or name one that is not legal there.
    pub fn decode(&self, variant: &dyn Variant, position: &Position) -> Option<Entry> {
        let (from, to, promotion) = unpack(self.mv)?;
        let mut moves = MoveList::new();
        variant.legal_moves(position, &mut moves);
        let mv = moves
            .iter()
            .copied()
            .find(|mv| mv.from() == from && mv.to() == to && mv.promotion() == promotion)?;
        Some(Entry {
            key: self.key,
            mv,
            weight: self.weight,
            learn: self.learn,
        })
    }

    fn from_bytes(bytes: &[u8]) -> Raw {
        Raw {
            key: u64::from_be_bytes(bytes[0..8].try_into().expect("eight bytes")),
            mv: u16::from_be_bytes(bytes[8..10].try_into().expect("two bytes")),
            weight: u16::from_be_bytes(bytes[10..12].try_into().expect("two bytes")),
            learn: u32::from_be_bytes(bytes[12..16].try_into().expect("four bytes")),
        }
    }

    fn to_bytes(self) -> [u8; ENTRY_SIZE] {
        let mut bytes = [0u8; ENTRY_SIZE];
        bytes[0..8].copy_from_slice(&self.key.to_be_bytes());
        bytes[8..10].copy_from_slice(&self.mv.to_be_bytes());
        bytes[10..12].copy_from_slice(&self.weight.to_be_bytes());
        bytes[12..16].copy_from_slice(&self.learn.to_be_bytes());
        bytes
    }
}

/// One entry whose move has been read against a position.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Entry {
    /// The Polyglot key of the position the move belongs to.
    pub key: u64,
    /// The move.
    pub mv: Move,
    /// How good, or how often played, the move is, relative to the other
    /// entries of this key.
    pub weight: u16,
    /// Four bytes the format reserves for a program's own use.
    pub learn: u32,
}

impl Entry {
    /// An entry naming `mv` in the position `key` identifies.
    pub fn new(key: u64, mv: Move, weight: u16, learn: u32) -> Entry {
        Entry {
            key,
            mv,
            weight,
            learn,
        }
    }
}

impl From<Entry> for Raw {
    fn from(entry: Entry) -> Raw {
        Raw {
            key: entry.key,
            mv: Raw::pack(entry.mv),
            weight: entry.weight,
            learn: entry.learn,
        }
    }
}

/// An opening book: entries sorted by key, looked up by position.
#[derive(Debug)]
pub struct Book {
    bytes: Bytes,
}

#[derive(Debug)]
enum Bytes {
    Mapped(Mmap),
    Owned(Vec<u8>),
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        match self {
            Bytes::Mapped(map) => map,
            Bytes::Owned(vec) => vec,
        }
    }
}

impl Book {
    /// The book at `path`, memory-mapped.
    ///
    /// The file is read as it is on disk for as long as the book lives, so
    /// truncating it under a live `Book` is the caller's problem.
    pub fn open(path: &Path) -> io::Result<Book> {
        let file = fs::File::open(path)?;
        // Safety: nothing here writes the file, and the caller is told above
        // not to shorten it while the book is open.
        let map = unsafe { Mmap::map(&file)? };
        check_length(map.len())?;
        Ok(Book {
            bytes: Bytes::Mapped(map),
        })
    }

    /// The book `bytes` holds.
    pub fn from_bytes(bytes: Vec<u8>) -> io::Result<Book> {
        check_length(bytes.len())?;
        Ok(Book {
            bytes: Bytes::Owned(bytes),
        })
    }

    /// How many entries the book holds.
    pub fn len(&self) -> usize {
        self.bytes.as_ref().len() / ENTRY_SIZE
    }

    /// Whether the book holds no entries.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The entry at `index`, counting from the start of the file.
    pub fn get(&self, index: usize) -> Option<Raw> {
        if index >= self.len() {
            return None;
        }
        let start = index * ENTRY_SIZE;
        Some(Raw::from_bytes(
            &self.bytes.as_ref()[start..start + ENTRY_SIZE],
        ))
    }

    /// Every entry, in file order.
    pub fn iter(&self) -> impl Iterator<Item = Raw> + '_ {
        (0..self.len()).map(|index| self.get(index).expect("an index below the length"))
    }

    /// The entries at `key`, in the order the file gives them.
    pub fn raw_entries(&self, key: u64) -> Vec<Raw> {
        let mut index = self.lower_bound(key);
        let mut out = Vec::new();
        while let Some(entry) = self.get(index) {
            if entry.key != key {
                break;
            }
            out.push(entry);
            index += 1;
        }
        out
    }

    /// The entries at this position's key that name a move legal in it, in
    /// the order the file gives them.
    pub fn entries(&self, variant: &dyn Variant, position: &Position) -> Vec<Entry> {
        self.raw_entries(position.polyglot_key())
            .into_iter()
            .filter_map(|raw| raw.decode(variant, position))
            .collect()
    }

    /// The heaviest entry of this position; ties go to the earlier one.
    pub fn best(&self, variant: &dyn Variant, position: &Position) -> Option<Entry> {
        self.entries(variant, position)
            .into_iter()
            .reduce(|best, entry| {
                if entry.weight > best.weight {
                    entry
                } else {
                    best
                }
            })
    }

    /// One entry of this position, drawn by weight: the one whose running
    /// weight total first exceeds `seed` reduced modulo the total.
    ///
    /// A pure function of the entries and the seed, so variety is the
    /// caller's to supply. When every entry weighs 0 it returns the first.
    pub fn pick(&self, variant: &dyn Variant, position: &Position, seed: u64) -> Option<Entry> {
        let entries = self.entries(variant, position);
        let total: u64 = entries.iter().map(|entry| u64::from(entry.weight)).sum();
        if total == 0 {
            return entries.into_iter().next();
        }
        let mut drawn = seed % total;
        entries.into_iter().find(|entry| {
            match drawn.checked_sub(u64::from(entry.weight)) {
                // The draw fell inside this entry's share of the total.
                None => true,
                Some(rest) => {
                    drawn = rest;
                    false
                }
            }
        })
    }

    /// Writes `entries` to `path`, sorted by key, then by descending weight,
    /// then by the encoded move; entries sharing a key and a move are merged,
    /// their weights added and saturated at `u16::MAX`.
    pub fn write(path: &Path, entries: &[Entry]) -> io::Result<()> {
        write_raw(path, entries.iter().copied().map(Raw::from).collect())
    }

    /// The index of the first entry whose key is at least `key`.
    fn lower_bound(&self, key: u64) -> usize {
        let (mut low, mut high) = (0, self.len());
        while low < high {
            let middle = low + (high - low) / 2;
            if self.get(middle).expect("an index below the length").key < key {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        low
    }
}

fn check_length(length: usize) -> io::Result<()> {
    if length % ENTRY_SIZE == 0 {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("a book is a whole number of {ENTRY_SIZE}-byte entries"),
        ))
    }
}

/// Sorts, merges and writes; the one place a book is laid out.
pub(crate) fn write_raw(path: &Path, mut entries: Vec<Raw>) -> io::Result<()> {
    entries.sort_by_key(|entry| (entry.key, std::cmp::Reverse(entry.weight), entry.mv));
    let mut merged: Vec<Raw> = Vec::with_capacity(entries.len());
    let mut block = 0;
    for entry in entries {
        if merged.last().is_none_or(|last| last.key != entry.key) {
            block = merged.len();
        }
        match merged[block..].iter().position(|kept| kept.mv == entry.mv) {
            Some(offset) => {
                let kept = &mut merged[block + offset];
                kept.weight = kept.weight.saturating_add(entry.weight);
            }
            None => merged.push(entry),
        }
    }
    merged.sort_by_key(|entry| (entry.key, std::cmp::Reverse(entry.weight), entry.mv));

    let mut file = io::BufWriter::new(fs::File::create(path)?);
    for entry in merged {
        file.write_all(&entry.to_bytes())?;
    }
    file.flush()
}

fn unpack(bits: u16) -> Option<(Square, Square, Option<Role>)> {
    let promotion = match (bits >> 12) & 7 {
        0 => None,
        1 => Some(Role::Knight),
        2 => Some(Role::Bishop),
        3 => Some(Role::Rook),
        4 => Some(Role::Queen),
        _ => return None,
    };
    let square = |file: u16, rank: u16| {
        Some(Square::new(
            File::from_index(file as usize)?,
            Rank::from_index(rank as usize)?,
        ))
    };
    let to = square(bits & 7, (bits >> 3) & 7)?;
    let from = square((bits >> 6) & 7, (bits >> 9) & 7)?;
    Some((from, to, promotion))
}
