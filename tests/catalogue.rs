//! The catalogue, the documentation it writes, and the groups it describes.

use esca::{Facts, catalogue};

#[test]
fn the_features_document_is_the_catalogue() {
    // A checkout may rewrite the file's line endings; the words are the contract.
    assert_eq!(
        include_str!("../docs/features.md").replace("\r\n", "\n"),
        esca::facts::catalogue::markdown(),
        "docs/features.md is stale: run `cargo run --example catalogue`"
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
