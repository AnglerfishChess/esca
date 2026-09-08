//! The facts as data: the name, type and meaning of every field of every
//! group.
//!
//! The entries are written by the `facts_group!` macro from the struct
//! definitions themselves, so a field and its catalogue entry cannot disagree.

use crate::types::{ByColour, Colour, File, FileSet, PerFile, PerRank, Role, Square, SquareSet};

use super::types::{Dtype, FieldType, Named, parse};

/// One fact: what it is called, what it holds and what it means.
///
/// Everything a field says about itself is fixed at compile time and handed
/// out as `&'static`, so walking the catalogue allocates nothing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Field {
    /// The name the field carries in Rust and in Python.
    pub name: &'static str,
    ty: &'static str,
    doc: &'static str,
}

impl Field {
    /// The entry for a field declared as `name: ty` with `doc` above it.
    #[doc(hidden)]
    pub const fn new(name: &'static str, ty: &'static str, doc: &'static str) -> Field {
        Field { name, ty, doc }
    }

    /// The field's type, as the declaration spells it.
    pub const fn type_name(&self) -> &'static str {
        self.ty
    }

    /// What the field holds, as a tree.
    ///
    /// Panics for a type no rule covers; teaching this module the type is part
    /// of adding one.
    pub fn kind(&self) -> FieldType {
        match parse(self.ty) {
            Ok(kind) => kind,
            Err(why) => panic!("{}: {why}", self.name),
        }
    }

    /// The element type a tensor of this fact holds.
    pub fn dtype(&self) -> Dtype {
        self.kind().dtype()
    }

    /// The shape of one position's value, the batch axis left off.
    ///
    /// Panics for a shape no fact has had yet; teaching this module the shape
    /// is part of adding one.
    pub fn shape(&self) -> &'static [usize] {
        let shape = self.kind().shape();
        SHAPES
            .iter()
            .copied()
            .find(|known| *known == shape.as_slice())
            .unwrap_or_else(|| panic!("{}: an unlisted shape {shape:?}", self.name))
    }

    /// What the field holds, in one sentence.
    pub fn meaning(&self) -> &'static str {
        self.doc.trim()
    }
}

/// Every shape a fact's value takes, so that one can be handed out as
/// `&'static`.
const SHAPES: [&[usize]; 10] = [
    &[],
    &[2],
    &[3],
    &[8],
    &[64],
    &[2, 2],
    &[2, 3],
    &[2, 6],
    &[2, 8],
    &[2, 64],
];

/// One group of facts.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Group {
    /// The name the group carries on [`Facts`](crate::Facts).
    pub name: &'static str,
    doc: &'static str,
    /// Its fields, in the order they are declared.
    pub fields: &'static [Field],
}

impl Group {
    /// The entry for a group named `name`, documented by `doc`.
    #[doc(hidden)]
    pub const fn new(name: &'static str, doc: &'static str, fields: &'static [Field]) -> Group {
        Group { name, doc, fields }
    }

    /// What the group is about, in one sentence.
    pub fn meaning(&self) -> &'static str {
        self.doc.trim()
    }

    /// The field of that name.
    pub fn field(&self, name: &str) -> Option<&'static Field> {
        self.fields.iter().find(|field| field.name == name)
    }
}

