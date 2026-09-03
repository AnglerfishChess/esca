# Feature catalogue v1

Fixed-width vectors of cheaply derivable position facts. Everything here is
derivable from the current position plus at most one ply of lookahead (the
legal move list, the attack maps and one exchange loop per square), or from a
supplied game history. Rust is the single source of truth; the Python trainer
calls the same code through the PyO3 binding.

Two schemas are defined:

| Schema | Shape | Consumer |
|---|---|---|
| `position` | one vector of 1930 f32 per position | value head, policy head |
| `move` | one vector of 24 f32 per legal move | policy head |

The raw board is the `placement` group, first in the schema order, so that a
run measuring the augmentation against the board alone selects one group
rather than building its own input.

[`features-v1.md`](features-v1.md) is the candidate list this catalogue is
being filled from; it never overrides an entry here.

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
| **value order** | P=1, N=B=3, R=5, Q=9, K=∞. Used only for comparisons, never as an evaluation. Where a value is added or subtracted instead of compared, a king contributes 0, except as a forked or hanging target, where it counts as 9 so that the feature's scale holds. |
| **en prise** | A unit of X is en prise if it is hanging, or if it is attacked by an enemy unit of strictly lower value. Kings are never en prise. |
| **destination** | The square the moved unit ends on. For castling that is the king's landing square, c1 or g1 in the mover's frame, never the rook's square the move is written with. |
| **safe destination** | A move of piece *p* to square *t* is safe if, in the position after the move, *t* is not attacked by an enemy pawn, *t* is not attacked by an enemy piece of value below value(*p*), and *t* is not both attacked by them and undefended by us. *p* is the unit standing on *t* after the move, so a promotion is valued as the piece it becomes. No exchange sequence is played out: a defender that is pinned or overloaded is still counted as a defender. This is a 1-ply approximation of "does not lose material", and it is wrong exactly where a static exchange evaluation would be needed. |
| **safe check** | A checking move whose destination is a safe destination. |
| **king ring** | The up-to-8 squares adjacent to a king. A king does not defend its own ring: its own attacks are left out of "defended" there. |
| **ring attacker** | An enemy knight, bishop, rook or queen attacking a king ring square. Pawns and the enemy king do not count. The same set is what tropism averages over. |
| **king files** | The three files a king's shelter and storm are read on: the king's own file clamped to b–g, and its two neighbours, in ascending order. |
| **virtual mobility** | The number of squares a queen placed on our own king's square would attack. A cheap proxy for how exposed the king is. |

### Values and exchange

| Term | Definition |
|---|---|
| **value sum** | Σ over a set of units of P=1, N=B=3, R=5, Q=9, K=0. |
| **exchange on a square** | Both sides capture on one square in turn, each with its least valuable attacker of that square, until one has no attacker left or stops. Attackers are read from the occupancy of the moment, so a slider standing behind a departed attacker on the same ray joins in. Pins are ignored: a pinned defender still counts. A king captures only when the square is no longer attacked by the other side, so it is the last attacker of its side. A pawn capturing onto its relative rank 8 promotes to a queen: that capture wins 8 more, and a queen stands on the square from then on. |
| **SEE** | Static exchange evaluation, in value units, signed, positive for material won. Of a capture: the value its side wins from the exchange that capture begins, each side stopping as soon as going on would cost it material. The move itself is played whatever it costs, so a capture's SEE may be negative. Of a quiet move: the same reckoning with nothing captured, so 0 or negative. Of castling: 0. |
| **SEE of a unit** | The SEE of the exchange the opponent begins by capturing that unit. Never negative — the opponent may leave it alone. 0 for a square no enemy unit attacks, and for a king. |
| **signed SEE of a unit** | The same exchange with the opponent's first capture played whatever it costs, so it may be negative. Absent where the opponent has no capture there: an unattacked unit, a king, and a unit whose only attacker is a king the square is still defended against. |
| **max gain** | The largest SEE over a side's captures. |
| **threatened** | A unit whose SEE of a unit is above 0: the opponent wins material by taking it now. The exchange-exact form of *en prise*. |
| **loose unit** | A unit no unit of its own side defends, attacked or not. Never a king. |
| **attacker surplus** | On a unit: the enemy units attacking it, less the friendly units defending it, counting on both sides only units whose value order is at most the unit's own. A king is above every value order, so it counts on neither side. |

