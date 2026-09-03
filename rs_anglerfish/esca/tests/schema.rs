//! The v1 schema is a frozen contract: its text, its widths and its id.

use esca::{CHESS960, CLASSIC, Schema};

/// The id of the v1 schema. Changing it is changing what a trained net eats.
const SCHEMA_V1_ID: &str = "8030a54d6f6a11e0efa97d3f90117baa";

const CANONICAL: &str = include_str!("data/schema_v1.txt");

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

#[test]
fn the_manifest_matches_features_md() {
    let schema = Schema::v1();
    let expected = [
        ("placement", 768),
        ("state", 16),
        ("material", 26),
        ("pawns", 195),
        ("pieces", 35),
        ("king", 137),
        ("mobility", 39),
        ("attacks", 25),
        ("exchange", 8),
        ("threats", 24),
        ("tactics", 120),
        ("endgame", 15),
        ("history", 27),
        ("planes", 512),
    ];
    let named: Vec<(&str, usize)> = schema
        .groups()
        .iter()
        .map(|group| (group.name, group.width))
        .collect();
    assert_eq!(named, expected);
    assert_eq!(schema.width(), 1947);
    assert_eq!(schema.semver(), "1.0.0");
}

#[test]
fn subsets_have_their_own_widths() {
    let schema = Schema::v1();
    assert_eq!(schema.width_of(schema.all()), 1947);
    let without_planes = {
        let mut set = schema.all();
        set.remove(schema.group_index("planes").expect("planes is a group"));
        set
    };
    assert_eq!(schema.width_of(without_planes), 1435);
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
