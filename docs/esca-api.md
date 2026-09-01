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
    /// `e1g1`. Classic geometry only.
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
    Checkmate { winner: Color },
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

    pub fn side_to_move(&self) -> Color;
    pub fn piece_at(&self, sq: Square) -> Option<Piece>;
    pub fn by_role(&self, role: Role) -> SquareSet;
    pub fn by_color(&self, color: Color) -> SquareSet;
    pub fn by_piece(&self, piece: Piece) -> SquareSet;
    pub fn occupied(&self) -> SquareSet;
    pub fn king_of(&self, color: Color) -> Square;

    pub fn castling_rights(&self) -> CastlingRights;
    pub fn en_passant(&self) -> Option<Square>;
    pub fn halfmove_clock(&self) -> u32;
    pub fn fullmove_number(&self) -> u32;
    /// False when the position came from a four-field FEN.
    pub fn clocks_known(&self) -> bool;

    /// Zobrist key: equal for equal placement, side, castling rights and
    /// en-passant square; independent of the clocks. Valid as an identity
    /// within one process run — not across runs, and not stored.
    pub fn key(&self) -> Key;

    /// Facts of this position under `variant`.
    pub fn facts(&self, variant: &dyn Variant) -> Facts;
    /// Same, reusing buffers; no allocation.
    pub fn facts_in(&self, variant: &dyn Variant, scratch: &mut Scratch) -> Facts;

    /// Colours swapped and ranks flipped.
    pub fn mirrored(&self) -> Position;

    /// Board, side to move and state, for a human reader. Text not stable.
    pub fn summary(&self) -> String;
}

/// Chess960-compatible: each right names the rook's starting file.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CastlingRights { /* [[Option<File>; 2]; 2] */ }

impl CastlingRights {
    pub fn short(&self, color: Color) -> Option<File>;
    pub fn long(&self, color: Color) -> Option<File>;
    pub fn any(&self, color: Color) -> bool;
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

pub enum MoveKind { Quiet, Capture, EnPassant, Castling, Promotion }

/// Fixed capacity, never allocates.
pub struct MoveList { /* … */ }

impl MoveList {
    pub fn new() -> MoveList;
    pub fn clear(&mut self);
    pub fn as_slice(&self) -> &[Move];
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
    pub fn position(&self) -> &Position;
    pub fn start_position(&self) -> &Position;
    pub fn moves(&self) -> &[Move];
    /// Every position from the start to the current one.
    pub fn positions(&self) -> impl Iterator<Item = &Position>;
    pub fn ply(&self) -> u32;

    pub fn legal_moves(&self) -> MoveList;
    pub fn annotated_moves(&self) -> Vec<AnnotatedMove>;
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

---

## 5. Facts

Grouped plain structs with public fields; the accessor *is* the field. Every
side-paired value is `[T; 2]`, indexed by `Side`. Definitions are those in
`features.md` §1, repeated in the doc comments.

```rust
pub enum Side { Us, Them }

pub struct Facts {
    pub state: StateFacts,
    pub material: MaterialFacts,
    pub pawns: PawnFacts,
    pub pieces: PieceFacts,
    pub king: KingFacts,
    pub mobility: MobilityFacts,
    pub attacks: AttackFacts,
    pub tactics: [TacticsFacts; 2],
    pub planes: PlaneFacts,
    pub moves: Vec<AnnotatedMove>,
}

pub struct PawnFacts {
    pub pawns: [SquareSet; 2],
    pub passed: [SquareSet; 2],
    pub candidates: [SquareSet; 2],
    pub doubled: [SquareSet; 2],
    pub isolated: [SquareSet; 2],
    pub backward: [SquareSet; 2],
    pub open_files: FileSet,
    pub semi_open_files: [FileSet; 2],
    pub islands: [u8; 2],
    pub levers: [u8; 2],
    pub rams: u8,
    /* … */
}

pub struct AttackFacts {
    pub by: [SquareSet; 2],
    pub by_pawns: [SquareSet; 2],
    pub by_role: [[SquareSet; 6]; 2],
    pub hanging: [SquareSet; 2],
    pub en_prise: [SquareSet; 2],
    pub pinned: [SquareSet; 2],
}

impl AttackFacts {
    pub fn attackers_of(&self, sq: Square, side: Side) -> SquareSet;
    pub fn is_hanging(&self, sq: Square) -> bool;
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
}

impl Facts {
    /// A page of prose: material, structure, king safety, threats. Text not stable.
    pub fn summary(&self) -> String;
}

/// Reusable buffers. One per thread; a search keeps one per node stack.
pub struct Scratch { /* … */ }
impl Scratch { pub fn new() -> Scratch; }
```

Facts are computed from the position and the variant's legal move list, so a
group holds under every variant whose rules its definitions assume; the schema
names which those are (§6). A `Variant` supplies rules, never facts.

---

## 6. Schema and encoding

```rust
pub struct Schema { /* groups, versions, widths, offsets */ }
pub struct GroupSet(u16);
pub struct SchemaId([u8; 16]);

impl Schema {
    /// The v0 schema of `features.md`: 9 groups, 1065 values.
    pub fn v0() -> &'static Schema;
    pub fn id(&self) -> SchemaId;
    pub fn semver(&self) -> &str;
    pub fn width(&self) -> usize;
    pub fn width_of(&self, groups: GroupSet) -> usize;
    pub fn groups(&self) -> &[GroupSpec];
    pub fn group(&self, name: &str) -> Option<&GroupSpec>;
    /// The canonical text `id` hashes.
    pub fn canonical(&self) -> String;
    pub fn all(&self) -> GroupSet;
    /// The groups whose definitions hold under `variant`.
    pub fn groups_for(&self, variant: &dyn Variant) -> GroupSet;
}

pub struct GroupSpec {
    pub name: &'static str,
    pub version: u16,
    pub width: usize,
    pub features: &'static [FeatureSpec],
    /// Variant names the group is defined for; empty means all of them.
    pub variants: &'static [&'static str],
}

impl Facts {
    /// Writes the selected groups in schema order; returns values written.
    /// A group not defined for the facts' variant is written as zeros, so
    /// widths and offsets do not depend on the variant.
    /// Panics if `out` is shorter than `schema.width_of(groups)`.
    pub fn encode_into(&self, schema: &Schema, groups: GroupSet, out: &mut [f32]) -> usize;
    pub fn encode(&self, schema: &Schema, groups: GroupSet) -> Vec<f32>;
}

impl MoveFacts {
    pub const WIDTH: usize = 24;
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

---

## 7. `lichess` — evaluation dump reader (feature `lichess`)

```rust
pub mod lichess {
    pub struct Record { pub epd: String, pub evals: Vec<Eval> }
    pub struct Eval { pub depth: u32, pub knodes: u64, pub pvs: Vec<Pv> }
    pub struct Pv { pub score: Score, pub line: String }
    pub enum Score { Cp(i32), Mate(i32) }

