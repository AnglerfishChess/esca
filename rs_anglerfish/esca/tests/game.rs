//! Repetition, the move-count rules, outcomes and undo.

use esca::{
    CLASSIC, Colour, DrawClaim, Game, IllegalMove, Outcome, Position, Variant, chess960, classic,
};

/// Knights out and back: four plies return to the same position.
const SHUFFLE: [&str; 4] = ["Nf3", "Nf6", "Ng1", "Ng8"];

fn shuffle(game: &mut Game, plies: usize) {
    for text in SHUFFLE.iter().cycle().take(plies) {
        game.play_san(text).expect("the shuffle is legal");
    }
}

#[test]
fn threefold_repetition_is_claimable() {
    let mut game = Game::new(classic());
    assert_eq!(game.repetitions(), 1);
    shuffle(&mut game, 4);
    assert_eq!(game.repetitions(), 2);
    assert!(game.claims().is_empty());
    shuffle(&mut game, 4);
    assert_eq!(game.repetitions(), 3);
    assert_eq!(game.claims(), [DrawClaim::ThreefoldRepetition]);
    // Claimable, so the game is still playable.
    assert_eq!(game.outcome(), None);
}

#[test]
fn fivefold_repetition_ends_the_game() {
    let mut game = Game::new(classic());
    shuffle(&mut game, 16);
    assert_eq!(game.repetitions(), 5);
    assert_eq!(game.outcome(), Some(Outcome::FivefoldRepetition));
}

/// A position repeated with different castling rights is a different one.
#[test]
fn repetition_counts_castling_rights() {
    let mut game = Game::from_fen(classic(), "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1")
        .expect("the FEN is well formed");
    for text in ["a1b1", "a8b8", "b1a1", "b8a8"] {
        game.play_uci(text).expect("the rook moves are legal");
    }
    // Same placement, but neither side may castle long any more.
    assert_eq!(game.position().castling_rights().to_fen_field(), "Kk");
    assert_eq!(game.repetitions(), 1);
}

/// The en-passant square counts only while a pawn could actually take.
#[test]
fn repetition_ignores_an_unplayable_en_passant_square() {
    let with_ep =
        Position::from_fen("4k3/8/8/8/3pP3/8/8/4K3 b - e3 0 1").expect("the FEN is well formed");
    let without_ep =
        Position::from_fen("4k3/8/8/8/3pP3/8/8/4K3 b - - 0 1").expect("the FEN is well formed");
    assert_ne!(with_ep, without_ep);

    let playable = Game::from_position(classic(), with_ep).expect("the position is playable");
    let plain = Game::from_position(classic(), without_ep).expect("the position is playable");
    // d4 takes e3 en passant, so the square is part of the position.
    assert_eq!(playable.repetitions(), 1);
    assert_eq!(plain.repetitions(), 1);
    assert_ne!(playable.position().key(), plain.position().key());
}

#[test]
fn fifty_moves_is_claimable_and_seventy_five_is_not() {
    let mut game = Game::from_fen(classic(), "4k3/8/8/8/8/8/1R6/4K3 w - - 99 60")
        .expect("the FEN is well formed");
    game.play_uci("b2b3").expect("the rook move is legal");
    assert_eq!(game.position().halfmove_clock(), 100);
    assert_eq!(game.claims(), [DrawClaim::FiftyMoves]);
    assert_eq!(game.outcome(), None);

    let mut game = Game::from_fen(classic(), "4k3/8/8/8/8/8/1R6/4K3 w - - 149 100")
        .expect("the FEN is well formed");
    assert_eq!(game.outcome(), None);
    game.play_uci("b2b3").expect("the rook move is legal");
    assert_eq!(game.outcome(), Some(Outcome::SeventyFiveMoves));
}

