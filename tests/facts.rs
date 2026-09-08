//! The facts as a whole: what hand-picked positions must carry, and the
//! catalogue that names every one of them.

mod common;

use common::{files, position_of};
use esca::{
    ByColour, CHESS960, CLASSIC, Colour, Facts, Game, Position, Rank, Role, Scratch, Square,
    Variant, chess960, classic,
};
use rstest::rstest;

/// The FEN lines of a corpus file, `#` comments and blanks dropped.
fn corpus(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

fn facts_of(fen: &str) -> Facts {
    Position::from_fen(fen)
        .expect("a test FEN is legal")
        .facts(&CLASSIC)
}

#[test]
fn the_start_position_is_symmetric() {
    let facts = facts_of("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    assert_eq!(facts.material.value.white, facts.material.value.black);
    assert_eq!(facts.material.phase_points, 24);
    assert_eq!(facts.pawns.islands, ByColour::new(1, 1));
    assert_eq!(facts.pawns.rams, 0);
    assert!(facts.pawns.open_files.is_empty());
    assert_eq!(facts.pieces.minors_undeveloped, ByColour::new(4, 4));
    assert_eq!(facts.pieces.queen_developed, ByColour::new(false, false));
    assert_eq!(facts.king.on_home_square, ByColour::new(true, true));
    assert_eq!(facts.king.distance, 7);
    assert_eq!(facts.tactics.legal_move_count, ByColour::new(20, 20));
    assert_eq!(facts.tactics.available, ByColour::new(true, true));
}

#[test]
fn a_fact_names_the_colour_it_is_about_whoever_is_to_move() {
    for fen in [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1",
    ] {
        let facts = facts_of(fen);
        assert!(
            facts
                .pawns
                .pawns
                .black
                .into_iter()
                .all(|square| square.rank() == Rank::Seventh),
            "{fen}"
        );
        assert!(
            facts
                .pawns
                .pawns
                .of(Colour::White)
                .into_iter()
                .all(|square| square.rank() == Rank::Second),
            "{fen}"
        );
    }
}

#[test]
fn the_side_to_move_is_named() {
    let white = facts_of("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    assert_eq!(white.side_to_move(), Colour::White);
    let black = facts_of("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1");
    assert_eq!(black.side_to_move(), Colour::Black);
}

#[test]
fn a_check_leaves_the_other_sides_tactics_unread() {
    let facts = facts_of("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3");
    assert!(facts.state.in_check);
    assert!(!facts.state.double_check);
    assert!(
        !facts.tactics.available.black,
        "no null move while in check"
    );
    assert_eq!(facts.tactics.legal_move_count.black, 0);
    assert_eq!(facts.tactics.check_count.black, 0);
}

#[test]
fn a_promotion_is_available_on_its_own_file() {
    let facts = facts_of("8/P6k/8/8/8/8/6K1/8 w - - 0 1");
    let tactics = &facts.tactics;
    assert!(tactics.promotion_available().white);
    assert!(tactics.promotes_to_queen.white);
    assert!(tactics.promotes_to_rook.white);
    assert!(tactics.promotes_to_bishop.white);
    assert!(tactics.promotes_to_knight.white);
    assert_eq!(tactics.promotion_files.white.len(), 1);
    assert!(tactics.promotion_files.white.contains(esca::File::A));
    assert!(facts.pawns.passed.white.contains(Square::A7));
    assert!(facts.pawns.passer_unstoppable.white);
    assert_eq!(facts.pawns.passer_lead_rank.white, Some(7));
}

#[test]
fn an_en_passant_capture_is_legal_when_it_is() {
    let facts = facts_of("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3");
    assert_eq!(facts.state.en_passant, Some(esca::File::F));
    assert!(facts.state.en_passant_capture_legal);
    let annotated = position_of("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3")
        .annotated_moves(&CLASSIC);
    assert!(annotated.iter().any(|m| m.facts.is_en_passant));

    let quiet = facts_of("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
    assert_eq!(quiet.state.en_passant, Some(esca::File::E));
    assert!(!quiet.state.en_passant_capture_legal);
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
    assert!(facts.tactics.mate_in_1.white);
    assert!(
        facts.king.back_rank_risk.black,
        "the black king is boxed in"
    );
    assert!(!facts.king.back_rank_risk.white);
}

#[test]
fn a_stalemate_is_seen_one_move_ahead() {
    let facts = facts_of("7k/8/5K2/8/8/8/8/6Q1 w - - 0 1");
    assert!(
        facts.tactics.stalemate_in_1.white,
        "Qg6 leaves Black with no move"
    );
    let mated = facts_of("7k/6Q1/5K2/8/8/8/8/8 b - - 0 1");
    assert_eq!(mated.tactics.legal_move_count.black, 0);
    assert!(mated.state.in_check);
}

#[test]
fn bare_kings_have_no_material() {
    let facts = facts_of("8/8/8/4k3/8/8/8/4K3 w - - 0 1");
    assert_eq!(facts.material.value, ByColour::new(0, 0));
    assert_eq!(facts.material.phase_points, 0);
    assert!(facts.material.pawns_only);
    assert_eq!(
        facts.material.insufficient_material,
        ByColour::new(true, true)
    );
    assert_eq!(facts.pawns.islands, ByColour::new(0, 0));
    assert_eq!(facts.attacks.hanging, ByColour::default());
}

#[test]
fn hanging_and_pinned_units_are_found() {
    // White's rook on e1 pins the black rook to the king on e8.
    let facts = facts_of("4k3/8/8/8/8/8/4r3/K3R3 w - - 0 1");
    assert!(facts.attacks.pinned.black.contains(Square::E2));
    assert!(facts.attacks.hanging.black.contains(Square::E2));
    assert!(facts.attacks.en_prise.black.contains(Square::E2));
}

#[rstest]
#[case::classic(&CLASSIC, include_str!("data/fens_classic.txt"))]
#[case::chess960(&CHESS960, include_str!("data/fens_chess960.txt"))]
fn facts_in_matches_facts(#[case] variant: &dyn Variant, #[case] fens: &str) {
    let mut scratch = Scratch::new();
    for fen in corpus(fens) {
        let position = Position::from_fen(fen).expect("a corpus FEN is legal");
        assert_eq!(
            position.facts(variant),
            position.facts_in(variant, &mut scratch),
            "{fen}"
        );
    }
}

#[test]
fn a_game_supplies_the_repetition_facts_a_position_cannot() {
    let mut game = Game::new(classic());
    let facts = game.facts();
    assert!(facts.history.known);
    assert!(!facts.history.repetition_seen);
    assert!(!facts.history.repetition_available);

    for uci in ["g1f3", "g8f6", "f3g1", "f6g8"] {
        game.play_uci(uci).expect("a legal move");
    }
    let facts = game.facts();
    assert!(facts.history.repetition_seen);
    assert!(facts.history.known);
    assert!(!game.position().facts(game.variant()).history.known);

    assert!(facts.history.repetition_available, "Nf3 repeats");
    assert_eq!(game.annotated_moves().len(), game.legal_moves().len());
}

#[test]
fn a_promotion_and_a_king_move_are_both_annotated() {
    let annotated = position_of("r3k3/1P6/8/8/8/8/6K1/8 w q - 0 1").annotated_moves(&CLASSIC);
    assert!(annotated.iter().any(|m| m.facts.promotion.is_some()));
    assert!(annotated.iter().any(|m| m.facts.mover == Role::King));
}

#[test]
fn chess960_start_positions_are_facts_too() {
    for seed in 0..8u64 {
        let game = Game::with_seed(chess960(), seed);
        let facts = game.facts();
        assert_eq!(facts.variant_name(), "chess960");
        assert_eq!(facts.material.phase_points, 24);
        assert_eq!(facts.pawns.islands, ByColour::new(1, 1));
        assert!(facts.tactics.legal_move_count.white > 0);
    }
}

#[test]
fn every_group_of_the_catalogue_is_a_group_of_the_facts() {
    let names: Vec<&str> = Facts::groups().collect();
    assert_eq!(
        names,
        [
            "placement",
            "state",
            "history",
            "material",
            "pawns",
            "pieces",
            "king",
            "mobility",
            "attacks",
            "exchange",
            "threats",
            "tactics",
            "endgame",
            "maps",
        ]
    );
    for group in esca::catalogue() {
        assert!(!group.fields.is_empty(), "{} has no fields", group.name);
        assert!(!group.meaning().is_empty(), "{} has no meaning", group.name);
        for field in group.fields {
            assert!(
                !field.meaning().is_empty(),
                "{}.{} has no meaning",
                group.name,
                field.name
            );
            assert!(
                !field.type_name().contains(" <") && !field.type_name().contains("> "),
                "{}.{} has an untidy type",
                group.name,
                field.name
            );
        }
    }
}

#[test]
fn the_worked_example_reads_the_files_of_a_passer() {
    let facts = facts_of("8/pp3ppk/8/3P4/8/8/PP3PPK/8 w - - 0 1");
    assert_eq!(facts.pawns.passed.white.files(), files("d"));
    assert!(facts.pawns.passed.black.files().is_empty());
}
