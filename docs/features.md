# Feature catalogue v0

Fixed-width vectors of cheaply derivable position facts, fed to the net
alongside the raw board. Everything here is derivable from the current
position plus at most one ply of lookahead (the legal move list and the
attack maps). Rust is the single source of truth; the Python trainer calls
the same code through the PyO3 binding.

Two schemas are defined:

| Schema | Shape | Consumer |
|---|---|---|
| `position` | one vector of 1065 f32 per position | value head, policy head |
| `move` | one vector of 24 f32 per legal move | policy head |

The raw board (12 piece-colour planes × 64 squares = 768) is **not** part of
either schema; it is produced separately and is the baseline the augmentation
is measured against.

---

## 1. Glossary

Defined once, used everywhere below. Where a definition is a cheap
approximation of a costlier truth, the approximation *is* the contract.

### Sides and orientation

| Term | Definition |
|---|---|
| **us** | The side to move. |
| **them** | The side not to move. |
| **mover's view** | The board as seen by *us*: when Black is to move, the board is flipped vertically (rank *r* becomes rank 9−*r*) and the colours are swapped. Files are **not** mirrored — file a stays file a. Every square index, file index and rank index below is in the mover's view. |
| **relative rank** | Rank counted from the owner's own back rank: our relative rank 1 is where our king starts, relative rank 8 is where our pawns promote. In the mover's view our relative rank equals the absolute rank. |
| **side-relative** | Every feature exists for us and for them; the two are computed by the same code with the sides exchanged. |
| **wing** | Files a–d are the queen-side, files e–h the king-side. A unit stands on the wing its own file falls in. |

Because of the flip, "side to move" is constant and is not emitted, and no
feature distinguishes actual White from actual Black.

### Attacks and safety

| Term | Definition |
|---|---|
| **attacks** | Square *s* is attacked by side X if some piece of X could capture a piece standing on *s*, ignoring pins and ignoring whether X's king would be left in check. Pawns attack diagonally only. Sliders stop at the first occupied square; occupancy includes both colours, and the enemy king is *not* removed from it. |
| **attack map** | For a side, the union of its attacked squares; kept per piece type as well. |
| **defended** | A unit of X on *s* is defended if *s* is attacked by X. A piece never defends itself. |
| **hanging** | A unit of X on *s* is attacked by the opponent and not defended by X. Kings are never hanging. |
| **value order** | P=1, N=B=3, R=5, Q=9, K=∞. Used only for comparisons, never as an evaluation. |
| **en prise** | A unit of X is en prise if it is hanging, or if it is attacked by an enemy unit of strictly lower value. Kings are never en prise. |
| **destination** | The square the moved unit ends on. For castling that is the king's landing square, c1 or g1 in the mover's frame, never the rook's square the move is written with. |
| **safe destination** | A move of piece *p* to square *t* is safe if, in the position after the move, *t* is not attacked by an enemy pawn, *t* is not attacked by an enemy piece of value below value(*p*), and *t* is not both attacked by them and undefended by us. *p* is the unit standing on *t* after the move, so a promotion is valued as the piece it becomes. No exchange sequence is played out: a defender that is pinned or overloaded is still counted as a defender. This is a 1-ply approximation of "does not lose material", and it is wrong exactly where a static exchange evaluation would be needed. |
| **safe check** | A checking move whose destination is a safe destination. |
| **king ring** | The up-to-8 squares adjacent to a king. A king does not defend its own ring: its own attacks are left out of "defended" there. |
| **ring attacker** | An enemy knight, bishop, rook or queen attacking a king ring square. Pawns and the enemy king do not count. The same set is what tropism averages over. |
| **king files** | The three files a king's shelter and storm are read on: the king's own file clamped to b–g, and its two neighbours, in ascending order. |
| **virtual mobility** | The number of squares a queen placed on our own king's square would attack. A cheap proxy for how exposed the king is. |

### Pawns

