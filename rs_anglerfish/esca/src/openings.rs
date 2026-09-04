//! The bundled ECO catalogue: the code and name of a named position.
//!
//! ```
//! use esca::{Game, classic, openings};
//!
//! let mut game = Game::new(classic());
//! for san in ["e4", "e5", "Nf3", "Nc6", "Bb5"] {
//!     game.play_san(san).unwrap();
//! }
//! assert_eq!(openings::lookup(game.position()).unwrap().name, "Ruy Lopez");
//! assert_eq!(game.opening().unwrap().eco, "C60");
//! ```

use core::fmt;
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::game::Game;
use crate::position::Position;
use crate::variant::classic;

/// The data set, one file per ECO volume.
const VOLUMES: [&str; 5] = [
    include_str!("../data/openings/a.tsv"),
    include_str!("../data/openings/b.tsv"),
    include_str!("../data/openings/c.tsv"),
    include_str!("../data/openings/d.tsv"),
    include_str!("../data/openings/e.tsv"),
];

/// An ECO code and the name that goes with it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Opening {
    /// The ECO classification: a volume letter A to E and two digits.
    pub eco: &'static str,
    /// The name, in English, as `Family: Variation, Subvariation`.
    pub name: &'static str,
}

impl fmt::Display for Opening {
    /// The code and the name, one space apart.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.eco, self.name)
    }
}

/// The opening `position` is named after, if it has a name.
///
/// The catalogue is keyed by position, so a line that transposes into a named
/// position is named. It is classic chess, and holds no name for the starting
/// array.
pub fn lookup(position: &Position) -> Option<Opening> {
    catalogue().get(&position.polyglot_key()).copied()
}

/// How many named positions the catalogue holds.
pub fn count() -> usize {
    catalogue().len()
}

impl Game {
    /// The opening of the deepest named position this game has reached.
    #[cfg_attr(docsrs, doc(cfg(feature = "openings")))]
    pub fn opening(&self) -> Option<Opening> {
        self.positions().filter_map(lookup).last()
    }
}

/// The catalogue, built from the bundled text on first use.
fn catalogue() -> &'static HashMap<u64, Opening> {
    static CATALOGUE: OnceLock<HashMap<u64, Opening>> = OnceLock::new();
    CATALOGUE.get_or_init(|| {
        let mut catalogue = HashMap::new();
        for volume in VOLUMES {
            for line in volume.lines().skip(1) {
                let mut fields = line.split('\t');
                let (Some(eco), Some(name), Some(moves)) =
                    (fields.next(), fields.next(), fields.next())
                else {
                    continue;
                };
                if let Some(key) = key_after(moves) {
                    // The data set names each position once; where it does
                    // not, the first line wins.
                    catalogue.entry(key).or_insert(Opening { eco, name });
                }
            }
        }
        catalogue
    })
}

/// The Polyglot key of the position `movetext` reaches, playing its SAN moves
/// from the classic start; `None` when one of them does not read.
fn key_after(movetext: &str) -> Option<u64> {
    let mut game = Game::new(classic());
    for token in movetext.split_whitespace() {
        // Move numbers and their `...` continuations start with a digit.
        if token.starts_with(|c: char| c.is_ascii_digit()) {
            continue;
        }
        game.play_san(token).ok()?;
    }
    Some(game.position().polyglot_key())
}