### History

| Term | Definition |
|---|---|
| **quiet plies** | Plies since the last ply of the supplied history that captured or gave check; the whole history when it holds none. Distinct from the halfmove clock, which counts captures and pawn moves. |
| **material trend** | The material balance now, less the balance eight plies ago, both as a value sum of the side to move's units less the other side's. Fewer than eight plies means the start of the supplied history. |

An exchange is a one-square reckoning, not a search: it answers "what does
taking here cost", never "is taking here best".

### Defenders, x-rays and batteries

| Term | Definition |
|---|---|
| **sole defender** | The one unit defending a unit that exactly one friendly unit defends. |
| **overloaded defender** | The sole defender of two or more friendly units that are each attacked. |
| **removable defender** | A sole defender of an attacked friendly unit, itself attacked and with a signed SEE of a unit of at least 0: taking it costs the capturer nothing. |
| **x-ray attack** | A slider's attack on a square that exists once the single unit standing between them is removed. *Through own*: that unit is the slider's own. *Through enemy*: it is not. Counted per (slider, target) pair. |
| **battery** | Two friendly sliders on one rank, file or diagonal that both move along it — Q+R and R+R on a rank or file, Q+B and B+B on a diagonal, Q+Q on either — with nothing between them but friendly sliders that also move along it. Counted per pair, so three sliders in a row are three batteries. |

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
| **stop square** | The square directly ahead of a pawn on its own file: file *f*, relative rank *r*+1. |
| **ram** | A pawn whose stop square holds an enemy pawn. |
| **fixed pawn** | A pawn whose stop square holds any unit, its own side's or the enemy's. A ram is the case where that unit is an enemy pawn. |
| **blocked passer** | A passed pawn whose stop square holds an enemy unit. |
| **pawn chain** | A maximal run of friendly pawns each defending the next, along one diagonal direction. Its **length** is the number of pawns in it. Every pawn heads a run of at least itself, so a side with pawns none of which defends another has a longest chain of 1, and a side with no pawns has 0. |
| **chain base** | The rearmost pawn of a chain of two or more: the one no other pawn of that chain defends. |
| **majority** | On a wing, strictly more own pawns than enemy pawns. |
| **hole for X** | A square on X's relative ranks 3 to 6 that no pawn of X can ever attack: no pawn of X stands on either adjacent file at a lower relative rank. Occupancy does not matter — a hole may carry a unit of either side, X's own pawn included. |
| **promotion distance** | For a pawn on relative rank *r*: 8 − *r*, the pushes it still needs. |
| **lead passer** | A side's most advanced passer: the one of greatest relative rank, and among equals the one nearest file a. |
| **in the square** | Of the king defending against a passer on relative rank *r*: `dist(king, promotion square) − (defender to move ? 1 : 0) ≤ 8 − r`, the rule of the square. The defender is the passer owner's opponent. An `unstoppable passer` is this test failing with a bare-king defender besides. |
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

### Endgame

| Term | Definition |
|---|---|
| **centralisation distance** | Chebyshev distance from a square to the nearest of d4, e4, d5, e5. At most 3, so a corner is as far out as a square gets. |
| **race plies** | For a side, the plies its most advanced passer needs to promote unopposed: 8 − its relative rank, one less when that side is to move. Nothing on the board is asked to get out of the way. |
| **opposition** | The kings on one file, rank or diagonal, every square between them empty and their number odd. *Direct* when that number is 1, *distant* when it is 3 or 5. The side **not** to move holds it. |
| **key square** | Of a pawn on relative rank 4 or below: the three squares two ranks ahead, on its own file and both neighbours. Of a pawn above that: the three squares one rank ahead. A rook pawn has none. |
| **wrong-colour bishop** | A side with a bishop and a pawn, whose bishops all stand on one square colour, whose pawns are all rook pawns, and none of whose pawns promotes on that colour. Two colours are compared, so the mover's view does not enter. |

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
| **E** | One exchange loop on one square: least valuable attacker, x-rays refreshed, at most about 32 steps. | ~50–500 ns |

