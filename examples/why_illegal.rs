//! List the legal moves of a FEN with what each one does, then explain why a
//! move that is not among them is illegal — an en-passant capture by its SAN,
//! or `O-O`/`O-O-O`. Both arguments have defaults.
//!
//! ```text
//! cargo run --example why_illegal -- "8/8/8/K2pP2r/8/8/8/7k w - d6 0 1" exd6
//! ```

use esca::explain::EnPassant;
use esca::{CLASSIC, Position, Wing};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let fen = args
        .next()
        .unwrap_or_else(|| "8/8/8/K2pP2r/8/8/8/7k w - d6 0 1".into());
    let text = args.next().unwrap_or_else(|| "exd6".into());

    let position: Position = fen.parse()?;
    let moves = position.annotated_moves(&CLASSIC);

    println!(
        "{} legal moves for {}:",
        moves.len(),
        position.side_to_move()
    );
    for annotated in moves.iter() {
        let mv = annotated.mv;
        let facts = annotated.facts;
        let mut what = Vec::new();
        if facts.is_castling {
            what.push("castling".to_string());
        }
        if let Some(victim) = facts.victim {
            what.push(if facts.is_en_passant {
                "en passant".into()
            } else {
                format!("takes a {victim}")
            });
        }
        if let Some(role) = facts.promotion {
            what.push(format!("promotion to {role}"));
        }
        if facts.gives_check {
            what.push("check".into());
        }
        println!("  {:<8} {}", position.san(&CLASSIC, mv), what.join(", "));
    }

    println!("\nwhy {text} is illegal:");
    match text.as_str() {
        "O-O" | "O-O-O" => {
            let wing = if text == "O-O" {
                Wing::Short
            } else {
                Wing::Long
            };
            let c = position.castling(position.side_to_move(), wing);
            println!(
                "  right {} rook_present {} allowed {}",
                c.right, c.rook_present, c.allowed
            );
            println!("  king attacked by {:?}", c.king_in_check_by);
            for (square, by) in &c.path_attacked {
                println!("  crosses {square}, covered by {by:?}");
            }
            println!("  path blocked on {:?}", c.path_blocked);
            println!("  {}", c.describe());
        }
        _ => match position.en_passant_status() {
            EnPassant::None => println!("  no en-passant capture is on offer"),
            EnPassant::Available { target, captures } => {
                println!("  the target is {target}");
                for capture in &captures {
                    println!("  from {}: legal {}", capture.from, capture.legal);
                    println!("    {}", capture.describe());
                    if let Some(reason) = &capture.forbidden_by {
                        println!("    {reason:?}");
                        println!("    {}", reason.describe());
                    }
                }
            }
        },
    }
    Ok(())
}
