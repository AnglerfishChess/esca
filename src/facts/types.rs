//! What a fact holds, as data: the declared type parsed into a small tree.
//!
//! Everything generated about the facts — the type column of
//! `docs/features.md`, the JSON schema, the tensor layout and the Python
//! stub — is read off this tree, so the four cannot drift apart or from the
//! declaration they all come from.

use core::fmt;

/// A value that is one of a fixed list of names.
///
/// The names are in declaration order, and that order is the code a tensor
/// carries: the first name is 0.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Named {
    /// A piece kind, written as its lower-case letter.
    Role,
    /// A column of the board, written as its letter.
    File,
    /// A cell of the board, written as its name.
    Square,
    /// The side a king's square and its spent castling rights read as.
    CastledSide,
    /// Which opposition the kings stand in.
    Opposition,
    /// A material configuration that draws although one side is ahead.
    DrawishMaterial,
}

impl Named {
    /// The name the type carries in Rust.
    pub const fn type_name(self) -> &'static str {
        match self {
            Named::Role => "Role",
            Named::File => "File",
            Named::Square => "Square",
            Named::CastledSide => "CastledSide",
            Named::Opposition => "Opposition",
            Named::DrawishMaterial => "DrawishMaterial",
        }
    }

    /// The names it can take, in declaration order: the first is code 0.
    pub const fn codes(self) -> &'static [&'static str] {
        match self {
            Named::Role => &["p", "n", "b", "r", "q", "k"],
            Named::File => &["a", "b", "c", "d", "e", "f", "g", "h"],
            Named::Square => &SQUARE_NAMES,
            Named::CastledSide => &["short", "long"],
            Named::Opposition => &["direct", "distant"],
            Named::DrawishMaterial => &["two_knights", "wrong_bishop", "opposite_bishops"],
        }
    }
}

/// Every square name, a1 first.
const SQUARE_NAMES: [&str; 64] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", //
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2", //
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3", //
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4", //
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5", //
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6", //
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", //
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8", //
];

/// What the positions of a fixed run of values mean.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Run {
    /// One value per file, file a first.
    PerFile,
    /// One value per rank, rank 1 first.
    PerRank,
    /// A run of `n` values whose order the fact's own meaning states.
    Array,
}

impl Run {
    /// How the positions are to be read.
    pub const fn order(self) -> &'static str {
        match self {
            Run::PerFile => "file a first, through file h",
            Run::PerRank => "rank 1 first, through rank 8",
            Run::Array => "the order the fact's meaning states",
        }
    }
}

/// What one fact holds.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FieldType {
    /// True or false.
    Bool,
    /// A whole number of `bits` bits.
    Int {
        /// Whether it can be negative.
        signed: bool,
        /// Its width in bits.
        bits: u32,
    },
    /// A fractional number, 32 bits wide.
    Float,
    /// One of a fixed list of names.
    Named(Named),
    /// A set of squares.
    SquareSet,
    /// A set of files.
    FileSet,
    /// A value that may be absent.
    Optional(Box<FieldType>),
    /// A fixed run of `len` values.
    Fixed {
        /// How many values.
        len: usize,
        /// What each of them holds.
        item: Box<FieldType>,
        /// What their positions mean.
        run: Run,
    },
    /// One value for White and one for Black.
    ByColour(Box<FieldType>),
}