Class D applies to exactly two features (`mate_in_1`, `stalemate_in_1`). They
sit in their own sub-group so that a search that cannot afford them can turn
them off without changing any other offset. `C·E` is one exchange loop per
capture in the move list.

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

Fourteen groups, in schema order. "Head" says which head the feature is
expected to serve: **V** value, **P** policy, **B** both.

A group with no features is named and ordered anyway: its width is 0, it
writes nothing, and the group after it keeps its offset when the empty one is
filled.

### 2.1 `placement` — piece planes (width 768)

The raw board: twelve 64-square planes in the mover's view, ours before
theirs and in the role order P, N, B, R, Q, K.

| Feature | Width | Encoding | Cost | Head |
|---|---|---|---|---|
| `our_pawns` | 64 | 64-plane | A | B |
| `our_knights` | 64 | 64-plane | A | B |
| `our_bishops` | 64 | 64-plane | A | B |
| `our_rooks` | 64 | 64-plane | A | B |
| `our_queens` | 64 | 64-plane | A | B |
| `our_king` | 64 | 64-plane | A | B |
| `their_pawns` | 64 | 64-plane | A | B |
| `their_knights` | 64 | 64-plane | A | B |
| `their_bishops` | 64 | 64-plane | A | B |
| `their_rooks` | 64 | 64-plane | A | B |
| `their_queens` | 64 | 64-plane | A | B |
| `their_king` | 64 | 64-plane | A | B |

Because the planes are read in the mover's view, the same structure with the
colours swapped gives the same 768 values.

### 2.2 `state` — game-state flags (width 16)

| Feature | Width | Encoding | Cost | Head |
|---|---|---|---|---|
| `in_check` | 1 | bit | A | B |
| `double_check` | 1 | bit: two or more checkers | A | B |
| `castle_rights` | 4 | bits: us short, us long, them short, them long | A | B |
| `ep_available` | 1 | bit: the FEN names an en-passant file | A | P |
| `ep_file` | 8 | one-hot, zeros when none | A | P |
| `ep_capture_legal` | 1 | bit: some legal move actually captures en passant | C | P |

The clock and the repetition facts are the `history` group's ([§2.13](#213-history--what-the-plies-before-say-width-27)).

### 2.3 `material` — material and phase (width 26)

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

### 2.4 `pawns` — pawn structure (width 195)

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
| `chain_max_length` | 2 | longest pawn chain per side, / 5 | B | V |
| `chain_base_attacked` | 2 | bit per side: an enemy unit attacks a chain base | B | V |
| `majority_by_wing` | 4 | bits per side: majority on the queen-side, on the king-side | A | V |
| `holes` | 2 | count of holes for the side, / 16 | A | V |
| `holes_occupied` | 2 | enemy knights and bishops standing on those holes, / 4 | B | V |
| `fixed_pawns` | 2 | count per side, / 8 | A | V |
| `blocked_passers` | 2 | count per side, / 2 | A | V |
| `passer_distance` | 2 | promotion distance of the lead passer, / 6; 0 when the side has none, which no passer's distance is | A | V |
| `passer_king_distance` | 4 | per side: Chebyshev distance to the lead passer's promotion square from its own king, then from the enemy king, / 8; 8 when the side has no passer, one more than any distance on a board | A | V |
| `passer_in_square` | 2 | bit per side: the defending king is in the square of the lead passer; 0 when the side has no passer | A | V |
| `passer_free_path` | 2 | passers whose whole front span is empty, / 2 | A | V |
| `half_open_at_enemy_king` | 2 | files semi-open for the side among the enemy king files, / 3 | A | V |
| `backward_on_semi_open` | 2 | backward pawns on a file semi-open for the enemy, / 4 | A | V |