| Term | Definition |
|---|---|
| **front span** | For a pawn of X on file *f*, relative rank *r*: all squares on file *f* with relative rank > *r*. |
| **passed** | No enemy pawn stands on files *f*−1, *f*, *f*+1 at any relative rank greater than *r*. |
| **candidate passer** | Not passed, no enemy pawn on file *f* ahead of *r*, and the number of our pawns on adjacent files at relative rank ≤ *r* is at least the number of enemy pawns on adjacent files at relative rank > *r*. |
| **unstoppable passer** | A passed pawn whose promotion square the enemy king cannot reach in time by the rule of the square: `dist(enemy_king, promo_sq) − (defender to move ? 1 : 0) > 8 − r`, and no piece other than kings remains for the defender. The defender is the passer owner's opponent, whichever side that is; its own pawns do not stop the pawn being unstoppable. |
| **doubled** | Two or more pawns of the same colour on one file. All of them are marked. |
| **isolated** | No friendly pawn on either adjacent file. |
| **backward** | Not passed, no friendly pawn on an adjacent file at relative rank ≤ *r*, and the square directly ahead is attacked by an enemy pawn. |
| **open file** | No pawn of either colour on it. |
| **semi-open for X** | No pawn of X on it, at least one pawn of the opponent. |
| **pawn island** | A maximal run of adjacent files carrying at least one pawn of the side. |
| **lever** | A pawn one of whose attacked squares carries an enemy pawn. Counted once per such pawn, from the attack alone. |
| **ram** | A pawn whose stop square holds an enemy pawn. |
| **outpost square for X** | A square on relative ranks 4–6, attacked by one of X's pawns, and never attackable by an enemy pawn (no enemy pawn on either adjacent file at that relative rank or ahead of it). |
| **pawn shield** | For each of the king files: the nearest friendly pawn ahead of the king on that file. |
| **pawn storm** | The same three files: the nearest enemy pawn ahead of the king. |
| **behind a passer** | On the passer's file, at a lower relative rank in the passer owner's frame. Nothing has to stand clear between the two. |

### Tactical patterns (all measured over one ply)

| Term | Definition |
|---|---|
| **fork** | A legal move after which the moved piece attacks two or more enemy units that are each either of greater value than the moved piece, or undefended. The enemy king counts as greater value. |
| **royal fork** | A fork one of whose targets is the king (i.e. a forking check). |
| **absolute pin** | A unit that may not legally move because its own king would be exposed. |
| **relative pin** | A unit that, if it moved off the ray, would expose a strictly more valuable non-king unit to the pinning slider. |
| **skewer** | A slider attacks a unit that, if it moved off the ray, would expose a unit of lower or equal value behind it. Counted per (slider, front unit, back unit) triple, for the side whose slider it is. |
| **discovered check available** | Some friendly unit stands on a ray between one of our sliders and the enemy king, and has at least one legal move off that ray. |
| **mate in 1** | Some legal move leaves the opponent checkmated. |
| **stalemate in 1** | Some legal move leaves the opponent stalemated. |

### Phase

`phase_points = 4·Q + 2·R + 1·B + 1·N` summed over both sides, capped at 24.
`phase = phase_points / 24`; 1.0 is a full opening set, 0.0 is a bare pawn
endgame.

### Them-facts and the null move

Facts about *their* one-ply options are computed in the position after a null
move (they move next, everything else unchanged). When we are in check the
null move does not exist; the whole `tactics.them` block is then zero and its
`facts_available` bit is 0. Safety of their moves is judged against our
pieces where they currently stand.

### Cost classes

Marginal cost of one feature, given a shared scratch buffer that is built
once per position (per-piece attack sets, per-side attack maps, pawn spans,
the legal move list).

| Class | Meaning | Rough budget |
|---|---|---|
| **A** | A fixed number of bitwise ops, popcounts or table lookups. | ~1–20 ns |
| **B** | One pass over the pieces present (≤32). | ~20–100 ns |
| **C** | One pass over the legal move list (typically 20–40 moves), a few ops each. | ~50–300 ns |
| **D** | Make-move plus movegen for a *subset* of moves (checking moves only). | ~0.2–3 µs |

Class D applies to exactly two features (`mate_in_1`, `stalemate_in_1`). They
sit in their own sub-group so that a search that cannot afford them can turn
them off without changing any other offset.

### Encoding rules

