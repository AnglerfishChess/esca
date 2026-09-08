//! Read a PGN file and report, per game: opening, final FEN, how it ended,
//! and which side has passed pawns at the end.
//!
//! ```text
//! cargo run --example pgn_report --features pgn,openings -- examples/games.pgn
//! ```

use std::path::Path;

use esca::{Colour, pgn};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "examples/games.pgn".into());

    for (n, read) in pgn::read(Path::new(&path))?.enumerate() {
        let game = read?.mainline_game()?;
        let facts = game.facts();

        println!("game {}", n + 1);
        println!(
            "  opening: {}",
            game.opening().map_or("—".to_string(), |o| o.to_string())
        );
        println!("  final:   {}", game.position().fen());
        println!("  ending:  {}", ending(&game));
        for colour in Colour::ALL {
            let files: Vec<String> = facts
                .pawns
                .passed
                .of(colour)
                .files()
                .into_iter()
                .map(|f| f.to_string())
                .collect();
            println!(
                "  passed ({colour}): {}",
                if files.is_empty() {
                    "none".into()
                } else {
                    files.join(", ")
                }
            );
        }
    }
    Ok(())
}

/// How the game stands after its last move. Both enums say this themselves
/// now, so the ten-arm match of the first review is gone.
fn ending(game: &esca::Game) -> String {
    match game.outcome() {
        Some(outcome) => outcome.describe(),
        None if !game.claims().is_empty() => format!(
            "claimable draw: {}",
            game.claims()
                .iter()
                .map(|claim| claim.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        None => "unfinished".into(),
    }
}