### 2.5 `pieces` — bishops, rooks, knights, queens (width 35)

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
| `minors_on_outpost` | 2 | knights and bishops on an own outpost square, count / 2 | B | V |
| `outpost_squares_free` | 2 | count of unoccupied outpost squares, / 4 | A | P |
| `knights_on_rim` | 2 | count on files a/h or relative ranks 1/8, / 2 | A | V |
| `minors_undeveloped` | 2 | knights and bishops still on their classic starting squares b1, c1, f1, g1 relative, / 4 | A | V |
| `queen_developed` | 2 | bit: a queen stands off its classic starting square d1 relative | A | V |

### 2.6 `king` — king safety and shelter (width 120)

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
| `king_distance` | 6 | one-hot over Chebyshev distance 2–7 between the kings, shared; two kings stand neither nearer nor further apart | A | V |
| `king_tropism` | 2 | mean Chebyshev distance of the enemy's knights, bishops, rooks and queens to this king, 0 when it has none, / 8 | B | V |
| `virtual_mobility` | 2 | see glossary, / 27 | A | V |

### 2.7 `mobility` — mobility and space (width 39)

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

`immobile_pieces` reads a unit's destinations off its attack map alone, so a
piece under an absolute pin is not immobile.

### 2.8 `attacks` — attack-map summary (width 25)

| Feature | Width | Encoding | Cost | Head |
|---|---|---|---|---|
| `attacked_square_count` | 3 | us, them, difference, / 48 | A | V |
| `attacked_count` | 2 | per side, own units the opponent attacks, defended or not, / 8 | A | B |
| `attacked_value` | 2 | per side, value sum of those, / 20 | B | B |
| `hanging_count` | 2 | per side, / 4 | A | B |
| `hanging_value` | 2 | per side, value sum of hanging units, / 20 | B | B |
| `en_prise_count` | 2 | per side, / 4 | B | B |
| `en_prise_value` | 2 | per side, value sum of the units en prise, / 20 | B | B |
| `en_prise_max_value` | 2 | per side, largest value en prise, / 9 | B | B |
| `pinned_count` | 2 | per side, absolute pins, / 4 | B | B |
| `pinned_value` | 2 | per side, value sum of those, / 20 | B | B |
| `skewer_candidates` | 2 | per side, / 4 | B | P |
| `defended_count` | 2 | per side, own units that are defended, / 16 | A | V |

### 2.9 `exchange` — static exchange evaluation (width 8)

The same 4-wide block twice: `exchange.us` then `exchange.them`, the second
computed after a null move. The schema names the two blocks' features
`us.<feature>` and `them.<feature>`. When we are in check the null move does
not exist and the `them` block is zero, which `tactics.them.facts_available`
reports.

| Feature | Width | Encoding | Cost | Head |
|---|---|---|---|---|
| `see_best_capture` | 1 | diff / 9: the largest SEE over the side's captures, 0 when it has none | C·E | B |
| `see_positive_capture_count` | 1 | count / 8: captures whose SEE is above 0 | C·E | P |
| `see_equal_capture_count` | 1 | count / 8: captures whose SEE is 0 | C·E | P |
| `see_positive_total` | 1 | count / 20: Σ of the SEEs above 0, the value twin of the count above | C·E | V |

Only one of a side's captures can be played, so `see_positive_total` says how
many ways there are to win material, not how much is to be won.

### 2.10 `threats` — what is about to be lost (width 24)

Every row is a pair: us then them. A threat is read on the units it is against,
so `threats.threatened_count[us]` counts what *we* stand to lose. Kings are left
out of every set: they cannot be captured. The last three rows are threats not
yet made — the geometry a pin, a skewer or a piled-up attack comes from.