/// The type a declaration spells, or an explanation of what could not be read.
pub fn parse(ty: &str) -> Result<FieldType, String> {
    let ty = ty.trim();
    let inner = |head: &str| -> Option<&str> {
        ty.strip_prefix(head)
            .and_then(|rest| rest.strip_prefix('<'))
            .and_then(|rest| rest.strip_suffix('>'))
    };
    if let Some(item) = inner("ByColour") {
        return Ok(FieldType::ByColour(Box::new(parse(item)?)));
    }
    if let Some(item) = inner("Option") {
        return Ok(FieldType::Optional(Box::new(parse(item)?)));
    }
    if let Some(item) = inner("PerFile") {
        return Ok(FieldType::Fixed {
            len: 8,
            item: Box::new(parse(item)?),
            run: Run::PerFile,
        });
    }
    if let Some(item) = inner("PerRank") {
        return Ok(FieldType::Fixed {
            len: 8,
            item: Box::new(parse(item)?),
            run: Run::PerRank,
        });
    }
    if let Some(body) = ty.strip_prefix('[').and_then(|rest| rest.strip_suffix(']')) {
        let (item, len) = body
            .rsplit_once(';')
            .ok_or_else(|| format!("an array without a length: {ty}"))?;
        let len = len
            .trim()
            .parse()
            .map_err(|_| format!("an array without a numeric length: {ty}"))?;
        return Ok(FieldType::Fixed {
            len,
            item: Box::new(parse(item)?),
            run: Run::Array,
        });
    }
    Ok(match ty {
        "bool" => FieldType::Bool,
        "f32" => FieldType::Float,
        "SquareSet" => FieldType::SquareSet,
        "FileSet" => FieldType::FileSet,
        "Role" => FieldType::Named(Named::Role),
        "File" => FieldType::Named(Named::File),
        "Square" => FieldType::Named(Named::Square),
        "CastledSide" => FieldType::Named(Named::CastledSide),
        "Opposition" => FieldType::Named(Named::Opposition),
        "DrawishMaterial" => FieldType::Named(Named::DrawishMaterial),
        "u8" => FieldType::Int {
            signed: false,
            bits: 8,
        },
        "i8" => FieldType::Int {
            signed: true,
            bits: 8,
        },
        "u16" => FieldType::Int {
            signed: false,
            bits: 16,
        },
        "i16" => FieldType::Int {
            signed: true,
            bits: 16,
        },
        "u32" => FieldType::Int {
            signed: false,
            bits: 32,
        },
        "i32" => FieldType::Int {
            signed: true,
            bits: 32,
        },
        other => return Err(format!("no rule for the type {other}")),
    })
}

/// The element type of a tensor buffer, named the way NumPy names it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dtype {
    /// One byte per value, 0 or 1.
    Bool,
    /// An unsigned whole number of `bits` bits.
    Uint(u32),
    /// A signed whole number of `bits` bits.
    Int(u32),
    /// A 32-bit float.
    Float32,
}

impl fmt::Display for Dtype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Dtype::Bool => f.write_str("bool"),
            Dtype::Uint(bits) => write!(f, "uint{bits}"),
            Dtype::Int(bits) => write!(f, "int{bits}"),
            Dtype::Float32 => f.write_str("float32"),
        }
    }
}

impl Dtype {
    /// How many bytes one value takes.
    pub const fn size(self) -> usize {
        match self {
            Dtype::Bool => 1,
            Dtype::Uint(bits) | Dtype::Int(bits) => (bits / 8) as usize,
            Dtype::Float32 => 4,
        }
    }

    /// The name the safetensors header spells it with.
    pub const fn safetensors(self) -> &'static str {
        match self {
            Dtype::Bool => "BOOL",
            Dtype::Uint(8) => "U8",
            Dtype::Uint(16) => "U16",
            Dtype::Uint(32) => "U32",
            Dtype::Uint(64) => "U64",
            Dtype::Int(8) => "I8",
            Dtype::Int(16) => "I16",
            Dtype::Int(32) => "I32",
            Dtype::Int(64) => "I64",
            Dtype::Float32 => "F32",
            _ => "U8",
        }
    }
}

/// How a fact is packed when the caller asks for the packed layout.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Packing {
    /// The expanded form already, value for value.
    Plain,
    /// One `uint64` bitboard: bit *i* is square *i*, a1 = bit 0.
    Bitboard,
    /// One `uint8` mask: bit *i* is file *i*, file a = bit 0.
    FileMask,
    /// Every value one bit, eight to a byte, the whole array flattened in C
    /// order and the last byte padded with zeroes.
    Bits,
}

impl FieldType {
    /// The element type a tensor of this fact holds.
    pub fn dtype(&self) -> Dtype {
        match self {
            FieldType::Bool | FieldType::SquareSet | FieldType::FileSet => Dtype::Bool,
            FieldType::Int {
                signed: false,
                bits,
            } => Dtype::Uint(*bits),
            FieldType::Int { signed: true, bits } => Dtype::Int(*bits),
            FieldType::Float => Dtype::Float32,
            // A code, and −1 where the value is absent, so it must be signed
            // and wide enough for a square.
            FieldType::Optional(_) => Dtype::Int(16),
            FieldType::Named(_) => Dtype::Uint(8),
            FieldType::Fixed { item, .. } | FieldType::ByColour(item) => item.dtype(),
        }
    }

