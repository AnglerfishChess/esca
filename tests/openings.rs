//! The ECO catalogue, case by case.
//!
//! Every code and name is the one the bundled data set gives for the line
//! above it.

#![cfg(feature = "openings")]

use esca::{Game, Position, chess960, classic, openings};
use rstest::rstest;

/// The Queen's Gambit Declined as the data set writes it.
const QGD: &str = "d4 d5 c4 e6 Nc3 Nf6 Bg5 Be7 Nf3";

/// The same nine moves in another order: the knights and the bishop come out
/// when it suits them, and the ninth move stands on the same board.
const QGD_TRANSPOSED: &str = "Nf3 d5 d4 Nf6 c4 e6 Nc3 Be7 Bg5";

/// A Najdorf, then a move nobody has named.
const NAJDORF_THEN_OFF_BOOK: &str = "e4 c5 Nf3 d6 d4 cxd4 Nxd4 Nf6 Nc3 a6 h4";

/// The classic array is Chess960 arrangement 518.
const CLASSIC_ARRANGEMENT: u64 = 518;

/// The classic game the space-separated SAN `moves` reach.
fn played(moves: &str) -> Game {
    let mut game = Game::new(classic());
    for text in moves.split_whitespace() {
        game.play_san(text)
            .unwrap_or_else(|_| panic!("{text} is legal"));
    }
    game
}

#[rstest]
#[case::kings_pawn("e4", "B00", "King's Pawn Game")]
#[case::queens_pawn("d4", "A40", "Queen's Pawn Game")]
#[case::sicilian("e4 c5", "B20", "Sicilian Defense")]
#[case::ruy_lopez("e4 e5 Nf3 Nc6 Bb5", "C60", "Ruy Lopez")]
#[case::italian("e4 e5 Nf3 Nc6 Bc4", "C50", "Italian Game")]
#[case::kings_indian("d4 Nf6 c4 g6 Nc3", "E61", "King's Indian Defense")]
#[case::queens_gambit_declined(QGD, "D53", "Queen's Gambit Declined")]
#[case::najdorf(
    "e4 c5 Nf3 d6 d4 cxd4 Nxd4 Nf6 Nc3 a6",
    "B90",
    "Sicilian Defense: Najdorf Variation"
)]
fn a_named_position_answers_with_its_code_and_its_name(
    #[case] moves: &str,
    #[case] eco: &str,
    #[case] name: &str,
) {
    let opening = openings::lookup(played(moves).position()).expect("the line is named");
    assert_eq!(opening.eco, eco);
    assert_eq!(opening.name, name);
}

#[test]
fn a_line_that_transposes_into_a_named_position_is_named() {
    let direct = played(QGD);
    let transposed = played(QGD_TRANSPOSED);
    assert_ne!(direct.moves(), transposed.moves());
    assert_eq!(
        openings::lookup(direct.position()),
        openings::lookup(transposed.position())
    );
    assert_eq!(
        openings::lookup(transposed.position()).map(|opening| opening.eco),
        Some("D53")
    );
}

#[test]
fn the_starting_array_has_no_name() {
    assert_eq!(openings::lookup(Game::new(classic()).position()), None);
}

#[test]
fn a_position_nobody_has_named_has_no_name() {
    let position = Position::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").expect("a legal position");
    assert_eq!(openings::lookup(&position), None);
}

#[test]
fn a_game_keeps_the_deepest_name_it_reached() {
    let game = played(NAJDORF_THEN_OFF_BOOK);
    assert_eq!(openings::lookup(game.position()), None);
    let opening = game.opening().expect("the game went through the Najdorf");
    assert_eq!(opening.eco, "B90");
    assert_eq!(opening.name, "Sicilian Defense: Najdorf Variation");
}

#[test]
fn a_game_that_has_reached_no_named_position_has_no_opening() {
    assert_eq!(Game::new(classic()).opening(), None);
}

#[test]
fn the_catalogue_is_keyed_by_position_and_not_by_the_rules_in_force() {
    let mut game = Game::with_seed(chess960(), CLASSIC_ARRANGEMENT);
    assert_eq!(game.position().fen(), Game::new(classic()).position().fen());
    game.play_san("e4").expect("e4 is legal under either rules");
    assert_eq!(
        game.opening().map(|opening| opening.name),
        Some("King's Pawn Game")
    );
}

#[test]
fn every_row_of_the_data_set_names_a_position_of_its_own() {
    // The bundled volumes hold 3,810 rows between them.
    assert_eq!(openings::count(), 3810);
}

#[test]
fn an_opening_reads_as_its_code_and_then_its_name() {
    let opening = openings::lookup(played("e4 e5 Nf3 Nc6 Bb5").position()).expect("a named line");
    assert_eq!(opening.to_string(), "C60 Ruy Lopez");
}