| Feature | Width | Encoding | Cost | Head |
|---|---|---|---|---|
| `threatened_count` | 2 | per side, own threatened units, / 4 | B·E | B |
| `threatened_value` | 2 | per side, value sum of those, / 20 | B·E | B |
| `threat_max_gain` | 2 | per side, largest SEE of a unit over its own units, / 9 | B·E | B |
| `attacked_by_lesser_count` | 2 | per side, own units an enemy unit of strictly lower value order attacks, / 4 | B | B |
| `queen_attacked_by_lesser` | 2 | bit per side: one of those units is a queen | A | B |
| `overloaded_defenders` | 2 | per side, / 4 | B | V |
| `removable_defenders` | 2 | per side, / 4 | B·E | V |
| `loose_units` | 2 | per side, / 8 | A | V |
| `attacker_surplus_count` | 2 | per side, own units whose attacker surplus is above 0, / 4 | B | V |
| `xray_through_enemy` | 2 | per side, x-rays through one enemy unit onto an enemy unit, / 4 | B | P |
| `battery_count` | 2 | per side, / 4 | B | P |
| `battery_at_king` | 2 | bit per side: a battery whose line holds a square of the enemy king ring | B | B |

`threatened_count` answers exactly what `attacks.en_prise_count` approximates:
a unit an enemy pawn attacks is en prise whatever defends it, and threatened
only when the exchange on its square wins material.

### 2.11 `tactics` — one-ply tactics (width 120)

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
| `winning_capture_available` | 1 | bit: a capture whose SEE is above 0 | C·E | B |
| `winning_capture_max_gain` | 1 | max SEE over the captures, at least 0, / 9 | C·E | B |
| `captures_hanging_available` | 1 | bit: a capture of a hanging unit | C | B |
| `hanging_victim_max_value` | 1 | largest hanging victim capturable now, / 9 | C | B |
| `equal_capture_count` | 1 | captures whose SEE is 0, / 8 | C·E | P |
| `losing_capture_count` | 1 | captures whose SEE is below 0, / 8 | C·E | P |
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

### 2.12 `endgame` — endgame facts (width 15)

Always emitted; what it says decides games at low `phase` and is merely true
above it. Every pair is us then them.

| Feature | Width | Encoding | Cost | Head |
|---|---|---|---|---|
| `king_centralisation` | 2 | count/3: centralisation distance of the king | A | V |
| `race_plies` | 2 | count/8: race plies | A | V |
| `race_plies_diff` | 1 | diff/8: ours less theirs, so below 0 is our promotion first | A | V |
| `opposition` | 3 | one-hot: direct / distant / none | A | V |
| `key_square_occupied` | 2 | bit per side: the king stands on a key square of a passer of its own | B | V |
| `wrong_colour_bishop` | 2 | bit per side | A | V |
| `drawish_material` | 3 | one-hot: two knights against a bare king / a wrong-colour bishop with its rook pawns against a bare king / one bishop each on opposite colours with no other piece; zeros for any other material | A | V |

A side with no passer has `race_plies` 8: a real race runs 0 to 6 plies, so
the top of the scale is free to say there is none, and the difference of two
sides without a passer is 0.

The opposition is held by the side not to move, always, so the one-hot says
which kind stands rather than who has it; its third slot is "none", and one of
the three is always set. `drawish_material` is all zeros unless one of its
three configurations stands, and no two of them can stand at once — each of
the first two needs a bare king, and the third a bishop on both sides.
`key_square_occupied` and `wrong_colour_bishop` are 0 for a side whose
material the definition does not fit, and neither reads the material beyond
it: a bishop is the wrong colour with a rook still on the board, where
`drawish_material` names nothing.

### 2.13 `history` — what the plies before say (width 27)

The halfmove clock is the position's own; every other value is 0 with
`history_known` = 0 unless the caller supplies a game.