    /// The shape of one position's value, the batch axis left off.
    pub fn shape(&self) -> Vec<usize> {
        match self {
            FieldType::Bool
            | FieldType::Int { .. }
            | FieldType::Float
            | FieldType::Named(_)
            | FieldType::Optional(_) => Vec::new(),
            FieldType::SquareSet => vec![64],
            FieldType::FileSet => vec![8],
            FieldType::Fixed { len, item, .. } => {
                let mut shape = vec![*len];
                shape.extend(item.shape());
                shape
            }
            FieldType::ByColour(item) => {
                let mut shape = vec![2];
                shape.extend(item.shape());
                shape
            }
        }
    }

    /// How many values one position contributes.
    pub fn width(&self) -> usize {
        self.shape().iter().product::<usize>().max(1)
    }

    /// How the fact is packed in the packed layout.
    pub fn packing(&self) -> Packing {
        match self {
            FieldType::SquareSet => Packing::Bitboard,
            FieldType::FileSet => Packing::FileMask,
            FieldType::Bool => Packing::Bits,
            FieldType::Fixed { item, .. } | FieldType::ByColour(item) => item.packing(),
            _ => Packing::Plain,
        }
    }

    /// The element type of the packed form.
    pub fn packed_dtype(&self) -> Dtype {
        match self.packing() {
            Packing::Bitboard => Dtype::Uint(64),
            Packing::FileMask | Packing::Bits => Dtype::Uint(8),
            Packing::Plain => self.dtype(),
        }
    }

    /// The shape of one position's packed value, or `None` where the packing
    /// runs across the batch and no per-position shape exists.
    pub fn packed_shape(&self) -> Option<Vec<usize>> {
        match self.packing() {
            Packing::Bitboard | Packing::FileMask => {
                let mut shape = self.shape();
                shape.pop();
                Some(shape)
            }
            Packing::Bits => None,
            Packing::Plain => Some(self.shape()),
        }
    }

    /// What each axis of the expanded shape means, outermost first. Empty for
    /// a fact that is one value.
    pub fn order(&self) -> String {
        let mut axes = Vec::new();
        self.axes(&mut axes);
        axes.join("; ")
    }

    fn axes(&self, out: &mut Vec<String>) {
        match self {
            FieldType::ByColour(item) => {
                out.push(format!("axis {}: white, black", out.len()));
                item.axes(out);
            }
            FieldType::Fixed { item, run, .. } => {
                out.push(format!("axis {}: {}", out.len(), run.order()));
                item.axes(out);
            }
            FieldType::SquareSet => out.push(format!(
                "axis {}: the 64 squares, a1 first, then b1 through h1, then a2",
                out.len()
            )),
            FieldType::FileSet => {
                out.push(format!("axis {}: the eight files, file a first", out.len()))
            }
            _ => {}
        }
    }

    /// What the values themselves mean, where reading them needs more than the
    /// shape: the codes of a named value, or that −1 stands for absent.
    pub fn encoding(&self) -> Option<String> {
        match self {
            FieldType::Named(named) => Some(format!(
                "{} as its code: {}",
                named.type_name(),
                named
                    .codes()
                    .iter()
                    .enumerate()
                    .map(|(code, name)| format!("{code} = {name}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            FieldType::Optional(item) => Some(match item.encoding() {
                Some(inner) => format!("−1 where the value is absent, otherwise {inner}"),
                None => "−1 where the value is absent".to_string(),
            }),
            FieldType::Fixed { item, .. } | FieldType::ByColour(item) => item.encoding(),
            _ => None,
        }
    }

    /// Whether the positions of the value carry a meaning the fact's own
    /// wording has to state: a fixed run whose order is not in its type.
    pub fn needs_order_note(&self) -> bool {
        match self {
            FieldType::Fixed { run, .. } => *run == Run::Array,
            FieldType::ByColour(item) | FieldType::Optional(item) => item.needs_order_note(),
            _ => false,
        }
    }

    /// Whether the value is a run of positions at all, in any of its forms.
    pub fn is_run(&self) -> bool {
        match self {
            FieldType::Fixed { .. } => true,
            FieldType::ByColour(item) | FieldType::Optional(item) => item.is_run(),
            _ => false,
        }
    }
}
