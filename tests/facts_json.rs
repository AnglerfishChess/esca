//! The one JSON form: what `serde::Serialize` writes for a position's facts.
//!
//! `tests/data/facts/<name>.json` is the answer for each position
//! `tests/data/facts/positions.json` names, and the Python suite compares its
//! own `to_dict()` against the very same files.

mod common;

use common::{facts_of, facts_under};
use esca::CHESS960;
use rstest::rstest;
use serde_json::Value;

/// The golden text of a named position, with its line endings normalised.
fn golden(name: &str) -> String {
    let path = format!(
        "{}/tests/data/facts/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("{path} is readable"))
        .replace("\r\n", "\n")
}

/// The facts of a named position, as the JSON form writes them.
fn written(fen: &str, chess960: bool) -> String {
    let facts = if chess960 {
        facts_under(&CHESS960, fen)
    } else {
        facts_of(fen)
    };
    let mut text = serde_json::to_string_pretty(&facts).expect("the facts serialise");
    text.push('\n');
    text
}

#[rstest]
#[case::start(
    "start",
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    false
)]
#[case::start_black_to_move(
    "start_black_to_move",
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1",
    false
)]
#[case::kiwipete(
    "kiwipete",
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    false
)]
#[case::passer("passer", "8/pp3ppk/8/3P4/8/8/PP3PPK/8 w - - 0 1", false)]
#[case::chess960(
    "chess960",
    "bqnbnrkr/pppppppp/8/8/8/8/PPPPPPPP/BQNBNRKR w HFhf - 0 1",
    true
)]
fn the_golden_files_are_what_the_facts_serialise_to(
    #[case] name: &str,
    #[case] fen: &str,
    #[case] chess960: bool,
) {
    assert_eq!(
        golden(name),
        written(fen, chess960),
        "tests/data/facts/{name}.json is stale: run `cargo run --example catalogue --all-features`"
    );
}

#[rstest]
#[case::same_placement_either_way("start", "start_black_to_move")]
fn the_same_placement_writes_the_same_facts(#[case] one: &str, #[case] other: &str) {
    // Nothing but the state of play differs, so every group but `state` and
    // the tactics read after a null move answers the same either way.
    let one: Value = serde_json::from_str(&golden(one)).expect("the golden file parses");
    let other: Value = serde_json::from_str(&golden(other)).expect("the golden file parses");
    for group in ["placement", "material", "pawns", "pieces", "king", "maps"] {
        assert_eq!(one[group], other[group], "{group} reads the side to move");
    }
}

#[rstest]
#[case::squares_in_board_order("passer", "pawns", "pawns", &["a2", "b2", "f2", "g2", "d5"])]
#[case::a_set_with_no_members("passer", "pawns", "passed", &[])]
fn a_square_set_is_the_list_of_its_squares(
    #[case] name: &str,
    #[case] group: &str,
    #[case] field: &str,
    #[case] expected: &[&str],
) {
    let facts: Value = serde_json::from_str(&golden(name)).expect("the golden file parses");
    let side = if expected.is_empty() {
        "black"
    } else {
        "white"
    };
    let members: Vec<&str> = facts[group][field][side]
        .as_array()
        .expect("a list of squares")
        .iter()
        .map(|square| square.as_str().expect("a square name"))
        .collect();
    assert_eq!(members, expected);
}

#[test]
fn a_run_of_values_is_a_list_in_the_stated_order() {
    let facts: Value = serde_json::from_str(&golden("passer")).expect("the golden file parses");
    // a2 b2 f2 g2 and the passer on d5: one pawn on each of a, b, d, f, g.
    assert_eq!(
        facts["pawns"]["count_by_file"]["white"],
        serde_json::json!([1, 1, 0, 1, 0, 1, 1, 0])
    );
    assert_eq!(
        facts["king"]["shield_files"]["white"],
        serde_json::json!(["f", "g", "h"])
    );
}

#[test]
fn an_absent_value_is_null_and_a_named_one_is_its_name() {
    let facts: Value = serde_json::from_str(&golden("passer")).expect("the golden file parses");
    assert_eq!(facts["endgame"]["opposition"], Value::Null);
    assert_eq!(facts["endgame"]["drawish_material"], Value::Null);
    assert_eq!(facts["state"]["en_passant"], Value::Null);

    let mate: Value = serde_json::from_str(&golden("kiwipete")).expect("the golden file parses");
    assert_eq!(mate["king"]["castled_side"]["white"], Value::Null);
    assert_eq!(mate["history"]["last_move_victim"], Value::Null);
}

/// The keys of the golden text at `indent` spaces, in the order it writes them.
fn keys_at(text: &str, indent: usize) -> Vec<String> {
    let pad = " ".repeat(indent);
    text.lines()
        .filter_map(|line| {
            let rest = line.strip_prefix(&pad)?;
            let name = rest.strip_prefix('"')?;
            let (name, _) = name.split_once('"')?;
            (!name.starts_with(' ')).then(|| name.to_string())
        })
        .collect()
}

#[test]
fn every_group_and_field_the_catalogue_names_is_in_the_json_in_declaration_order() {
    let text = golden("kiwipete");
    let named: Vec<String> = esca::catalogue()
        .iter()
        .map(|group| group.name.to_string())
        .collect();
    assert_eq!(keys_at(&text, 2), named);

    let mut declared: Vec<String> = Vec::new();
    for group in esca::catalogue() {
        declared.extend(group.fields.iter().map(|field| field.name.to_string()));
    }
    assert_eq!(keys_at(&text, 4), declared);
}