| Feature | Width | Encoding | Cost | Head |
|---|---|---|---|---|
| `halfmove_bucket` | 8 | one-hot over 0 / 1–3 / 4–9 / 10–19 / 20–39 / 40–69 / 70–89 / 90 and above | A | V |
| `halfmove_known` | 1 | bit: the position carried a halfmove clock | A | V |
| `repetition_seen` | 1 | bit: this position occurred before in the supplied history | A | V |
| `repetition_available_us` | 1 | bit: some legal move reaches a position in the history | C | V |
| `captures_in_last_8` | 1 | count / 8: captures among the last eight plies | A | V |
| `checks_in_last_8` | 1 | count / 8: plies among the last eight that gave check | A | V |
| `quiet_plies` | 1 | count / 16: see glossary | A | V |
| `material_trend` | 1 | diff / 9: see glossary | A | V |
| `last_move_victim` | 5 | one-hot P,N,B,R,Q; zeros for a quiet move | A | B |
| `last_move_mover` | 6 | one-hot P,N,B,R,Q,K; zeros when no move was played | A | B |
| `history_known` | 1 | bit: the caller supplied a position history | A | V |

A window shorter than the history it is read over is the history itself: a
game three plies long counts captures over three plies and reads its trend
from its own start.

The Lichess evaluation dump carries 4-field FENs (no halfmove clock, no move
number), so the whole group is constant across it — see
[§5](#5-open-questions).

### 2.14 `planes` — attack and status bitboards (width 512)

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

### 2.15 Totals

| Group | Width | Dominant cost |
|---|---|---|
| `placement` | 768 | A |
| `state` | 16 | A |
| `material` | 26 | A |
| `pawns` | 195 | A |
| `pieces` | 35 | A/B |
| `king` | 120 | A/B |
| `mobility` | 39 | B |
| `attacks` | 25 | B |
| `exchange` | 8 | C·E |
| `threats` | 24 | B·E |
| `tactics` | 120 | C, plus 2 features at D |
| `endgame` | 15 | A |
| `history` | 27 | A |
| `planes` | 512 | A |
| **total** | **1930** | |
| total without `placement` and `planes` | 650 | |

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
| Forced mate in 2 or more | Needs search. |
| Threats after the opponent's best reply | Needs 2 plies plus a choice of "best". |
| Zugzwang, fortress, corresponding squares | Needs search or endgame theory. The opposition is not among them: `endgame.opposition` reads it off the two king squares. |
| Tablebase results | External data. |
| Piece-square tables and any hand-tuned score | The net learns them from the board planes. |
| Move number / opening classification | Absent from the training source, and phase already covers what it would proxy. |
| Absolute colour | The mover's-view flip removes it; nothing in chess depends on it. |
| Chess960 castling geometry | Four features assume the classic starting squares and are defined for classic chess only: `pieces.minors_undeveloped`, `pieces.queen_developed`, `king.king_on_home_square`, `king.king_castled_zone`. Under another variant they are written as zeros, so widths and offsets do not move. Every other feature is 960-safe. |
| Game history beyond the plies a `Game` holds | The library does not track games; the caller passes what it knows, and `history` reports what the plies it was given say. |

---

## 5. Open questions

1. **Clock and history in training.** The Lichess dump has 4-field FENs, so
   the whole `history` group is constant across the training set. A feature
   that is always "unknown" during training is unusable at play time. Either
   omit the group from the trained schema — it is a group of its own so that
   this costs one name — or find a second source that carries clocks (game
   PGNs).
2. **`planes` width.** 512 of 1930 values. Ablation (§7) decides whether it
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
schema_semver = "1.0.0"
groups = [
  { name = "placement", version = 1, width = 768, offset =   0 },
  { name = "state",     version = 2, width =  16, offset = 768 },
  { name = "material",  version = 1, width =  26, offset = 784 },
  ...
]
schema_id = "16606f2b054a3281622fd2296f5ca13d"   # 128-bit, hex
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
`rs_anglerfish/esca/tests/data/schema_v1.txt`.

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
| Give a group of width 0 its first features | Bump that group's version; every later group's offset moves | stop working; refused at load |
| Remove a group | Bump `schema_semver` major | stop working; refused at load |

At most two versions of any group are kept compiled. Dropping an old version
is a major release.

---

## 7. Evaluation plan

### Runs

| Run | Input |
|---|---|
| `B0` baseline | `placement` only |
| `B1` augmented | every group |
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
