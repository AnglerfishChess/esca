//! The JSON form of the facts, as a JSON Schema.
//!
//! `docs/facts.schema.json` is [`json_schema`]'s text. It describes exactly
//! what `serde::Serialize` writes and what Python's `to_dict()` answers, so a
//! consumer can validate either without knowing which produced it.

use serde_json::{Map, Value, json};

use super::MoveFacts;
use super::catalogue::Field;
use super::types::{FieldType, Named};

/// The schema of the whole [`Facts`](crate::Facts) object, as pretty-printed
/// JSON with a trailing newline.
pub fn json_schema() -> String {
    let mut groups = Map::new();
    for group in super::catalogue() {
        let mut node = object_of(group.fields);
        node["description"] = Value::String(group.meaning().to_string());
        groups.insert(group.name.to_string(), node);
    }
    let mut moves = object_of(MoveFacts::FIELDS);
    moves["description"] = Value::String(
        "What one legal move does, beyond what the move itself says. Every annotated move of \
         facts.moves carries these."
            .to_string(),
    );
    let schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://raw.githubusercontent.com/AnglerfishChess/esca/main/docs/facts.schema.json",
        "title": "esca facts",
        "description": "What Position::facts() answers about one position, as Rust's \
                        serde::Serialize writes it and Python's to_dict() answers it. Generated \
                        from the fact declarations: cargo run --example catalogue --all-features.",
        "type": "object",
        "additionalProperties": false,
        "required": super::catalogue().iter().map(|group| group.name).collect::<Vec<_>>(),
        "properties": Value::Object(groups),
        "$defs": { "move": moves },
    });
    let mut out = serde_json::to_string_pretty(&schema).expect("a schema serialises");
    out.push('\n');
    out
}

/// One group of fields as a closed object schema.
fn object_of(fields: &[Field]) -> Value {
    let mut properties = Map::new();
    for field in fields {
        let mut node = node_of(&field.kind());
        node["description"] = Value::String(field.meaning().to_string());
        properties.insert(field.name.to_string(), node);
    }
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": fields.iter().map(|field| field.name).collect::<Vec<_>>(),
        "properties": Value::Object(properties),
    })
}

/// The schema of one value.
fn node_of(kind: &FieldType) -> Value {
    match kind {
        FieldType::Bool => json!({ "type": "boolean" }),
        FieldType::Int { signed: false, .. } => json!({ "type": "integer", "minimum": 0 }),
        FieldType::Int { signed: true, .. } => json!({ "type": "integer" }),
        FieldType::Float => json!({ "type": "number" }),
        FieldType::Named(named) => json!({ "type": "string", "enum": named.codes() }),
        FieldType::SquareSet => json!({
            "type": "array",
            "uniqueItems": true,
            "$comment": "the members in board order: a1, b1 through h1, then a2",
            "items": { "type": "string", "enum": Named::Square.codes() },
        }),
        FieldType::FileSet => json!({
            "type": "array",
            "uniqueItems": true,
            "$comment": "the members from file a upwards",
            "items": { "type": "string", "enum": Named::File.codes() },
        }),
        FieldType::Optional(item) => json!({
            "oneOf": [{ "type": "null" }, node_of(item)],
        }),
        FieldType::Fixed { len, item, run } => json!({
            "type": "array",
            "minItems": len,
            "maxItems": len,
            "$comment": run.order(),
            "items": node_of(item),
        }),
        FieldType::ByColour(item) => json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["white", "black"],
            "properties": { "white": node_of(item), "black": node_of(item) },
        }),
    }
}
