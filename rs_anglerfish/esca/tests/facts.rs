//! The v0 facts: golden vectors, and the values hand-picked positions must
//! carry.

use esca::{
    CHESS960, CLASSIC, Colour, Facts, Game, Position, Rank, Role, Schema, Scratch, Side, Square,
    Variant, chess960, classic, encode_fens, encode_positions,
};

/// The FEN lines of a corpus file, `#` comments and blanks dropped.
fn corpus(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

/// The rows of a fixture, one per position.
fn vectors(bytes: &[u8], width: usize) -> Vec<Vec<f32>> {
    assert_eq!(bytes.len() % (width * 4), 0, "a fixture is whole rows");
    bytes
        .chunks_exact(width * 4)
        .map(|row| {
            row.chunks_exact(4)
                .map(|value| f32::from_le_bytes(value.try_into().expect("four bytes")))
                .collect()
        })
        .collect()
}

/// The name of the feature the value at `index` belongs to.
fn feature_at(index: usize) -> String {
    let mut at = 0;
    for group in Schema::v0().groups() {
        if index < at + group.width {
            let inside = index - at;
            for feature in group.features {
                if inside < feature.offset + feature.width {
                    return format!(
                        "{}.{} [{}]",
                        group.name,
                        feature.name,
                        inside - feature.offset
                    );
                }
            }
        }
        at += group.width;
    }
    format!("value {index}")
}

fn check_golden(variant: &dyn Variant, fens: &str, expected: &[u8]) {
    let schema = Schema::v0();
    let fens = corpus(fens);
    let expected = vectors(expected, schema.width());
    assert_eq!(fens.len(), expected.len(), "corpus and fixture disagree");

    let positions: Vec<Position> = fens
        .iter()
        .map(|fen| Position::from_fen(fen).expect("a corpus FEN is legal"))
        .collect();
    let mut actual = vec![0.0f32; positions.len() * schema.width()];
    encode_positions(variant, &positions, schema, schema.all(), &mut actual);

    for (row, fen) in fens.iter().enumerate() {
        let row_values = &actual[row * schema.width()..(row + 1) * schema.width()];
        for (index, (&got, &want)) in row_values.iter().zip(&expected[row]).enumerate() {
            assert_eq!(
                got.to_bits(),
                want.to_bits(),
                "row {row} {fen}: {} is {got}, not {want}",
                feature_at(index)
            );
        }
    }
}

#[test]
fn the_classic_golden_vectors_still_hold() {
    check_golden(
        &CLASSIC,
        include_str!("data/fens_classic.txt"),
        include_bytes!("data/vectors_classic.bin"),
    );
}

#[test]
fn the_chess960_golden_vectors_still_hold() {
    check_golden(
        &CHESS960,
        include_str!("data/fens_chess960.txt"),
        include_bytes!("data/vectors_chess960.bin"),
    );
}

#[test]
fn chess960_zeroes_the_features_it_does_not_define() {
    let schema = Schema::v0();
    let fen = corpus(include_str!("data/fens_chess960.txt"))[0];
    let position = Position::from_fen(fen).expect("a corpus FEN is legal");
    let values = position.facts(&CHESS960).encode(schema, schema.all());

    let mut at = 0;
    for group in schema.groups() {
        for feature in group.features {
            if !feature.defined_for("chess960") {
                let start = at + feature.offset;
                assert!(
                    values[start..start + feature.width]
                        .iter()
                        .all(|v| *v == 0.0),
                    "{}.{} is not zeroed",
                    group.name,
                    feature.name
                );
            }
        }
        at += group.width;
    }
}

fn facts_of(fen: &str) -> Facts {
    Position::from_fen(fen)
        .expect("a test FEN is legal")
        .facts(&CLASSIC)
}

#[test]
fn the_start_position_is_symmetric() {
    let facts = facts_of("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    assert_eq!(facts.material.value[0], facts.material.value[1]);
    assert_eq!(facts.material.phase, 1.0);
    assert_eq!(facts.pawns.islands, [1, 1]);
    assert_eq!(facts.pawns.rams, 0);
    assert!(facts.pawns.open_files.is_empty());
    assert_eq!(facts.pieces.minors_undeveloped, [4, 4]);
    assert_eq!(facts.pieces.queen_developed, [false, false]);
    assert!(facts.king.on_home_square[0] && facts.king.on_home_square[1]);
    assert_eq!(facts.king.distance, 7);
    assert_eq!(facts.tactics[0].legal_move_count, 20);
    assert_eq!(facts.tactics[1].legal_move_count, 20);
    assert!(facts.tactics[0].available && facts.tactics[1].available);
    assert_eq!(facts.moves.len(), 20);
}

#[test]
fn the_side_to_move_plays_us() {
    let white = facts_of("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    assert_eq!(white.side(Colour::White), Side::Us);
    assert_eq!(white.side(Colour::Black), Side::Them);

    let black = facts_of("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1");
    assert_eq!(black.side(Colour::Black), Side::Us);
    assert_eq!(black.side(Colour::White), Side::Them);
}

#[test]
fn a_side_indexes_a_named_colours_facts_whoever_is_to_move() {
    for fen in [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1",
    ] {
        let facts = facts_of(fen);
        let black = facts.side(Colour::Black).index();
        assert!(
            facts.pawns.pawns[black]
                .into_iter()
                .all(|square| square.rank() == Rank::Seventh),
            "{fen}"
        );
    }
}

#[test]
fn a_check_is_seen_and_the_them_block_is_not() {
    let facts = facts_of("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3");
    assert!(facts.state.in_check);
    assert!(!facts.state.double_check);
    assert!(!facts.tactics[1].available, "no null move while in check");
    assert_eq!(facts.tactics[1], Default::default());
}

#[test]
fn a_promotion_is_available_on_its_own_file() {
    let facts = facts_of("8/P6k/8/8/8/8/6K1/8 w - - 0 1");
    let tactics = &facts.tactics[0];
    assert!(tactics.promotion_available());
    assert!(tactics.promotion_roles.iter().all(|got| *got));
    assert_eq!(tactics.promotion_files.len(), 1);
    assert!(tactics.promotion_files.contains(esca::File::A));
    assert!(facts.pawns.passed[0].contains(Square::A7));
    assert!(facts.pawns.passer_unstoppable[0]);
    assert_eq!(facts.pawns.passer_lead_rank[0], Some(7));
}

#[test]
fn an_en_passant_capture_is_legal_when_it_is() {
    let facts = facts_of("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3");
    assert_eq!(facts.state.en_passant, Some(esca::File::F));
    assert!(facts.state.ep_capture_legal);
    assert!(facts.moves.iter().any(|m| m.facts.is_en_passant));

    let quiet = facts_of("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
    assert_eq!(quiet.state.en_passant, Some(esca::File::E));
    assert!(!quiet.state.ep_capture_legal);
}

#[test]
fn opposite_coloured_bishops_are_recognised() {
    let facts = facts_of("8/2k5/3b4/8/8/5B2/2K5/8 w - - 0 1");
    assert!(facts.pieces.opposite_coloured_bishops);
    let same = facts_of("8/2k5/3b4/8/8/4B3/2K5/8 b - - 0 1");
    assert!(!same.pieces.opposite_coloured_bishops);
}

#[test]
fn a_back_rank_mate_is_seen_one_move_ahead() {
    let facts = facts_of("6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1");
    assert!(facts.tactics[0].mate_in_1);
    assert!(facts.king.back_rank_risk[1], "the black king is boxed in");
    assert!(!facts.king.back_rank_risk[0]);
}

#[test]
fn a_stalemate_is_seen_one_move_ahead() {
    let facts = facts_of("7k/8/5K2/8/8/8/8/6Q1 w - - 0 1");
    assert!(
        facts.tactics[0].stalemate_in_1,
        "Qg6 leaves Black with no move"
    );
    let mated = facts_of("7k/6Q1/5K2/8/8/8/8/8 b - - 0 1");
    assert_eq!(mated.tactics[0].legal_move_count, 0);
    assert!(mated.state.in_check);
}

#[test]
fn bare_kings_have_no_material() {
    let facts = facts_of("8/8/8/4k3/8/8/8/4K3 w - - 0 1");
    assert_eq!(facts.material.value, [0, 0]);
    assert_eq!(facts.material.phase, 0.0);
    assert!(facts.material.pawns_only);
    assert_eq!(facts.material.insufficient, [true, true]);
    assert_eq!(facts.pawns.islands, [0, 0]);
    assert_eq!(facts.attacks.hanging, [Default::default(); 2]);
}

#[test]
fn hanging_and_pinned_units_are_found() {
    // White's rook on e1 pins the black rook to the king on e8.
    let facts = facts_of("4k3/8/8/8/8/8/4r3/K3R3 w - - 0 1");
    assert!(facts.attacks.pinned[1].contains(Square::E2));
    assert!(facts.attacks.hanging[1].contains(Square::E2));
    assert!(facts.attacks.is_hanging(Square::E2));
    assert_eq!(
        facts.attacks.attackers_of(Square::E2, Side::Us),
        Square::E1.to_set()
    );
    assert!(facts.attacks.en_prise[1].contains(Square::E2));
}

#[test]
fn facts_in_matches_facts() {
    let mut scratch = Scratch::new();
    for fen in corpus(include_str!("data/fens_classic.txt")) {
        let position = Position::from_fen(fen).expect("a corpus FEN is legal");
        assert_eq!(
            position.facts(&CLASSIC),
            position.facts_in(&CLASSIC, &mut scratch),
            "{fen}"
        );
    }
}

#[test]
fn a_move_encodes_to_its_declared_width() {
    let facts = facts_of("r3k3/1P6/8/8/8/8/6K1/8 w q - 0 1");
    let mut row = vec![0.0f32; esca::MoveFacts::WIDTH];
    for annotated in facts.moves.iter() {
        annotated.facts.encode_into(&mut row);
        assert!(row.iter().all(|v| (0.0..=1.0).contains(v)));
        assert_eq!(row[0], f32::from(annotated.facts.victim.is_some()));
        // Exactly one mover role, and at most one victim and promotion role.
        assert_eq!(row[6..12].iter().sum::<f32>(), 1.0);
        assert!(row[1..6].iter().sum::<f32>() <= 1.0);
        assert!(row[12..16].iter().sum::<f32>() <= 1.0);
    }
    assert!(facts.moves.iter().any(|m| m.facts.promotion.is_some()));
    assert!(facts.moves.iter().any(|m| m.facts.mover == Role::King));
}

#[test]
fn a_game_supplies_the_repetition_facts_a_position_cannot() {
    let mut game = Game::new(classic());
    let facts = game.facts();
    assert!(facts.state.history_known);
    assert!(!facts.state.repetition_seen);
    assert!(!facts.state.repetition_available[0]);

    for uci in ["g1f3", "g8f6", "f3g1", "f6g8"] {
        game.play_uci(uci).expect("a legal move");
    }
    let facts = game.facts();
    assert!(facts.state.repetition_seen);
    assert!(facts.state.history_known);
    assert!(!game.position().facts(game.variant()).state.history_known);

    assert!(
        facts.state.repetition_available[Side::Us.index()],
        "Nf3 repeats"
    );
    assert_eq!(game.annotated_moves().len(), game.legal_moves().len());

    // A king triangulation, so that after a null move Black's opponent can
    // walk back into a position the game has already held.
    let mut game =
        Game::from_fen(classic(), "k7/8/8/8/8/8/8/K7 b - - 0 1").expect("a legal position");
    for uci in ["a8b8", "a1b1", "b8a8", "b1b2", "a8b8", "b2a1"] {
        game.play_uci(uci).expect("a legal move");
    }
    assert!(game.facts().state.repetition_available[Side::Them.index()]);
}

#[test]
fn a_selected_group_writes_only_its_own_width() {
    let schema = Schema::v0();
    let facts = facts_of("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let all = facts.encode(schema, schema.all());
    let mut at = 0;
    for (index, group) in schema.groups().iter().enumerate() {
        let one = facts.encode(schema, esca::GroupSet::only(index));
        assert_eq!(one.len(), group.width);
        assert_eq!(one, all[at..at + group.width], "group {}", group.name);
        at += group.width;
    }
    assert_eq!(at, all.len());
}

#[test]
fn a_bad_row_names_itself() {
    let schema = Schema::v0();
    let fens = ["8/8/8/4k3/8/8/8/4K3 w - - 0 1", "not a fen"];
    let mut out = vec![0.0f32; fens.len() * schema.width()];
    let error = encode_fens(&CLASSIC, &fens, schema, schema.all(), &mut out)
        .expect_err("the second row is not a FEN");
    assert_eq!(error.row, 1);
}

#[test]
fn chess960_start_positions_are_facts_too() {
    for seed in 0..8u64 {
        let game = Game::with_seed(chess960(), seed);
        let facts = game.facts();
        assert_eq!(facts.variant(), "chess960");
        assert_eq!(facts.material.phase, 1.0);
        assert_eq!(facts.pawns.islands, [1, 1]);
        assert!(facts.tactics[0].legal_move_count > 0);
    }
}