| Kind | Encoding |
|---|---|
| bit | 0.0 or 1.0 |
| count | `min(n, SCALE) / SCALE`, `SCALE` named per feature |
| difference | `clamp(d / SCALE, −1, 1)`, sign kept |
| one-hot *k* | exactly one 1.0, or all zeros when the feature is absent |
| 8-per-file | index 0 = file a, in the mover's view |
| 64-plane | index = square index in the mover's view (a1 = 0 … h8 = 63) |
| square colour | light and dark are read in the mover's view too, so the rank flip does not swap them |

All values are `f32` in [−1, 1]. No NaN, no infinity, ever. Side-paired
features are written us-block first, them-block second.

---

## 2. Groups

Nine groups, in schema order. "Head" says which head the feature is expected
to serve: **V** value, **P** policy, **B** both.

### 2.1 `state` — game-state flags (width 29)

| Feature | Width | Encoding | Cost | Head |
|---|---|---|---|---|
| `in_check` | 1 | bit | A | B |
| `double_check` | 1 | bit: two or more checkers | A | B |
| `castle_rights` | 4 | bits: us short, us long, them short, them long | A | B |
| `ep_available` | 1 | bit: the FEN names an en-passant file | A | P |
| `ep_file` | 8 | one-hot, zeros when none | A | P |
| `ep_capture_legal` | 1 | bit: some legal move actually captures en passant | C | P |
| `halfmove_bucket` | 8 | one-hot over 0 / 1–3 / 4–9 / 10–19 / 20–39 / 40–69 / 70–89 / 90 and above | A | V |
| `halfmove_known` | 1 | bit: the caller supplied a halfmove clock | A | V |
| `repetition_seen` | 1 | bit: this position occurred before in the supplied history | A | V |
| `repetition_available_us` | 1 | bit: some legal move reaches a position in the history | C | V |
| `repetition_available_them` | 1 | bit: same, after a null move | C | V |
| `history_known` | 1 | bit: the caller supplied a position history | A | V |

