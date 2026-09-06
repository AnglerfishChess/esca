//! The facts as data: the name, type and meaning of every field of every
//! group.
//!
//! The entries are written by the [`facts_group!`] macro from the struct
//! definitions themselves, so a field and its catalogue entry cannot disagree.

/// One fact: what it is called, what it holds and what it means.
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
    pub fn type_name(&self) -> String {
        tidy_type(self.ty)
    }

    /// What the field holds, in one sentence.
    pub fn meaning(&self) -> String {
        sentence(self.doc)
    }
}

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
    pub fn meaning(&self) -> String {
        sentence(self.doc)
    }

    /// The field of that name.
    pub fn field(&self, name: &str) -> Option<&'static Field> {
        self.fields.iter().find(|field| field.name == name)
    }
}

/// Doc-comment lines joined into one line of single-spaced words.
fn sentence(doc: &str) -> String {
    doc.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A stringified type with the spaces `stringify!` leaves around its
/// punctuation taken out again.
fn tidy_type(ty: &str) -> String {
    ty.replace(" <", "<")
        .replace("< ", "<")
        .replace(" >", ">")
        .replace("> ", ">")
        .replace(" ;", ";")
        .replace(" ,", ",")
}

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
            /// The name this group carries on [`Facts`](crate::Facts).
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
    };
}

pub(crate) use facts_group;

/// The catalogue as a Markdown document: a table of field, type and meaning
/// per group. `docs/features.md` is this text.
pub fn markdown() -> String {
    let mut out = String::from(PREAMBLE);
    for group in super::catalogue() {
        out.push_str(&format!("## `{}`\n\n{}\n\n", group.name, group.meaning()));
        out.push_str(&table(group.fields));
        out.push('\n');
    }
    out.push_str(MOVES);
    out.push_str(&table(super::MoveFacts::FIELDS));
    out
}

/// One group's fields as a Markdown table.
fn table(fields: &[Field]) -> String {
    let mut out = String::from("| field | type | meaning |\n| --- | --- | --- |\n");
    for field in fields {
        out.push_str(&format!(
            "| `{}` | `{}` | {} |\n",
            field.name,
            field.type_name(),
            field.meaning()
        ));
    }
    out
}

/// What the reader needs before the first table.
const PREAMBLE: &str = "\
# The facts

`Position::facts()` answers what is true about a position. Every fact belongs
to a group, and every fact names the colour it is about.

- `ByColour<T>` is one `T` for White and one for Black: `.white`, `.black`, or
  `.of(colour)`. In Python it is an `esca.ByColour`, read the same way, and
  `to_dict()` writes it as `{\"white\": …, \"black\": …}`.
- `SquareSet` is a set of squares, `FileSet` a set of files. Both are objects
  in Python too; `to_dict()` writes them as lists of names.
- `Option<T>` is a value that may be absent — `None` in Python.
- `[T; n]` is a fixed run of `n` values, a Python list.
- `Role` is a piece kind, written as its lower-case letter: `p n b r q k`.
- The named values are `Opposition` (`direct`, `distant`), `DrawishMaterial`
  (`two_knights`, `wrong_bishop`, `opposite_bishops`) and `CastledSide`
  (`short`, `long`). In Python each is that name as text.
- Where a fact is about the side to move rather than about a colour — that it
  stands in check, say — it is a plain value, and its meaning says so.
- A rank \"counted from a side's own back rank\" is rank 1 at that side's
  first rank and rank 8 at the promotion rank.

Every group is also a Python attribute of `Facts`, listed by `Facts.groups()`
and read by `facts.group(name)`; `fields()` and `to_dict()` walk one group in
the order below. This file is generated from the group definitions themselves:
`cargo run --example catalogue`.

";

/// The heading of the move table.
const MOVES: &str = "\
## `move`

What one legal move does, beyond what the move itself says. The mover is the
side to move; the enemy is the other side. Every annotated move of
`facts.moves` carries these.

";