#[test]
fn checkmate_and_stalemate() {
    let mut game = Game::new(classic());
    for text in ["f3", "e5", "g4", "Qh4#"] {
        game.play_san(text).expect("the fool's mate is legal");
    }
    assert_eq!(
        game.outcome(),
        Some(Outcome::Checkmate {
            winner: Colour::Black
        })
    );
    assert!(game.legal_moves().is_empty());

    let stalemate = Game::from_fen(classic(), "7k/5Q2/6K1/8/8/8/8/8 b - - 0 1")
        .expect("the FEN is well formed");
    assert_eq!(stalemate.outcome(), Some(Outcome::Stalemate));
}

#[test]
fn insufficient_material() {
    for fen in [
        "4k3/8/8/8/8/8/8/4K3 w - - 0 1",
        "4k3/8/8/8/8/8/8/3BK3 w - - 0 1",
        "4k3/8/8/8/8/8/8/3NK3 w - - 0 1",
        // Both bishops on light squares: no mate is possible.
        "4k3/8/4b3/8/8/8/8/3BK3 w - - 0 1",
    ] {
        let game = Game::from_fen(classic(), fen).expect("the FEN is well formed");
        assert_eq!(game.outcome(), Some(Outcome::InsufficientMaterial), "{fen}");
    }
    for fen in [
        // Opposite square colours: a helpmate exists.
        "4k3/8/3b4/8/8/8/8/3BK3 w - - 0 1",
        "4k3/8/8/8/8/8/8/3RK3 w - - 0 1",
        "4k3/8/8/8/8/8/8/1N1NK3 w - - 0 1",
    ] {
        let game = Game::from_fen(classic(), fen).expect("the FEN is well formed");
        assert_eq!(game.outcome(), None, "{fen}");
    }
}

#[test]
fn undo_restores_the_previous_position() {
    let mut game = Game::new(classic());
    let start = game.position().clone();
    game.play_san("e4").expect("the move is legal");
    let mv = game.moves()[0];
    assert_eq!(game.ply(), 1);
    assert_eq!(game.undo(), Some(mv));
    assert_eq!(game.ply(), 0);
    assert_eq!(game.position(), &start);
    assert_eq!(game.undo(), None);

    // Claims follow the history back.
    let mut game = Game::new(classic());
    shuffle(&mut game, 8);
    assert_eq!(game.claims(), [DrawClaim::ThreefoldRepetition]);
    game.undo();
    assert!(game.claims().is_empty());
}

#[test]
fn a_game_refuses_a_move_that_is_not_legal() {
    let mut game = Game::new(classic());
    let mut opening = Game::new(classic());
    opening.play_san("e4").expect("the move is legal");
    let black_move = *opening
        .legal_moves()
        .as_slice()
        .first()
        .expect("black has moves");
    assert_eq!(game.play(black_move), Err(IllegalMove));
}

#[test]
fn positions_run_from_the_start_to_now() {
    let mut game = Game::new(classic());
    shuffle(&mut game, 4);
    let positions: Vec<&Position> = game.positions().collect();
    assert_eq!(positions.len(), 5);
    assert_eq!(positions[0], game.start_position());
    assert_eq!(positions[4], game.position());
    assert_eq!(positions[0].key(), positions[4].key());
}

#[test]
fn a_chess960_game_starts_from_its_arrangement() {
    let game = Game::with_seed(chess960(), 1000);
    assert_eq!(game.variant().name(), "chess960");
    // The seed is taken modulo 960.
    assert_eq!(game.start_position(), &game.variant().start_position(40));
}

#[test]
fn classic_refuses_a_shuffled_start_position() {
    let shuffled =
        Position::from_fen("1rqbkrbn/1ppppp1p/1n6/p1N3p1/8/2P4P/PP1PPPP1/1RQBKRBN w FBfb - 0 9")
            .expect("the FEN is well formed");
    assert!(CLASSIC.validate(&shuffled).is_err());
    assert!(Game::from_position(classic(), shuffled.clone()).is_err());
    assert!(Game::from_position(chess960(), shuffled).is_ok());
}
