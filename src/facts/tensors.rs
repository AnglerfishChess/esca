//! The facts of many positions as one typed array per fact.
//!
//! A fact keeps the type it was declared with: a count is an integer of its own
//! width, a flag is a `bool`, a set of squares is 64 flags. Nothing is scaled,
//! normalised or turned into a float, and nothing is written in the mover's
//! view — the leading axis of a per-colour fact is White then Black, always.
//!
//! [`layout`] says what every array is called, what it holds and how to read
//! its axes; [`Batch`] is the arrays themselves. A batch is written in the
//! safetensors format — a JSON header naming each array, then the raw bytes —
//! which NumPy, PyTorch and `esca.tensors` all read.

use std::sync::OnceLock;

use serde_json::Value;

use super::catalogue::{Field, shape_text};
use super::types::{Dtype, FieldType, Named, Packing, Run};
use crate::game::Game;
use crate::position::Position;
use crate::variant::Variant;

/// What one array of a batch holds.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Entry {
    /// The array's name: the group and the field, `pawns.passed`.
    pub name: &'static str,
    /// The group the fact belongs to.
    pub group: &'static str,
    /// The fact's own name.
    pub field: &'static str,
    /// What the fact holds, as the declaration spells it.
    pub declared: &'static str,
    /// The element type, expanded.
    pub dtype: Dtype,
    /// The shape of one position's value, the batch axis left off.
    pub shape: &'static [usize],
    /// What each axis means, numbered the way [`Batch::shape_of`] numbers
    /// them: axis 0 is the run of positions, so these start at 1. Empty for a
    /// fact that is one value.
    pub order: &'static str,
    /// How to read the values themselves, where the shape does not say.
    pub encoding: Option<&'static str>,
    /// The element type of the packed form.
    pub packed_dtype: Dtype,
    /// The shape of one position's packed value, or `None` where the packing
    /// runs across the batch and there is no per-position shape.
    pub packed_shape: Option<Vec<usize>>,
    /// The fact's own meaning, as the catalogue states it.
    pub meaning: &'static str,
    kind: FieldType,
}

impl Entry {
    /// How many values of one position the expanded array holds.
    pub fn width(&self) -> usize {
        self.shape.iter().product::<usize>().max(1)
    }

    /// How the fact is packed.
    pub fn packing(&self) -> Packing {
        self.kind.packing()
    }
}

impl Entry {
    /// The entry as data: what the array is called, what it holds, how to read
    /// its axes, and the tree of the type it came from.
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "name": self.name,
            "group": self.group,
            "field": self.field,
            "declared": self.declared,
            "dtype": self.dtype.to_string(),
            "shape": self.shape,
            "order": self.order,
            "encoding": self.encoding,
            "packed_dtype": self.packed_dtype.to_string(),
            "packed_shape": self.packed_shape,
            "meaning": self.meaning,
            "kind": kind_json(&self.kind),
        })
    }
}

/// One node of a field's type tree, as data.
fn kind_json(kind: &FieldType) -> Value {
    match kind {
        FieldType::Bool => serde_json::json!({ "kind": "bool" }),
        FieldType::Int { signed, bits } => {
            serde_json::json!({ "kind": "int", "signed": signed, "bits": bits })
        }
        FieldType::Float => serde_json::json!({ "kind": "float" }),
        FieldType::Named(named) => serde_json::json!({
            "kind": "named",
            "type": named.type_name(),
            "codes": named.codes(),
        }),
        FieldType::SquareSet => serde_json::json!({ "kind": "square_set" }),
        FieldType::FileSet => serde_json::json!({ "kind": "file_set" }),
        FieldType::Optional(item) => {
            serde_json::json!({ "kind": "optional", "item": kind_json(item) })
        }
        FieldType::Fixed { len, item, run } => serde_json::json!({
            "kind": "fixed",
            "len": len,
            "run": match run {
                Run::PerFile => "per_file",
                Run::PerRank => "per_rank",
                Run::Array => "array",
            },
            "order": run.order(),
            "item": kind_json(item),
        }),
        FieldType::ByColour(item) => {
            serde_json::json!({ "kind": "by_colour", "item": kind_json(item) })
        }
    }
}

