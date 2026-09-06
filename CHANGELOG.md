# Changelog

## Unreleased

- The typed facts are the API. `Position::facts()` answers 221 named facts in
  14 groups, and 27 more about every legal move, each with its own type: a
  `bool`, a count, a `SquareSet`, a `FileSet`, an `Option` or a named enum.
  Nothing is scaled, capped or one-hot, and nothing has to be looked up in a
  manifest to be read.
- Every fact names the colour it is about. A value that differs between the
  two sides is a `ByColour<T>` — `.white`, `.black`, `.of(colour)`, `[colour]`
  — so the same placement reads the same whoever is to move. `Side`, `Us`,
  `Them`, `esca.US`, `esca.THEM` and every side-to-move-relative index are
  gone; a fact that really is about the side to move, such as `state.in_check`,
  is a plain value that says so.
- The facts describe themselves. A catalogue carries every group, field, type
  and meaning as data, `docs/features.md` is generated from it, and Python
  walks a group without knowing it: `Facts.groups()`, `facts.group(name)` and
  `to_dict()`. Rust serialises the same shape behind the new `serde` feature.
- Python gains `esca.ByColour` and a first-class `esca.FileSet`, and
  `SquareSet.files` says which files a set of squares stands on. A colour is
  written `"white"` or `"black"` everywhere it is answered, and read from
  either name, or from `"w"` or `"b"`, in any case.
- The `f32` machinery is gone: `Schema`, `SchemaId`, `GroupSet`, `FeatureSet`,
  the feature manifests, `Facts::encode`, `encode_positions`, `encode_fens`,
  `MoveFacts::encode_into`, the Python `encode`, `encode_into`, `encode_moves`,
  `features_for`, `SCHEMA`, `WIDTH` and `esca.lichess.batches`, with `numpy`
  off the dependency list. Nothing a net eats is normalised here any more; what
  replaces it is the typed tensor export below, which keeps every fact's own
  width and sign. The Rust `lichess` reader, which hands over typed records,
  stays.
- `bishops_light` and `bishops_dark` now count the square colours they name.
  They were read in the mover's view, so with Black to move they answered the
  other way round.
- Renames the same audit made: `state.can_castle_short` and `can_castle_long`,
  `state.en_passant_capture_legal`, `material.phase_points` (the count, not a
  fraction), `material.insufficient_material`, `pawns.queenside_majority` and
  `kingside_majority`, `pawns.passer_own_king_distance` and
  `passer_enemy_king_distance`, `pieces.rooks_on_seventh_rank`,
  `pieces.rook_on_seventh_with_king_on_eighth`, `pieces.knight_pair`,
  `king.shield_file_open` and `shield_file_semi_open_for_enemy`,
  `attacks.controlled` with `by_pawns` … `by_kings` beside it, and the
  per-role fields of `placement`, `material` and `mobility`. A move's deltas
  are named for the side they are about: `mover_hanging_delta`,
  `enemy_hanging_delta`, `mover_king_attackers_delta`,
  `enemy_king_attackers_delta`.
- `endings` names the ending a position is, needing no feature: a canonical
  material signature (`KRPvKR`, stronger side first), a catalogue of 32 named
  classes from `KvK` to the pawn endings, what theory says the result is, and
  the method it is played by — the box method, Lucena, Philidor, key squares,
  the opposition, the rule of the square, the wrong bishop and the wrong rook
  pawn. A position is an ending while neither side has more than two pieces;
  above that it says so and still answers the material. The few
  position-specific facts that overturn theory — the pawn's race, the bishops'
  square colours, the opposition — are grouped as evidence, each behind the
  reason it belongs to, and the verdict and the technique are adjusted by
  them. `endings::classify`, `Position::ending` and `Game::ending` in Rust,
  `esca.endings` in Python.
- Every value the `explain` layer answers with says what it means in one plain
  English sentence, through `describe()`: each enum, and each aggregate a
  summary reads naturally off. A sentence names the squares its value carries,
  and an aggregate lists every reason that applies, so a reader who knows no
  chess gets the whole situation in words as well as in fields. The enum value
  stays the answer; the sentence is derived from it.
- `explain::Castling` carries the `colour` and the `wing` it answers for, which
  is what lets it say "White cannot castle short: …" on its own.
- `mcp/` is a second distribution from this repository: `chess-esca-mcp`, an MCP server over the
  library, serving the whole state of a position, a move's legality with every reason behind it,
  the named facts by group, ECO names, Polyglot book moves and PGN both ways, as JSON grouped by
  reason with the squares each was read off. No engine and no search. Its version is the crate's
  and it pins the matching `esca`, so one tag releases both; the plugin that installs it lives in
  the `plugins` repository.
