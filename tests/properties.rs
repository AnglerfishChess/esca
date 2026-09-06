//! Invariants over randomly played legal games.

use esca::{
    CastlingOutput, Game, MAX_MOVES, MoveFacts, MoveList, Outcome, Position, Schema, Scratch, Side,
    Variant, chess960, classic,
};
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

/// Everything the v1 facts of any position must satisfy.
fn check_facts(variant: &dyn Variant, position: &Position) -> Result<(), TestCaseError> {
    let fen = position.fen();
    let schema = Schema::v1();
    let facts = position.facts(variant);
    let mut scratch = Scratch::new();

    // Deterministic, and the buffered path agrees with the allocating one.
    prop_assert_eq!(&facts, &position.facts(variant), "{}", fen);
    prop_assert_eq!(&facts, &position.facts_in(variant, &mut scratch), "{}", fen);

    let values = facts.encode(schema, schema.all());
    prop_assert_eq!(values.len(), schema.width_of(schema.all()));
    for (index, value) in values.iter().enumerate() {
        prop_assert!(
            value.is_finite() && (-1.0..=1.0).contains(value),
            "value {} is {} in {}",
            index,
            value,
            fen
        );
    }

    // Writing into a caller's buffer writes exactly the declared width.
    let mut buffer = vec![f32::NAN; schema.width() + 8];
    let written = facts.encode_into(schema, schema.all(), &mut buffer);
    prop_assert_eq!(written, schema.width());
    prop_assert_eq!(&buffer[..written], &values[..]);
    prop_assert!(buffer[written..].iter().all(|v| v.is_nan()));

    // Colour-and-rank mirroring exchanges the sides and nothing else.
    let mirrored = position
        .mirrored()
        .facts(variant)
        .encode(schema, schema.all());
    prop_assert_eq!(&mirrored, &values, "mirror of {}", fen);

    for side in Side::ALL {
        let i = side.index();
        let pawns = facts.pawns.pawns[i];
        for derived in [
            facts.pawns.passed[i],
            facts.pawns.candidates[i],
            facts.pawns.doubled[i],
            facts.pawns.isolated[i],
            facts.pawns.backward[i],
            facts.pawns.defended[i],
        ] {
            prop_assert!(derived.is_subset(pawns), "{}", fen);
        }
        let units = facts.attacks.units(side);
        for derived in [
            facts.attacks.hanging[i],
            facts.attacks.en_prise[i],
            facts.attacks.pinned[i],
            facts.attacks.defended[i],
        ] {
            prop_assert!(derived.is_subset(units), "{}", fen);
        }
        prop_assert!(facts.attacks.hanging[i].is_subset(facts.attacks.en_prise[i]));
        prop_assert!(facts.attacks.by_pawns[i].is_subset(facts.attacks.by[i]));
        prop_assert!(facts.pieces.outposts[i].is_subset(facts.attacks.by_pawns[i]));
        prop_assert!(facts.king.ring[i].is_subset(!facts.king.square[i].to_set()));
    }

    // Every annotated move is one of the legal moves, once.
    let mut legal = MoveList::new();
    variant.legal_moves(position, &mut legal);
    prop_assert_eq!(facts.moves.len(), legal.len(), "{}", fen);
    prop_assert_eq!(
        facts.tactics[0].legal_move_count as usize,
        legal.len(),
        "{}",
        fen
    );
    for annotated in facts.moves.iter() {
        prop_assert!(legal.contains(&annotated.mv), "{}", fen);
        let mut row = vec![0.0f32; MoveFacts::WIDTH];
        annotated.facts.encode_into(&mut row);
        prop_assert!(
            row.iter()
                .all(|v| v.is_finite() && (-1.0..=1.0).contains(v))
        );
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    #[test]
    fn the_facts_of_a_random_classic_game_keep_their_invariants(
        choices in prop::collection::vec(any::<u8>(), 0..24),
    ) {
        let mut game = Game::new(classic());
        check_facts(game.variant(), game.position())?;
        for &choice in &choices {
            if game.outcome().is_some() {
                break;
            }
            let legal = game.legal_moves();
            let mv = legal[choice as usize % legal.len()];
            game.play(mv).expect("a generated move is legal");
            check_facts(game.variant(), game.position())?;
        }
    }

    #[test]
    fn the_facts_of_a_random_chess960_game_keep_their_invariants(
        seed in 0u64..960,
        choices in prop::collection::vec(any::<u8>(), 0..12),
    ) {
        let mut game = Game::with_seed(chess960(), seed);
        check_facts(game.variant(), game.position())?;
        for &choice in &choices {
            if game.outcome().is_some() {
                break;
            }
            let legal = game.legal_moves();
            let mv = legal[choice as usize % legal.len()];
            game.play(mv).expect("a generated move is legal");
            check_facts(game.variant(), game.position())?;
        }
    }
}
