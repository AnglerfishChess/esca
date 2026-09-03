# esca vocabulary

Every term esca's API, docs and tests use. Standard chess and chess-engine
terms are defined here in esca's exact sense; where a term has several
readings in the wild, the reading below is the one esca means.

Terms are grouped by area. Within a group they are ordered so that a term is
defined before it is used.

---

## 1. Board and pieces

| Term | Definition |
|---|---|
| **board** | The 8×8 grid of squares. Files a–h run left to right from White's side, ranks 1–8 run from White's side to Black's. |
| **square** | One of the 64 cells, named file-then-rank (`e4`). Indexed 0–63 with a1 = 0, h1 = 7, a8 = 56, h8 = 63. |
| **file** | A column of 8 squares, a–h. |
| **rank** | A row of 8 squares, 1–8. |
| **square colour** | Light or dark. a1 is dark. A bishop never leaves the colour it stands on. |
| **colour** (of a player) | White or Black. Also called *side*. |
| **role** | What a piece is, without its colour: pawn, knight, bishop, rook, queen, king. |
| **piece** | A role plus a colour, e.g. a black knight. |
| **unit** | Any piece on the board, used when the distinction between a pawn and a piece would otherwise confuse ("no piece other than kings" excludes pawns; "no unit" excludes nothing). |
| **occupancy** | The set of squares carrying a unit, of either colour or of one. |
| **back rank** | The rank a colour's king and rooks start on: rank 1 for White, rank 8 for Black. |
| **relative rank** | A rank counted from the owner's own back rank: relative rank 1 is that colour's back rank, relative rank 8 is where its pawns promote. |
| **mover's view** | The board as seen by the side to move: when Black is to move, ranks are flipped (rank *r* becomes rank 9−*r*) and colours are swapped. Files are not mirrored. |
| **promotion** | Replacing a pawn that reaches its relative rank 8 with another piece of the same colour. |
| **material** | The units a side has, or their summed conventional value (P=1, N=B=3, R=5, Q=9). |

---

## 2. Moves and notation

| Term | Definition |
|---|---|
| **move** | One player's single action: moving a unit, possibly capturing, possibly promoting, including castling and en passant. |
| **ply** | One move by one side. The unit in which search depth and game length are counted. |
| **half-move** | A synonym of *ply*. |
| **full move** | A ply by White plus the reply by Black. |
| **full-move number** | The ordinal of the current full move, starting at 1 and incremented after each Black move. Part of a FEN. |
| **quiet move** | A move that neither captures nor promotes. |
| **capture** | A move whose destination holds an enemy unit, which is removed. |
| **en passant** | A pawn that has just advanced two squares may be captured, on the very next ply only, by an enemy pawn moving to the square it skipped. |
| **castling** | The king and one rook move in one ply, subject to castling rights, an empty path, and the king neither standing in, passing through, nor landing in check. Named *king-side* (short) and *queen-side* (long) after the rook's starting wing. |
| **null move** | A search device, not a chess move: the side to move passes, everything else unchanged. Illegal when that side is in check. |
| **pseudo-legal move** | A move legal by the moving unit's movement rules and by occupancy, ignoring whether it leaves its own king in check. |
| **legal move** | A pseudo-legal move that does not leave its own king attacked. Only legal moves may be played. |
| **UCI move notation** | Origin square, destination square, and for a promotion the lower-case promotion role: `e2e4`, `e7e8q`. The move format engines speak. |
| **UCI castling encoding** | In classic chess, castling is written as the king's two-square move (`e1g1`). In Chess960 it is written king-to-rook (`e1h1`), because the king's destination can coincide with its origin. |
| **SAN** | *Standard Algebraic Notation*: the human notation used in books and PGN — `Nf3`, `exd5`, `O-O`, `Qxh7#` — with only as much origin information as disambiguation needs. |
| **PGN** | *Portable Game Notation*: the text format for a whole game — a tag section of `[Name "value"]` pairs, then the movetext in SAN, ending in a result. |
| **tag pair** | One `[Name "value"]` line of a PGN tag section. |
| **seven-tag roster** | The tags every PGN game carries, in this order: `Event`, `Site`, `Date`, `Round`, `White`, `Black`, `Result`. |
| **movetext** | The moves of a PGN game, with their move numbers, comments, glyphs and variations, and the result marker that ends them. |
| **RAV** (recursive annotation variation) | A `( … )` alternative inside movetext, replacing the move it follows and holding movetext of its own, variations included. |
| **NAG** (numeric annotation glyph) | A `$n` annotation of a move. `$1`–`$6` are the `!`, `?`, `!!`, `??`, `!?` and `?!` suffixes; esca keeps every glyph as its number. |
| **export format** | The strict PGN a program writes: the seven-tag roster first, one blank line, movetext wrapped at 80 columns, one space between tokens. |
| **PV** (principal variation) | The sequence of moves an engine considers best from a position; its first move is the best move. |
| **multi-PV** | Several PVs from one position, ranked, each with its own score. |
| **centipawn** (cp) | A score unit: 100 cp is nominally one pawn. `mate n` is a separate kind of score meaning forced mate in *n* moves. |