- The crate lives at the root of its own repository,
  <https://github.com/AnglerfishChess/esca>; the engine and its trainer go on
  at <https://github.com/AnglerfishChess/anglerfish>.

- Facts are typed for their purpose. A per-file count is a `PerFile<u8>` and a
  per-rank count a `PerRank<u8>` — eight values whose order is in the type, not
  in a comment — and every fact whose value is a run of positions states which
  end is which in its own meaning, which a test enforces. The three shield
  values keep their `[T; 3]` and now say "queenside first; `shield_files` names
  them". `pawns.count_by_file` and `count_by_rank` are those types in Rust and
  eight-element tuples, not lists, in Python.
- One JSON form. Rust's `serde::Serialize` (feature `serde`) and Python's
  `to_dict()` write exactly the same bytes: a `ByColour` keyed by colour, a
  `SquareSet` as its square names in board order, a `FileSet` as its letters, a
  run as a list in the order stated, an absent `Option` as `null`, a named
  value as its name. `docs/facts.schema.json` is that format as a JSON Schema,
  and `tests/data/facts/` holds the answer for five named positions, which both
  suites check. `fields()` is gone; `to_dict()` is the one entry point, per
  group and whole.
- A typed tensor export, `esca::facts::tensors` (feature `tensors`) and
  `esca.tensors` (extra `esca[tensors]`, NumPy). One array per fact, batch
  first, each keeping the width and sign it was declared with: nothing is
  scaled, normalised or cast to a float, and sub-byte only ever means bit
  packing. `layout()` names every array, its dtype, its shape and what each
  axis means; a batch is written and read as safetensors, which both languages
  do by hand rather than take a dependency for.
- The catalogue parses each declared type once into a tree, and
  `docs/features.md` (now with `dtype` and `shape` columns), the JSON schema,
  the tensor layout and the facts section of `python/esca/_esca.pyi` are all
  generated from it: `cargo run --example catalogue --all-features`, with a
  test in each language failing when any of them is stale.
- The Python facts are typed exactly: `ByColour[SquareSet]`,
  `ByColour[PerFile[int]]`, `Literal` unions where a value is one of a fixed
  list of names, and a `TypedDict` per group for what `to_dict()` answers, so
  `facts.to_dict()["pawns"]["passed"]["white"]` is a `list[str]` to a checker.
  Every facts class gained `__eq__`, `__hash__` and a `__repr__` that shows the
  values; `ByColour` gained `__match_args__` and `ByColour[T]` at runtime.
- `history.last_move_victim` and the other role-valued facts keep their letter
  spelling (`p n b r q k`); only the three named enums are snake-case names.
- `tactics.winning_capture_max_gain` reads "the largest victim value less
  capturer value", the same quantity said without a Unicode minus.
- NumPy is no longer a dependency of `esca`; it is the `tensors` extra.
- The MCP `facts` tool answers in that one JSON form, its hand-written
  positional labels dropped, and carries a `field_notes` entry giving the
  catalogue meaning of every field whose value is a run of positions.
- pyrefly now checks `python/`, which the paths it had named did not; that
  turned up a test helper still calling the encoder removed at 0.3.0.

## 0.3.0 (2026-09-04)

- The `explain` layer answers a rules question with the evidence behind it,
  every reason that applies at once and each carrying the squares it was read
  off. `Position::castling` names the right, the rook, the check, every
  covered square of the king's path with the units covering it, and every
  unit standing on the path; `en_passant_status` names each pawn that could
  take and what forbids it, the rank pin that binds neither pawn alone
  included. `checkers`, `attackers`, `between`, `pins` and `skewers` hand over
  the squares behind an answer. Chess960 throughout: the castling path is the
  one the variant's rule names, so a king that never moves and a rook the king
  crosses are answered by the same fields.
- `Game::repetition_status` lists the plies the position has stood at and the
  earlier plies with the same placement that do not count, with what differs;
  `fifty_move_status` counts the clock down to the claim and to the automatic
  draw and names the move that last cleared it; `draw_status` lists every draw
  that holds rather than the first, a stalemate carrying the king's covered
  escape squares and what holds each other unit; `claims_after` says what a
  move would earn without playing it.
- Python has it as `esca.explain`, reached from methods on `Position` and
  `Game`. None of it is part of the feature schema.
- The UCI client speaks asynchronously on both surfaces. Rust gains the `tokio`
  feature and `uci::tokio::Engine`: the same conversation, the same values and
  the same `Error` as the blocking `Engine`, with every method awaited and
  every future cancellation-safe. A search let go of unanswered asks the engine
  to stop, and the engine settles on its next call, since a drop cannot wait.
