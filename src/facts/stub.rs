//! The facts section of `python/esca/_esca.pyi`, written from the catalogue.
//!
//! The extension exposes one class per group with one property per fact; this
//! writes those classes, the `TypedDict` each group's `to_dict()` answers in,
//! and the aliases the properties are typed with. `python/esca/_esca.pyi`
//! carries the text between the two marker lines, and a test fails when it is
//! stale.

use super::MoveFacts;
use super::catalogue::Field;
use super::types::{FieldType, Named, Run};

/// The line the generated section starts after.
pub const BEGIN: &str = "# --- generated from the facts catalogue: do not edit by hand ---";
/// The line it ends before.
pub const END: &str = "# --- end of the generated facts section ---";

/// The generated section, marker lines included, ending in a newline.
pub fn python_stub() -> String {
    let mut out = String::new();
    out.push_str(BEGIN);
    out.push_str("\n#\n# cargo run --example catalogue --all-features\n\n");
    out.push_str(HEADER);

    for group in super::catalogue() {
        out.push_str(&typed_dict(
            &format!("{}Dict", class_of(group.name)),
            &format!(
                "The plain data `{}.to_dict()` answers.",
                class_of(group.name)
            ),
            group.fields,
        ));
    }
    out.push_str(&typed_dict(
        "MoveFactsDict",
        "The plain data `MoveFacts.to_dict()` answers.",
        MoveFacts::FIELDS,
    ));

    out.push_str("class FactsDict(TypedDict):\n");
    out.push_str(
        "    \"\"\"The plain data `Facts.to_dict()` answers, one entry per group.\"\"\"\n\n",
    );
    for group in super::catalogue() {
        out.push_str(&format!(
            "    {}: {}Dict\n",
            group.name,
            class_of(group.name)
        ));
    }
    out.push('\n');

    for group in super::catalogue() {
        let class = class_of(group.name);
        out.push_str(&facts_class(&class, group.meaning(), group.fields, &class));
    }
    out.push_str(&facts_class(
        "MoveFacts",
        &move_meaning(),
        MoveFacts::FIELDS,
        "MoveFacts",
    ));

    out.push_str(
        "#: Any one group of facts, as `Facts.group()` hands one over.\ntype AnyFacts = (\n",
    );
    let classes: Vec<String> = super::catalogue()
        .iter()
        .map(|group| class_of(group.name))
        .collect();
    out.push_str(&format!("    {}\n", classes.join("\n    | ")));
    out.push_str(")\n\n");

    out.push_str(END);
    out.push('\n');
    out
}

/// What the move group is about, in one sentence.
fn move_meaning() -> String {
    "What one legal move does, beyond what the move itself says. The mover is the side to move; \
     the enemy is the other side."
        .to_string()
}

/// The class name a group's facts carry in Python.
fn class_of(group: &str) -> String {
    match group {
        "placement" => "PlacementFacts",
        "state" => "StateFacts",
        "history" => "HistoryFacts",
        "material" => "MaterialFacts",
        "pawns" => "PawnFacts",
        "pieces" => "PieceFacts",
        "king" => "KingFacts",
        "mobility" => "MobilityFacts",
        "attacks" => "AttackFacts",
        "exchange" => "ExchangeFacts",
        "threats" => "ThreatFacts",
        "tactics" => "TacticsFacts",
        "endgame" => "EndgameFacts",
        "maps" => "MapFacts",
        other => panic!("no Python class name for the group {other}"),
    }
    .to_string()
}

/// One `TypedDict` describing what a group's `to_dict()` answers.
fn typed_dict(name: &str, doc: &str, fields: &[Field]) -> String {
    let mut out = format!("class {name}(TypedDict):\n");
    out.push_str(&format!("    \"\"\"{doc}\"\"\"\n\n"));
    for field in fields {
        out.push_str(&format!(
            "    {}: {}\n",
            field.name,
            json_type(&field.kind())
        ));
    }
    out.push('\n');
    out
}

/// One facts class: a property per fact, and the methods every group carries.
fn facts_class(name: &str, doc: &str, fields: &[Field], dict: &str) -> String {
    let mut out = format!("@final\nclass {name}:\n");
    out.push_str(&docstring(doc, 4));
    out.push('\n');
    for field in fields {
        out.push_str("    @property\n");
        out.push_str(&format!(
            "    def {}(self) -> {}:\n",
            field.name,
            attribute_type(&field.kind())
        ));
        out.push_str(&docstring(field.meaning(), 8));
    }
    for (class, extra) in EXTRAS {
        if *class == name {
            out.push_str(extra);
        }
    }
    out.push_str(&format!("    def to_dict(self) -> {dict}Dict: ...\n"));
    out.push_str("    def __eq__(self, other: object) -> bool: ...\n");
    out.push_str("    def __hash__(self) -> int: ...\n\n");
    out
}