/// Every fact of every group, in the order they are declared.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Layout {
    /// One entry per fact.
    pub entries: Vec<Entry>,
}

impl Layout {
    /// The entry of that name, `group.field`.
    pub fn entry(&self, name: &str) -> Option<&Entry> {
        self.entries.iter().find(|entry| entry.name == name)
    }

    /// The entries of one group.
    pub fn group(&self, name: &str) -> impl Iterator<Item = &Entry> {
        self.entries.iter().filter(move |entry| entry.group == name)
    }

    /// The layout as a table a reader can check an array against.
    pub fn describe(&self) -> String {
        let mut out = String::from("| array | dtype | shape | order | meaning |\n");
        out.push_str("| --- | --- | --- | --- | --- |\n");
        for entry in &self.entries {
            out.push_str(&format!(
                "| `{}` | `{}` | `{}` | {} | {} |\n",
                entry.name,
                entry.dtype,
                shape_text(entry.shape),
                if entry.order.is_empty() {
                    "one value"
                } else {
                    entry.order
                },
                entry.meaning,
            ));
        }
        out
    }
}

/// The layout every batch follows.
pub fn layout() -> &'static Layout {
    static LAYOUT: OnceLock<Layout> = OnceLock::new();
    LAYOUT.get_or_init(|| {
        let mut entries = Vec::new();
        for group in super::catalogue() {
            for field in group.fields {
                entries.push(entry_of(group.name, field));
            }
        }
        Layout { entries }
    })
}

fn entry_of(group: &'static str, field: &'static Field) -> Entry {
    let kind = field.kind();
    Entry {
        name: kept(format!("{group}.{}", field.name)),
        group,
        field: field.name,
        declared: field.type_name(),
        dtype: kind.dtype(),
        shape: field.shape(),
        order: kept(kind.order()),
        encoding: kind.encoding().map(kept),
        packed_dtype: kind.packed_dtype(),
        packed_shape: kind.packed_shape(),
        meaning: field.meaning(),
        kind,
    }
}

