//! The pair by colour, the file set, and the shape they serialise to.

mod common;

use common::facts_of;
use esca::{ByColour, Colour, File, FileSet, Square, SquareSet};
use rstest::rstest;

#[test]
fn a_pair_is_read_by_name_by_colour_and_by_index() {
    let pair = ByColour::new(3, 5);
    assert_eq!(pair.white, 3);
    assert_eq!(pair.black, 5);
    assert_eq!(pair.of(Colour::White), 3);
    assert_eq!(pair.of(Colour::Black), 5);
    assert_eq!(pair[Colour::Black], 5);
}

#[test]
fn a_pair_maps_both_sides_at_once() {
    let pair = ByColour::new(2u8, 4u8);
    assert_eq!(pair.map(|value| value * 2), ByColour::new(4, 8));
    assert_eq!(
        ByColour::from_fn(|colour| colour == Colour::White),
        ByColour::new(true, false)
    );
}

#[test]
fn a_pair_is_written_to_by_colour() {
    let mut pair = ByColour::default();
    pair[Colour::Black] = 7;
    assert_eq!(pair, ByColour::new(0, 7));
}

#[rstest]
#[case::empty("", "")]
#[case::one("d4", "d")]
#[case::a_file("d4 d5 d6", "d")]
#[case::several("a1 c3 h8", "ach")]
fn the_files_of_a_square_set_are_the_files_its_members_stand_on(
    #[case] squares: &str,
    #[case] files: &str,
) {
    assert_eq!(common::squares(squares).files(), common::files(files));
}

#[test]
fn a_file_set_holds_files_and_nothing_else() {
    let files: FileSet = [File::B, File::G].into_iter().collect();
    assert_eq!(files.len(), 2);
    assert!(files.contains(File::B));
    assert!(!files.contains(File::C));
    assert_eq!(files.into_iter().collect::<Vec<_>>(), [File::B, File::G]);
    assert!(FileSet::EMPTY.is_empty());
}

#[test]
fn the_worked_example_reads_the_files_of_a_passer() {
    let facts = facts_of("8/pp3ppk/8/3P4/8/8/PP3PPK/8 w - - 0 1");
    assert_eq!(facts.pawns.passed.white, Square::D5.to_set());
    assert_eq!(facts.pawns.passed.white.files(), common::files("d"));
    assert_eq!(facts.pawns.passed.black, SquareSet::EMPTY);
}

#[cfg(feature = "serde")]
#[test]
fn a_group_serialises_as_the_python_to_dict_writes_it() {
    let facts = facts_of("8/pp3ppk/8/3P4/8/8/PP3PPK/8 w - - 0 1");
    let json = serde_json::to_value(facts.pawns).expect("the pawns group serialises");

    assert_eq!(json["passed"]["white"], serde_json::json!(["d5"]));
    assert_eq!(json["passed"]["black"], serde_json::json!([]));
    assert_eq!(json["open_files"], serde_json::json!(["c", "e", "h"]));
    assert_eq!(json["rams"], serde_json::json!(0));
    assert_eq!(json["passer_lead_rank"]["black"], serde_json::json!(null));
}

#[cfg(feature = "serde")]
#[test]
fn every_group_serialises_under_its_own_name() {
    let facts = facts_of("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let json = serde_json::to_value(&facts).expect("the facts serialise");
    for group in esca::catalogue() {
        let block = &json[group.name];
        assert!(block.is_object(), "{} is not an object", group.name);
        for field in group.fields {
            assert!(
                block.get(field.name).is_some(),
                "{}.{} is missing",
                group.name,
                field.name
            );
        }
    }
    assert_eq!(json["endgame"]["opposition"], serde_json::json!(null));
    assert_eq!(json["state"]["in_check"], serde_json::json!(false));
}
