# esca API sketch

`esca` is the position-facts library: a Rust crate at `rs_anglerfish/esca/`
and a Python package of the same name built from it. Terms used below are
defined in [`esca-vocabulary.md`](esca-vocabulary.md); features and their
encodings in [`features.md`](features.md).

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
| `serde` | no | `Serialize`/`Deserialize` for `Position`, `Move`, `Schema` and the manifest types. |

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
pub enum Outcome {
    Checkmate { winner: Colour },
    Stalemate,
    InsufficientMaterial,
    SeventyFiveMoves,
    FivefoldRepetition,
}

pub enum DrawClaim { FiftyMoves, ThreefoldRepetition }
```

---

## 2. Position

Immutable and variant-agnostic: placement and state, no rules. Every query
that needs rules takes a `&dyn Variant`, or is asked of a `Game`.

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

    /// Static exchange evaluation, in value units, as `features.md` §1
    /// defines it: of the unit on `sq`, and of a move of this position.
    pub fn see(&self, sq: Square) -> i32;
    pub fn see_capture(&self, mv: Move) -> i32;

    /// Facts of this position under `variant`.
    pub fn facts(&self, variant: &dyn Variant) -> Facts;
    /// Same, reusing buffers; no allocation.
    pub fn facts_in(&self, variant: &dyn Variant, scratch: &mut Scratch) -> Facts;

    /// Colours swapped and ranks flipped.
    pub fn mirrored(&self) -> Position;

    /// Board, side to move and state, for a human reader. Text not stable.
    pub fn summary(&self) -> String;
}

/// A Zobrist key.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Key(u64);

impl Key {
    pub fn get(self) -> u64;
}

/// An evaluation of a position. Positive favours the side to move; `Mate(n)`
/// is a forced mate in *n* moves, negative when it is against the side to move.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Score { Cp(i32), Mate(i32) }

/// Chess960-compatible: each right names the rook's starting file.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CastlingRights { /* [[Option<File>; 2]; 2] */ }

impl CastlingRights {
    pub fn short(&self, colour: Colour) -> Option<File>;
    pub fn long(&self, colour: Colour) -> Option<File>;
    pub fn any(&self, colour: Colour) -> bool;
    /// `KQkq` when the rook files are the classic ones, `AHah` otherwise.
    pub fn to_fen_field(&self) -> String;
}
```

---

## 3. Move

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
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
}

/// `kind` is the first case that applies, in the order castling, en passant,
/// promotion, capture, quiet; `is_capture` is true for a capturing promotion
/// and for en passant as well.
pub enum MoveKind { Quiet, Capture, EnPassant, Castling, Promotion }

/// Inline storage for the largest legal move count; never allocates.
pub struct MoveList<T = Move> { /* … */ }