/// Text the layout hands out for the life of the process. The layout is built
/// once, so this runs once per fact.
fn kept(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

/// One array of a batch, typed as the layout says.
#[derive(Clone, PartialEq, Debug)]
pub enum Buffer {
    /// One byte per value, 0 or 1.
    Bool(Vec<bool>),
    /// Unsigned bytes.
    U8(Vec<u8>),
    /// Signed bytes.
    I8(Vec<i8>),
    /// Unsigned 16-bit values.
    U16(Vec<u16>),
    /// Signed 16-bit values.
    I16(Vec<i16>),
    /// Unsigned 32-bit values.
    U32(Vec<u32>),
    /// Signed 32-bit values.
    I32(Vec<i32>),
    /// 32-bit floats.
    F32(Vec<f32>),
    /// Unsigned 64-bit values: a bitboard, one per set.
    U64(Vec<u64>),
}

impl Buffer {
    /// An empty buffer of `dtype`.
    fn empty(dtype: Dtype) -> Buffer {
        match dtype {
            Dtype::Bool => Buffer::Bool(Vec::new()),
            Dtype::Uint(8) => Buffer::U8(Vec::new()),
            Dtype::Uint(16) => Buffer::U16(Vec::new()),
            Dtype::Uint(32) => Buffer::U32(Vec::new()),
            Dtype::Uint(_) => Buffer::U64(Vec::new()),
            Dtype::Int(8) => Buffer::I8(Vec::new()),
            Dtype::Int(16) => Buffer::I16(Vec::new()),
            Dtype::Int(_) => Buffer::I32(Vec::new()),
            Dtype::Float32 => Buffer::F32(Vec::new()),
        }
    }

    /// How many values it holds.
    pub fn len(&self) -> usize {
        match self {
            Buffer::Bool(values) => values.len(),
            Buffer::U8(values) => values.len(),
            Buffer::I8(values) => values.len(),
            Buffer::U16(values) => values.len(),
            Buffer::I16(values) => values.len(),
            Buffer::U32(values) => values.len(),
            Buffer::I32(values) => values.len(),
            Buffer::F32(values) => values.len(),
            Buffer::U64(values) => values.len(),
        }
    }

    /// Whether it holds none.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Its element type.
    pub fn dtype(&self) -> Dtype {
        match self {
            Buffer::Bool(_) => Dtype::Bool,
            Buffer::U8(_) => Dtype::Uint(8),
            Buffer::I8(_) => Dtype::Int(8),
            Buffer::U16(_) => Dtype::Uint(16),
            Buffer::I16(_) => Dtype::Int(16),
            Buffer::U32(_) => Dtype::Uint(32),
            Buffer::I32(_) => Dtype::Int(32),
            Buffer::F32(_) => Dtype::Float32,
            Buffer::U64(_) => Dtype::Uint(64),
        }
    }

    /// Its bytes, little-endian, as safetensors carries them.
    pub fn bytes(&self) -> Vec<u8> {
        match self {
            Buffer::Bool(values) => values.iter().map(|&flag| u8::from(flag)).collect(),
            Buffer::U8(values) => values.clone(),
            Buffer::I8(values) => values.iter().map(|&v| v as u8).collect(),
            Buffer::U16(values) => values.iter().flat_map(|v| v.to_le_bytes()).collect(),
            Buffer::I16(values) => values.iter().flat_map(|v| v.to_le_bytes()).collect(),
            Buffer::U32(values) => values.iter().flat_map(|v| v.to_le_bytes()).collect(),
            Buffer::I32(values) => values.iter().flat_map(|v| v.to_le_bytes()).collect(),
            Buffer::F32(values) => values.iter().flat_map(|v| v.to_le_bytes()).collect(),
            Buffer::U64(values) => values.iter().flat_map(|v| v.to_le_bytes()).collect(),
        }
    }

    /// The flags of a `bool` buffer.
    fn flags(&self) -> Option<&[bool]> {
        match self {
            Buffer::Bool(values) => Some(values),
            _ => None,
        }
    }

    /// One whole number pushed, widened or narrowed to the buffer's own type.
    fn push_int(&mut self, value: i64) {
        match self {
            Buffer::Bool(values) => values.push(value != 0),
            Buffer::U8(values) => values.push(value as u8),
            Buffer::I8(values) => values.push(value as i8),
            Buffer::U16(values) => values.push(value as u16),
            Buffer::I16(values) => values.push(value as i16),
            Buffer::U32(values) => values.push(value as u32),
            Buffer::I32(values) => values.push(value as i32),
            Buffer::F32(values) => values.push(value as f32),
            Buffer::U64(values) => values.push(value as u64),
        }
    }
}

/// Which form a batch's arrays are in.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Form {
    /// One value per element, as the layout's `dtype` and `shape` say.
    Expanded,
    /// The same facts as bits: a square set is one `uint64` bitboard, a file
    /// set one `uint8` mask, and every other `bool` fact eight values to a
    /// byte, the array flattened in C order.
    Packed,
}

/// The facts of a run of positions, one array per fact, batch axis first.
#[derive(Clone, PartialEq, Debug)]
pub struct Batch {
    count: usize,
    form: Form,
    buffers: Vec<Buffer>,
}

impl Batch {
    /// The facts of every position of `positions` under `variant`.
    pub fn from_positions(positions: &[Position], variant: &dyn Variant) -> Batch {
        let mut scratch = super::Scratch::new();
        let mut batch = Batch::empty();
        for position in positions {
            let facts = position.facts_in(variant, &mut scratch);
            batch.push_facts(&facts);
        }
        batch
    }

    /// The facts of every game's current position, history and all.
    pub fn from_games(games: &[Game]) -> Batch {
        let mut batch = Batch::empty();
        for game in games {
            batch.push_facts(&game.facts());
        }
        batch
    }

    /// An empty batch in the expanded form.
    fn empty() -> Batch {
        Batch {
            count: 0,
            form: Form::Expanded,
            buffers: layout()
                .entries
                .iter()
                .map(|entry| Buffer::empty(entry.dtype))
                .collect(),
        }
    }

    /// One position's facts appended to every array.
    fn push_facts(&mut self, facts: &super::Facts) {
        let json = serde_json::to_value(facts).expect("the facts serialise");
        for (entry, buffer) in layout().entries.iter().zip(self.buffers.iter_mut()) {
            let value = json
                .get(entry.group)
                .and_then(|group| group.get(entry.field))
                .unwrap_or_else(|| panic!("{} is missing from the facts", entry.name));
            push(value, &entry.kind, buffer);
        }
        self.count += 1;
    }