/// A docstring, wrapped at 116 columns and indented by `indent`.
fn docstring(text: &str, indent: usize) -> String {
    let pad = " ".repeat(indent);
    let width = 116usize.saturating_sub(indent);
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.len() + 1 + word.chars().count() > width.saturating_sub(6) {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    lines.push(line);
    if lines.len() == 1 {
        return format!("{pad}\"\"\"{}\"\"\"\n", lines[0]);
    }
    let mut out = format!("{pad}\"\"\"{}\n", lines[0]);
    for line in &lines[1..] {
        out.push_str(&format!("{pad}{line}\n"));
    }
    out.push_str(&format!("{pad}\"\"\"\n"));
    out
}

/// The type an attribute of a facts class answers in.
fn attribute_type(kind: &FieldType) -> String {
    match kind {
        FieldType::Bool => "bool".to_string(),
        FieldType::Int { .. } => "int".to_string(),
        FieldType::Float => "float".to_string(),
        FieldType::Named(Named::Square) => "str".to_string(),
        FieldType::Named(named) => alias(*named),
        FieldType::SquareSet => "SquareSet".to_string(),
        FieldType::FileSet => "FileSet".to_string(),
        FieldType::Optional(item) => format!("{} | None", attribute_type(item)),
        FieldType::Fixed { len, item, run } => run_type(*run, *len, &attribute_type(item)),
        FieldType::ByColour(item) => format!("ByColour[{}]", attribute_type(item)),
    }
}

/// The type a value takes in the plain data `to_dict()` answers.
fn json_type(kind: &FieldType) -> String {
    match kind {
        FieldType::Bool => "bool".to_string(),
        FieldType::Int { .. } => "int".to_string(),
        FieldType::Float => "float".to_string(),
        FieldType::Named(Named::Square) => "str".to_string(),
        FieldType::Named(named) => alias(*named),
        FieldType::SquareSet | FieldType::FileSet => "list[str]".to_string(),
        FieldType::Optional(item) => format!("{} | None", json_type(item)),
        FieldType::Fixed { item, .. } => format!("list[{}]", json_type(item)),
        FieldType::ByColour(item) => format!("ColourPair[{}]", json_type(item)),
    }
}

/// The alias a fixed run of `len` values of `item` carries.
fn run_type(run: Run, len: usize, item: &str) -> String {
    match run {
        Run::PerFile => format!("PerFile[{item}]"),
        Run::PerRank => format!("PerRank[{item}]"),
        Run::Array => format!(
            "tuple[{}]",
            std::iter::repeat_n(item, len)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

/// The alias the header declares a named value under.
fn alias(named: Named) -> String {
    match named {
        Named::Role => "RoleLetter",
        Named::File => "FileLetter",
        Named::CastledSide => "CastledSide",
        Named::Opposition => "Opposition",
        Named::DrawishMaterial => "DrawishMaterial",
        Named::Square => "str",
    }
    .to_string()
}

/// The values a group answers that are not facts of its own but a reading of
/// the ones beside them. The extension exposes each as a property.
const EXTRAS: &[(&str, &str)] = &[
    (
        "KingFacts",
        "    @property\n    def ring_attacker_surplus(self) -> ByColour[int]:\n        \"\"\"The weight besieging each king\'s ring, less the weight defending it.\"\"\"\n",
    ),
    (
        "TacticsFacts",
        "    @property\n    def check_available(self) -> ByColour[bool]:\n        \"\"\"Whether the side has a checking move.\"\"\"\n\
         \n    @property\n    def safe_check_available(self) -> ByColour[bool]:\n        \"\"\"Whether it has a checking move with a safe destination.\"\"\"\n\
         \n    @property\n    def promotion_available(self) -> ByColour[bool]:\n        \"\"\"Whether it has a promotion.\"\"\"\n\
         \n    @property\n    def safe_promotion_available(self) -> ByColour[bool]:\n        \"\"\"Whether it has a promotion with a safe destination.\"\"\"\n\
         \n    @property\n    def capture_available(self) -> ByColour[bool]:\n        \"\"\"Whether it has a capture.\"\"\"\n\
         \n    @property\n    def fork_available(self) -> ByColour[bool]:\n        \"\"\"Whether it has a forking move.\"\"\"\n\
         \n    @property\n    def pin_creation_available(self) -> ByColour[bool]:\n        \"\"\"Whether it has a move creating a pin.\"\"\"\n\
         \n    @property\n    def only_moves(self) -> ByColour[bool]:\n        \"\"\"Whether the side has at most two legal moves. False for a side whose values could not be read.\"\"\"\n",
    ),
];

/// The aliases every generated class is written in terms of.
const HEADER: &str = "\
#: A colour, as every esca surface writes one.
type Colour = Literal[\"white\", \"black\"]

#: A colour, as every esca surface reads one: either spelling, either case.
type ColourName = Literal[\"white\", \"black\", \"White\", \"Black\", \"WHITE\", \"BLACK\", \"w\", \"b\", \"W\", \"B\"]

#: A file letter.
type FileLetter = Literal[\"a\", \"b\", \"c\", \"d\", \"e\", \"f\", \"g\", \"h\"]

#: A file, as a file argument reads one: either case.
type FileName = Literal[
    \"a\",
    \"b\",
    \"c\",
    \"d\",
    \"e\",
    \"f\",
    \"g\",
    \"h\",
    \"A\",
    \"B\",
    \"C\",
    \"D\",
    \"E\",
    \"F\",
    \"G\",
    \"H\",
]

#: A role letter, as every esca surface spells a piece kind.
type RoleLetter = Literal[\"p\", \"n\", \"b\", \"r\", \"q\", \"k\"]

#: A role, as a role argument reads one: either case.
type RoleName = Literal[\"p\", \"n\", \"b\", \"r\", \"q\", \"k\", \"P\", \"N\", \"B\", \"R\", \"Q\", \"K\"]

#: A square name, a1 through h8.
type Square = Literal[
    \"a1\",
    \"b1\",
    \"c1\",
    \"d1\",
    \"e1\",
    \"f1\",
    \"g1\",
    \"h1\",
    \"a2\",
    \"b2\",
    \"c2\",
    \"d2\",
    \"e2\",
    \"f2\",
    \"g2\",
    \"h2\",
    \"a3\",
    \"b3\",
    \"c3\",
    \"d3\",
    \"e3\",
    \"f3\",
    \"g3\",
    \"h3\",
    \"a4\",
    \"b4\",
    \"c4\",
    \"d4\",
    \"e4\",
    \"f4\",
    \"g4\",
    \"h4\",
    \"a5\",
    \"b5\",
    \"c5\",
    \"d5\",
    \"e5\",
    \"f5\",
    \"g5\",
    \"h5\",
    \"a6\",
    \"b6\",
    \"c6\",
    \"d6\",
    \"e6\",
    \"f6\",
    \"g6\",
    \"h6\",
    \"a7\",
    \"b7\",
    \"c7\",
    \"d7\",
    \"e7\",
    \"f7\",
    \"g7\",
    \"h7\",
    \"a8\",
    \"b8\",
    \"c8\",
    \"d8\",
    \"e8\",
    \"f8\",
    \"g8\",
    \"h8\",
]

#: The side a king's square and its spent castling rights read as.
type CastledSide = Literal[\"short\", \"long\"]

#: Which opposition the kings stand in.
type Opposition = Literal[\"direct\", \"distant\"]

#: A material configuration that draws although one side is ahead.
type DrawishMaterial = Literal[\"two_knights\", \"wrong_bishop\", \"opposite_bishops\"]

#: Eight values, file a first, through file h.
type PerFile[T] = tuple[T, T, T, T, T, T, T, T]

#: Eight values, rank 1 first, through rank 8.
type PerRank[T] = tuple[T, T, T, T, T, T, T, T]

class ColourPair[T](TypedDict):
    \"\"\"One value per colour, as `to_dict()` writes a `ByColour`.\"\"\"

    white: T
    black: T

@final
class ByColour[T]:
    \"\"\"One value for White and one for Black.

    Iterating gives the two values, White first, so `white, black = pair`
    unpacks; `items()` gives each with the colour it belongs to. A pair hashes
    when both of its values do.
    \"\"\"

    __match_args__: Final[tuple[Literal[\"white\"], Literal[\"black\"]]]

    def __new__(cls, white: T, black: T) -> ByColour[T]: ...
    @property
    def white(self) -> T: ...
    @property
    def black(self) -> T: ...
    def of(self, colour: ColourName) -> T: ...
    def items(self) -> list[tuple[Colour, T]]: ...
    def __getitem__(self, colour: ColourName) -> T: ...
    def __iter__(self) -> Iterator[T]: ...
    def __len__(self) -> int: ...
    def __contains__(self, colour: str) -> bool: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...
    def __class_getitem__(cls, item: Any) -> Any: ...

";