/// One fact's value, read without naming its type.
///
/// The shapes are the ones [`FieldType`] describes and the JSON form writes: a
/// named value is its text, an absent `Option` is [`FieldValue::Absent`], and
/// a `PerFile`, a `PerRank` or a fixed run is a [`FieldValue::Run`] in the
/// order the field's type or meaning states.
#[derive(Clone, PartialEq, Debug)]
pub enum FieldValue {
    /// True or false.
    Bool(bool),
    /// A whole number.
    Int(i64),
    /// A fractional number.
    Float(f32),
    /// One of a fixed list of names, as its text: `n`, `d`, `e4`, `short`.
    Name(&'static str),
    /// A set of squares.
    Squares(SquareSet),
    /// A set of files.
    Files(FileSet),
    /// An optional value that is absent.
    Absent,
    /// A fixed run of values.
    Run(Vec<FieldValue>),
    /// One value for White and one for Black.
    ByColour {
        /// White's value.
        white: Box<FieldValue>,
        /// Black's value.
        black: Box<FieldValue>,
    },
}

/// One group of facts, read without naming its type.
///
/// [`Facts::group`](crate::Facts::group) hands one out by name; the fields
/// come back in the order the catalogue lists them.
pub trait FactsGroup {
    /// The group's catalogue entry.
    fn entry(&self) -> &'static Group;

    /// The value of the field of that name.
    fn value(&self, field: &str) -> Option<FieldValue>;