---

## 3. Game state and rules

| Term | Definition |
|---|---|
| **side to move** | The colour whose turn it is. |
| **castling rights** | Which castlings remain available, per colour and per wing. A right is lost permanently when its king or its rook moves, and is not restored. |
| **en-passant square** | The square a pawn skipped over on the immediately preceding ply, if any. |
| **halfmove clock** | Plies since the last capture or pawn move, reset to 0 by either. |
| **check** | The side to move's king stands on a square attacked by the opponent. |
| **double check** | Check given by two units at once; only a king move can answer it. |
| **discovered check** | Check given by a unit that did not move, uncovered by the unit that did. |
| **checkmate** | The side to move is in check and has no legal move. It loses. |
| **stalemate** | The side to move is not in check and has no legal move. Draw. |
| **insufficient material** | Neither side has material that could ever deliver mate. Draw, immediately. |
| **repetition** | The same position — same placement, same side to move, same castling rights, same en-passant possibility — occurring more than once in a game. |
| **threefold repetition** | Three occurrences; a player may *claim* a draw. |
| **fivefold repetition** | Five occurrences; the draw is automatic, no claim needed. |
| **fifty-move rule** | At a halfmove clock of 100 (fifty full moves without capture or pawn move) a player may *claim* a draw. |
| **seventy-five-move rule** | At a halfmove clock of 150 the draw is automatic. |
| **claimable vs automatic** | A claimable draw ends the game only if a player asks for it, so the position is still playable; an automatic draw ends it regardless. esca reports both kinds and marks which is which. |
| **outcome** | How a game ended: checkmate (with the winner), or one of the draw conditions. A position with legal moves and no automatic draw has no outcome. |
| **FEN** | *Forsyth–Edwards Notation*: one line of six space-separated fields — placement, side to move, castling rights, en-passant square, halfmove clock, full-move number — describing a position completely. |
| **EPD** | *Extended Position Description*: the first four FEN fields plus optional named operations. A four-field FEN, i.e. one without clocks, is an EPD without operations. |
| **Zobrist key** | A hash of a position, built by XOR-ing per-piece-per-square random constants plus side, castling and en-passant constants. Equal positions have equal keys; distinct positions collide only by chance. |

---

## 4. Variants

| Term | Definition |
|---|---|
| **variant** | A complete set of chess rules: initial position, movement, castling semantics, promotion roles, and terminal conditions. |
| **classic chess** | The standard game. esca's default variant everywhere. |
| **Chess960** (Fischer Random) | The back rank is one of 960 shuffles, mirrored for both colours, with the king between the rooks and bishops on opposite square colours. Castling keeps the classic *destination* squares (king to c1/g1, rook to d1/f1) whatever the starting files were. |
| **X-FEN / Shredder-FEN** | FEN dialects that write castling rights as the rook's *file* (`AHah`) instead of `KQkq`, so a shuffled back rank is unambiguous. esca stores rights in this form and renders classic-compatible `KQkq` when the position permits it. |

---

## 5. esca structures

| Term | Definition |
|---|---|
| **Variant** (esca type) | The Rust trait whose implementations *are* the variants: initial position, legal move generation, castling and promotion semantics, terminal conditions, move text. Built-in implementations: `Classic` and `Chess960`. A new variant is a new implementation and nothing else. |
| — naming | One name on both sides, Rust and Python: *variant* is the word chess players, PGN tags and the UCI protocol already use for the thing being selected. |
| **Position** | An immutable, variant-agnostic snapshot: placement, side to move, castling rights (stored Chess960-compatibly), en-passant square, halfmove clock, full-move number. It answers everything that needs no rules; anything that needs rules takes a `Variant` argument, or is asked of a `Game`. |
| **Game** | A `Variant` plus a start position plus the moves played from it, and therefore the positions reached. It answers rule questions without being handed a variant, and it is what knows about repetition and about claimable draws. |
| **Round** | Not an esca type. `Game` is one played game; a match, round or tournament is a sequence of `Game`s with pairing and scoring data, which callers keep themselves. "Round" is also not used for a full move — that is *full move*. |
| **Move** (esca type) | One legal action, stored as origin, destination, optional promotion role, and kind (quiet, capture, castling, en passant). Castling is stored king-to-rook so it is unambiguous in every variant; its text form is produced by the `Variant`. |
| **MoveList** | An ordered list sized inline for the largest legal move count and filled without allocating. It holds plain moves or annotated ones. |
| **Side** | `Us` or `Them`, relative to the side to move. Facts are expressed in these terms, so no fact distinguishes actual White from actual Black. |
| **SquareSet** | A typed set of squares — one bit per square — supporting union, intersection, difference, membership, count and iteration. The idiomatic name for what engines call a *bitboard*. |
| **FileSet** | The same for the eight files. |
| **Facts** | esca's word for cheap, position-derived features: everything true about a position that follows from the position plus at most one ply of lookahead. Grouped into plain structs with public fields (`facts.pawns.passed[Us]`). |
| **MoveFacts** | The same for one legal move: what it captures, whether it checks, whether its destination is safe. |
| **annotated legal move** | A `Move` paired with its `MoveFacts`. |
| **Scratch** | Reusable buffers — attack maps, pawn spans, the move list — that let repeated fact extraction run without allocating. |
| **Schema** | A versioned, ordered list of feature groups with their widths and offsets: the contract between the extractor that writes a feature vector and the net that consumes one. |
| **SchemaId** | A 128-bit hash over a schema's canonical text. It changes when any group name, order, width or encoding changes, and only then. |
| **GroupSet** | A subset of a schema's groups, selecting which of them an encoding emits. |
| **FeatureSet** | A subset of a schema's features, e.g. those whose definitions hold under one variant. A feature outside the set is encoded as zeros, so widths and offsets are unaffected. |
| **summary** | A human-readable rendering of a position or its facts, for reading and for diagnostics. Its exact text is not a stable format. |