- `esca.uci.AsyncEngine` is asyncio all the way down: it drives the process
  itself, with no worker thread and no blocking call under it, so `run`, which
  named that thread, is gone. Cancelling an awaited `play`, `analyse` or
  `answer` stops the search and leaves the engine usable. Its `timeout` is
  settable, as the blocking `Engine`'s is.
- `esca.uci.protocol` puts the protocol core on the Python surface, for a
  client of one's own: `Command` constructors that render a line, `parse`
  reading one line of engine output into a `Message`, and `Session` saying what
  may come next.
- Both Rust clients and the asyncio one cap the lines an engine has written and
  the client has not read at 4096. The oldest line that carries no part of the
  conversation goes first, and `dropped_lines` counts what went, so an engine
  writing faster than it is read costs a bounded amount of memory rather than
  its answer.
- `Position::polyglot_key`, needing no feature: the key the Polyglot format
  fixes, the same number in every run and in every program that implements it,
  so it is a key to store and to send where `Position::key` is not. Python
  reads it as `position.polyglot_key`.
- Opening books, behind the `polyglot` feature and always in the wheel:
  `polyglot::Book` memory-maps a book and answers a position with its entries
  in book order, its heaviest, or one drawn by weight from a seed; a move that
  is not legal in the position asked about is refused, never guessed.
  `Book::write` and `polyglot::Builder` go the other way, counting the moves
  of played games or of a PGN source into a sorted, merged book with a maximum
  ply and a minimum count. Python mirrors it as `esca.polyglot`, which also
  has `download` for fetching a book to a path with an optional digest.
- The ECO catalogue, behind the `openings` feature and always in the wheel:
  `openings::lookup` gives a position its code and name, and `Game::opening`
  the deepest one a game has reached. Keyed by position, so a transposition is
  named. Python mirrors it as `esca.openings` and `game.opening()`.

## 0.2.1 (2026-09-04)

- The PyPI wheels carry `esca.pgn` and `esca.uci`; 0.2.0's were built
  without them.

## 0.2.0 (2026-09-03)

- PGN, behind the `pgn` feature and always in the wheel: `pgn::Reader` over a
  file or a string, tolerant of the archives in the wild and, with
  `skipping()`, of one bad game among many; `pgn::Game` with headers,
  mainline, variations, comments and NAGs, `mainline_game()` to play it and
  `Game::to_pgn()` to write one, X-FEN setups included. Python mirrors it as
  `esca.pgn`.
- A UCI client, behind the `uci` feature and always in the wheel:
  `uci::protocol` is the protocol as values and does no I/O; `uci::Engine`
  runs the process with every wait bounded, so a silent or dead engine is an
  error, not a hang. `set_position` sets `UCI_Chess960` and the castling
  spelling from the game's variant, and refuses a Chess960 game to an engine
  that does not offer the option. Python has `esca.uci.Engine` (GIL released
  around every wait) and `AsyncEngine`, the same surface as coroutines, with
  no async runtime under it.
- The v1 schema, `schema_semver` 1.0.0: fourteen groups in the order
  `features.md` §2 gives them. `Schema::v1` replaces `Schema::v0`, which is
  gone, as is every net trained on it; Python names it `esca.SCHEMA_V1` and
  `esca.SCHEMA`.
- The clock and repetition facts move out of `state` into the new `history`
  group, so a training source without clocks omits one group name.
  `Facts::history` carries them; `StateFacts` no longer does. The group also
  says what the recent plies did: captures and checks in the last eight, the
  plies since either, the material trend over them, and what the last move
  moved, took and gave. A `Game` supplies them; a bare `Position` does not.
- `pawns` gains the structure a player names out loud: the longest chain and
  whether its base is under attack, the majority on each wing, the holes a
  side can never cover again and the enemy minors sitting on them, fixed and
  blockaded pawns, the lead passer's distance to promotion, both kings'
  distances to its promotion square, whether the defending king is in its
  square and whether its path is empty, the files semi-open at the enemy king,
  and the backward pawns the enemy can actually attack.
- `pieces.knights_on_outpost` becomes `pieces.minors_on_outpost`, which counts
  bishops too; `king.king_distance` drops the two distances two kings can
  never stand at; `state.repetition_available_them` is gone.
- `pieces` gains what makes a minor good or bad and a rook worth its rank: the
  own fixed pawns standing on an own bishop's colour, the bishop pair against a
  knight pair as one signed value, a rook on the seventh with the enemy king
  shut on the eighth, and the units with nowhere safe to go, by count and by
  value. A trapped unit is `mobility.immobile_pieces` with safety asked of the
  squares it does reach.
