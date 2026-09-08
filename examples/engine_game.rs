//! Play ten plies of a UCI engine against itself at 100 ms a move, then report
//! the ending, the facts as JSON, and the facts as a tensor batch. Without a
//! usable engine it says so and stops.
//!
//! ```text
//! cargo run --example engine_game --features uci,serde,tensors -- /usr/local/bin/stockfish
//! ```

use std::time::Duration;

use esca::facts::tensors::Batch;
use esca::uci::{Launch, Limits};
use esca::{Game, classic};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(path) = std::env::args().nth(1) else {
        println!("no engine given: pass the path to a UCI engine, e.g. /usr/local/bin/stockfish");
        return Ok(());
    };
    // Missing, or there and not runnable, reads the same from here.
    let mut engine = match Launch::new(&path).spawn() {
        Ok(engine) => engine,
        Err(error) => {
            println!("{path} does not run as a UCI engine: {error}");
            return Ok(());
        }
    };
    engine.handshake()?;
    engine.new_game()?;

    let mut game = Game::new(classic());
    let limits = Limits::movetime(Duration::from_millis(100));
    let mut san = Vec::new();

    for _ in 0..10 {
        let Some(mv) = engine.play(&game, &limits)?.best else {
            break;
        };
        san.push(game.move_to_san(mv));
        game.play(mv)?;
    }
    engine.quit()?;

    println!("moves: {}", san.join(" "));
    println!("fen:   {}", game.position().fen());
    println!("\nending: {}", game.ending().describe());

    let facts = game.facts();
    println!(
        "\nfacts JSON: {} bytes",
        serde_json::to_string(&facts)?.len()
    );
    println!(
        "pawns.passed = {}",
        serde_json::to_string(&facts.pawns.passed)?
    );
    let material = facts.group("material").expect("a material group");
    println!("material fields: {}", material.fields().len());

    let batch = Batch::from_games(std::slice::from_ref(&game));
    let entry = batch
        .layout()
        .entry("pawns.passed")
        .expect("a passed-pawn array");
    println!(
        "\ntensor {}: {:?} {:?} ({})",
        entry.name,
        entry.dtype,
        batch.shape_of(entry),
        entry.order
    );
    println!("safetensors: {} bytes", batch.to_safetensors().len());
    Ok(())
}