impl<T: Copy + Default> MoveList<T> {
    pub fn new() -> MoveList<T>;
    pub fn clear(&mut self);
    pub fn push(&mut self, item: T);
    pub fn as_slice(&self) -> &[T];
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

Text is produced through the `Variant`, which owns the castling encoding and
SAN disambiguation.

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
side-paired value is `[T; 2]`, subscripted by `Side::index()`. Definitions are
those in `features.md` §1, repeated in the doc comments.

```rust
pub enum Side { Us, Them }

impl Side {
    pub const ALL: [Side; 2];
    /// 0 for us, 1 for them.
    pub const fn index(self) -> usize;
}

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
    pub exchange: [ExchangeFacts; 2],
    pub threats: ThreatFacts,
    pub tactics: [TacticsFacts; 2],
    pub endgame: EndgameFacts,
    pub planes: PlaneFacts,
    pub moves: MoveList<AnnotatedMove>,
}

pub struct PlacementFacts {
    /// Each side's units, by role P, N, B, R, Q, K.
    pub by_role: [[SquareSet; 6]; 2],
}

impl PlacementFacts {
    pub fn of(&self, side: Side, role: Role) -> SquareSet;
}

/// The `us` block, then the `them` block, the second after a null move.
pub struct ExchangeFacts {
    pub see_best_capture: i32,
    pub see_positive_capture_count: u16,
    pub see_equal_capture_count: u16,
    pub see_positive_total: i32,
}

pub enum Opposition { Direct, Distant }
pub enum DrawishMaterial { TwoKnights, WrongBishop, OppositeBishops }

pub struct EndgameFacts {
    pub king_centralisation: [u8; 2],
    pub race_plies: [u8; 2],
    pub opposition: Option<Opposition>,
    pub key_square_occupied: [bool; 2],
    pub wrong_colour_bishop: [bool; 2],
    pub drawish_material: Option<DrawishMaterial>,
}

impl EndgameFacts {
    /// Our race plies less theirs: negative when we promote first.
    pub fn race_plies_diff(&self) -> i32;
}

pub struct PawnFacts {
    pub pawns: [SquareSet; 2],
    pub passed: [SquareSet; 2],
    pub candidates: [SquareSet; 2],
    pub doubled: [SquareSet; 2],
    pub isolated: [SquareSet; 2],
    pub backward: [SquareSet; 2],
    pub defended: [SquareSet; 2],
    pub count_by_file: [[u8; 8]; 2],
    pub count_by_rank: [[u8; 8]; 2],
    pub open_files: FileSet,
    pub semi_open_files: [FileSet; 2],
    pub islands: [u8; 2],
    pub levers: [u8; 2],
    pub rams: u8,
    pub passer_lead_rank: [Option<u8>; 2],
    pub passer_protected: [u8; 2],
    pub passers_connected: [bool; 2],
    pub passer_unstoppable: [bool; 2],
    pub chain_max_length: [u8; 2],
    pub chain_base_attacked: [bool; 2],
    pub majority_by_wing: [[bool; 2]; 2],
    pub holes: [SquareSet; 2],
    pub holes_occupied: [u8; 2],
    pub fixed_pawns: [u8; 2],
    pub blocked_passers: [u8; 2],
    pub passer_distance: [Option<u8>; 2],
    pub passer_king_distance: [[Option<u8>; 2]; 2],
    pub passer_in_square: [bool; 2],
    pub passer_free_path: [u8; 2],
    pub half_open_at_enemy_king: [u8; 2],
    pub backward_on_semi_open: [u8; 2],
}

pub struct AttackFacts {
    pub by: [SquareSet; 2],
    pub by_pawns: [SquareSet; 2],
    pub by_role: [[SquareSet; 6]; 2],
    pub attacked: [SquareSet; 2],
    pub hanging: [SquareSet; 2],
    pub en_prise: [SquareSet; 2],
    pub pinned: [SquareSet; 2],
    pub defended: [SquareSet; 2],
    pub attacked_value: [i32; 2],
    pub hanging_value: [i32; 2],
    pub en_prise_value: [i32; 2],
    pub en_prise_max_value: [i32; 2],
    pub pinned_value: [i32; 2],
    pub skewer_candidates: [u8; 2],
    /* the placement the sets were read from */
}

impl AttackFacts {
    pub fn attackers_of(&self, sq: Square, side: Side) -> SquareSet;
    pub fn is_hanging(&self, sq: Square) -> bool;
    pub fn units(&self, side: Side) -> SquareSet;
}

/// Every set is read on the units it is about: index 0 is what we stand to
/// lose. Kings are in none of them.
pub struct ThreatFacts {
    pub threatened: [SquareSet; 2],
    pub threatened_value: [i32; 2],
    pub threat_max_gain: [i32; 2],
    pub attacked_by_lesser: [SquareSet; 2],
    pub queen_attacked_by_lesser: [bool; 2],
    pub overloaded_defenders: [SquareSet; 2],
    pub removable_defenders: [SquareSet; 2],
    pub loose: [SquareSet; 2],
    pub attacker_surplus: [SquareSet; 2],
    pub xray_through_enemy: [u8; 2],
    pub battery_count: [u8; 2],
    pub battery_at_king: [bool; 2],
}

pub struct AnnotatedMove {
    pub mv: Move,
    pub facts: MoveFacts,
}

pub struct MoveFacts {
    pub victim: Option<Role>,
    pub mover: Role,
    pub promotion: Option<Role>,
    pub gives_check: bool,
    pub gives_safe_check: bool,
    pub is_safe: bool,
    pub captures_hanging: bool,
    pub escapes_attack: bool,
    pub to_attacked_by_pawn: bool,
    pub is_castling: bool,
    pub is_en_passant: bool,
    pub see: i32,
    pub threat_created_max: i32,
    pub moves_attacked_unit: bool,
    pub blocks_check: bool,
    pub advances_passer: bool,
    pub creates_passer: bool,
    pub creates_isolated: bool,
    pub creates_doubled: bool,
    pub creates_backward: bool,
    pub opens_file_at_enemy_king: bool,
    pub our_ring_attackers_delta: i32,
    pub their_ring_attackers_delta: i32,
    pub own_hanging_delta: i32,
    pub their_hanging_delta: i32,
    pub leaves_unit_hanging: bool,
    pub gives_discovered_attack: bool,
}

impl Facts {
    /// The variant the facts were computed under.
    pub fn variant(&self) -> &'static str;
    /// The colour that plays `Side::Us`.
    pub fn side_to_move(&self) -> Colour;
    /// The side `colour` plays: the index into every side-paired fact.
    pub fn side(&self, colour: Colour) -> Side;
    /// A page of prose: material, structure, king safety, threats. Text not stable.
    pub fn summary(&self) -> String;
}

/// The `available`/`count` fields of a `TacticsFacts` carry the numbers; the
/// schema's `*_available` and `only_moves` bits are predicates over them.
impl TacticsFacts {
    pub fn check_available(&self) -> bool;
    pub fn safe_check_available(&self) -> bool;
    pub fn promotion_available(&self) -> bool;
    pub fn safe_promotion_available(&self) -> bool;
    pub fn capture_available(&self) -> bool;
    pub fn fork_available(&self) -> bool;
    pub fn pin_creation_available(&self) -> bool;
    pub fn only_moves(&self) -> bool;
}

/// Reusable buffers. One per thread; a search keeps one per node stack.
pub struct Scratch { /* … */ }
impl Scratch { pub fn new() -> Scratch; }
```

Facts are computed from the position and the variant's legal move list, so a
feature holds under every variant whose rules its definition assumes; the
schema names which those are (§6). A `Variant` supplies rules, never facts.

The `history` facts other than the halfmove clock come from `Game::facts` and
`Game::facts_in`, which have the history; `Position::facts` and
`Position::facts_in` emit them as zero, `known` included.

---

## 6. Schema and encoding

```rust
pub struct Schema { /* groups, versions, widths, offsets */ }
pub struct GroupSet(u16);
pub struct SchemaId([u8; 16]);

impl Schema {
    /// The v1 schema of `features.md`: 14 groups, 1930 values.
    pub fn v1() -> &'static Schema;
    pub fn id(&self) -> SchemaId;
    pub fn semver(&self) -> &str;
    pub fn width(&self) -> usize;
    pub fn width_of(&self, groups: GroupSet) -> usize;
    pub fn groups(&self) -> &[GroupSpec];
    pub fn group(&self, name: &str) -> Option<&GroupSpec>;
    /// The canonical text `id` hashes.
    pub fn canonical(&self) -> String;
    pub fn all(&self) -> GroupSet;
    /// Where a group sits in the schema order.
    pub fn group_index(&self, name: &str) -> Option<usize>;
    /// The named groups; `None` when a name is not the schema's.
    pub fn group_set(&self, names: &[&str]) -> Option<GroupSet>;
    pub fn feature_count(&self) -> usize;
    /// The features whose definitions hold under `variant`.
    pub fn features_for(&'static self, variant: &dyn Variant) -> FeatureSet;
}

impl GroupSet {
    pub const EMPTY: GroupSet;
    pub const fn only(index: usize) -> GroupSet;
    pub const fn contains(self, index: usize) -> bool;
    pub fn insert(&mut self, index: usize);
    pub fn remove(&mut self, index: usize);
    pub const fn is_empty(self) -> bool;
    pub const fn len(self) -> u32;
}

impl SchemaId {
    pub const fn bytes(self) -> [u8; 16];
}

pub struct GroupSpec {
    pub name: &'static str,
    pub version: u16,
    pub width: usize,
    pub features: &'static [FeatureSpec],
}

pub struct FeatureSpec {
    pub name: &'static str,
    /// Within the group, in values.
    pub offset: usize,
    pub width: usize,
    /// The encoding kind and scale, as `features.md` §6 spells it.
    pub encoding: &'static str,
    /// Variant names the feature is defined for; empty means all of them.
    pub variants: &'static [&'static str],
}

impl FeatureSpec {
    pub fn defined_for(&self, variant_name: &str) -> bool;
}

/// A subset of a schema's features.
pub struct FeatureSet { /* … */ }

impl FeatureSet {
    pub fn contains(&self, group: &str, feature: &str) -> bool;
    pub fn names(&self) -> impl Iterator<Item = (&'static str, &'static str)>;
}

impl Facts {
    /// Writes the selected groups in schema order; returns values written.
    /// A feature not defined for the facts' variant is written as zeros, so
    /// widths and offsets do not depend on the variant.
    /// Panics if `out` is shorter than `schema.width_of(groups)`.
    pub fn encode_into(&self, schema: &Schema, groups: GroupSet, out: &mut [f32]) -> usize;
    pub fn encode(&self, schema: &Schema, groups: GroupSet) -> Vec<f32>;
}

impl MoveFacts {
    pub const WIDTH: usize = 40;
    /// Panics if `out` is shorter than `WIDTH`.
    pub fn encode_into(&self, out: &mut [f32]);
}

/// One row per position, row-major, into `out` of `positions.len() * width`.
pub fn encode_positions(
    variant: &dyn Variant,
    positions: &[Position],
    schema: &Schema,
    groups: GroupSet,
    out: &mut [f32],
);

/// As above from FEN text; the error names the offending row.
pub fn encode_fens(
    variant: &dyn Variant,
    fens: &[&str],
    schema: &Schema,
    groups: GroupSet,
    out: &mut [f32],
) -> Result<(), RowError>;

pub struct RowError { pub row: usize, pub source: FenError }
```

Rows are independent and the crate spawns no threads; the caller parallelises.
`features.md` §4 names the features defined for classic chess only.

The v1 id is `16606f2b054a3281622fd2296f5ca13d`; its canonical text is checked
in as `rs_anglerfish/esca/tests/data/schema_v1.txt`.

---

## 7. `lichess` — evaluation dump reader (feature `lichess`)

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

## 8. Adding a variant

A variant is one more `Variant` implementation. `Position`, `Facts`,
`MoveFacts`, `Schema` and the encoders are untouched, because `Position` is
data and every fact derives from the variant's own legal move list.

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
let defined = Schema::v1().features_for(game.variant());
```

The model a variant must fit: an 8×8 board, the six standard roles, two
colours, one unit per square. Rules beyond that need the model widened first.

---

## 9. Python surface

Built from the same crate with feature `python`; distributed with `.pyi` stubs,
so every signature below is typed and checkable. Squares, roles, colours,
outcomes and castling styles are text on this surface, and a file set is the
string of its letters; the classes are `Variant`, `SquareSet`, `Move`,
`Position`, `Game`, `Schema`, `Facts` with its groups, `lichess.Batch`,
`pgn.Game`, and `uci.Engine` with its `Limits`, `Option`, `Info` and `Answer`.

```python
import esca
import numpy as np

esca.CLASSIC  # Variant
esca.CHESS960
esca.US, esca.THEM  # 0 and 1, the index of a side-paired fact

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
g.annotated_moves()  # list[AnnotatedMove]
g.outcome()  # "checkmate", "stalemate", … or None
g.claims()  # ["threefold_repetition", …]
print(g.position.summary())

# Position on its own; rules come from the variant argument
p = esca.Position.from_fen(fen)
p.fen, p.epd, p.side_to_move, p.en_passant, p.in_check  # str, str, "w", "e6", bool
p.piece_at("e1")  # "K"
f = p.facts(esca.CLASSIC)
f = g.facts()  # variant taken from the game

p.see("e5"), p.see_capture(mv)  # static exchange evaluation, in value units

f.side("b")  # esca.US or esca.THEM, whichever Black plays
f.pawns.passed[esca.US]  # SquareSet
list(f.attacks.hanging[esca.THEM])  # ["e5", …]
list(f.threats.threatened[esca.US])  # what we stand to lose
f.king.ring_attack_weight[esca.US]
print(f.summary())

# Schema and batch encoding
esca.SCHEMA  # Schema, also as esca.SCHEMA_V1
esca.SCHEMA_ID  # "16a7…", 32 hex chars
esca.WIDTH  # 1930
esca.MOVE_WIDTH  # 40
esca.schema()  # [{"name", "version", "width", "offset"}, …]
esca.features_for(esca.CHESS960)  # [("state", "in_check"), …]

# (n, w) float32
x = esca.encode(fens, variant=esca.CHESS960, groups=["state", "pawns"])
esca.encode_into(fens, out, groups=["state", "pawns"])  # caller's array
moves, mx = esca.encode_moves(fen)  # list[Move], (m, 40)
moves, mx, cuts = esca.encode_moves(fens)  # per FEN, (total, 40), (n + 1,) int64

# Lichess dump
for batch in esca.lichess.batches(path, batch_size=8192, min_depth=20):
    batch.fens, batch.features, batch.cp, batch.mate, batch.best_moves

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

