//! Invariants over randomly played legal games.

use esca::{CastlingOutput, Game, MAX_MOVES, Outcome, Position, Variant, chess960, classic};
use proptest::prelude::*;

/// Everything that must hold of any position esca can reach.
fn check_position(variant: &dyn Variant, position: &Position) -> Result<(), TestCaseError> {
    let fen = position.fen();
    let parsed = Position::from_fen(&fen).expect("esca reads back the FEN it writes");
    prop_assert_eq!(&parsed, position);
    prop_assert_eq!(parsed.key(), position.key());
    prop_assert_eq!(&position.mirrored().mirrored(), position);
    prop_assert_eq!(
        Position::from_fen(&position.epd())
            .expect("esca reads back the EPD it writes")
            .key(),
        position.key()
    );

    let legal = {
        let mut list = esca::MoveList::new();
        variant.legal_moves(position, &mut list);
        list
    };
    prop_assert!(legal.len() <= MAX_MOVES);

    for &mv in legal.as_slice() {
        prop_assert!(variant.is_legal(position, mv), "{}", fen);

        let uci = variant.move_to_uci(position, mv, CastlingOutput::KingToRook);
        prop_assert_eq!(variant.move_from_uci(position, &uci), Ok(mv), "{}", fen);
        let uci = variant.move_to_uci(position, mv, CastlingOutput::KingTwoSquares);
        prop_assert_eq!(variant.move_from_uci(position, &uci), Ok(mv), "{}", fen);

        let san = variant.move_to_san(position, mv);
        prop_assert_eq!(variant.move_from_san(position, &san), Ok(mv), "{}", fen);

        // Playing a legal move yields a position of the same standing.
        let after = variant.play(position, mv);
        prop_assert_eq!(after.side_to_move(), !position.side_to_move());
    }

    match variant.outcome(position) {
        Some(Outcome::Checkmate { .. }) | Some(Outcome::Stalemate) => {
            prop_assert!(legal.is_empty(), "{}", fen)
        }
        _ => prop_assert!(!legal.is_empty(), "{}", fen),
    }
    Ok(())
}

/// Plays `choices` as indices into the legal moves, checking every position
/// on the way.
fn play_out(mut game: Game, choices: &[u8]) -> Result<(), TestCaseError> {
    check_position(game.variant(), game.position())?;
    for &choice in choices {
        if game.outcome().is_some() {
            break;
        }
        let legal = game.legal_moves();
        let mv = legal[choice as usize % legal.len()];
        game.play(mv).expect("a generated move is legal");
        check_position(game.variant(), game.position())?;

        // The history holds every position, in order.
        prop_assert_eq!(game.positions().count(), game.ply() as usize + 1);
        prop_assert!(game.repetitions() >= 1);
    }

    // Undoing the whole game returns the start position.
    let start = game.start_position().clone();
    while game.undo().is_some() {}
    prop_assert_eq!(game.position(), &start);
    prop_assert_eq!(game.ply(), 0);
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn a_random_classic_game_keeps_its_invariants(choices in prop::collection::vec(any::<u8>(), 0..40)) {
        play_out(Game::new(classic()), &choices)?;
    }

    #[test]
    fn a_random_chess960_game_keeps_its_invariants(
        seed in 0u64..960,
        choices in prop::collection::vec(any::<u8>(), 0..40),
    ) {
        play_out(Game::with_seed(chess960(), seed), &choices)?;
    }
}
