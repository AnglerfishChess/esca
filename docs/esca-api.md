# esca API sketch

`esca` is the position-facts library: a Rust crate at the root of this
repository and a Python package of the same name built from it. Terms used below are
defined in [`esca-vocabulary.md`](esca-vocabulary.md); every fact, its type and
its meaning in [`features.md`](features.md).

## How to read this

Rust and Python are one API under one set of names. The signature blocks below
are Rust.

- **A Rust reader** starts here and reads §1–§8 in order: variant, position,
  move, game, facts, catalogue, JSON form, tensors.
- **A Python reader** starts at [§11, the Python surface](#11-python-surface),
  which is the whole of it in one place, and comes back to a numbered section
  for what a name means.

Four things exist on the Python surface only: `uci.Engine.analyse`,
`uci.AsyncEngine` (`uci::tokio::Engine` is the Rust counterpart, not the same
type), `Facts.to_dict()` (Rust serialises the same bytes through `serde`), and
`esca.tensors`, whose arrays are NumPy's. Three exist on the Rust surface only:
the `Scratch` buffers and the `*_in` methods that reuse them, `uci::tokio`, and
`facts::tensors::Batch`.

Board representation and move generation come from `cozy-chess`. Its types
are an implementation detail: no `cozy_chess` item appears in any signature,
constant or error of esca's public API, in either language.

Cargo features:

| Feature | Default | Effect |
|---|---|---|
| `python` | no | The PyO3 module, built by maturin. |
| `lichess` | no | The `lichess` module: streaming reader for the evaluation dump. |
| `pgn` | no | The `pgn` module: reading and writing games. |
| `uci` | no | The `uci` module: the protocol as values, and engines as subprocesses. |
| `tokio` | no | `uci::tokio`: the same client on a tokio runtime. Implies `uci`. |
| `polyglot` | no | The `polyglot` module: opening books read, picked from and built. |
| `openings` | no | The `openings` module: the bundled ECO code and name of a position. |
| `serde` | no | `Serialize` for the facts and the board vocabulary, `Serialize`/`Deserialize` for `Position` and `Move`, and `facts::schema::json_schema()`. |
| `tensors` | no | `facts::tensors`: a run of positions as one typed array per fact. Implies `serde`. |

`Position::polyglot_key` needs no feature: the key is part of the position,
and the `polyglot` and `openings` modules are indexed by it.

---

## 1. Variant

The trait that *is* a variant. One implementation per set of rules; adding
rules means adding an implementation.

```rust
pub trait Variant: Send + Sync + 'static {
    /// Stable identifier, lower-case, as used in PGN and UCI: "chess", "chess960".
    fn name(&self) -> &'static str;

    /// A position to start a game from. `Classic` ignores the seed and
    /// returns the standard array; `Chess960` returns arrangement
    /// `seed % 960`.
    fn start_position(&self, seed: u64) -> Position;

    /// Appends every legal move in `pos` to `out`.
    fn legal_moves(&self, pos: &Position, out: &mut MoveList);
    fn is_legal(&self, pos: &Position, mv: Move) -> bool;

    /// The position after `mv`. Panics if `mv` is not legal in `pos`.
    fn play(&self, pos: &Position, mv: Move) -> Position;

    /// Terminal state of `pos` judged from the position alone: checkmate,
    /// stalemate, insufficient material, and the automatic move-count draw.
    /// Repetition and claimable draws need history and belong to `Game`.
    fn outcome(&self, pos: &Position) -> Option<Outcome>;

    fn promotion_roles(&self) -> &'static [Role];

    /// Whether `pos` is a position this variant can reach and play on.
    fn validate(&self, pos: &Position) -> Result<(), PositionError>;

    /// Castling is written as `style` asks; every other move is unaffected.
    fn move_to_uci(&self, pos: &Position, mv: Move, style: CastlingOutput) -> String;
    /// Accepts both castling spellings whatever the output style.
    fn move_from_uci(&self, pos: &Position, text: &str) -> Result<Move, MoveParseError>;
    fn move_to_san(&self, pos: &Position, mv: Move) -> String;
    fn move_from_san(&self, pos: &Position, text: &str) -> Result<Move, MoveParseError>;
}

/// How castling is spelled in UCI text.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CastlingOutput {
    /// `e1h1`. Correct in every variant.
    #[default]
    KingToRook,
    /// `e1g1`. Classic geometry only, and `Chess960` writes king-to-rook
    /// whatever the style asks for: a two-square destination there can be
    /// another legal king move, or the king's own origin.
    KingTwoSquares,
}

pub struct Classic;
pub struct Chess960;

pub static CLASSIC: Classic = Classic;
pub static CHESS960: Chess960 = Chess960;

/// `Classic`, shared. The default wherever a variant is optional.
pub fn classic() -> Arc<dyn Variant>;
pub fn chess960() -> Arc<dyn Variant>;
```

`Game` holds `Arc<dyn Variant>`: one concrete `Game` type, variants selectable
at runtime, cloning free, and the same shape usable from Python. Dispatch is
one indirect call per rules question, amortised over the work behind it; code
that cannot pay it calls a concrete implementation (`Classic.legal_moves(…)`)
directly.

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Outcome {
    Checkmate { winner: Colour },
    Stalemate,
    InsufficientMaterial,
    SeventyFiveMoves,
    FivefoldRepetition,
}

impl Outcome {
    /// `checkmate`, `stalemate`, `insufficient_material`,
    /// `seventy_five_moves`, `fivefold_repetition`.
    pub const fn name(self) -> &'static str;
    /// The colour the result favours; `None` for the draws.
    pub const fn winner(self) -> Option<Colour>;
    pub fn describe(self) -> String;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DrawClaim { FiftyMoves, ThreefoldRepetition }

impl DrawClaim {
    /// `fifty_moves`, `threefold_repetition`.
    pub const fn name(self) -> &'static str;
    pub fn describe(self) -> &'static str;
}

/// The name, for both.
impl fmt::Display for Outcome { /* … */ }
impl fmt::Display for DrawClaim { /* … */ }
```

Each draw condition has one spelling everywhere it is named — in `Outcome`, in
`DrawClaim`, in `explain`'s two draw enums, in the Python strings and in the
MCP payloads: `stalemate`, `insufficient_material`, `fivefold_repetition`,
`seventy_five_moves`, `threefold_repetition`, `fifty_moves`.

---

## 2. Position

Immutable and variant-agnostic: placement and state, no rules. A question that
needs rules is asked of the position and takes the variant that answers it; a
question that needs the moves played before it — repetition, claimable draws,
the opening — belongs to a `Game`.

```rust
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Position { /* … */ }

impl Position {
    /// Six-field FEN; also accepts a four-field FEN (EPD), taking
    /// halfmove clock 0, full move 1, and marking the clocks unknown.
    pub fn from_fen(fen: &str) -> Result<Position, FenError>;
    pub fn fen(&self) -> String;
    pub fn epd(&self) -> String;

    pub fn side_to_move(&self) -> Colour;
    pub fn piece_at(&self, sq: Square) -> Option<Piece>;
    pub fn by_role(&self, role: Role) -> SquareSet;
    pub fn by_colour(&self, colour: Colour) -> SquareSet;
    pub fn by_piece(&self, piece: Piece) -> SquareSet;
    pub fn occupied(&self) -> SquareSet;
    pub fn king_of(&self, colour: Colour) -> Square;

    pub fn castling_rights(&self) -> CastlingRights;
    pub fn en_passant(&self) -> Option<Square>;
    pub fn in_check(&self) -> bool;
    pub fn halfmove_clock(&self) -> u32;
    pub fn fullmove_number(&self) -> u32;
    /// False when the position came from a four-field FEN.
    pub fn clocks_known(&self) -> bool;

    /// Zobrist key: equal for equal placement, side, castling rights and
    /// en-passant square; independent of the clocks. Valid as an identity
    /// within one process run — not across runs, and not stored.
    pub fn key(&self) -> Key;

    /// The Polyglot key, as §13 defines it: fixed by that format, so it is
    /// the same number in every run and in every program that implements it.
    pub fn polyglot_key(&self) -> u64;

    /// Static exchange evaluation, in value units, as `features.md` §1
    /// defines it: of the unit on `sq`, and of a move of this position.
    pub fn see(&self, sq: Square) -> i32;
    pub fn see_capture(&self, mv: Move) -> i32;

    /// Colours swapped and ranks flipped.
    pub fn mirrored(&self) -> Position;

    /// Board, side to move and state, for a human reader. Text not stable.
    pub fn summary(&self) -> String;
}

/// The rules questions, each answered by the `variant` that defines it.
impl Position {
    pub fn legal_moves(&self, variant: &dyn Variant) -> MoveList;
    pub fn is_legal(&self, variant: &dyn Variant, mv: Move) -> bool;
    /// Panics if `mv` is not legal here.
    pub fn play(&self, variant: &dyn Variant, mv: Move) -> Position;

    /// The SAN text of `mv`, with its check or mate mark.
    pub fn san(&self, variant: &dyn Variant, mv: Move) -> String;
    pub fn parse_san(&self, variant: &dyn Variant, text: &str) -> Result<Move, MoveParseError>;
    /// The UCI text of `mv`, castling spelled as `style` asks.
    pub fn uci(&self, variant: &dyn Variant, mv: Move, style: CastlingOutput) -> String;
    /// Either castling spelling is read.
    pub fn parse_uci(&self, variant: &dyn Variant, text: &str) -> Result<Move, MoveParseError>;

    /// The terminal condition this position has reached on its own. Repetition
    /// needs the moves played, so it is `Game::outcome`.
    pub fn outcome(&self, variant: &dyn Variant) -> Option<Outcome>;

    /// Facts of this position under `variant`.
    pub fn facts(&self, variant: &dyn Variant) -> Facts;
    /// Every legal move with what it does.
    pub fn annotated_moves(&self, variant: &dyn Variant) -> MoveList<AnnotatedMove>;
    /// The same two, reusing buffers; nothing is allocated.
    pub fn facts_in(&self, variant: &dyn Variant, scratch: &mut Scratch) -> Facts;
    pub fn annotated_moves_in(&self, variant: &dyn Variant, scratch: &mut Scratch)
        -> MoveList<AnnotatedMove>;
}

/// The six-field FEN, and the FEN read back.
impl fmt::Display for Position { /* … */ }
impl FromStr for Position { type Err = FenError; /* … */ }

/// A Zobrist key.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Key(u64);

impl Key {
    pub fn get(self) -> u64;
}

/// An evaluation of a position. Positive favours the side to move; `Mate(n)`
/// is a forced mate in *n* moves, negative when it is against the side to move.
/// The UCI spelling is its `Display`: `cp 25`, `mate -3`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Score { Cp(i32), Mate(i32) }
```

Scores order the way a searcher wants them: a mate to give beats every
centipawn score and the sooner of two such mates is the greater; a mate to
suffer loses to every centipawn score and the later of two such mates is the
greater. So `slice.sort()` and `iter.max()` mean what they read as.

```rust
/// Chess960-compatible: each right names the rook's starting file.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct CastlingRights { /* [[Option<File>; 2]; 2] */ }

impl CastlingRights {
    pub const EMPTY: CastlingRights;
    /// The rights of a classic starting array.
    pub const CLASSIC: CastlingRights;
    /// The file of the rook `colour` may castle on `wing` with.
    pub fn of(&self, colour: Colour, wing: Wing) -> Option<File>;
    pub fn short(&self, colour: Colour) -> Option<File>;
    pub fn long(&self, colour: Colour) -> Option<File>;
    pub fn any(&self, colour: Colour) -> bool;
    pub fn is_empty(&self) -> bool;
    /// `KQkq` when the rook files are the classic ones, `AHah` otherwise.
    pub fn to_fen_field(&self) -> String;
}
```

---

## 3. Move

```rust
#[derive(Clone, Copy)]
pub struct Move { /* from, to, promotion, kind */ }

impl Move {
    pub fn from(&self) -> Square;
    /// For castling, the rook's square: unambiguous in every variant.
    pub fn to(&self) -> Square;
    pub fn promotion(&self) -> Option<Role>;
    pub fn kind(&self) -> MoveKind;
    pub fn is_capture(&self) -> bool;
    pub fn is_castling(&self) -> bool;
    pub fn is_en_passant(&self) -> bool;

    /// The UCI text, castling spelled as `output` asks.
    pub fn uci(&self, output: CastlingOutput) -> String;
}

/// Origin, destination and promotion only: the flags say what the move did in
/// one position, and the same move is the same move in another.
impl PartialEq for Move { /* … */ }
impl Eq for Move {}
impl Hash for Move { /* … */ }

/// The UCI text, castling king-to-rook.
impl fmt::Debug for Move { /* … */ }

/// Origin, destination and promotion letter, four or five characters.
impl FromStr for Move { type Err = ParseError; /* … */ }

/// `kind` is the first case that applies, in the order castling, en passant,
/// promotion, capture, quiet; `is_capture` is true for a capturing promotion
/// and for en passant as well.
pub enum MoveKind { Quiet, Capture, EnPassant, Castling, Promotion }

/// Inline storage for the largest legal move count; never allocates.
/// Derefs to `[T]`, so `iter`, `first`, `contains` and `sort_by_key` are there.
pub struct MoveList<T = Move> { /* … */ }

impl<T: Copy> MoveList<T> {
    pub fn new() -> MoveList<T>;
    pub fn clear(&mut self);
    pub fn push(&mut self, item: T);
    pub fn as_slice(&self) -> &[T];
    pub fn as_mut_slice(&mut self) -> &mut [T];
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

A move has no `Display` and no `Default`: the spelling is a choice, so it is
made explicitly, by `Move::uci` for a move standing alone or by the `Variant`,
which owns the castling encoding and SAN disambiguation. A move read with
`FromStr` carries no kind — only a position says what a move captures and
whether it castles — so a move to *play* comes from `Variant::move_from_uci`,
`Position::parse_uci` or `Game::play_uci`. It still compares equal to the
generated move it names, which is what makes a parsed move usable as a key and
as a lookup.

---

## 4. Game

Owns the variant and the history, and is therefore the only thing that can
answer repetition and claim questions.

```rust
pub struct Game { /* … */ }

impl Game {
    /// `variant.start_position(0)`.
    pub fn new(variant: Arc<dyn Variant>) -> Game;
    pub fn with_seed(variant: Arc<dyn Variant>, seed: u64) -> Game;
    pub fn from_position(variant: Arc<dyn Variant>, start: Position) -> Result<Game, PositionError>;
    pub fn from_fen(variant: Arc<dyn Variant>, fen: &str) -> Result<Game, FenError>;

    pub fn variant(&self) -> &dyn Variant;

    /// Castling spelling for this game's UCI output; `KingToRook` until set.
    pub fn castling_output(&self) -> CastlingOutput;
    pub fn set_castling_output(&mut self, style: CastlingOutput);
    pub fn move_to_uci(&self, mv: Move) -> String;
    pub fn move_to_san(&self, mv: Move) -> String;
    pub fn position(&self) -> &Position;
    pub fn start_position(&self) -> &Position;
    pub fn moves(&self) -> &[Move];
    /// Every position from the start to the current one.
    pub fn positions(&self) -> impl Iterator<Item = &Position>;
    pub fn ply(&self) -> u32;

    pub fn legal_moves(&self) -> MoveList;
    pub fn annotated_moves(&self) -> MoveList<AnnotatedMove>;
    pub fn play(&mut self, mv: Move) -> Result<(), IllegalMove>;
    pub fn play_uci(&mut self, text: &str) -> Result<(), MoveParseError>;
    pub fn play_san(&mut self, text: &str) -> Result<(), MoveParseError>;
    pub fn undo(&mut self) -> Option<Move>;

    /// Automatic terminal conditions, repetition included.
    pub fn outcome(&self) -> Option<Outcome>;
    /// Draws a player could claim now, in no particular order.
    pub fn claims(&self) -> &[DrawClaim];
    /// How often the current position has occurred in this game.
    pub fn repetitions(&self) -> u32;

    pub fn facts(&self) -> Facts;
    pub fn facts_in(&self, scratch: &mut Scratch) -> Facts;
}
```

### Errors

Every one of these is `Copy`, `Display` and `std::error::Error`.

```rust
pub enum FenError {
    FieldCount, Placement, SideToMove, Castling,
    EnPassant, HalfmoveClock, FullmoveNumber,
    /// The placement is not a legal chess position.
    Position,
}

/// Why a position is not one a variant can play on.
pub enum PositionError { CastlingRights }

pub enum MoveParseError {
    /// Not shaped like a move at all.
    Syntax,
    /// Well formed, but names no legal move here.
    Illegal,
    /// Names more than one legal move.
    Ambiguous,
}

pub struct IllegalMove;
```

---

## 5. Facts

Grouped plain structs with public fields; the accessor *is* the field. Every
value that differs between the two sides is a `ByColour<T>` — `.white`,
`.black`, `.of(colour)` or `[colour]`. Where a fact is about the side to move
rather than about a colour it is a plain value, and its meaning says so.

Each fact is typed for what it is *for*, not reduced to the plainest type that
would hold it: a set of squares stays a `SquareSet`, a per-file count is a
`PerFile<u8>` whose order is in its type, and a run whose order the type cannot
carry — the three files around a king — states it in its own meaning.
[`features.md`](features.md) has every field, its type, its `dtype`, its
`shape` and its meaning, and is generated from these definitions.

### The vocabulary types

`Colour`, `Role`, `Piece`, `File`, `Rank`, `Square` and `Move` each write
themselves as the text esca reads back, and `Debug` is that same text — `{:?}`
on a square is `e4`, not a wrapper around an index. `FromStr` parses it:

| `Colour` | `Role` | `Piece` | `File` | `Rank` | `Square` | `Move` |
|---|---|---|---|---|---|---|
| `w` | `n` | `N` | `d` | `4` | `e4` | `e2e4` |

```rust
/// The text does not name a value of the type it was read as.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ParseError {
    /// `colour`, `role`, `piece`, `file`, `rank`, `square` or `move`.
    pub expected: &'static str,
    pub input: String,
}

/// Which side of the board a castling is played on, named after where the king
/// lands. The rook it castles with is the outermost one on that side, whatever
/// file it starts on, so the name holds in Chess960 too.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Wing { Short, Long }

impl Wing {
    pub const ALL: [Wing; 2];
    /// `G` for `Short`, `C` for `Long`.
    pub const fn king_file(self) -> File;
    /// `F` for `Short`, `D` for `Long`.
    pub const fn rook_file(self) -> File;
    /// `short` or `long`, which is its `Display` too.
    pub const fn name(self) -> &'static str;
}
```

`Wing` is a crate-root type — `esca::Wing`, re-exported as `esca::explain::Wing`
for the layer that reads castling.

### The per-something containers

```rust
pub struct ByColour<T> {
    pub white: T,
    pub black: T,
}

impl<T> ByColour<T> {
    pub const fn new(white: T, black: T) -> ByColour<T>;
    pub const fn of(&self, colour: Colour) -> &T;
    pub fn map<U>(self, f: impl FnMut(T) -> U) -> ByColour<U>;
    pub fn from_fn(f: impl FnMut(Colour) -> T) -> ByColour<T>;
    /// Both values with the colour each belongs to, White first.
    pub fn iter(&self) -> impl Iterator<Item = (Colour, &T)>;
}

impl<T> IntoIterator for ByColour<T> { type Item = (Colour, T); /* … */ }
impl<T> Index<Colour> for ByColour<T> { /* … */ }
impl<T> IndexMut<Colour> for ByColour<T> { /* … */ }

/// Eight values, file a first. `PerRank<T>` is the same, rank 1 first, and
/// `PerRole<T>` is six, pawn first, in `Role::ALL` order.
pub struct PerFile<T>(/* [T; 8] */);

impl<T> PerFile<T> {
    pub const fn new(values: [T; 8]) -> PerFile<T>;
    pub const fn of(&self, file: File) -> &T;
    pub fn into_array(self) -> [T; 8];
    pub fn iter(&self) -> impl Iterator<Item = &T>;
    pub fn map<U>(self, f: impl FnMut(T) -> U) -> PerFile<U>;
}

impl<T: Copy + Default> PerFile<T> {
    pub fn from_fn(f: impl FnMut(File) -> T) -> PerFile<T>;
}

impl<T> Index<File> for PerFile<T> { /* … */ }
impl<T> IndexMut<File> for PerFile<T> { /* … */ }
```

`of` answers with a reference whatever `T` is, so a `ByColour<Vec<_>>` reads the
same way a `ByColour<u8>` does; a `Copy` value is copied out by the caller.

```rust
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Facts {
    pub placement: PlacementFacts,
    pub state: StateFacts,
    pub history: HistoryFacts,
    pub material: MaterialFacts,
    pub pawns: PawnFacts,
    pub pieces: PieceFacts,
    pub king: KingFacts,
    pub mobility: MobilityFacts,
    pub attacks: AttackFacts,
    pub exchange: ExchangeFacts,
    pub threats: ThreatFacts,
    pub tactics: TacticsFacts,
    pub endgame: EndgameFacts,
    pub maps: MapFacts,
}

pub struct PlacementFacts {
    pub units: ByColour<SquareSet>,
    pub pawns: ByColour<SquareSet>,
    pub knights: ByColour<SquareSet>,
    pub bishops: ByColour<SquareSet>,
    pub rooks: ByColour<SquareSet>,
    pub queens: ByColour<SquareSet>,
    pub kings: ByColour<SquareSet>,
}

impl PlacementFacts {
    pub fn of(&self, colour: Colour, role: Role) -> SquareSet;
}

/// The values for the side not to move are read after a null move; when the
/// side to move is in check that move does not exist and they are zero.
pub struct ExchangeFacts {
    pub see_best_capture: ByColour<i32>,
    pub see_positive_capture_count: ByColour<u16>,
    pub see_equal_capture_count: ByColour<u16>,
    pub see_positive_total: ByColour<i32>,
}

pub enum Opposition { Direct, Distant }
pub enum DrawishMaterial { TwoKnights, WrongBishop, OppositeBishops }
pub enum CastledSide { Short, Long }

pub struct EndgameFacts {
    pub king_centralisation: ByColour<u8>,
    pub race_plies: ByColour<u8>,
    pub opposition: Option<Opposition>,
    pub key_square_occupied: ByColour<bool>,
    pub wrong_colour_bishop: ByColour<bool>,
    pub drawish_material: Option<DrawishMaterial>,
}

/// A legal move and its facts.
pub struct AnnotatedMove {
    pub mv: Move,
    pub facts: MoveFacts,
}

impl Facts {
    /// The name of the variant the facts were computed under: `chess` or
    /// `chess960`.
    pub fn variant_name(&self) -> &'static str;
    /// The colour to move.
    pub fn side_to_move(&self) -> Colour;
    /// The names of the groups, in the order they are declared.
    pub fn groups() -> impl Iterator<Item = &'static str>;
    /// The group of that name, read without naming its type.
    pub fn group(&self, name: &str) -> Option<&dyn FactsGroup>;
    /// A page of prose: material, structure, king safety, threats. Text not stable.
    pub fn summary(&self) -> String;
}

/// The counts carry the numbers; these read a question off them.
impl TacticsFacts {
    pub fn check_available(&self) -> ByColour<bool>;
    pub fn safe_check_available(&self) -> ByColour<bool>;
    pub fn promotion_available(&self) -> ByColour<bool>;
    pub fn safe_promotion_available(&self) -> ByColour<bool>;
    pub fn capture_available(&self) -> ByColour<bool>;
    pub fn fork_available(&self) -> ByColour<bool>;
    pub fn pin_creation_available(&self) -> ByColour<bool>;
    pub fn only_moves(&self) -> ByColour<bool>;
}

impl KingFacts {
    /// The weight besieging each king's ring, less the weight defending it.
    pub fn ring_attacker_surplus(&self) -> ByColour<i16>;
}

/// Reusable buffers. One per thread; a search keeps one per node stack.
pub struct Scratch { /* … */ }
impl Scratch { pub fn new() -> Scratch; }
```

`Facts` describes one position and holds no move data: what each legal move
*does* is a separate answer, `Position::annotated_moves` or
`Game::annotated_moves`, one `MoveFacts` per move. That is why the JSON form
(§7) and the tensor form (§8) hold no move data either.

Facts are computed from the position and the variant's legal move list, so a
fact holds under every variant whose rules its definition assumes. A `Variant`
supplies rules, never facts.

The `history` facts other than the halfmove clock come from `Game::facts` and
`Game::facts_in`, which have the history; `Position::facts` and
`Position::facts_in` emit them as zero, `known` included.

### Walking a group without knowing it

```rust
/// One group of facts, read without naming its type.
pub trait FactsGroup {
    fn entry(&self) -> &'static Group;
    fn value(&self, field: &str) -> Option<FieldValue>;
    /// Every field's name and value, in catalogue order.
    fn fields(&self) -> Vec<(&'static str, FieldValue)>;
}

/// One fact's value, read without naming its type: the shapes `FieldType`
/// describes and the JSON form writes.
pub enum FieldValue {
    Bool(bool),
    Int(i64),
    Float(f32),
    /// One of a fixed list of names, as its text: `n`, `d`, `e4`, `short`.
    Name(&'static str),
    Squares(SquareSet),
    Files(FileSet),
    /// An `Option` that is absent.
    Absent,
    /// A fixed run, in the order the field's type or meaning states.
    Run(Vec<FieldValue>),
    ByColour { white: Box<FieldValue>, black: Box<FieldValue> },
}
```

```rust
let facts = position.facts(&CLASSIC);
for (name, value) in facts.group("material").unwrap().fields() {
    println!("{name}: {value:?}");
}
```

### Worked example: which files carry a passed pawn

```rust
use esca::{CLASSIC, File, Position};

let position = Position::from_fen("8/pp3ppk/8/3P4/8/8/PP3PPK/8 w - - 0 1").unwrap();
let facts = position.facts(&CLASSIC);

let files: Vec<File> = facts.pawns.passed.white.files().into_iter().collect();
assert_eq!(files, [File::D]);
assert!(facts.pawns.passed.black.is_empty());
```

```python
import esca

facts = esca.Position.from_fen("8/pp3ppk/8/3P4/8/8/PP3PPK/8 w - - 0 1").facts()

assert list(facts.pawns.passed.white.files) == ["d"]
assert not facts.pawns.passed.black

assert facts.to_dict()["pawns"]["passed"] == {"white": ["d5"], "black": []}
```

---

## 6. The catalogue

Every group, field, type and meaning as data. `docs/features.md` is generated
from it, and the MCP server serves it as `esca://facts-catalogue`.

```rust
pub struct Field {
    pub name: &'static str,
    /* … */
}

impl Field {
    /// The field's type, as the declaration spells it.
    pub const fn type_name(&self) -> &'static str;
    /// What the field holds, in one sentence.
    pub fn meaning(&self) -> &'static str;
}

pub struct Group {
    pub name: &'static str,
    pub fields: &'static [Field],
    /* … */
}

impl Group {
    pub fn meaning(&self) -> &'static str;
    pub fn field(&self, name: &str) -> Option<&'static Field>;
}

/// Every group, in the order `Facts` declares them.
pub fn catalogue() -> &'static [Group];

/// The catalogue as the Markdown of `docs/features.md`.
pub fn markdown() -> String;
```

Each group type carries its own entry, so a field and its documentation cannot
disagree: `PawnFacts::GROUP`, `PawnFacts::FIELDS` and `PawnFacts::ENTRY`, and
`MoveFacts::FIELDS` for a move.

`Field::kind()` parses the declared type into a tree, and everything generated
about the facts is read off that one tree:

```rust
pub mod types {
    pub enum FieldType {
        Bool,
        Int { signed: bool, bits: u32 },
        Float,
        Named(Named),          // Role, File, Square, CastledSide, Opposition, DrawishMaterial
        SquareSet,
        FileSet,
        Optional(Box<FieldType>),
        Fixed { len: usize, item: Box<FieldType>, run: Run },  // [T; n], PerFile, PerRank
        ByColour(Box<FieldType>),
    }

    impl FieldType {
        pub fn dtype(&self) -> Dtype;
        pub fn shape(&self) -> Vec<usize>;
        pub fn order(&self) -> String;      // what each axis means
        pub fn encoding(&self) -> Option<String>;  // the codes, and that −1 is absent
    }
}

impl Field {
    pub fn kind(&self) -> FieldType;
    pub fn dtype(&self) -> Dtype;
    /// The shape of one position's value, the batch axis left off.
    pub fn shape(&self) -> &'static [usize];
}
```

Everything a field or a group says about itself is fixed at compile time and
handed out as `&'static`, so walking the catalogue allocates nothing.

`cargo run --example catalogue --all-features` rewrites all of it:
`docs/features.md`, `docs/facts.schema.json`, the generated section of
`python/esca/_esca.pyi`, the golden JSON of `tests/data/facts/` and the golden
tensor batch beside it. A test in each language fails when any of them is
stale.

---

## 7. The JSON form

One wire format, written by Rust's `serde::Serialize` (feature `serde`) and by
Python's `to_dict()`, byte for byte the same:

| value | written as |
|---|---|
| a group, `Facts`, `MoveFacts` | an object, keys in declaration order |
| `ByColour<T>` | `{"white": …, "black": …}` |
| `SquareSet` | the list of square names in board order: a1, b1 … h1, then a2 |
| `FileSet` | the list of file letters, file a first |
| `PerFile<T>`, `PerRank<T>`, `[T; n]` | a list of `n`, in the order stated |
| `Option<T>` | `null` when absent |
| `Role`, `File` | the letter; `Square` its name |
| `Opposition`, `DrawishMaterial`, `CastledSide` | the snake-case name |

`Facts` writes its fourteen groups and no move data; a `MoveFacts` writes the
same way when it is serialised on its own.

`docs/facts.schema.json` is that format as a JSON Schema (draft 2020-12),
generated from the same catalogue. `tests/data/facts/<name>.json` holds the
answer for each position `tests/data/facts/positions.json` names; the Rust
suite writes and compares them, the Python suite compares its own `to_dict()`
against the very same files and validates both against the schema.

```rust
let facts = position.facts(&CLASSIC);
let text = serde_json::to_string_pretty(&facts)?;
```

```python
facts.to_dict()["pawns"]["passed"]  # {"white": ["d5"], "black": []}
facts.pawns.to_dict()["count_by_file"]  # {"white": [1, 1, 0, 1, 0, 1, 1, 0], …}
```

---

## 8. Tensors (feature `tensors` in Rust, extra `tensors` in Python)

One typed array per fact, batch axis first. A fact keeps the width and sign it
was declared with; nothing is scaled, normalised or cast to a float, and no
value is written in the mover's view.

| fact type | dtype | shape |
|---|---|---|
| `bool` | `bool` | `()` |
| `u8`, `i8`, `u16`, `u32`, `i32` | the same width and sign | `()` |
| `f32` (only `king.tropism`) | `float32` | `()` |
| `Option<T>` | `int16`, −1 where absent | `()` |
| `[T; n]`, `PerFile<T>`, `PerRank<T>` | as `T` | `(n,)` |
| `FileSet` | `bool` | `(8,)`, file a first |
| `SquareSet` | `bool` | `(64,)`, a1 first |
| a named value | `uint8`, its code | `()` |
| `ByColour<T>` | as `T` | `(2, …)`, White then Black |

A named value's code is its position in the list `features.md` gives:
`p` is 0 and `k` is 5, `direct` is 0 and `distant` is 1. The packed form keeps
the same facts as bits: a `SquareSet` is one `uint64` bitboard with a1 as bit
0, a `FileSet` one `uint8` mask with file a as bit 0, and every other flag one
bit, eight to a byte over the array flattened in C order.

```rust
use esca::facts::tensors::{Batch, Form, layout};

pub fn layout() -> &'static Layout;      // one Entry per fact, in declaration order

pub struct Entry {
    pub name: &'static str,        // "pawns.passed"
    pub group: &'static str,
    pub field: &'static str,
    pub declared: &'static str,    // "ByColour<SquareSet>"
    pub dtype: Dtype,
    pub shape: &'static [usize],   // batch axis left off
    pub order: &'static str,       // "axis 1: white, black; axis 2: the 64 squares, a1 first, …"
    pub encoding: Option<&'static str>,
    pub packed_dtype: Dtype,
    pub packed_shape: Option<Vec<usize>>,
    pub meaning: &'static str,
}

impl Batch {
    pub fn from_positions(positions: &[Position], variant: &dyn Variant) -> Batch;
    pub fn from_games(games: &[Game]) -> Batch;
    pub fn len(&self) -> usize;
    pub fn form(&self) -> Form;
    pub fn layout(&self) -> &'static Layout;
    pub fn buffer(&self, name: &str) -> Option<&Buffer>;
    /// The shape one array has in this form, batch axis included.
    pub fn shape_of(&self, entry: &Entry) -> Vec<usize>;
    pub fn packed(&self) -> Batch;
    pub fn expanded(&self) -> Batch;
    pub fn to_safetensors(&self) -> Vec<u8>;
}
```

**The batch axis.** `Entry::shape` is one position's value, batch axis left
off; `Batch::shape_of` is the array as it is stored, batch axis included. A
`ByColour<SquareSet>` is `shape` `(2, 64)` and `shape_of` `(n, 2, 64)`.
`Entry::order` names the axes of the stored array, so it counts the batch axis
as axis 0 and starts at 1: "axis 1: white, black; axis 2: the 64 squares".
The Python `layout()` entries carry the same `shape` and `order`, and the
arrays `facts()` answers with are the `shape_of` ones.

```python
import esca.tensors as tensors

tensors.names()  # every array name, "group.field", in declaration order
tensors.names(["pawns"])  # the names of one group
tensors.layout()  # the same entries, as dicts, each with its `kind` type tree
arrays = tensors.facts(fens_or_games, groups=["pawns"], layout="expanded")
arrays["pawns.passed"].shape  # (n, 2, 64), dtype bool
arrays["pawns.count_by_file"][0, 0]  # array([1, 1, 0, 1, 0, 1, 1, 0], dtype=uint8)

packed = tensors.facts(fens, layout="packed")
packed["pawns.passed"].dtype  # uint64, one bitboard per side
tensors.expand(packed, len(fens))  # back to one value per element

tensors.save(arrays, path, metadata={"count": "2", "form": "expanded"})
tensors.load(path)  # {name: ndarray}
tensors.metadata(path)
```

The safetensors writing and reading are the format itself — an eight-byte
header length, a JSON header, then the raw bytes — written by hand on both
sides, so neither the crate nor the wheel takes a dependency for it. A Rust
test writes `tests/data/facts/batch.safetensors` and the Python suite reads it
back and recomputes it, so the two agree on every byte.

---

## 9. `lichess` — evaluation dump reader (feature `lichess`)

```rust
pub mod lichess {
    pub struct Record { pub epd: String, pub evals: Vec<Eval> }
    pub struct Eval { pub depth: u32, pub knodes: u64, pub pvs: Vec<Pv> }
    pub struct Pv { pub score: Score, pub line: String }

    impl Record {
        /// The four-field FEN parsed; `clocks_known()` is false.
        pub fn position(&self) -> Result<Position, FenError>;
    }
    impl Pv {
        pub fn best_move(&self, variant: &dyn Variant, pos: &Position)
            -> Result<Move, MoveParseError>;
    }

    /// Streams a Zstandard-compressed JSON-lines dump; never holds the file.
    pub fn read(path: &Path) -> io::Result<impl Iterator<Item = io::Result<Record>> + use<>>;
    /// Streams decompressed JSON lines; blank ones are skipped.
    pub fn read_from<R: BufRead>(reader: R) -> impl Iterator<Item = io::Result<Record>>;
}
```

Every `evals` entry of a record is exposed; choosing among depths is the
caller's policy.

The dump writes `cp` and `mate` from White's point of view; `Score` is
side-relative, so the reader negates the scores of a record whose position has
Black to move. A malformed line is one `Err` and the stream goes on. A small
share of dump rows — around one in ten thousand — describe placements no game
can reach, and `Record::position` returns `FenError::Position` for those.

---

## 10. Adding a variant

A variant is one more `Variant` implementation. `Position`, `Facts` and
`MoveFacts` are untouched, because `Position` is data and every fact derives
from the variant's own legal move list.

```rust
pub struct Horde;

impl Variant for Horde {
    fn name(&self) -> &'static str { "horde" }
    fn start_position(&self, _seed: u64) -> Position { /* … */ }
    fn legal_moves(&self, pos: &Position, out: &mut MoveList) { /* … */ }
    fn play(&self, pos: &Position, mv: Move) -> Position { /* … */ }
    fn outcome(&self, pos: &Position) -> Option<Outcome> { /* … also "no units left" */ }
    /* … */
}

let game = Game::new(Arc::new(Horde));
let facts = game.facts();     // unchanged code
```

The model a variant must fit: an 8×8 board, the six standard roles, two
colours, one unit per square. Rules beyond that need the model widened first.

---

## 11. Python surface

This is the whole Python surface. Every name in it is the name the Rust
sections above give the same thing; a numbered section says what it means.

Built from the same crate with feature `python`; distributed with `.pyi` stubs,
so every signature below is typed and checkable. Squares, roles, colours,
outcomes and castling styles are text on this surface; a colour is **written**
`"white"` or `"black"` everywhere it is answered, and **read** from either, or
from `"w"` or `"b"`, in any case. The classes are `Variant`, `SquareSet`, `Move`,
`Position`, `Game`, `Facts` with its groups, `ByColour`, `FileSet`,
`pgn.Game`, `polyglot.Book` with its `Entry`, `Raw` and
`Builder`, `openings.Opening`, `endings.Ending` with its class, verdict, technique and
evidence, `uci.Engine` and `uci.AsyncEngine` with their
`Limits`, `Option`, `Info` and `Answer`, and `uci.protocol` with `Command`,
`Message` and `Session`. `esca.tensors` is a module of its own, behind the
`tensors` extra.

**Method or property.** Cheap constant data a value already holds is a
property: `position.fen`, `move.origin`, `game.ply`. Anything that takes an
argument or does work is a method: `move.uci()`, `position.legal_moves()`,
`game.outcome()`. So `Move.uci` is a method — it takes the castling spelling.

**A rules question takes its variant by keyword**, and `variant` defaults to
classic chess wherever it is optional: `position.facts(variant=esca.CHESS960)`,
never positionally.

### Errors

`esca.Error` is the base of every exception the library raises, and each member
is a `ValueError` as well, so `except ValueError` keeps working:

| exception | raised when |
|---|---|
| `esca.FenError` | the text is not a FEN esca can read |
| `esca.IllegalMove` | the move is not legal in the position |
| `esca.MoveParseError` | the text names no one legal move of the position |
| `esca.PositionError` | the position is not one this variant can play on |
| `esca.ParseError` | the text names no square, file, rank, role, colour or move |

A message names the offending input and, where a position is what refused it,
that position's FEN. The UCI client has its own family, rooted at
`esca.uci.UciError`: `EngineTimeout`, `EngineDied` and `ProtocolError`. It is
not part of `esca.Error`.

```python
import esca

esca.CLASSIC  # Variant
esca.CHESS960

# Game — variant defaults to classic everywhere it is optional
g = esca.Game()
g = esca.Game(variant=esca.CHESS960, seed=518)
g = esca.Game.from_fen(fen, variant=esca.CHESS960)

g.castling_output = esca.KING_TO_ROOK  # or esca.KING_TWO_SQUARES
g.play("e2e4")  # UCI or a Move
g.play_san("Nf3")
g.move_to_uci(mv)
g.undo()
g.position  # Position, immutable and hashable
g.moves  # list[Move] played
g.legal_moves()  # list[Move]
g.annotated_moves()  # list[AnnotatedMove]: am.move, am.facts
g.outcome()  # see below, or None
g.claims()  # ["threefold_repetition", …]
print(g.position.summary())

# A game copies and pickles: variant name, start FEN, moves and castling style
import copy, pickle

g2 = g.copy()  # or copy.copy(g), copy.deepcopy(g), pickle.loads(pickle.dumps(g))

# Position on its own; a rules question takes `variant` by keyword
p = esca.Position.from_fen(fen)
p.fen, p.epd, p.side_to_move, p.en_passant, p.in_check  # str, str, "white", "e6", bool
p.piece_at("e1")  # "K"

p.legal_moves()  # list[Move]; every one of these takes variant=…
p.annotated_moves()  # list[AnnotatedMove]
p.san(mv), p.parse_san("Nf3")  # str, Move
p.uci(mv), p.uci(mv, castling="king_two_squares")  # str
p.parse_uci("e1h1")  # Move; either castling spelling is read
p.play("e2e4")  # Position after the move, from a Move or UCI text
p.outcome()  # "checkmate", "stalemate", … or None
f = p.facts(variant=esca.CLASSIC)  # keyword-only
f = g.facts()  # variant taken from the game

# A Move: origin, destination and promotion are what it is
mv = esca.Move.from_uci("e7e8q")  # only a position says what a move takes or castles,
mv.origin, mv.destination, mv.promotion  # so read one with p.parse_uci to play it
mv.uci()  # "e7e8q"; mv.uci(castling="king_two_squares") for the other spelling
esca.Move("e1", "h1", kind="castling")  # a kind and an is_capture that disagree are refused

p.see("e5"), p.see_capture(mv)  # static exchange evaluation, in value units

f.pawns.passed.white  # SquareSet
f.pawns.passed.of("black")  # the same by colour; f.pawns.passed["b"] too
white, black = f.pawns.passed  # a pair iterates as its two values, White first
f.pawns.passed.items()  # [("white", SquareSet(…)), ("black", SquareSet(…))]
len(f.pawns.passed), "w" in f.pawns.passed  # 2, True
list(f.attacks.hanging.black)  # ["e5", …]
f.king.ring_attack_weight.white  # int
list(f.pawns.open_files)  # FileSet, iterated as ["c", "e", "h"]
print(f.summary())

f.pawns.count_by_file.white  # PerFile: (1, 1, 0, 1, 0, 1, 1, 0), file a first
f.king.shield.white  # three values, queenside first; f.king.shield_files names them
f.pawns == other.pawns  # groups compare and hash by the facts they hold
repr(f.pawns)  # PawnFacts(pawns=ByColour(white=SquareSet([…]), …), …)

# Walking the groups without knowing them
f.groups()  # ["placement", "state", …]
f.group("pawns").to_dict()  # {"pawns": {"white": [...], …}, "passed": …}
f.to_dict()  # {group: {field: value}} for every group, the one JSON form
esca.facts_catalogue()  # [{"name", "meaning", "fields": [{"name", "type", "meaning"}]}]
esca.move_facts_catalogue()  # the same for a move's facts

# Typed all the way down: the facts section of `_esca.pyi` is generated from
# the catalogue, so a stub cannot claim a type the extension does not answer.
f.pawns.passed  # ByColour[SquareSet]
f.pawns.count_by_file.white[0]  # int
f.to_dict()["pawns"]["passed"]["white"]  # list[str]

# Typed arrays, with `pip install esca[tensors]`
import esca.tensors as tensors

tensors.facts([fen], groups=["pawns"])["pawns.passed"]  # (1, 2, 64) of bool

# PGN
for pg in esca.pgn.read(path):  # or read_string(text); skip_errors=True drops bad games
    pg.headers  # dict[str, str], in the order the tags were set
    pg.comment, pg.result  # "", "1-0"
    pg.variant, pg.start_position  # Variant, Position
    for node in pg.mainline():
        node.move, node.san, node.nags, node.comment_before, node.comment_after
        node.variations  # list[list[Node]]
    pg.game()  # esca.Game of the mainline
    print(pg.to_string())
esca.pgn.count(path)  # games that read without error
esca.pgn.Game.from_game(g)  # or g.to_pgn()

# Opening books. A key is an int, and a raw entry's move is UCI text.
p.polyglot_key  # int; no feature is needed for the key itself

book = esca.polyglot.Book(path)
for raw in book:  # len(book) entries, in file order
    raw.key, raw.bits, raw.uci, raw.weight, raw.learn
    raw.decode(p, variant=esca.CLASSIC)  # Entry | None
book.entries(p)  # list[Entry], in book order, illegal moves dropped
book.best(p)  # Entry | None
book.pick(p, seed)  # Entry | None, drawn by weight
entry.key, entry.move, entry.bits, entry.weight, entry.learn
esca.polyglot.Book.write(path, entries)

builder = esca.polyglot.Builder(max_ply=20, min_count=2)
builder.add_game(g)
builder.add_pgn(path)  # or add_pgn_string(text); returns games counted
builder.write(path)

# Streams url to a temporary file beside path, renaming it on success.
esca.polyglot.download(url, path, sha256=None)

# Opening names
esca.openings.lookup(p)  # Opening | None, with .eco and .name
esca.openings.count()
g.opening()  # Opening | None, the deepest name the game reached

# Named endings, no feature
esca.endings.classify(p)  # Ending, with .class_, .verdict, .technique, .evidence
g.ending()  # the same for the current position

# Engines. Times are seconds, moves are Move objects, scores are cp/mate.
from esca import uci

with uci.Engine("stockfish") as engine:
    engine.handshake()
    engine.name, engine.author, engine.options  # str | None, str | None, dict[str, uci.Option]
    engine.set_option("Hash", 256)  # bool / int / str / None, by the option's type
    engine.new_game()

    answer = engine.play(game, uci.Limits(movetime=0.5))  # uci.Answer
    answer.best, answer.ponder  # Move | None, Move | None

    lines = engine.analyse(game, uci.Limits(depth=20), multipv=3)  # list[uci.Info]
    lines[0].cp, lines[0].mate, lines[0].pv  # int | None, int | None, list[Move]

    # A search is stopped through the Search it gave, not through the engine
    engine.set_position(game)
    with engine.go(uci.Limits(infinite=True)) as search:  # streams reports
        for report in search:
            if report.depth and report.depth >= 12:
                search.stop()
        best = search.answer()  # search.done says whether it has one

# The same surface, awaited: asyncio all the way down, no thread and no
# blocking call under it.
async with uci.AsyncEngine("stockfish") as engine:
    await engine.handshake()
    await engine.set_option("Hash", 256)
    answer = await engine.play(game, uci.Limits(movetime=0.5))
    # `go` is not a coroutine: it hands back an AsyncSearch that sends `go`
    # when it is first read, so each of these is one whole call.
    async for report in engine.go(uci.Limits(depth=20)):
        print(report.depth, report.cp)
    answer = await engine.go(uci.Limits(depth=20)).answer()
    async with engine.go(uci.Limits(infinite=True)) as search:
        await search.stop()  # and ponderhit(), on the search
    engine.timeout = 30.0  # bounds every wait that takes no timeout of its own
    engine.dropped_lines  # reports the engine wrote faster than they were read

# The protocol as values, for a client of one's own
from esca.uci import protocol

protocol.Command.position(game).to_line()  # "position startpos moves e2e4"
protocol.Command.go(uci.Limits(depth=8)).to_line()  # "go depth 8"
message = protocol.parse("info depth 3 score cp 12 pv e2e4", game)
message.kind, message.info, message.answer, message.option  # str, Info | None, …

session = protocol.Session()
session.sent(protocol.Command.uci())
session.received(protocol.parse("uciok"))
session.state  # "idle"
```

### How a game ended

`Game.outcome()` and `Position.outcome()` answer with one of `"checkmate"`,
`"stalemate"`, `"insufficient_material"`, `"seventy_five_moves"`,
`"fivefold_repetition"`, or `None`.

`None` does not mean the game is playable. A claimable draw leaves the position
playable until a player asks for it, so `outcome()` stays `None` while
`claims()` is non-empty. "How did this game end" needs both: `outcome()` for
what ended it on its own, `claims()` for what either player may end it with.
`game.draw_status()` (§14) answers with every condition of both kinds at once,
each carrying its evidence.

| Contract | |
|---|---|
| `Position`, `Move`, `Facts` and their groups are immutable and picklable; `Position` and `Move` are hashable. | |
| `Game` copies and pickles: `copy()`, `copy.copy`, `copy.deepcopy`, `pickle`. The state is the variant's name, the start FEN, the moves in UCI and the castling spelling. | |
| `ByColour` iterates as its two values, White first, so `white, black = pair` unpacks; `items()` pairs each with its colour name, and a pair hashes when both its values do. | |
| A malformed game raises `ValueError` naming the line and column; the stream goes on with the next game. | |

---

## 12. `pgn` — games as text (feature `pgn`)

```rust
pub mod pgn {
    /// The game-termination marker.
    pub enum GameResult { White, Black, Draw, Unknown }

    impl GameResult {
        pub fn from_text(text: &str) -> Option<GameResult>;
        pub fn as_str(self) -> &'static str;
    }

    /// The tag pairs, in the order they were set.
    pub struct Headers { /* … */ }

    impl Headers {
        pub const SEVEN_TAG_ROSTER: [&'static str; 7];
        pub fn new() -> Headers;
        pub fn get(&self, name: &str) -> Option<&str>;
        pub fn contains(&self, name: &str) -> bool;
        pub fn set(&mut self, name: &str, value: &str);
        pub fn remove(&mut self, name: &str) -> Option<String>;
        pub fn iter(&self) -> impl Iterator<Item = (&str, &str)>;
        pub fn len(&self) -> usize;
        pub fn is_empty(&self) -> bool;
        /// The roster first, in roster order, then the rest as they were set.
        pub fn export_order(&self) -> Vec<(&str, &str)>;
    }

    /// One move of a game tree.
    pub struct Node {
        pub mv: Move,
        /// The move's own text, as written, less any `!`/`?` suffix.
        pub san: String,
        pub nags: Vec<u16>,
        pub comment_before: String,
        pub comment_after: String,
        /// Alternatives to this move, each a line from the same position.
        pub variations: Vec<Vec<Node>>,
    }

    impl Node { pub fn new(mv: Move, san: &str) -> Node; }

    pub struct Game {
        pub headers: Headers,
        /// The comment before the first move.
        pub comment: String,
        pub moves: Vec<Node>,
        pub result: GameResult,
    }

    impl Game {
        pub fn new() -> Game;
        pub fn mainline(&self) -> &[Node];
        /// The rules and the start position the `Variant` and `FEN` tags name.
        pub fn setup(&self) -> Result<(Arc<dyn Variant>, Position), PgnError>;
        pub fn mainline_game(&self) -> Result<crate::Game, PgnError>;
        pub fn from_game(game: &crate::Game) -> Game;
    }

    /// Export format: tag pairs one per line, a blank line, then the movetext
    /// wrapped at `EXPORT_WIDTH`.
    impl fmt::Display for Game { /* … */ }
    pub const EXPORT_WIDTH: usize = 80;

    impl crate::Game { pub fn to_pgn(&self) -> Game; }

    /// Games read one at a time.
    pub struct Reader<R> { /* … */ }

    impl<R: BufRead> Reader<R> {
        pub fn new(input: R) -> Reader<R>;
        /// Drops malformed games instead of reporting them.
        pub fn skipping(self) -> Reader<R>;
        pub fn read_game(&mut self) -> Option<Result<Game, PgnError>>;
    }

    impl<R: BufRead> Iterator for Reader<R> { type Item = Result<Game, PgnError>; }

    pub fn read(path: &Path) -> io::Result<Reader<BufReader<File>>>;
    pub fn read_str(text: &str) -> Reader<&[u8]>;
    /// Games that read without error.
    pub fn count_games<R: BufRead>(input: R) -> usize;

    pub struct PgnError { pub line: usize, pub column: usize, pub kind: ErrorKind }

    pub enum ErrorKind {
        UnterminatedComment, UnterminatedString, MalformedTag,
        UnterminatedVariation, UnexpectedVariationEnd, StrayVariation,
        IllegalMove(String), AmbiguousMove(String), Syntax(String),
        UnknownVariant(String), BadFen(FenError), BadSetup(PositionError),
        Io(String),
    }
}
```

`Variant` selects the rules: absent, `Chess`, `Standard`, `Normal` or
`From Position` is classic chess, and `Chess960`, `Chess 960` or
`Fischerandom` is Chess960, case and separators ignored; any other value is
`UnknownVariant` naming it. A `FEN` tag is read in either castling dialect —
`KQkq`, where a letter means the outermost rook on that wing, or the rook
files `AHah` — and is used whatever `SetUp` says. The FEN `from_game` writes
is the one `Position::fen` renders: `KQkq` where the rook files are the
classic ones, rook files otherwise.

The reader tolerates the forms that occur in the wild: a missing result, a
move number glued to its move, `...` continuations, `%` escape lines, `{}`
comments spanning lines, `;` comments to the end of a line, `$` glyphs and
`!`/`?` suffixes, and glyphs after a variation. It resynchronises on the next
tag section after a malformed game, so one bad game does not lose the rest of
the stream. Line and column are 1-based, and 0 when the error did not come
from text.

Writing is deterministic, and reading its own output changes nothing. It
writes the tag pairs the game holds — `from_game` is what supplies a roster —
a `12.` before every White move and a `12...` before a Black move that
follows a comment or a variation, and a comment as one `{…}` with its
whitespace collapsed. A move number and its move are one token, so a line
break never falls between them.

---

## 13. `uci` — talking to an engine (feature `uci`)

Three layers: the protocol as values, which does no I/O; the engine as a
subprocess, which bounds every wait; and the variant handling that keeps
Chess960 honest. There are two ways to hold an engine, over the same values and
the same errors — blocking, and on a runtime: `uci::Engine` and
`uci::tokio::Engine` in Rust, `uci.Engine` and `uci.AsyncEngine` in Python.

```rust
pub mod uci {
    pub mod protocol {
        pub enum Command { Uci, Debug(bool), IsReady, SetOption { name: String, value: Option<String> },
                           Register(Register), NewGame, Position(Setup), Go(Limits),
                           Stop, PonderHit, Quit }
        impl Command { pub fn to_line(&self) -> String; pub fn keyword(&self) -> &'static str; }

        pub struct Setup { pub fen: Option<String>, pub moves: Vec<String> }
        impl Setup {
            /// The moves played, each written where it was played, castling as `style` asks.
            pub fn of_game(game: &Game, style: CastlingOutput) -> Setup;
        }

        /// Every limit a `go` may name; they combine.
        pub struct Limits { pub search_moves: Vec<String>, pub ponder: bool,
                            pub white_time: Option<Duration>, /* … */ pub infinite: bool }

        pub enum Message { Id { key: String, value: String }, UciOk, ReadyOk, Option(OptionSpec),
                           Info(Box<Info>), BestMove(BestMove), Registration(Status),
                           CopyProtection(Status), Raw(String) }

        pub struct OptionSpec { pub name: String, pub kind: OptionKind }
        pub enum OptionKind { Check { .. }, Spin { .. }, Combo { .. }, Button, String { .. } }
        impl OptionSpec { pub fn accepts(&self, value: &OptionValue) -> Result<(), String>; }

        /// Every standard `info` field; moves stay text until a game reads them.
        pub struct Info { pub depth: Option<u32>, pub score: Option<Score>, pub bound: Option<Bound>,
                          pub pv: Vec<String>, /* … */ pub unknown: Vec<String> }
        impl Info { pub fn pv_moves(&self, game: &Game) -> Vec<Move>; }

        /// Which commands may go out and which messages may come in.
        pub struct Session { /* … */ }
        impl Session {
            pub fn state(&self) -> State;
            pub fn sent(&mut self, command: &Command) -> Result<(), ProtocolError>;
            pub fn received(&mut self, message: &Message) -> Result<(), ProtocolError>;
        }
        pub enum State { Started, Identifying, Idle, Searching, Pondering, Quitting }

        /// Never fails: a line it cannot read is `Message::Raw`.
        pub fn parse(line: &str) -> Message;
    }

    pub struct Engine { /* … */ }
    impl Engine {
        pub fn spawn(program: impl AsRef<OsStr>, args: impl IntoIterator<Item = impl AsRef<OsStr>>)
            -> Result<Engine, Error>;
        pub fn handshake(&mut self) -> Result<&Identity, Error>;
        pub fn options(&self) -> &[OptionSpec];
        pub fn set_option(&mut self, name: &str, value: OptionValue) -> Result<(), Error>;
        pub fn new_game(&mut self) -> Result<(), Error>;
        pub fn set_position(&mut self, game: &Game) -> Result<(), Error>;
        /// Starts a search. The `Search` is the whole of the answer.
        pub fn go(&mut self, limits: &Limits) -> Result<Search<'_>, Error>;
        pub fn play(&mut self, game: &Game, limits: &Limits) -> Result<Answer, Error>;
        pub fn is_ready(&mut self) -> Result<(), Error>;
        /// How long a wait that is not given its own limit may take.
        pub fn timeout(&self) -> Duration;
        pub fn set_timeout(&mut self, timeout: Duration);
        pub fn quit(&mut self) -> Result<Option<i32>, Error>;
        pub fn kill(&mut self);
        /// The line-level interface, for tools and diagnostics.
        pub fn send_line(&mut self, text: &str) -> Result<(), Error>;
        pub fn next_line(&mut self, timeout: Duration) -> Result<Option<String>, Error>;
        pub fn dropped_lines(&self) -> u64;
    }

    /// A search in flight: the whole of one search, and the only way to steer
    /// one. Iterating gives every report up to the move the engine chooses.
    pub struct Search<'a> { /* … */ }

    impl Search<'_> {
        pub fn next_info(&mut self) -> Result<Option<Info>, Error>;
        /// The answer, with the reports still to come dropped.
        pub fn answer(&mut self) -> Result<Answer, Error>;
        pub fn stop(&mut self) -> Result<(), Error>;
        pub fn ponderhit(&mut self) -> Result<(), Error>;
        pub fn is_done(&self) -> bool;
        pub fn game(&self) -> &Game;
        pub fn state(&self) -> State;
        pub fn dropped_lines(&self) -> u64;
    }

    /// Blocking only; the tokio client awaits `next_info` instead.
    impl Iterator for Search<'_> { type Item = Result<Info, Error>; }

    pub struct Answer { pub best: Option<Move>, pub ponder: Option<Move> }

    pub enum Error { Io(io::Error), Timeout { .. }, Died { .. }, Protocol(ProtocolError),
                     NotIdentified, NoSuchOption(String), BadValue { .. } }

    /// The same client on a tokio runtime (feature `tokio`): the same values,
    /// the same `Error`, every method awaited.
    pub mod tokio {
        pub struct Engine { /* … */ }
        impl Engine {
            pub async fn spawn(program: impl AsRef<OsStr>,
                               args: impl IntoIterator<Item = impl AsRef<OsStr>>)
                -> Result<Engine, Error>;
            pub async fn handshake(&mut self) -> Result<&Identity, Error>;
            pub async fn set_option(&mut self, name: &str, value: OptionValue) -> Result<(), Error>;
            pub async fn set_position(&mut self, game: &Game) -> Result<(), Error>;
            pub async fn go(&mut self, limits: &Limits) -> Result<Search<'_>, Error>;
            pub async fn play(&mut self, game: &Game, limits: &Limits)
                -> Result<Answer, Error>;
            pub async fn quit(&mut self) -> Result<Option<i32>, Error>;
            pub fn timeout(&self) -> Duration;
            pub fn set_timeout(&mut self, timeout: Duration);
            pub fn dropped_lines(&self) -> u64;
            /* new_game, is_ready, send_line, next_line, kill: the blocking
               ones, awaited */
        }

        /// The same `Search`, its reports one awaited `next_info` at a time;
        /// no `Iterator`, which cannot await.
        pub struct Search<'a> { /* … */ }
    }

    impl Launch { pub async fn spawn_tokio(self) -> Result<tokio::Engine, Error>; }
}
```

| Contract | |
|---|---|
| One way to search: `go` starts one and hands back the `Search`, which is the whole of it — iterate it for the reports, ask it for the answer, stop it, tell it a ponder hit. `play` is `set_position` plus `go` plus the answer. Nothing else drives a search. | |
| Every wait is bounded, and the bound is on **silence**, not on the search. A search fails with `Error::Timeout` once the engine has said nothing for what its limits asked for (`Limits::movetime`) plus the engine's `timeout`; every other wait takes the `timeout` alone. `set_timeout` is the per-call knob. | |
| An engine that has exited is `Error::Died` on every call after, and is killed when the `Engine` is dropped. | |
| A line that breaks the grammar is `Message::Raw`, and a token that does lands in `Info::unknown`. Reading engine output never fails. | |
| A message the [`Session`] has no room for — a second `bestmove`, a `readyok` no `isready` asked for — is logged and dropped, not delivered. | |
| Dropping a `Search` stops it and waits for the answer, so the engine is left idle. | |
| `set_position` on a Chess960 game sets `UCI_Chess960` and writes castling king-to-rook; an engine that does not offer the option is an error, not a game played by the wrong rules. Classic games are written king-two-squares, and both spellings are read. | |
| Every wire line is logged through `log` at debug, `>>` out and `<<` in. | |
| The unread lines are capped at 4096. When the engine writes faster than it is read, the oldest line that carries no part of the conversation goes; `bestmove`, `readyok`, `uciok`, `id`, `option`, `copyprotection` and `registration` are never dropped. `dropped_lines()` counts what went. | |
| `AsyncEngine` starts its process on the first call and quits with `async with`. Cancelling an awaited `play`, `analyse` or `answer` asks the engine to stop, and the engine's next call waits that search out, as on the tokio client. | |
| On the tokio client every future is cancellation-safe: one dropped mid-wait loses no line and leaves the session where it stood. A `Search` let go of unanswered asks the engine to stop, and the engine's next call waits that search out — a drop cannot await, so the blocking client's draining `Drop` becomes a settling next call. | |

---

## 14. `explain` — the evidence behind a rules answer

Where `Facts` answers *what is true*, `explain` answers *why*, and hands over
the squares the answer was read off. Every categorical answer is an enum,
every distinct reason is its own field carrying its own evidence, and every
reason that applies is filled in — a caller reads the whole situation from one
value instead of asking again for the next reason. Nothing here is part of the
feature schema.

```rust
pub mod explain {
    /// `esca::Wing`, re-exported: which castling, named by where the king
    /// lands — `Short` on the g-file, `Long` on the c-file. Not the wing the
    /// rook starts on, which a shuffled back rank can put on either side of
    /// the king.
    pub use crate::Wing;

    /// One castling of one colour, and everything standing in its way.
    pub struct Castling {
        /// Whose castling this is.
        pub colour: Colour,
        /// Which castling of that colour.
        pub wing: Wing,
        /// The position still holds this castling right.
        pub right: bool,
        /// The rook the right names stands on its square. False without a
        /// right, which names no rook.
        pub rook_present: bool,
        /// The enemy units attacking the king where it stands.
        pub king_in_check_by: SquareSet,
        /// Each square the king crosses or lands on that the enemy covers,
        /// with the units covering it, in ascending square order. The king's
        /// own square is `king_in_check_by`, not a member here, and without
        /// the right there is no path at all.
        pub path_attacked: Vec<(Square, SquareSet)>,
        /// The units standing on squares the king or the rook must pass or
        /// land on, the castling king and rook themselves excepted.
        pub path_blocked: SquareSet,
        /// Nothing above prevents the castling. Whose turn it is is not part
        /// of it, so for the side to move this is exactly legality.
        pub allowed: bool,
    }

    /// The en-passant capture a position offers the side to move.
    pub enum EnPassant {
        /// The previous ply was not a double pawn step.
        None,
        /// A pawn skipped `target`, and `captures` holds every pawn of the
        /// side to move standing beside it.
        Available { target: Square, captures: Vec<EpCapture> },
    }

    impl EnPassant {
        pub fn target(&self) -> Option<Square>;
        pub fn captures(&self) -> &[EpCapture];
    }

    /// One pawn's en-passant capture of the target.
    pub struct EpCapture {
        pub from: Square,
        pub legal: bool,
        /// The first of `InCheck`, `Pinned`, `ExposesKing` that applies;
        /// `None` when the capture is legal.
        pub forbidden_by: Option<EpObstacle>,
    }

    pub enum EpObstacle {
        /// The pawn is pinned against its own king and the target is off the
        /// pinning ray.
        Pinned { ray: SquareSet, pinner: Square },
        /// Both pawns leave one rank at once and uncover the king: the pin
        /// that binds neither pawn alone.
        ExposesKing { attacker: Square },
        /// The side to move is in check and this capture does not answer it.
        InCheck { by: SquareSet },
    }

    /// A unit that may not move off the line between an enemy slider and its
    /// own king.
    pub struct Pin {
        pub pinned: Square,
        pub pinner: Square,
        pub king: Square,
        /// Between pinner and king, exclusive.
        pub ray: SquareSet,
    }

    /// A unit attacked with a less valuable one of the same colour directly
    /// behind it on the slider's line.
    pub struct Skewer {
        pub attacker: Square,
        pub front: Square,
        pub behind: Square,
        /// Between attacker and `behind`, exclusive; holds `front`.
        pub ray: SquareSet,
    }

    /// How often the current position has stood, and what nearly counted.
    pub struct Repetition {
        pub count: u32,
        /// Every ply the current position occurred at, this one last.
        pub plies: Vec<u32>,
        pub near_misses: Vec<NearMiss>,
    }

    /// An earlier ply with the same placement that is not a repetition.
    pub struct NearMiss { pub ply: u32, pub differs: Vec<Difference> }

    pub enum Difference { CastlingRights, EnPassant, SideToMove }

    /// The halfmove clock, and how far it is from ending the game.
    pub struct FiftyMove {
        pub clock: u32,
        /// Plies until a player may claim; 0 once one may.
        pub plies_to_claim: u32,
        /// Plies until the draw is automatic; 0 once it is.
        pub plies_to_automatic: u32,
        /// The last move of this game that set the clock to 0. `None` when no
        /// move did, which leaves the clock the start position carried.
        pub last_reset: Option<Reset>,
    }

    pub struct Reset { pub ply: u32, pub kind: ResetKind }

    /// A capturing pawn move is a `Capture`.
    pub enum ResetKind { Capture, PawnMove }

    /// Every draw condition that holds, not the first of them. Both lists are
    /// empty when the side to move is checkmated.
    pub struct DrawStatus {
        pub automatic: Vec<AutomaticDraw>,
        pub claimable: Vec<ClaimableDraw>,
    }

    pub enum AutomaticDraw {
        Stalemate(StalemateDetail),
        InsufficientMaterial(MaterialConfig),
        FivefoldRepetition(Repetition),
        SeventyFiveMoves(FiftyMove),
    }

    pub enum ClaimableDraw { ThreefoldRepetition(Repetition), FiftyMoves(FiftyMove) }

    /// The one spelling every surface uses, the same names `Outcome` and
    /// `DrawClaim` answer with (§1).
    impl AutomaticDraw { pub const fn name(&self) -> &'static str; }
    impl ClaimableDraw { pub const fn name(&self) -> &'static str; }

    /// The material `Variant::outcome` calls insufficient, named. Either side
    /// may be the one holding the minor.
    pub enum MaterialConfig {
        KvK,
        KNvK,
        KBvK,
        /// Bishops and kings only, every bishop on one square colour.
        KBvKBSameColour,
    }

    /// Why the side to move has no move.
    pub struct StalemateDetail {
        pub king: Square,
        /// Each square beside the king that none of its own units holds, with
        /// the enemy units covering it, the king itself out of the way.
        pub escape_squares: Vec<(Square, SquareSet)>,
        /// Every other unit of the side to move, and what holds it.
        pub stuck_units: Vec<(Square, Stuck)>,
    }

    pub enum Stuck {
        /// Pinned against its own king, whatever else is true of it.
        Pinned { ray: SquareSet, pinner: Square },
        /// Occupancy leaves it no move at all.
        Blocked,
        /// It has moves, and none of them is legal.
        NoMoves,
    }
}

impl Position {
    /// What stands in the way of `colour` castling on `wing`.
    pub fn castling(&self, colour: Colour, wing: Wing) -> Castling;
    pub fn en_passant_status(&self) -> EnPassant;
    /// The units giving check to the side to move.
    pub fn checkers(&self) -> SquareSet;
    /// The units of `colour` attacking `square`, pins ignored.
    pub fn attackers(&self, square: Square, colour: Colour) -> SquareSet;
    /// The squares strictly between `a` and `b`; empty when they share no
    /// rank, file or diagonal.
    pub fn between(&self, a: Square, b: Square) -> SquareSet;
    /// The absolute pins on `colour`'s units: the pinned unit's own king is
    /// what stands behind it. Relative pins are not counted.
    pub fn pins(&self, colour: Colour) -> Vec<Pin>;
    /// The skewers on `colour`'s units, the more valuable one in front.
    pub fn skewers(&self, colour: Colour) -> Vec<Skewer>;
}

impl Game {
    pub fn repetition_status(&self) -> Repetition;
    pub fn fifty_move_status(&self) -> FiftyMove;
    pub fn draw_status(&self) -> DrawStatus;
    /// What could be claimed once `mv` is played. Empty when `mv` is not
    /// legal here.
    pub fn claims_after(&self, mv: Move) -> Vec<ClaimableDraw>;
}
```

A ply number counts positions, not moves: the start of a game is ply 0, and
the position after *n* moves is ply *n*.

The castling path is the one the variant's rule names, so both variants are
answered by the same fields: the king crosses every square between where it
stands and the g- or c-file square it lands on, and the rook every square up
to the f- or d-file square, which on a shuffled back rank can be squares the
king never touches, or none at all.

The types are `esca.explain` on the Python side, the methods stay on
`Position` and `Game`. Python mirrors the field names, `EpCapture.origin`
apart, `from` being a keyword there. Squares are text, square sets are `SquareSet`,
and an enum that carries nothing is its name in `snake_case`; an enum that
carries something is one class with a `kind` naming the case and the payload
of every case as attributes, empty where the case does not carry it.

```python
from esca import explain  # Castling, Pin, DrawStatus and their kin

c = game.position.castling("w", "short")
c.right, c.rook_present, c.allowed  # True, True, False
c.path_attacked  # [("f1", SquareSet(["b5"]))]
list(c.path_blocked)  # ["g1"]

ep = game.position.en_passant_status()
ep.target  # "d6" or None
ep.captures[0].origin, ep.captures[0].legal  # "e5", False
ep.captures[0].forbidden_by.kind  # "exposes_king"
ep.captures[0].forbidden_by.attacker  # "h5"

game.position.checkers()  # SquareSet
game.position.attackers("e4", "b")
game.position.between("a1", "d4")  # SquareSet(["b2", "c3"])
game.position.pins("w")[0].pinned  # "d2"
game.position.skewers("b")[0].behind  # "h8"

rep = game.repetition_status()
rep.count, rep.plies  # 2, [4, 8]
rep.near_misses[0].ply, rep.near_misses[0].differs  # 0, ["castling_rights"]

fifty = game.fifty_move_status()
fifty.clock, fifty.plies_to_claim, fifty.last_reset.kind  # 12, 88, "pawn_move"

draws = game.draw_status()
[d.kind for d in draws.automatic]  # ["stalemate", "insufficient_material"]
draws.automatic[0].stalemate.stuck_units  # [("a7", Stuck)]
draws.automatic[1].material  # "kb_v_k"
[d.kind for d in game.claims_after(mv)]  # ["threefold_repetition"]
```

A `kind` is the condition's one spelling: `stalemate`,
`insufficient_material`, `fivefold_repetition`, `seventy_five_moves`,
`threefold_repetition`, `fifty_moves` — the same names `Game.outcome()` and
`Game.claims()` answer with.

Every value of every enum here, and every aggregate a one-line summary reads
naturally off, answers `describe()` with one plain English sentence: what the
value means, in the words a beginner or a language model would use. The enum
value stays the answer; the sentence is derived from it and never replaces it.

**The `describe()` rule**, which `endings` (§15) follows too: a sentence is
about the value it is called on and nothing else. A value never repeats what a
value it holds would say — `DrawStatus::describe` says whether the game is
drawn, and each condition in its lists says for itself what it is. So printing
a value and then its parts reads as a report rather than as an echo, and
printing only the part that matters loses nothing else.

```rust
impl Wing { pub fn describe(self) -> &'static str; }
impl Castling { pub fn describe(&self) -> String; }
impl EnPassant { pub fn describe(&self) -> String; }
impl EpCapture { pub fn describe(&self) -> String; }
impl EpObstacle { pub fn describe(&self) -> String; }
impl Pin { pub fn describe(&self) -> String; }
impl Skewer { pub fn describe(&self) -> String; }
impl Repetition { pub fn describe(&self) -> String; }
impl NearMiss { pub fn describe(&self) -> String; }
impl Difference { pub fn describe(self) -> &'static str; }
impl FiftyMove { pub fn describe(&self) -> String; }
impl Reset { pub fn describe(&self) -> String; }
impl ResetKind { pub fn describe(self) -> &'static str; }
impl DrawStatus { pub fn describe(&self) -> String; }
impl AutomaticDraw { pub fn describe(&self) -> String; }
impl ClaimableDraw { pub fn describe(&self) -> String; }
impl MaterialConfig { pub fn describe(self) -> &'static str; }
impl StalemateDetail { pub fn describe(&self) -> String; }
impl Stuck { pub fn describe(&self) -> String; }
```

A value that carries squares names them; an aggregate lists every reason that
applies, in the order its fields carry them.

```python
c = game.position.castling("w", "short")
c.colour, c.wing  # "white", "short" — a colour is answered in full, read either way
c.describe()
# "White cannot castle short: the king is in check from e8, and a king may not
#  castle out of check; the enemy covers f1, which the king must cross or land
#  on; the path is blocked on g1."

ep.captures[0].describe()  # "The pawn on e5 may not take en passant. …"
game.draw_status().describe()
```

Python has `describe()` on the class each enum lives on: `Difference` reads as
`NearMiss.describe()`, `ResetKind` as `Reset.describe()`, `MaterialConfig` as
`AutomaticDraw.describe()` and `Wing` as `Castling.describe()`, since a value
that carries nothing is a `snake_case` string there and has no methods of its
own.

---

## 15. `endings` — named endings, no feature

The books' name for the material on the board, what theory says the result is,
and the method it is played by. Everything here is the general case adjusted
by the few position-specific facts that are cheap and well defined; it is not
a search, and it is not part of the feature schema.

```rust
pub mod endings {
    /// The most pieces one side may have for the position to be an ending. A
    /// piece is any unit that is neither a king nor a pawn.
    pub const ENDING_PIECES: u32 = 2;

    /// The material both sides hold, written the way endings are named.
    pub struct Signature {
        /// The canonical spelling, stronger side first: `KRPvKR`.
        pub text: String,
        /// The side written first: the one with more conventional material;
        /// on a tie the one whose pieces are worth more one by one, queen
        /// before rook before bishop before knight before pawn; White when
        /// even that is even.
        pub stronger: Colour,
        /// Units per role per colour: `counts.white.of(Role::Rook)`.
        pub counts: ByColour<PerRole<u8>>,
        /// Conventional material per colour, the king counting nothing.
        pub value: ByColour<u32>,
    }

    impl Signature {
        pub fn count(&self, colour: Colour, role: Role) -> u8;
        /// Everything that is neither a king nor a pawn.
        pub fn pieces(&self, colour: Colour) -> u32;
        pub fn pawns(&self) -> u32;
        pub fn describe(&self) -> String;
    }

    /// The named endings this version of the catalogue tells apart. A class
    /// names material only: which side holds which half of it is `Verdict`'s
    /// answer.
    pub enum Class {
        KvK, KQvK, KRvK, KBBvK, KBNvK, KNNvK, KBvK, KNvK, KPvK, KBPvK,
        KQvKQ, KQvKR, KQvKB, KQvKN, KQvKP, KQvTwoMinors,
        KRvKR, KRvKB, KRvKN, KRvKP, KRvTwoMinors, KRPvKR,
        KBBvKN, KBvKN, KBvKBSameColour, KBvKBOppositeColour, KNvKN,
        KBvKP, KNvKP,
        /// Kings and pawns only, in any number.
        Pawns,
        /// An ending the catalogue does not name.
        Other,
        /// More material than `ENDING_PIECES` allows.
        NotAnEnding,
    }

    impl Class {
        pub const ALL: [Class; 32];
        /// The name in `snake_case`, as the Python surface spells it.
        pub fn name(self) -> &'static str;
        pub fn describe(self) -> &'static str;
    }

    /// What theory says the result is, played out by both sides at their
    /// best. The answer is for the ending, not a search of the position.
    pub enum Verdict {
        Win(Colour),
        UsuallyWin(Colour),
        /// Usually drawn; the positions that are not are won by that colour.
        UsuallyDraw(Colour),
        Draw,
        Unknown,
    }

    impl Verdict {
        pub fn winner(self) -> Option<Colour>;
        pub fn name(self) -> &'static str;
        pub fn describe(self) -> String;
    }

    /// The named method an ending is played by.
    pub enum Technique {
        None,
        BoxMethod,
        TwoBishopMate,
        BishopKnightMate,
        KeySquares,
        Opposition,
        RuleOfTheSquare,
        Lucena,
        Philidor,
        WrongBishop,
        WrongRookPawn,
    }

    impl Technique {
        pub const ALL: [Technique; 11];
        pub fn name(self) -> &'static str;
        pub fn describe(self) -> &'static str;
    }

    /// The position-specific facts an ending's verdict and technique are read
    /// off. Each group is present only when the material puts it in question.
    pub struct Evidence {
        pub pawn: Option<PawnRace>,
        pub bishops: Option<Bishops>,
        /// The kings stand on one file, rank or diagonal with exactly one
        /// empty square between them, so the side not to move holds the
        /// opposition.
        pub opposition: bool,
    }

    /// The race of the only pawn on the board.
    pub struct PawnRace {
        pub pawn: Square,
        pub colour: Colour,
        pub promotion: Square,
        /// It stands on the a- or the h-file.
        pub rook_pawn: bool,
        /// Pawn moves left to promotion, a double first step counted as one.
        pub steps: u32,
        /// The defending king reaches the promotion square no later than the
        /// pawn does, both racing straight there and neither king counted as
        /// an obstacle: the rule of the square.
        pub defender_inside_square: bool,
        pub attacker_in_front: bool,
        pub defender_in_front: bool,
        /// The defending king stands on the promotion square or beside it.
        pub defender_holds_the_corner: bool,
    }

    /// The bishops on the board, when at least one stands on it.
    pub struct Bishops {
        /// One bishop each, on opposite square colours.
        pub opposite_colours: bool,
        /// Every bishop on the board stands on one square colour.
        pub same_colour: bool,
        /// The only pawn on the board belongs to the side with the bishops,
        /// is a rook pawn, and no bishop of that side stands on the colour of
        /// the square it promotes on.
        pub wrong_bishop: bool,
    }

    pub struct Ending {
        pub class: Class,
        pub signature: Signature,
        pub verdict: Verdict,
        pub technique: Technique,
        pub evidence: Evidence,
    }

    impl Ending { pub fn describe(&self) -> String; }
    impl PawnRace { pub fn describe(&self) -> String; }
    impl Bishops { pub fn describe(&self) -> String; }
    impl Evidence { pub fn describe(&self) -> String; }

    pub fn classify(position: &Position) -> Ending;
}

impl Position { pub fn ending(&self) -> Ending; }
impl Game { pub fn ending(&self) -> Ending; }
```

| Rule | |
|---|---|
| A position is an ending while **neither side has more than two pieces**, a piece being any unit that is neither a king nor a pawn. Above that the class is `NotAnEnding`, and the signature still says what is on the board. | |
| A signature writes the stronger side first, `K` then the pieces by descending value then the pawns: `KRPvKR`. The stronger side is the one with more conventional material (Q 9, R 5, B and N 3, P 1); on a tie the one holding the more valuable piece, queen before rook before bishop before knight before pawn; White when even that is even. | |
| A class names material only. `KBvKN` is the class whichever side has the bishop, and `Verdict` names the colour. | |
| `Class::KPvK` is one pawn and nothing else; two pawns or more, still with no pieces, is `Class::Pawns`. | |
| The verdict is the class's own result, overturned only by the facts in `Evidence`: two bishops on one square colour draw; a bishop that cannot cover its rook pawn's promotion square draws; a lone king on or beside a rook pawn's promotion square draws; a lone king outside the square of the pawn loses to it. | |
| The technique follows the class, and the same facts pick between two of them: `Lucena` unless the defending king stands in front of the pawn, which is `Philidor`; `RuleOfTheSquare` for a pawn the lone king cannot catch, `WrongRookPawn` for a corner it has reached, `KeySquares` otherwise. | |
| `Evidence::pawn` is filled in when exactly one pawn stands on the board, `Evidence::bishops` when at least one bishop does. Neither depends on the class. | |

```python
from esca import endings

e = endings.classify(position)  # also position.ending(), game.ending()
e.class_, e.verdict, e.technique  # "krp_v_kr", "usually_win", "lucena"
e.verdict.winner  # "white"
e.signature.text  # "KRPvKR"
e.signature.count("w", "p")  # 1
e.signature.pieces("b")  # 1
e.evidence.pawn.defender_inside_square  # True
e.evidence.bishops  # None
print(e.describe())

endings.EndingClass.all()  # every class, for a catalogue
e.class_.describe()
```

The three enums are objects on the Python surface rather than bare strings,
because each carries its own `describe()`. One compares equal to, hashes as,
and prints as its `snake_case` name, so `e.class_ == "krp_v_kr"` is the
comparison to write. `class_` carries the trailing underscore because `class`
is a Python keyword.

---

## 16. `polyglot` — opening books (feature `polyglot`)

The Polyglot book format: a file of 16-byte entries sorted by a key that the
format fixes, so a book written by one program is read by every other.

### The key

`Position::polyglot_key` needs no feature. It XORs 781 published constants:
one per piece per square, one per castling right, one per en-passant file,
and one for White to move.

| Rule | |
|---|---|
| A piece contributes `piece[kind][square]`, the kinds running black pawn, white pawn, black knight, … white king. | |
| A castling right contributes one constant per wing per colour, whatever file its rook starts on, so Chess960 rights map onto the same four. | |
| The en-passant file contributes only when a pawn of the side to move stands beside the pawn that has just advanced two squares — beside it, whether or not the capture would leave its own king in check. | |
| White to move contributes the turn constant; Black to move contributes nothing. | |
| The clocks and the full-move number contribute nothing. | |

The format's own test vectors, from the starting position through
`1. e4 d5 2. e5 f5 3. Ke2 Kf7`, are the cases of `tests/polyglot.rs` and
`python/tests/test_polyglot.py`.

### Entries and books

```rust
pub mod polyglot {
    /// The bytes one entry occupies.
    pub const ENTRY_SIZE: usize = 16;

    /// One entry as the file holds it: the move is still the format's 16 bits.
    pub struct Raw { pub key: u64, pub mv: u16, pub weight: u16, pub learn: u32 }

    impl Raw {
        /// Origin, destination and promotion, castling king-to-rook; `None`
        /// when the bits name no move.
        pub fn uci(&self) -> Option<String>;
        /// The bits of a move, for a book written by hand.
        pub fn pack(mv: Move) -> u16;
        /// The move read against `position`; `None` when it is not legal there.
        pub fn decode(&self, variant: &dyn Variant, position: &Position) -> Option<Entry>;
    }

    /// One entry whose move has been read against a position.
    pub struct Entry { pub key: u64, pub mv: Move, pub weight: u16, pub learn: u32 }

    impl Entry { pub fn new(key: u64, mv: Move, weight: u16, learn: u32) -> Entry; }
    impl From<Entry> for Raw { /* … */ }

    pub struct Book { /* … */ }

    impl Book {
        /// Memory-maps the file at `path`.
        pub fn open(path: &Path) -> io::Result<Book>;
        pub fn from_bytes(bytes: Vec<u8>) -> io::Result<Book>;
        pub fn len(&self) -> usize;
        pub fn is_empty(&self) -> bool;
        pub fn get(&self, index: usize) -> Option<Raw>;
        /// Every entry, in file order.
        pub fn iter(&self) -> impl Iterator<Item = Raw> + '_;
        /// The entries at `key`, in book order.
        pub fn raw_entries(&self, key: u64) -> Vec<Raw>;
        /// The entries at this position's key that name a legal move there.
        pub fn entries(&self, variant: &dyn Variant, position: &Position) -> Vec<Entry>;
        /// The heaviest of them; ties go to the earlier entry.
        pub fn best(&self, variant: &dyn Variant, position: &Position) -> Option<Entry>;
        /// One of them, by weight: the entry whose running total first
        /// exceeds `seed % total`.
        pub fn pick(&self, variant: &dyn Variant, position: &Position, seed: u64)
            -> Option<Entry>;
        /// Writes `entries` sorted and merged.
        pub fn write(path: &Path, entries: &[Entry]) -> io::Result<()>;
    }

    /// Counts the moves of the games it is given and writes them as a book.
    pub struct Builder { /* … */ }

    impl Builder {
        /// Every move of every game, once each.
        pub fn new() -> Builder;
        /// Moves later than `plies` are not counted. Default: no limit.
        pub fn max_ply(self, plies: u32) -> Builder;
        /// A move played fewer than `count` times is not written. Default: 1.
        pub fn min_count(self, count: u32) -> Builder;
        pub fn add_game(&mut self, game: &Game);
        /// Every game of a PGN source; malformed ones are skipped. Returns
        /// how many were added. Needs feature `pgn`.
        pub fn add_pgn<R: BufRead>(&mut self, input: R) -> usize;
        /// How many distinct position-and-move pairs have been counted.
        pub fn len(&self) -> usize;
        pub fn is_empty(&self) -> bool;
        /// The book rows: sorted, merged, and filtered by `min_count`.
        pub fn entries(&self) -> Vec<Raw>;
        pub fn write(&self, path: &Path) -> io::Result<()>;
    }
}
```

| Contract | |
|---|---|
| A file whose length is not a multiple of `ENTRY_SIZE` is `InvalidData`; an empty file is an empty book. | |
| Entries are big-endian and sorted by key; the entries of one key keep the order the file gives them. | |
| The move's 16 bits are destination file and rank, origin file and rank, then the promotion role, and castling is written king-takes-rook — which is the spelling `Move` already stores, in every variant. | |
| An entry whose bits name no move, or a move that is not legal in the position asked about, is dropped by `entries` and is `None` from `decode`. Nothing panics and nothing is guessed. | |
| `pick` is a pure function of the entries and the seed: a caller that wants variety supplies a fresh seed. When every candidate weighs 0, it returns the first. | |
| Writing sorts by key, then by descending weight, then by the encoded move, and merges entries that share a key and a move by adding their weights, saturating at `u16::MAX`. | |
| `Builder` weighs a move by how many games played it, capped the same way, and writes `0` as the learn value. | |

---

## 17. `openings` — the ECO catalogue (feature `openings`)

The [lichess-org/chess-openings](https://github.com/lichess-org/chess-openings)
data set, bundled under CC0-1.0 as `data/openings/`: an ECO
code and a name for each of some 3,800 named positions, indexed by Polyglot
key.

```rust
pub mod openings {
    /// An ECO volume letter with its index, and the name that goes with it.
    pub struct Opening { pub eco: &'static str, pub name: &'static str }

    /// The name of `position`, if it has one.
    pub fn lookup(position: &Position) -> Option<Opening>;
    /// How many named positions the catalogue holds.
    pub fn count() -> usize;
}

impl Game {
    /// The name of the deepest named position the game has reached.
    pub fn opening(&self) -> Option<Opening>;
}
```

| Contract | |
|---|---|
| The catalogue is keyed by position, not by move order, so a line that transposes into a named position is named. | |
| The starting position has no name; a game names an opening only from the first named position on. | |
| `Game::opening` walks the game's own positions and keeps the last hit, so a game that leaves the book keeps the last name it reached. | |
| The catalogue is classic chess; a Chess960 position matches only by coincidence. | |
| It is built on first use, from the bundled text, and shared from then on. | |