    with engine.go(uci.Limits(infinite=True)) as search:  # streams reports
        for report in search:
            if report.depth and report.depth >= 12:
                search.stop()
        best = search.answer()

async with uci.AsyncEngine("stockfish") as engine:  # the same surface, awaited
    await engine.handshake()
    async for report in await engine.go(uci.Limits(depth=20)):
        print(report.depth, report.cp)
```

Every function that encodes takes keyword-only `variant`, `schema` and
`groups`, defaulting to `esca.CLASSIC`, `esca.SCHEMA` and every group:

```python
esca.encode(fens, *, variant=..., schema=..., groups=None) -> np.ndarray
esca.encode_into(fens, out, *, variant=..., schema=..., groups=None) -> None
esca.encode_moves(fen, *, variant=...) -> tuple[list[Move], np.ndarray]
esca.encode_moves(fens, *, variant=...) -> tuple[list[list[Move]], np.ndarray, np.ndarray]
esca.lichess.batches(path, *, batch_size=8192, min_depth=0,
                     variant=..., schema=..., groups=None) -> Iterator[Batch]
```

| Contract | |
|---|---|
| Returned arrays are C-contiguous `float32`, allocated in Rust and handed over without a copy. | |
| Batch calls release the GIL, parallelise rows, and reuse their buffers internally. | |
| A malformed FEN raises `ValueError` naming the row index. | |
| `encode_moves` takes one FEN or a sequence of them. A sequence stacks every position's move rows into one `(total, 40)` array and returns the `(n + 1,)` int64 offsets that cut it, so FEN `i` owns `rows[offsets[i]:offsets[i + 1]]`: no padding, and one array per call rather than one per position. | |
| `Position`, `Move`, `Facts` and their groups are immutable and picklable; `Position` and `Move` are hashable. | |
| A malformed game raises `ValueError` naming the line and column; the stream goes on with the next game. | |
| A batch row takes the deepest evaluation that reaches `min_depth`, and its best line; a record with none, with a placement no game can reach, or with an unreadable line is skipped. | |
| `batch.cp` and `batch.mate` are both `(n,)` float32 and side-relative: a row is a mate row when `mate` is not 0.0, and a centipawn row otherwise. | |

---

## 10. `pgn` — games as text (feature `pgn`)

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

## 11. `uci` — talking to an engine (feature `uci`)

Three layers: the protocol as values, which does no I/O; the engine as a
subprocess, which blocks and bounds every wait; and the variant handling that
keeps Chess960 honest. No async runtime, in either language.

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
        pub fn go(&mut self, limits: &Limits, budget: Duration) -> Result<Search<'_>, Error>;
        pub fn play(&mut self, game: &Game, limits: &Limits, budget: Duration)
            -> Result<Answer, Error>;
        pub fn is_ready(&mut self) -> Result<(), Error>;
        pub fn stop(&mut self) -> Result<(), Error>;
        pub fn ponderhit(&mut self) -> Result<(), Error>;
        pub fn quit(&mut self) -> Result<Option<i32>, Error>;
        /// The line-level interface, for tools and diagnostics.
        pub fn send_line(&mut self, text: &str) -> Result<(), Error>;
        pub fn next_line(&mut self, timeout: Duration) -> Result<Option<String>, Error>;
    }

    /// An iterator over a search's reports, ending with the engine's answer.
    pub struct Search<'a> { /* … */ }
    pub struct Answer { pub best: Option<Move>, pub ponder: Option<Move> }

    pub enum Error { Io(io::Error), Timeout { .. }, Died { .. }, Protocol(ProtocolError),
                     NotIdentified, NoSuchOption(String), BadValue { .. } }
}
```

| Contract | |
|---|---|
| Every wait is bounded. A search takes its own budget; everything else takes the engine's `timeout`. | |
| An engine that has exited is `Error::Died` on every call after, and is killed when the `Engine` is dropped. | |
| A line that breaks the grammar is `Message::Raw`, and a token that does lands in `Info::unknown`. Reading engine output never fails. | |
| A message the [`Session`] has no room for — a second `bestmove`, a `readyok` no `isready` asked for — is logged and dropped, not delivered. | |
| Dropping a `Search` stops it and waits for the answer, so the engine is left idle. | |
| `set_position` on a Chess960 game sets `UCI_Chess960` and writes castling king-to-rook; an engine that does not offer the option is an error, not a game played by the wrong rules. Classic games are written king-two-squares, and both spellings are read. | |
| Every wire line is logged through `log` at debug, `>>` out and `<<` in. | |
