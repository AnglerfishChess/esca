//! The catalogue, the documentation it writes, and the groups it describes.

use esca::facts::types::{Dtype, FieldType, Run};
use esca::{Facts, catalogue};

/// The phrases a fact whose value is a run of positions may state its order
/// with. One of them has to appear in the meaning, so that a reader of
/// `[1, 1, 1, 1, 1, 1, 1, 1]` knows which end is which.
const ORDER_PHRASES: [&str; 4] = [
    "file a first",
    "rank 1 first",
    "queenside first",
    "board order",
];

#[test]
fn the_features_document_is_the_catalogue() {
    // A checkout may rewrite the file's line endings; the words are the contract.
    assert_eq!(
        include_str!("../docs/features.md").replace("\r\n", "\n"),
        esca::facts::catalogue::markdown(),
        "docs/features.md is stale: run `cargo run --example catalogue --all-features`"
    );
}

#[test]
fn the_json_schema_is_the_catalogue() {
    assert_eq!(
        include_str!("../docs/facts.schema.json").replace("\r\n", "\n"),
        esca::facts::schema::json_schema(),
        "docs/facts.schema.json is stale: run `cargo run --example catalogue --all-features`"
    );
}

#[test]
fn the_python_stub_carries_the_generated_section() {
    let stub = include_str!("../python/esca/_esca.pyi").replace("\r\n", "\n");
    let generated = esca::facts::stub::python_stub();
    assert!(
        stub.contains(&generated),
        "python/esca/_esca.pyi is stale: run `cargo run --example catalogue --all-features`"
    );
}

#[test]
fn the_catalogue_names_the_groups_facts_carries() {
    let groups: Vec<&str> = Facts::groups().collect();
    let named: Vec<&str> = catalogue().iter().map(|group| group.name).collect();
    assert_eq!(groups, named);
}

#[test]
fn every_field_is_named_once_in_its_group() {
    for group in catalogue() {
        let mut names: Vec<&str> = group.fields.iter().map(|field| field.name).collect();
        let count = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), count, "{} names a field twice", group.name);
        for name in names {
            assert!(
                group.field(name).is_some(),
                "{}.{name} is not found by name",
                group.name
            );
        }
    }
}

#[test]
fn a_meaning_is_one_line_of_prose() {
    for group in catalogue() {
        for field in group.fields {
            let meaning = field.meaning();
            assert!(!meaning.is_empty(), "{}.{}", group.name, field.name);
            assert!(!meaning.contains('\n'), "{}.{}", group.name, field.name);
            assert!(
                meaning.ends_with('.'),
                "{}.{} does not end in a full stop",
                group.name,
                field.name
            );
        }
    }
}

#[test]
fn every_declared_type_has_a_rule() {
    for group in catalogue() {
        for field in group.fields {
            // Panics with the name of the type no rule covers.
            let kind = field.kind();
            assert!(
                kind.width() >= 1,
                "{}.{} has no width",
                group.name,
                field.name
            );
        }
    }
    for field in esca::MoveFacts::FIELDS {
        let _ = field.kind();
    }
}

#[test]
fn a_run_of_values_states_which_end_is_which() {
    for group in catalogue() {
        for field in group.fields {
            if !field.kind().is_run() {
                continue;
            }
            let meaning = field.meaning().to_lowercase();
            assert!(
                ORDER_PHRASES.iter().any(|phrase| meaning.contains(phrase)),
                "{}.{} is a run of values and its meaning names no order; one of {ORDER_PHRASES:?} \
                 has to appear in it",
                group.name,
                field.name,
            );
        }
    }
}

#[test]
fn the_shield_names_the_files_its_three_values_stand_on() {
    let king = catalogue()
        .iter()
        .find(|group| group.name == "king")
        .expect("a king group");
    for name in [
        "shield",
        "storm",
        "shield_file_open",
        "shield_file_semi_open_for_enemy",
    ] {
        let meaning = king.field(name).expect("a shield field").meaning();
        assert!(
            meaning.contains("shield_files"),
            "king.{name} does not point at shield_files"
        );
    }
}

#[test]
fn a_type_carries_its_own_dtype_and_shape() {
    let pawns = catalogue()
        .iter()
        .find(|group| group.name == "pawns")
        .expect("a pawns group");

    let passed = pawns.field("passed").expect("a passed field");
    assert_eq!(passed.dtype(), Dtype::Bool);
    assert_eq!(passed.shape(), vec![2, 64]);

    let by_file = pawns.field("count_by_file").expect("a count_by_file field");
    assert_eq!(by_file.dtype(), Dtype::Uint(8));
    assert_eq!(by_file.shape(), vec![2, 8]);
    assert!(matches!(
        by_file.kind(),
        FieldType::ByColour(item) if matches!(*item, FieldType::Fixed { run: Run::PerFile, .. })
    ));

    let lead = pawns
        .field("passer_lead_rank")
        .expect("a passer_lead_rank field");
    assert_eq!(lead.dtype(), Dtype::Int(16));
    assert_eq!(lead.shape(), vec![2]);

    let open = pawns.field("open_files").expect("an open_files field");
    assert_eq!(open.dtype(), Dtype::Bool);
    assert_eq!(open.shape(), vec![8]);

    let king = catalogue()
        .iter()
        .find(|group| group.name == "king")
        .expect("a king group");
    assert_eq!(king.field("tropism").unwrap().dtype(), Dtype::Float32);
    assert_eq!(king.field("square").unwrap().dtype(), Dtype::Uint(8));
    assert_eq!(king.field("shield").unwrap().shape(), vec![2, 3]);
}
