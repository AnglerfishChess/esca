//! The v1 schema is a frozen contract: its text, its widths and its id.

use esca::{CHESS960, CLASSIC, MoveFacts, Schema};

/// The id of the v1 schema. Changing it is changing what a trained net eats.
const SCHEMA_V1_ID: &str = "dbe7a74d1478ca3f083be1cb5df36a1d";

const CANONICAL: &str = include_str!("data/schema_v1.txt");

/// The move row as `features.md` §3 lists it, in the canonical form §6 spells.
const MOVE_CANONICAL: &str = "\
move:1:40
  is_capture:1:bit
  victim_type:5:one-hot
  mover_type:6:one-hot
  promotion_piece:4:one-hot
  gives_check:1:bit
  gives_safe_check:1:bit
  is_safe:1:bit
  captures_hanging:1:bit
  escapes_attack:1:bit
  to_attacked_by_pawn:1:bit
  is_castling:1:bit
  is_en_passant:1:bit
  see:1:diff/9
  threat_created_max:1:count/9
  moves_attacked_unit:1:bit
  blocks_check:1:bit
  advances_passer:1:bit
  creates_passer:1:bit
  creates_weakness:3:bits
  opens_file_at_enemy_king:1:bit
  ring_attack_delta:2:diff/4
  own_hanging_delta:1:diff/4
  their_hanging_delta:1:diff/4
  leaves_unit_hanging:1:bit
  gives_discovered_attack:1:bit
";

#[test]
fn the_id_is_the_checked_in_one() {
    assert_eq!(Schema::v1().id().to_string(), SCHEMA_V1_ID);
    assert_eq!(
        include_str!("data/schema_v1_id.txt").trim(),
        SCHEMA_V1_ID,
        "the fixture and the constant disagree"
    );
}

#[test]
fn the_canonical_text_is_the_golden_one() {
    assert_eq!(Schema::v1().canonical(), CANONICAL);
}

/// The move row is a section of the same text, so the one id covers both rows.
#[test]
fn the_move_sections_canonical_text_is_the_golden_ones_last_section() {
    let moves = Schema::v1().moves();
    assert_eq!(moves.canonical(), MOVE_CANONICAL);
    assert!(CANONICAL.ends_with(&moves.canonical()));
    assert_eq!(CANONICAL.matches("\nmove:").count(), 1);
}

/// A row of another width is a different contract, so the widths the section
/// names are the ones a move is written with.
#[test]
fn the_move_rows_features_are_as_wide_as_the_row() {
    let moves = Schema::v1().moves();
    assert_eq!(moves.name, "move");
    assert_eq!(moves.version, 1);
    assert_eq!(moves.width, MoveFacts::WIDTH);
    assert_eq!(
        moves.features.iter().map(|f| f.width).sum::<usize>(),
        MoveFacts::WIDTH
    );
    assert_eq!(moves.features.len(), 25);
    assert_eq!(
        moves
            .feature("see")
            .map(|spec| (spec.offset, spec.encoding)),
        Some((24, "diff/9"))
    );
    assert_eq!(moves.feature("nonesuch"), None);
}

#[test]
fn the_manifest_matches_features_md() {
    let schema = Schema::v1();
    let expected = [
        ("placement", 768),
        ("state", 16),
        ("material", 28),
        ("pawns", 195),
        ("pieces", 44),
        ("king", 137),
        ("mobility", 44),
        ("attacks", 25),
        ("exchange", 8),
        ("threats", 24),
        ("tactics", 132),
        ("endgame", 15),
        ("history", 27),
        ("planes", 576),
    ];
    let named: Vec<(&str, usize)> = schema
        .groups()
        .iter()
        .map(|group| (group.name, group.width))
        .collect();
    assert_eq!(named, expected);
    assert_eq!(schema.width(), 2039);
    assert_eq!(schema.semver(), "1.0.0");
}

#[test]
fn subsets_have_their_own_widths() {
    let schema = Schema::v1();
    assert_eq!(schema.width_of(schema.all()), 2039);
    let without_planes = {
        let mut set = schema.all();
        set.remove(schema.group_index("planes").expect("planes is a group"));
        set
    };
    assert_eq!(schema.width_of(without_planes), 1463);
    let pair = schema
        .group_set(&["state", "pawns"])
        .expect("both are groups");
    assert_eq!(schema.width_of(pair), 16 + 195);
    assert_eq!(schema.group_set(&["nonesuch"]), None);
}

#[test]
fn chess960_drops_the_features_that_assume_classic_squares() {
    let schema = Schema::v1();
    let classic = schema.features_for(&CLASSIC);
    let nine_sixty = schema.features_for(&CHESS960);

    for (group, feature) in [
        ("pieces", "minors_undeveloped"),
        ("pieces", "queen_developed"),
        ("king", "king_on_home_square"),
        ("king", "king_castled_zone"),
        ("king", "castled_side"),
    ] {
        assert!(classic.contains(group, feature), "{group}.{feature}");
        assert!(!nine_sixty.contains(group, feature), "{group}.{feature}");
    }
    assert!(nine_sixty.contains("pawns", "passed_files"));
    assert_eq!(classic.names().count(), schema.feature_count());
    assert_eq!(nine_sixty.names().count(), schema.feature_count() - 5);
}