    /// How many positions it holds.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Whether it holds none.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Which form its arrays are in.
    pub fn form(&self) -> Form {
        self.form
    }

    /// What every array is called and holds.
    pub fn layout(&self) -> &'static Layout {
        layout()
    }

    /// The array of the fact named `group.field`.
    pub fn buffer(&self, name: &str) -> Option<&Buffer> {
        let index = layout()
            .entries
            .iter()
            .position(|entry| entry.name == name)?;
        self.buffers.get(index)
    }

    /// Its arrays, in the layout's order.
    pub fn buffers(&self) -> &[Buffer] {
        &self.buffers
    }

    /// The shape one array has in this form, batch axis included.
    pub fn shape_of(&self, entry: &Entry) -> Vec<usize> {
        match self.form {
            Form::Expanded => {
                let mut shape = vec![self.count];
                shape.extend(entry.shape.iter().copied());
                shape
            }
            Form::Packed => match entry.packing() {
                Packing::Bits => vec![(self.count * entry.width() + 7) / 8],
                Packing::Plain => {
                    let mut shape = vec![self.count];
                    shape.extend(entry.shape.iter().copied());
                    shape
                }
                Packing::Bitboard | Packing::FileMask => {
                    let mut shape = vec![self.count];
                    shape.extend(entry.packed_shape.clone().unwrap_or_default());
                    shape
                }
            },
        }
    }

    /// The same facts packed to bits. A packed batch is returned unchanged.
    pub fn packed(&self) -> Batch {
        if self.form == Form::Packed {
            return self.clone();
        }
        let buffers = layout()
            .entries
            .iter()
            .zip(self.buffers.iter())
            .map(|(entry, buffer)| match (entry.packing(), buffer.flags()) {
                (Packing::Bitboard, Some(flags)) => Buffer::U64(
                    flags
                        .chunks(64)
                        .map(|chunk| {
                            chunk.iter().enumerate().fold(0u64, |bits, (index, &flag)| {
                                bits | (u64::from(flag) << index)
                            })
                        })
                        .collect(),
                ),
                (Packing::FileMask, Some(flags)) => Buffer::U8(
                    flags
                        .chunks(8)
                        .map(|chunk| {
                            chunk
                                .iter()
                                .enumerate()
                                .fold(0u8, |bits, (index, &flag)| bits | (u8::from(flag) << index))
                        })
                        .collect(),
                ),
                (Packing::Bits, Some(flags)) => Buffer::U8(
                    flags
                        .chunks(8)
                        .map(|chunk| {
                            chunk.iter().enumerate().fold(0u8, |bits, (index, &flag)| {
                                bits | (u8::from(flag) << (7 - index))
                            })
                        })
                        .collect(),
                ),
                _ => buffer.clone(),
            })
            .collect();
        Batch {
            count: self.count,
            form: Form::Packed,
            buffers,
        }
    }

    /// The same facts one value per element. An expanded batch is returned
    /// unchanged.
    pub fn expanded(&self) -> Batch {
        if self.form == Form::Expanded {
            return self.clone();
        }
        let buffers = layout()
            .entries
            .iter()
            .zip(self.buffers.iter())
            .map(|(entry, buffer)| match (entry.packing(), buffer) {
                (Packing::Bitboard, Buffer::U64(words)) => Buffer::Bool(
                    words
                        .iter()
                        .flat_map(|&word| (0..64).map(move |bit| (word >> bit) & 1 == 1))
                        .collect(),
                ),
                (Packing::FileMask, Buffer::U8(masks)) => Buffer::Bool(
                    masks
                        .iter()
                        .flat_map(|&mask| (0..8).map(move |bit| (mask >> bit) & 1 == 1))
                        .collect(),
                ),
                (Packing::Bits, Buffer::U8(bytes)) => {
                    let wanted = self.count * entry.width();
                    Buffer::Bool(
                        bytes
                            .iter()
                            .flat_map(|&byte| (0..8).map(move |bit| (byte >> (7 - bit)) & 1 == 1))
                            .take(wanted)
                            .collect(),
                    )
                }
                _ => buffer.clone(),
            })
            .collect();
        Batch {
            count: self.count,
            form: Form::Expanded,
            buffers,
        }
    }

    /// The batch as safetensors: a JSON header naming every array with its
    /// dtype and shape, then the values themselves.
    ///
    /// The header carries a `__metadata__` entry with the position count and
    /// the form the arrays are in.
    pub fn to_safetensors(&self) -> Vec<u8> {
        let mut header = String::from("{\"__metadata__\":{\"count\":\"");
        header.push_str(&self.count.to_string());
        header.push_str("\",\"form\":\"");
        header.push_str(match self.form {
            Form::Expanded => "expanded",
            Form::Packed => "packed",
        });
        header.push_str("\",\"producer\":\"esca ");
        header.push_str(env!("CARGO_PKG_VERSION"));
        header.push_str("\"}");

        let mut data: Vec<u8> = Vec::new();
        for (entry, buffer) in layout().entries.iter().zip(self.buffers.iter()) {
            let bytes = buffer.bytes();
            let start = data.len();
            data.extend_from_slice(&bytes);
            header.push_str(&format!(
                ",\"{}\":{{\"dtype\":\"{}\",\"shape\":[{}],\"data_offsets\":[{},{}]}}",
                entry.name,
                buffer.dtype().safetensors(),
                self.shape_of(entry)
                    .iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
                start,
                data.len(),
            ));
        }
        header.push('}');
        while header.len() % 8 != 0 {
            header.push(' ');
        }

        let mut out = Vec::with_capacity(8 + header.len() + data.len());
        out.extend_from_slice(&(header.len() as u64).to_le_bytes());
        out.extend_from_slice(header.as_bytes());
        out.extend_from_slice(&data);
        out
    }
}

