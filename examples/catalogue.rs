//! Rewrites everything generated from the facts catalogue.
//!
//! ```text
//! cargo run --example catalogue --all-features
//! ```
//!
//! - `docs/features.md`, the catalogue as a document.
//! - `docs/facts.schema.json`, the JSON form as a JSON Schema.
//! - the generated section of `python/esca/_esca.pyi`.
//! - `tests/data/facts/<name>.json`, the JSON form of each named position of
//!   `tests/data/facts/positions.json`.
//! - `tests/data/facts/batch.safetensors` and `batch.packed.safetensors`, the
//!   same positions as tensors.
//!
//! Tests in both languages fail whenever a committed file and the live
//! catalogue disagree.

use std::fs;
use std::path::{Path, PathBuf};

use esca::facts::{schema, stub, tensors};

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    write(
        &root.join("docs/features.md"),
        esca::facts::catalogue::markdown(),
    );
    write(&root.join("docs/facts.schema.json"), schema::json_schema());

    let stub_path = root.join("python/esca/_esca.pyi");
    let stub = fs::read_to_string(&stub_path).expect("the stub is readable");
    write(&stub_path, spliced(&stub, &stub::python_stub()));

    let named = named_positions(&root);
    for (name, fen, variant) in &named {
        let position = esca::Position::from_fen(fen).expect("a named position parses");
        let facts = position.facts(rules(variant).as_ref());
        let mut text = serde_json::to_string_pretty(&facts).expect("the facts serialise");
        text.push('\n');
        write(&root.join(format!("tests/data/facts/{name}.json")), text);
    }

    let positions: Vec<esca::Position> = named
        .iter()
        .filter(|(_, _, variant)| variant == "chess")
        .map(|(_, fen, _)| esca::Position::from_fen(fen).expect("a named position parses"))
        .collect();
    let batch = tensors::Batch::from_positions(&positions, esca::classic().as_ref());
    write_bytes(
        &root.join("tests/data/facts/batch.safetensors"),
        batch.to_safetensors(),
    );
    write_bytes(
        &root.join("tests/data/facts/batch.packed.safetensors"),
        batch.packed().to_safetensors(),
    );
}

/// The positions the golden files are written for: the name, the FEN and the
/// variant, in the order `tests/data/facts/positions.json` names them.
fn named_positions(root: &Path) -> Vec<(String, String, String)> {
    let text = fs::read_to_string(root.join("tests/data/facts/positions.json"))
        .expect("the named positions are readable");
    let named: serde_json::Value = serde_json::from_str(&text).expect("the named positions parse");
    named
        .as_array()
        .expect("a list of named positions")
        .iter()
        .map(|entry| {
            (
                entry["name"].as_str().expect("a name").to_string(),
                entry["fen"].as_str().expect("a fen").to_string(),
                entry["variant"].as_str().expect("a variant").to_string(),
            )
        })
        .collect()
}

/// The rules a named variant means.
fn rules(name: &str) -> std::sync::Arc<dyn esca::Variant> {
    match name {
        "chess960" => esca::chess960(),
        _ => esca::classic(),
    }
}

/// `text` with the generated section of the stub replaced by `section`.
fn spliced(text: &str, section: &str) -> String {
    let begin = text
        .find(stub::BEGIN)
        .unwrap_or_else(|| panic!("the stub carries no {:?} line", stub::BEGIN));
    let end = text
        .find(stub::END)
        .unwrap_or_else(|| panic!("the stub carries no {:?} line", stub::END))
        + stub::END.len()
        + 1;
    format!("{}{section}{}", &text[..begin], &text[end..])
}

fn write(path: &Path, text: String) {
    fs::write(path, text).unwrap_or_else(|why| panic!("{} is writable: {why}", path.display()));
    println!("wrote {}", path.display());
}

fn write_bytes(path: &Path, bytes: Vec<u8>) {
    fs::write(path, bytes).unwrap_or_else(|why| panic!("{} is writable: {why}", path.display()));
    println!("wrote {}", path.display());
}