    impl Record {
        /// The four-field FEN parsed; `clocks_known()` is false.
        pub fn position(&self) -> Result<Position, FenError>;
    }
    impl Pv {
        pub fn best_move(&self, variant: &dyn Variant, pos: &Position)
            -> Result<Move, MoveParseError>;
    }

    /// Streams a Zstandard-compressed JSON-lines dump; never holds the file.
    pub fn read(path: &Path) -> io::Result<impl Iterator<Item = io::Result<Record>>>;
    pub fn read_from<R: BufRead>(reader: R) -> impl Iterator<Item = io::Result<Record>>;
}
```

Every `evals` entry of a record is exposed; choosing among depths is the
caller's policy.

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
let groups = Schema::v0().groups_for(game.variant());
```

The model a variant must fit: an 8×8 board, the six standard roles, two
colours, one unit per square. Rules beyond that need the model widened first.

---

## 9. Python surface

Built from the same crate with feature `python`; distributed with `.pyi`
stubs, so every signature below is typed and checkable.

```python
import esca
import numpy as np

esca.CLASSIC            # Variant
esca.CHESS960
esca.US, esca.THEM      # Side

# Game — variant defaults to classic everywhere it is optional
g = esca.Game()
g = esca.Game(variant=esca.CHESS960, seed=518)
g = esca.Game.from_fen(fen, variant=esca.CHESS960)

g.castling_output = esca.KING_TO_ROOK   # or esca.KING_TWO_SQUARES
g.play("e2e4")                      # UCI or a Move
g.play_san("Nf3")
g.move_to_uci(mv)
g.undo()
g.position                          # Position, immutable and hashable
g.moves                             # tuple[Move, ...] played
g.legal_moves()                     # list[Move]
g.annotated_moves()                 # list[AnnotatedMove]
g.outcome()                         # Outcome | None
g.claims()                          # list[DrawClaim]
print(g.position.summary())

# Position on its own; rules come from the variant argument
p = esca.Position.from_fen(fen)
f = p.facts(esca.CLASSIC)
f = g.facts()                       # variant taken from the game

f.pawns.passed[esca.US]             # SquareSet
list(f.attacks.hanging[esca.THEM])  # [Square, …]
f.king.ring_attack_weight[esca.US]
print(f.summary())

# Schema and batch encoding
esca.SCHEMA_ID                      # "b7f0…", 32 hex chars
esca.WIDTH                          # 1065
esca.schema()                       # [{"name","version","width","offset"}, …]
esca.groups_for(esca.CHESS960)      # ["state", "material", …]

x = esca.encode(fens, groups=["state", "material", "pawns"])   # (n, w) float32
esca.encode_into(fens, out)                                    # caller's array
moves, mx = esca.encode_moves(fen)                             # list[Move], (m, 24)

# Lichess dump
for batch in esca.lichess.batches(path, batch_size=8192, min_depth=20):
    batch.fens, batch.features, batch.cp, batch.mate, batch.best_moves
```

| Contract | |
|---|---|
| Returned arrays are C-contiguous `float32`, allocated in Rust and handed over without a copy. | |
| Batch calls release the GIL, parallelise rows, and reuse their buffers internally. | |
| A malformed FEN raises `ValueError` naming the row index. | |
| `Position`, `Move`, `Facts` and their groups are immutable and picklable; `Position` and `Move` are hashable. | |