/// One value of the JSON form appended to `buffer`, following `kind`.
fn push(value: &Value, kind: &FieldType, buffer: &mut Buffer) {
    match kind {
        FieldType::Bool => buffer.push_int(i64::from(value.as_bool().expect("a flag"))),
        FieldType::Int { .. } => buffer.push_int(value.as_i64().expect("a whole number")),
        FieldType::Float => match buffer {
            Buffer::F32(values) => values.push(value.as_f64().expect("a number") as f32),
            other => other.push_int(value.as_i64().unwrap_or_default()),
        },
        FieldType::Named(named) => buffer.push_int(code_of(*named, value) as i64),
        FieldType::SquareSet => {
            let members = value.as_array().expect("a list of square names");
            let mut held = [false; 64];
            for name in members {
                held[index_in(Named::Square, name)] = true;
            }
            for flag in held {
                buffer.push_int(i64::from(flag));
            }
        }
        FieldType::FileSet => {
            let members = value.as_array().expect("a list of file letters");
            let mut held = [false; 8];
            for name in members {
                held[index_in(Named::File, name)] = true;
            }
            for flag in held {
                buffer.push_int(i64::from(flag));
            }
        }
        FieldType::Optional(item) => {
            if value.is_null() {
                buffer.push_int(-1);
            } else {
                match item.as_ref() {
                    FieldType::Named(named) => buffer.push_int(code_of(*named, value) as i64),
                    _ => buffer.push_int(value.as_i64().expect("a whole number")),
                }
            }
        }
        FieldType::Fixed { len, item, .. } => {
            let values = value.as_array().expect("a fixed run of values");
            assert_eq!(values.len(), *len, "a run of the wrong length");
            for value in values {
                push(value, item, buffer);
            }
        }
        FieldType::ByColour(item) => {
            for side in ["white", "black"] {
                push(&value[side], item, buffer);
            }
        }
    }
}

/// The code of a named value, its position in the list the catalogue gives.
fn code_of(named: Named, value: &Value) -> usize {
    index_in(named, value)
}

fn index_in(named: Named, value: &Value) -> usize {
    let name = value.as_str().expect("a name");
    named
        .codes()
        .iter()
        .position(|code| *code == name)
        .unwrap_or_else(|| panic!("{name} is not a {}", named.type_name()))
}