The Lichess evaluation dump carries 4-field FENs (no halfmove clock, no move
number). With that source `halfmove_known` and `history_known` are 0 for every
training row — see [§5](#5-open-questions).

### 2.2 `material` — material and phase (width 26)

| Feature | Width | Encoding | Cost | Head |
|---|---|---|---|---|
| `piece_count` | 10 | 5 types × 2 sides, count / 8 (pawns) or / 4 (pieces) | A | V |
| `piece_count_diff` | 5 | difference us − them per type, / 4 | A | V |
| `non_pawn_material` | 2 | value sum of N,B,R,Q per side, / 62 | A | V |
| `material_balance` | 1 | (us − them) value sum, / 20 | A | V |
| `phase` | 1 | see glossary | A | V |
| `phase_bucket` | 3 | one-hot: phase > 0.75 / 0.25 ≤ phase ≤ 0.75 / < 0.25 | A | V |
| `both_queens` | 1 | bit: both sides have at least one queen | A | V |
| `pawns_only` | 1 | bit: no piece other than kings and pawns | A | V |
| `insufficient_material` | 2 | bit per side: no pawn, rook or queen, and either at most one minor or no knight and bishops of a single square colour | A | V |

### 2.3 `pawns` — pawn structure (width 165)

| Feature | Width | Encoding | Cost | Head |
|---|---|---|---|---|
| `pawn_count_by_file` | 16 | 8 per side, count / 3 | A | V |
| `pawn_count_by_rank` | 16 | 8 per side, relative rank, count / 8 | A | V |
| `doubled_files` | 16 | 8-bit mask per side | A | V |
| `isolated_files` | 16 | 8-bit mask per side | A | V |
| `backward_files` | 16 | 8-bit mask per side | A | V |
| `passed_files` | 16 | 8-bit mask per side | A | B |
| `candidate_files` | 16 | 8-bit mask per side | A | V |
| `passer_lead_rank` | 16 | one-hot 8 per side: relative rank of the most advanced passer; zeros if none | A | V |
| `passer_protected` | 2 | count per side of passers defended by a friendly pawn, / 4 | A | V |
| `passers_connected` | 2 | bit per side: two passers on adjacent files | A | V |
| `passer_unstoppable` | 2 | bit per side | A | V |
| `open_files` | 8 | 8-bit mask, colour-independent | A | B |
| `semi_open_files_us` | 8 | 8-bit mask | A | B |
| `semi_open_files_them` | 8 | 8-bit mask | A | B |
| `pawn_islands` | 2 | count per side, / 4 | A | V |
| `defended_pawns` | 2 | count per side of pawns defended by a pawn, / 8 | A | V |
| `levers` | 2 | count per side, / 4 | A | P |
| `rams` | 1 | count of blocked pawn pairs, / 8 | A | V |

### 2.4 `pieces` — bishops, rooks, knights, queens (width 35)

Every row is a pair: us then them.

| Feature | Width | Encoding | Cost | Head |
|---|---|---|---|---|
| `bishop_pair` | 2 | bit: bishops on both square colours | A | V |
| `bishops_by_square_colour` | 4 | count on light, count on dark, / 2, per side | A | V |
| `opposite_coloured_bishops` | 1 | bit: exactly one bishop each, on different colours | A | V |
| `pawns_on_bishop_colour` | 2 | own pawns standing on the colour of own bishops, / 8 | A | V |
| `rooks_connected_rank` | 2 | bit: two rooks share a rank with nothing between | A | V |
| `rooks_connected_file` | 2 | bit: two rooks share a file with nothing between | A | V |
| `rooks_on_open_file` | 2 | count / 2 | A | B |
| `rooks_on_semi_open_file` | 2 | count / 2 | A | B |
| `rooks_on_relative_7th` | 2 | count / 2 | A | V |
| `rook_behind_own_passer` | 2 | count / 2 | A | V |
| `rook_behind_enemy_passer` | 2 | count / 2 | A | V |
| `trapped_rook` | 2 | bit: a rook with ≤2 non-capture destinations, on a file beyond its own king's on the wing the king stands on, its side having lost both castling rights | B | V |
| `knights_on_outpost` | 2 | count / 2 | B | V |
| `outpost_squares_free` | 2 | count of unoccupied outpost squares, / 4 | A | P |
| `knights_on_rim` | 2 | count on files a/h or relative ranks 1/8, / 2 | A | V |
| `minors_undeveloped` | 2 | knights and bishops still on their classic starting squares b1, c1, f1, g1 relative, / 4 | A | V |
| `queen_developed` | 2 | bit: a queen stands off its classic starting square d1 relative | A | V |

### 2.5 `king` — king safety and shelter (width 122)

Every row is a pair: our king then their king, unless noted.

| Feature | Width | Encoding | Cost | Head |
|---|---|---|---|---|
| `king_file` | 16 | one-hot 8 per side | A | V |
| `king_rank` | 16 | one-hot 8 per side, relative rank | A | V |
| `king_on_home_square` | 2 | bit: the king stands on e1 relative; classic chess only | A | V |
| `king_castled_zone` | 4 | bits: king on files a–c, king on files f–h, per side; classic chess only | A | V |
| `pawn_shield` | 24 | per side, for each of the king files: one-hot 4 over "friendly pawn 1 rank ahead / 2 ahead / 3+ ahead / none" | A | V |
| `king_file_openness` | 12 | per side, for each of the king files: bit open, bit semi-open for the enemy | A | V |
| `pawn_storm` | 24 | per side, for each of the king files: one-hot 4 over the distance of the nearest enemy pawn ahead (≤2 / 3 / 4 / 5+ or none) | A | V |
| `ring_attackers` | 2 | count of ring attackers, / 6 | B | V |
| `ring_attack_weight` | 2 | Σ over attackers of (N,B = 1, R = 2, Q = 4), / 16 | B | V |
| `ring_defended` | 2 | ring squares attacked by the king's own side, / 8 | A | V |
| `ring_holes` | 2 | ring squares attacked by the enemy and not defended, / 8 | A | V |
| `king_escape_squares` | 2 | adjacent squares that are empty or capturable and not attacked, / 8 | A | V |
| `back_rank_risk` | 2 | bit: king on its relative rank 1 with every forward-adjacent square occupied by a friendly unit | A | V |
| `king_distance` | 8 | one-hot over Chebyshev distance 1–8 between the kings, shared | A | V |
| `king_tropism` | 2 | mean Chebyshev distance of the enemy's knights, bishops, rooks and queens to this king, 0 when it has none, / 8 | B | V |
| `virtual_mobility` | 2 | see glossary, / 27 | A | V |

### 2.6 `mobility` — mobility and space (width 39)

| Feature | Width | Encoding | Cost | Head |
|---|---|---|---|---|
| `mobility_ratio` | 1 | our total mobility / (ours + theirs), zero when both are zero | B | V |
| `mobility_by_type` | 10 | per side, per type (P,N,B,R,Q): squares the type's attack map covers and own units do not occupy — a union over the pieces of that type, not a sum, / 16 | B | B |
| `safe_mobility_by_type` | 10 | as above, minus squares attacked by an enemy pawn, / 16 | B | B |
| `mobility_diff_by_type` | 5 | us − them per type, / 16 | A | V |
| `space` | 2 | per side: attacked squares in the opponent's half, / 32 | A | V |
| `controlled_squares` | 3 | attacked-square count us, them, and the difference, / 48 | A | V |
| `centre_control` | 2 | per side: attacks on d4, e4, d5, e5, / 4 | A | B |
| `extended_centre_control` | 2 | per side: attacks on c3–f6, / 16 | A | V |
| `immobile_pieces` | 2 | per side: non-pawn, non-king units with no destination, / 4 | B | V |
| `total_mobility` | 2 | per side: sum over types, / 96 | B | V |

### 2.7 `attacks` — attack-map summary (width 17)

| Feature | Width | Encoding | Cost | Head |
|---|---|---|---|---|
| `attacked_square_count` | 3 | us, them, difference, / 48 | A | V |
| `hanging_count` | 2 | per side, / 4 | A | B |
| `hanging_value` | 2 | per side, value sum of hanging units, / 20 | B | B |
| `en_prise_count` | 2 | per side, / 4 | B | B |
| `en_prise_max_value` | 2 | per side, largest value en prise, / 9 | B | B |
| `pinned_count` | 2 | per side, absolute pins, / 4 | B | B |
| `skewer_candidates` | 2 | per side, / 4 | B | P |
| `defended_count` | 2 | per side, own units that are defended, / 16 | A | V |

### 2.8 `tactics` — one-ply tactics (width 120)

The same 60-wide block twice: `tactics.us` then `tactics.them`, the second
computed after a null move. The schema names the two blocks' features
`us.<feature>` and `them.<feature>`.

| Feature | Width | Encoding | Cost | Head |
|---|---|---|---|---|
| `check_available` | 1 | bit | C | P |
| `check_count` | 1 | count / 8 | C | P |
| `check_by_piece` | 5 | bit per moving piece type P,N,B,R,Q | C | P |
| `safe_check_available` | 1 | bit | C | P |
| `safe_check_count` | 1 | count / 8 | C | P |
| `safe_check_by_piece` | 5 | bit per moving piece type | C | P |
| `double_check_available` | 1 | bit: a move giving check from two units at once | C | P |
| `discovered_check_available` | 1 | bit: a legal move after which a unit that did not move gives check | C | P |
| `mate_in_1` | 1 | bit | D | B |
| `stalemate_in_1` | 1 | bit | D | B |
| `promotion_available` | 1 | bit: some legal move promotes | C | B |
| `promotion_file` | 8 | 8-bit mask over promoting files | C | B |
| `promotion_piece` | 4 | bit per obtainable promotion piece Q,R,B,N | C | P |
| `safe_promotion_available` | 1 | bit: a promotion whose destination is safe | C | B |
| `safe_promotion_file` | 8 | 8-bit mask | C | B |
| `capture_available` | 1 | bit | C | P |
| `capture_count` | 1 | count / 16 | C | P |
| `winning_capture_available` | 1 | bit: a capture whose victim is worth more than the capturer, or is undefended | C | B |
| `winning_capture_max_gain` | 1 | max(victim − capturer, 0) over captures, / 9 | C | B |
| `captures_hanging_available` | 1 | bit: a capture of a hanging unit | C | B |
| `hanging_victim_max_value` | 1 | largest hanging victim capturable now, / 9 | C | B |
| `equal_capture_count` | 1 | captures of equal value, defended, / 8 | C | P |
| `losing_capture_count` | 1 | captures of lower value, defended, / 8 | C | P |
| `fork_available` | 1 | bit | C | B |
| `fork_count` | 1 | count / 4 | C | B |
| `fork_max_value` | 1 | largest single forked value, / 9 | C | B |
| `knight_fork_available` | 1 | bit: the forking piece is a knight | C | B |
| `royal_fork_available` | 1 | bit | C | B |
| `pin_creation_available` | 1 | bit: a move creating an absolute or relative pin | C | P |
| `pin_creation_count` | 1 | count / 4 | C | P |
| `skewer_creation_available` | 1 | bit | C | P |
| `discovered_attack_available` | 1 | bit: a move uncovering a slider's attack on a unit of value ≥ 3 | C | P |
| `legal_move_count` | 1 | count / 64 | C | V |
| `only_moves` | 1 | bit: at most 2 legal moves | C | V |
| `facts_available` | 1 | bit: 0 when the block could not be computed (in check, for the them block) | A | B |

### 2.9 `planes` — attack and status bitboards (width 512)

Eight 64-square planes, in the mover's view. This group carries most of the
width and is the natural first ablation target.

| Plane | Definition | Cost | Head |
|---|---|---|---|
| `attacked_by_us` | union of our attacks | A | B |
| `attacked_by_them` | union of their attacks | A | B |
| `attacked_by_our_pawns` | our pawn attacks only | A | B |
| `attacked_by_their_pawns` | their pawn attacks only | A | B |
| `our_hanging` | our units that are hanging | A | B |
| `their_hanging` | their units that are hanging | A | B |
| `our_pinned` | our units under an absolute pin | B | B |
| `their_pinned` | their units under an absolute pin | B | B |

### 2.10 Totals

| Group | Width | Dominant cost |
|---|---|---|
| `state` | 29 | A |
| `material` | 26 | A |
| `pawns` | 165 | A |
| `pieces` | 35 | A/B |
| `king` | 122 | A/B |
| `mobility` | 39 | B |
| `attacks` | 17 | B |
| `tactics` | 120 | C, plus 2 features at D |
| `planes` | 512 | A |
| **total** | **1065** | |
| total without `planes` | 553 | |

---

## 3. Per-move annotations (`move` schema, width 24)

One vector per legal move, for the policy head and for move ordering. Built
from the position's shared scratch; marginal cost per move is class A.

| Feature | Width | Encoding |
|---|---|---|
| `is_capture` | 1 | bit |
| `victim_type` | 5 | one-hot P,N,B,R,Q; zeros for a quiet move |
| `mover_type` | 6 | one-hot P,N,B,R,Q,K |
| `promotion_piece` | 4 | one-hot Q,R,B,N; zeros when not a promotion |
| `gives_check` | 1 | bit |
| `gives_safe_check` | 1 | bit |
| `is_safe` | 1 | bit: safe destination (see glossary) |
| `captures_hanging` | 1 | bit |
| `escapes_attack` | 1 | bit: the origin square is attacked by them and the destination is safe |
| `to_attacked_by_pawn` | 1 | bit |
| `is_castling` | 1 | bit |
| `is_en_passant` | 1 | bit |

---

## 4. Excluded on purpose

These are part of the contract: callers must not expect them, and the engine
computes them itself where it needs them.

| Excluded | Why |
|---|---|
| Static exchange evaluation (SEE) | An exchange loop over all attackers and defenders of a square; not a fixed number of bitboard ops. `winning_capture_*` and "safe" are the 1-ply stand-ins. |
| Forced mate in 2 or more | Needs search. |
| Threats after the opponent's best reply | Needs 2 plies plus a choice of "best". |
| Zugzwang, fortress, opposition, corresponding squares | Needs search or endgame theory. |
| Tablebase results | External data. |
| Piece-square tables and any hand-tuned score | The net learns them from the board planes. |
| Move number / opening classification | Absent from the training source, and phase already covers what it would proxy. |
| Absolute colour | The mover's-view flip removes it; nothing in chess depends on it. |
| Chess960 castling geometry | Four features assume the classic starting squares and are defined for classic chess only: `pieces.minors_undeveloped`, `pieces.queen_developed`, `king.king_on_home_square`, `king.king_castled_zone`. Under another variant they are written as zeros, so widths and offsets do not move. Every other feature is 960-safe. |
| Game history beyond a supplied repetition set | The library does not track games; the caller passes what it knows. |

---

## 5. Open questions

1. **Clock and repetition in training.** The Lichess dump has 4-field FENs, so
   `halfmove_*`, `repetition_*` and `history_known` are constant-zero across
   the whole training set. A feature that is always "unknown" during training
   is unusable at play time. Either drop the 13 affected values from the
   trained schema, or find a second source that carries clocks (game PGNs).
   v0 keeps them in the schema and excludes them from the trained group list.
2. **`planes` width.** 512 of 1065 values. Ablation (§7) decides whether it
   earns its place or shrinks to 4 planes.
3. **Mate-in-1 in the search loop.** Class D. Cheap enough for a training
   pass, possibly not for every search node; the sub-group toggle exists so
   the answer can be measured rather than guessed.

---

## 6. Schema versioning

### Manifest

A schema is an ordered list of groups. Each group has a name, an integer
version, a width and an ordered list of feature entries.

```
schema_semver = "0.1.0"
groups = [
  { name = "state",    version = 1, width =  29, offset =   0 },
  { name = "material", version = 1, width =  26, offset =  29 },
  ...
]
schema_id = "a40a02ef18e4219b754d0f32410d803f"   # 128-bit, hex
```

`schema_id` is a BLAKE3 hash over the canonical UTF-8 rendering

```
<group>:<version>:<width>\n
  <feature>:<width>:<encoding>\n   (for each feature, in order)
```

for every group in order. It changes when any name, order, width or encoding
changes, and only then.

`<encoding>` is the encoding kind and its scale, from a fixed vocabulary:
`bit`, `bits`, `one-hot`, `mask8`, `plane`, `ratio`, `count/S` and `diff/S`,
where `S` is the feature's scale; `count/8|4` marks `piece_count`, whose scale
is 8 for pawns and 4 for the rest. The full text is checked in as
`rs_anglerfish/esca/tests/data/schema_v0.txt`.

### Contract

| Rule | |
|---|---|
| The trained net stores the full manifest it was trained with. | |
| The engine compares `schema_id` on load and refuses a mismatch, naming both ids and the first differing group. | |
| The extractor can emit any *subset* of groups, in the order the manifest names, writing only those groups' widths. | |
| Offsets are derived from the emitted subset, never hard-coded. | |

### Evolution

| Change | Effect | Old nets |
|---|---|---|
| Append a new group at the end | New `schema_id`; old groups untouched | keep working — their manifest names a subset the extractor still emits |
| Add or reorder features inside a group | Bump that group's version; it becomes a distinct group id | keep working while the previous version's implementation is retained |
| Remove a group | Bump `schema_semver` major | stop working; refused at load |

At most two versions of any group are kept compiled. Dropping an old version
is a major release.

---

## 7. Evaluation plan

### Runs

| Run | Input |
|---|---|
| `B0` baseline | board planes only (768) |
| `B1` augmented | board planes + all groups (768 + 1065) |
| `A-<group>` | `B1` minus one group |
| `S-<group>` | `B0` plus one group only |

Both ablation directions are run: groups are strongly correlated, so leaving
one out often costs nothing while adding it alone gains a lot.

### Metrics

| Head | Metric |
|---|---|
| value | MAE and RMSE on the win-probability scale (`cp` mapped through a logistic whose scale is fitted once on a held-out slice; `mate n` mapped to ±(1 − n/1000)) |
| value | Spearman correlation with the label, and sign accuracy |
| policy | top-1 and top-3 agreement with the multi-PV best move |
| policy | cross-entropy, and mean rank of the labelled best move |

### Slices

Reported per slice, not only in aggregate.

| Slice | Definition |
|---|---|
| quiet | top-1 and top-2 multi-PV evals differ by < 50 cp |
| tactical | they differ by ≥ 200 cp |
| endgame | `phase` < 0.25 |
| in check | `state.in_check` = 1 |

Held-out positions are deduplicated by FEN and drawn from games disjoint from
the training split.

### Cost

Each run reports, from a criterion benchmark on a fixed 10k-position corpus:
nanoseconds per position for the whole extractor, nanoseconds per group, and
positions per second in batch mode. A group that costs more than 5 % of
engine nodes-per-second must show a matching gain, measured as Elo in a
fixed-node self-play match with error bars, not as validation loss.