    /// Every field's name and value, in catalogue order.
    fn fields(&self) -> Vec<(&'static str, FieldValue)> {
        self.entry()
            .fields
            .iter()
            .map(|field| {
                let value = self
                    .value(field.name)
                    .expect("a catalogued field is a field of its own group");
                (field.name, value)
            })
            .collect()
    }
}

/// A fact's own type, read as a [`FieldValue`]. Implemented for every type a
/// fact is declared with.
pub(crate) trait ToValue {
    fn to_value(&self) -> FieldValue;
}

macro_rules! int_values {
    ($($ty:ty),*) => {
        $(impl ToValue for $ty {
            fn to_value(&self) -> FieldValue {
                FieldValue::Int(i64::from(*self))
            }
        })*
    };
}

int_values!(u8, i8, u16, i16, u32, i32);

impl ToValue for bool {
    fn to_value(&self) -> FieldValue {
        FieldValue::Bool(*self)
    }
}

impl ToValue for f32 {
    fn to_value(&self) -> FieldValue {
        FieldValue::Float(*self)
    }
}

impl ToValue for SquareSet {
    fn to_value(&self) -> FieldValue {
        FieldValue::Squares(*self)
    }
}

impl ToValue for FileSet {
    fn to_value(&self) -> FieldValue {
        FieldValue::Files(*self)
    }
}

impl<T: ToValue> ToValue for Option<T> {
    fn to_value(&self) -> FieldValue {
        match self {
            Some(value) => value.to_value(),
            None => FieldValue::Absent,
        }
    }
}

impl<T: ToValue> ToValue for ByColour<T> {
    fn to_value(&self) -> FieldValue {
        FieldValue::ByColour {
            white: Box::new(self.white.to_value()),
            black: Box::new(self.black.to_value()),
        }
    }
}

impl<T: ToValue> ToValue for PerFile<T> {
    fn to_value(&self) -> FieldValue {
        FieldValue::Run(self.iter().map(T::to_value).collect())
    }
}

impl<T: ToValue> ToValue for PerRank<T> {
    fn to_value(&self) -> FieldValue {
        FieldValue::Run(self.iter().map(T::to_value).collect())
    }
}

impl<T: ToValue, const N: usize> ToValue for [T; N] {
    fn to_value(&self) -> FieldValue {
        FieldValue::Run(self.iter().map(T::to_value).collect())
    }
}

impl ToValue for Colour {
    fn to_value(&self) -> FieldValue {
        FieldValue::Name(match self {
            Colour::White => "white",
            Colour::Black => "black",
        })
    }
}

macro_rules! named_values {
    ($($ty:ty => $named:expr),* $(,)?) => {
        $(impl ToValue for $ty {
            fn to_value(&self) -> FieldValue {
                FieldValue::Name($named.codes()[self.index()])
            }
        })*
    };
}

named_values! {
    Role => Named::Role,
    File => Named::File,
    Square => Named::Square,
}

macro_rules! self_named_values {
    ($($ty:ty),* $(,)?) => {
        $(impl ToValue for $ty {
            fn to_value(&self) -> FieldValue {
                FieldValue::Name(self.name())
            }
        })*
    };
}

self_named_values!(
    super::CastledSide,
    super::Opposition,
    super::DrawishMaterial
);

/// Declares a facts group and its catalogue entry from one definition.
macro_rules! facts_group {
    (
        $(#[doc = $group_doc:literal])+
        derive($($derive:path),* $(,)?);
        pub struct $name:ident as $group:literal {
            $(
                $(#[doc = $doc:literal])+
                pub $field:ident : $ty:ty,
            )*
        }
    ) => {
        $(#[doc = $group_doc])+
        #[derive($($derive),*)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize))]
        pub struct $name {
            $(
                $(#[doc = $doc])+
                pub $field: $ty,
            )*
        }

        impl $name {
            /// The name this group carries on [`Facts`].
            pub const GROUP: &'static str = $group;

            /// Its fields, in the order they are declared.
            pub const FIELDS: &'static [$crate::facts::catalogue::Field] = &[
                $($crate::facts::catalogue::Field::new(
                    stringify!($field),
                    stringify!($ty),
                    concat!($($doc),*),
                ),)*
            ];

            /// The group's catalogue entry.
            pub const ENTRY: $crate::facts::catalogue::Group =
                $crate::facts::catalogue::Group::new(
                    $group,
                    concat!($($group_doc),*),
                    $name::FIELDS,
                );
        }

        impl $crate::facts::catalogue::FactsGroup for $name {
            fn entry(&self) -> &'static $crate::facts::catalogue::Group {
                &$name::ENTRY
            }

            fn value(&self, field: &str) -> Option<$crate::facts::catalogue::FieldValue> {
                use $crate::facts::catalogue::ToValue;
                match field {
                    $(stringify!($field) => Some(self.$field.to_value()),)*
                    _ => None,
                }
            }
        }
    };
}

pub(crate) use facts_group;

/// The catalogue as a Markdown document: a table of field, type and meaning
/// per group. `docs/features.md` is this text.
pub fn markdown() -> String {
    let mut out = String::from(PREAMBLE);
    for group in super::catalogue() {
        out.push_str(&format!("## `{}`\n\n{}\n\n", group.name, group.meaning()));
        out.push_str(&table(Some(group.name), group.fields));
        out.push('\n');
    }
    out.push_str(MOVES);
    out.push_str(&table(None, super::MoveFacts::FIELDS));
    out
}

/// One group's fields as a Markdown table. A group name gives every row the
/// name its tensor array carries.
fn table(group: Option<&str>, fields: &[Field]) -> String {
    let mut out = match group {
        Some(_) => String::from(
            "| field | array | type | dtype | shape | meaning |\n\
             | --- | --- | --- | --- | --- | --- |\n",
        ),
        None => String::from(
            "| field | type | dtype | shape | meaning |\n| --- | --- | --- | --- | --- |\n",
        ),
    };
    for field in fields {
        out.push_str(&format!("| `{}` | ", field.name));
        if let Some(group) = group {
            out.push_str(&format!("`{group}.{}` | ", field.name));
        }
        out.push_str(&format!(
            "`{}` | `{}` | `{}` | {} |\n",
            field.type_name(),
            field.dtype(),
            shape_text(field.shape()),
            field.meaning()
        ));
    }
    out
}

/// A shape written the way Python writes a tuple.
pub(crate) fn shape_text(shape: &[usize]) -> String {
    match shape {
        [] => "()".to_string(),
        [one] => format!("({one},)"),
        many => format!(
            "({})",
            many.iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

/// What the reader needs before the first table.
const PREAMBLE: &str = "\
# The facts

`Position::facts()` answers what is true about a position. Every fact belongs
to a group, and every fact names the colour it is about.

Each fact is typed for what it is *for*, not reduced to the plainest type that
would hold it: a set of squares stays a set of squares, a per-file count stays
eight counts in file order. Where a value is a run of positions, the type says
what the positions are, or the meaning does.

## The types

- `ByColour<T>` is one `T` for White and one for Black: `.white`, `.black`, or
  `.of(colour)`. In Python it is an `esca.ByColour`, read the same way.
- `SquareSet` is a set of squares, `FileSet` a set of files. Both are objects
  in Python too.
- `PerFile<T>` is eight values, file a first; `PerRank<T>` is eight values,
  rank 1 first. In Python each is an eight-element tuple in that same order.
- `[T; n]` is a fixed run of `n` values whose order the fact's own meaning
  states — a Python tuple.
- `Option<T>` is a value that may be absent — `None` in Python.
- `Role` is a piece kind, written as its lower-case letter: `p n b r q k`.
  `File` is a file letter, `Square` a square name such as `e4`.
- The named values are `Opposition` (`direct`, `distant`), `DrawishMaterial`
  (`two_knights`, `wrong_bishop`, `opposite_bishops`) and `CastledSide`
  (`short`, `long`). In Python each is that name as text.
- Where a fact is about the side to move rather than about a colour — that it
  stands in check, say — it is a plain value, and its meaning says so.
- A rank \"counted from a side's own back rank\" is rank 1 at that side's
  first rank and rank 8 at the promotion rank.

## The JSON form

One wire format serves Rust's `serde::Serialize` (feature `serde`) and
Python's `to_dict()`, and they emit exactly the same bytes. Keys are the field
names below, in the order below; a `ByColour` is `{\"white\": …, \"black\": …}`;
a `SquareSet` is the list of its square names in board order, a1 then b1
through h1, then a2; a `FileSet` is the list of its file letters; a `PerFile`,
a `PerRank` and an `[T; n]` are lists in the order stated above; a named value
is its name; an absent `Option` is `null`. `docs/facts.schema.json` is that
format as a JSON Schema.

## The tensor form

`esca::facts::tensors` (Rust) and `esca.tensors` (Python, with the `tensors`
extra) turn a run of positions into one typed array per fact. The `dtype` and
`shape` columns below are that layout, batch axis left off; nothing is scaled,
normalised or cast to float. The rules, from the type alone:

| fact type | dtype | shape |
| --- | --- | --- |
| `bool` | `bool` | `()` |
| `u8`, `i8`, `u16`, `u32`, `i32` | the same width and sign | `()` |
| `f32` | `float32` | `()` |
| `Option<T>` | `int16`, −1 where absent | `()` |
| `[T; n]`, `PerFile<T>`, `PerRank<T>` | as `T` | `(n,)` |
| `FileSet` | `bool` | `(8,)`, file a first |
| `SquareSet` | `bool` | `(64,)`, a1 first |
| a named value | `uint8`, its code | `()` |
| `ByColour<T>` | as `T` | `(2, …)`, White then Black |

A named value's code is its position in the list above: `p` is 0 and `k` is 5,
`direct` is 0 and `distant` is 1. The packed layout keeps the same facts as
bits: a `SquareSet` is one `uint64` bitboard with a1 as bit 0, a `FileSet` one
`uint8` mask with file a as bit 0, and every other `bool` fact eight values to
a byte.

The `array` column is the name that array carries in a batch and in a
safetensors file: the group and the field, joined by a dot. `esca.tensors`
lists them at runtime as `names()`, and Rust's
`facts::tensors::layout().entries()` does the same.

Every group is an attribute of `Facts` in both languages, listed by
`Facts::groups()` and read by `facts.group(name)`; `to_dict()` walks one group
in the order below. This file is generated from the group definitions
themselves: `cargo run --example catalogue`.

";

/// The heading of the move table.
const MOVES: &str = "\
## `move`

What one legal move does, beyond what the move itself says. The mover is the
side to move; the enemy is the other side. Every annotated move of
`annotated_moves()` carries these. They are not part of `Facts`, of the JSON
form or of the tensor form, all of which describe one position rather than one
move.

";