- `attacks` gains the value twins `attacked_count`, `attacked_value`,
  `en_prise_value` and `pinned_value`.
- The new `placement` group: the raw board as twelve 64-square planes in the
  mover's view, so a run measuring the augmentation against the board alone
  selects a group instead of building its own input. `Facts::placement`.
- The `endgame` group fills in: how central each king stands, the plies each
  side's leading passer still needs and the difference between them, the
  opposition the kings stand in, a king on a key square of its own passer, a
  wrong-colour bishop, and the three drawn material configurations
  `material.insufficient_material` deliberately excludes. `Facts::endgame`,
  with the `Opposition` and `DrawishMaterial` enums. The opposition is no
  longer on the excluded list of `features.md` §4.
- `history.last_move_was_check` is gone: with a history it is `state.in_check`
  read from the other end, and a check cannot survive the move that answers
  it.
- Static exchange evaluation: `Position::see` and `Position::see_capture`, the
  new `exchange` group over them, and `tactics.winning_capture_available`,
  `winning_capture_max_gain`, `equal_capture_count` and `losing_capture_count`
  redefined from the exchange instead of the 1-ply victim-versus-capturer
  test. SEE is no longer on the excluded list of `features.md` §4.
- The new `threats` group, `Facts::threats`: what each side stands to lose,
  read from the exchange on each of its own units — which are threatened, what
  they are worth and the largest gain against them — plus the units a lesser
  enemy unit attacks, the overloaded and removable defenders, the loose units,
  the units more cheap attackers bear on than defenders, and the slider
  geometry a threat comes from: x-rays through an enemy unit, batteries, and
  batteries whose line meets the enemy king ring.
- The `move` schema grows from 24 values to 40. Each legal move now also
  carries its SEE, the largest one it leaves us next, whether it moves an
  attacked unit or interposes on a check, whether it advances or creates a
  passer, the isolated, doubled and backward pawns it makes, a king file it
  opens for us, what it does to both king rings and to both hanging counts,
  whether it leaves a piece hanging, and whether it uncovers a slider.
  `esca.MOVE_WIDTH` and `MoveFacts::WIDTH` say 40.
- `king` gains the defensive half of the siege and the exposure a player reads
  off the board: the pieces defending their own king's ring and their weight,
  the besieging weight less that, the open rays off the king, whether it has
  luft, which side it reads as having castled to, and whether the two kings
  stand on opposite wings. `KingFacts` carries them, with the `CastledSide`
  enum and `KingFacts::ring_attacker_surplus`; `castled_side` joins the
  features `features.md` §4 keeps to classic chess.
- `tactics` gains six bits per side: a check that also captures safely, a
  discovery onto the enemy queen, a back-rank mate threat, a quiet move that
  leaves more to be won than stands to be won now, a side with no safe move at
  all, and a promotion the exchange on its square makes profitable.
- The move row is a section of the schema of its own, named `move`, with a
  version, named features, widths and encodings, and its text folded into the
  one `schema_id`. A checkpoint that stores the id now refuses a move row of
  another shape as surely as a position row of one. `Schema::moves`,
  `GroupSpec::canonical` and `GroupSpec::feature`; `esca.MOVE_SCHEMA` and
  `Schema.moves()` in Python.
- `material` gains the bishop-pair imbalance and the piece-value difference,
  `mobility` the safe-mobility difference per type, and `planes` a ninth
  plane: where their threatened units stand.

## 0.1.0 (2026-09-03)

First release. `esca` is published to crates.io and PyPI; the engine and the
trainer ship in the repository only.

- `esca`: `Position`, `Game` and the `Variant` trait, with `Classic` and
  `Chess960` behind it; FEN and EPD, X-FEN and Shredder-FEN castling, legal
  move generation, UCI and SAN move text, and every terminal and claimable
  outcome.
- `Facts`, the v0 schema: nine groups of side-relative position facts plus
  per-move facts, a `schema_id` pinned by golden fixtures, and batch encoders
  that write 1065-wide `f32` rows.
- The `esca` Python package: an abi3-py312 wheel built from the crate, typed,
  with NumPy batch encoding.
- The `lichess` reader: streams the Lichess evaluation dump into encoded
  batches with their targets.
- `anglerfish-core`: the `anglerfish` UCI binary — protocol loop, bounded
  search, move-picking strategies, and the `Evaluator`/`Policy` traits a net
  will plug into (`Uniform` and `Material` until then).