---

## 6. Features and facts

| Term | Definition |
|---|---|
| **feature** | One named value inside a schema group, with a fixed width and encoding. |
| **group** | A named, independently versioned block of features, e.g. `pawns` or `king`. |
| **width** | How many `f32` values a feature, a group or a whole schema occupies. |
| **one-hot** | A width-*k* encoding carrying exactly one 1.0, or all zeros when the feature is absent. |
| **plane** | A 64-value feature, one per square, in the mover's view. |
| **attack** | Side X attacks square *s* if some unit of X could capture a unit standing on *s*, ignoring pins and ignoring whether X's own king would be exposed. Pawns attack diagonally only; sliders stop at the first occupied square. |
| **attack map** | The union of a side's attacked squares, kept per role as well. |
| **defend** | A unit of X is defended when the square it stands on is attacked by X. A unit never defends itself. |
| **hanging** | Attacked by the opponent and not defended. Kings are never hanging. |
| **en prise** | Hanging, or attacked by an enemy unit of strictly lower value. |
| **safe destination** | A square a move can land on where, after the move, it is not attacked by an enemy pawn, not attacked by a cheaper enemy piece, and not both attacked and undefended. A one-ply approximation of "does not lose material". |
| **safe check** | A checking move whose destination is a safe destination. |
| **SEE** (static exchange evaluation) | Playing out the whole capture sequence on one square, cheapest attacker first, to score the material outcome. Not a fact: it is a loop, not a fixed number of set operations, and it belongs to search. |
| **mobility** | How many squares a side's units attack that its own units do not occupy. *Safe mobility* excludes squares attacked by enemy pawns. |
| **virtual mobility** | The number of squares a queen would attack from one's own king square: a proxy for king exposure. |
| **space** | Attacked squares in the opponent's half. |
| **pin** | A unit that should not, or may not, move off the line between an enemy slider and something behind it. *Absolute*: the thing behind is its own king, so moving is illegal. *Relative*: the thing behind is more valuable, so moving is merely bad. |
| **skewer** | The same geometry reversed: the more valuable unit is in front, and moving it exposes what stands behind. |
| **fork** | A move after which the moved unit attacks two or more enemy units that are each more valuable than it or undefended. A *royal fork* has the king among the targets. |
| **king ring** | The up-to-8 squares adjacent to a king. |
| **tropism** | How close a side's units are to the enemy king, as a distance average. |
| **phase** | How far from the opening a position is, from material: 1.0 for a full set of pieces, 0.0 for a pawn endgame. |
| **passed pawn** | A pawn with no enemy pawn ahead of it on its own or an adjacent file. |
| **doubled / isolated / backward pawn** | Two or more of a colour on one file / no friendly pawn on either adjacent file / unable to advance safely and unsupported by a friendly pawn beside or behind it. |
| **open / semi-open file** | No pawn of either colour on it / no pawn of one named colour, but at least one of the other. |
| **outpost** | A square on relative ranks 4–6 protected by one's own pawn that no enemy pawn can ever attack. |
| **lever / ram** | A pawn that can capture an enemy pawn / a pawn blocked head-on by one. |
| **pawn shield / pawn storm** | The friendly pawns in front of one's own king / the enemy pawns advancing on it. |
| **win probability** | A score mapped to [0, 1] (or [−1, 1]) through a fitted logistic, the scale value heads are trained on. |
