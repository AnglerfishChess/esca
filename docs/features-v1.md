# Feature catalogue v1 — candidates

Candidates only: a list for review, nothing decided and nothing implemented.
Terms, encodings, cost classes and group mechanics are those of
[`features.md`](features.md); this document adds to them and never
contradicts them.

esca facts serve two audiences of equal standing: a person or tool analysing
a game, who wants readable, precisely defined answers to "what is true
here"; and a net, which wants the same answers as a flat row. A fact
qualifies when a strong player would name it, when it has one unambiguous
definition, and when it costs at most about one ply.

Every candidate carries a verdict — `keep` or `maybe` — with a one-clause
reason. Columns: **Sd** sidedness (`us/them` a side-paired block, `shared`
one value for the position, `wings` a per-side per-wing block), **W** total
width, **Enc** the encoding token from `features.md` §6, **Cost** the class
(§5 adds E and F), **Var** the variants the definition holds under, **Twin**
the count/value partner (§2).

---

## 1. Glossary additions

### 1.1 Values and exchange

| Term | Definition |
|---|---|
| **value sum** | Σ over a set of units of P=1, N=B=3, R=5, Q=9, K=0; K=9 where a king is a target. |
| **value twin** | Of a feature counting the members of a set: the value sum over that same set. The **count twin** is the reverse. |
| **SEE** | Static exchange evaluation of a capture: the value the mover wins if both sides then continue capturing on the destination square, always with the least valuable attacker, x-rays behind a departed attacker included, either side free to stop at any point. Signed, in value units. |
| **SEE of a unit** | The SEE of the opponent's best capture of that unit; 0 when no enemy unit attacks it. |
| **threatened** | A unit whose SEE of a unit is > 0: the opponent wins material by taking it now. The exchange-exact form of *en prise*. |
| **max gain** | The largest SEE over a side's captures. |
| **attacker surplus** | On a unit: attackers minus defenders, counting only units of value at or below the unit's own. |
| **loose unit** | A unit no friendly unit defends, attacked or not. Kings excluded. |

### 1.2 Defenders, x-rays, batteries

| Term | Definition |
|---|---|
| **overloaded defender** | A unit that is the only defender of two or more friendly units that are each attacked. |
| **removable defender** | The only defender of an attacked friendly unit, itself capturable with SEE ≥ 0 for the capturer. |
| **x-ray attack** | A slider's attack on square *s* that exists once the single unit standing between them is removed. *Through own*: that unit is the slider's own. *Through enemy*: it is not. |
| **battery** | Two friendly sliders on one ray, both attacking along it with only their own kind of blocker between: Q+R on a rank or file, Q+B on a diagonal, R+R, B+Q, Q+Q. |

### 1.3 Pawns

| Term | Definition |
|---|---|
| **pawn chain** | A maximal run of friendly pawns each defending the next, along one diagonal direction. Its **length** is the number of pawns in it. |
| **chain base** | The rearmost pawn of a chain, i.e. the one no friendly pawn defends. |
| **duo** | Two friendly pawns on adjacent files at the same relative rank. |
| **majority** | On a wing (glossary §1 of `features.md`), strictly more own pawns than enemy pawns. |
| **hole for X** | A square on X's relative ranks 3–6 that no pawn of X can ever attack: no pawn of X stands on either adjacent file at a lower relative rank. |
| **tension** | A pair of opposing pawns each attacking the other's square. Counted per pair, once. |
| **fixed pawn** | A pawn whose stop square carries any unit. A *ram* is the case where that unit is an enemy pawn. |
| **blocked passer** | A passed pawn whose stop square carries an enemy unit. |
| **promotion distance** | 8 − relative rank: the pushes a pawn still needs. |
| **square of the passer** | The square of side 8 − *r* whose corner is the passer's promotion square. The defending king is *in the square* when it stands inside it, one file wider when the passer's owner is to move. The rule of the square is this test alone; *unstoppable passer* additionally requires a bare-king defender. |
| **key square** | For a pawn on relative rank ≤ 4: the three squares two ranks ahead, on its file and both neighbours. On ranks 5–7: the three squares one rank ahead. Rook pawns have none. |

### 1.4 Pieces and king

| Term | Definition |
|---|---|
| **trapped unit** | A non-pawn, non-king unit with no safe destination among its legal moves. |
| **lifted rook** | A rook on relative ranks 3–5 with no friendly pawn ahead of it on its file. |
| **fianchetto** | A friendly pawn on relative b2 or g2 with the long diagonal from that square clear of friendly pawns; *held* when a friendly bishop stands on that square. |
| **wrong-colour bishop** | A side whose only bishops stand on one square colour while all its pawns are rook pawns promoting on the other colour. |
| **open ray** | A direction from a king — one of eight — with no unit on it between the king and the board edge. |
| **luft** | An empty, unattacked square adjacent to a king on its relative rank 1, on the rank ahead. |
| **castled side** | Read statically: short when the king stands on relative files g–h with no castling right left, long when on a–c likewise, otherwise none. |
| **centralisation distance** | Chebyshev distance from a square to the nearest of d4, e4, d5, e5. |
| **opposition** | Kings on one file, rank or diagonal with an odd number of empty squares between them; held by the side *not* to move. *Direct* when that number is 1. |

### 1.5 History

| Term | Definition |
|---|---|
| **quiet plies** | Plies since the last capture or check in the supplied history. Distinct from the halfmove clock, which counts captures and pawn moves. |
| **material trend** | Material balance now minus material balance 8 plies ago, from the mover's side. |
| **race plies** | For a side, the plies its most advanced passer needs to promote unopposed: promotion distance, less 1 when that side is to move. |

---

## 2. Value scale and count/value twins

One scale everywhere: the value sum of §1.1. Encodings are fixed by role, so
a twin never needs its own scale argument.

| Aggregate over | Count encoding | Value encoding |
|---|---|---|
| Units of one side | `count/8` | `count/20` |
| A subset that is normally small (hanging, pinned, forked) | `count/4` | `count/20` |
| A single largest unit | — | `count/9` |
| A signed side difference | `diff/8` | `diff/20` |

Every v0 aggregate over units, and where its missing twin lands:

| v0 aggregate | Kind | Twin | Status |
|---|---|---|---|
| `attacks.hanging_count` | count | `attacks.hanging_value` | present |
| `attacks.hanging_value` | value | `attacks.hanging_max_value` (largest, not a twin) | §3.7 |
| `attacks.en_prise_count` | count | `attacks.en_prise_value` | §3.7 |
| `attacks.en_prise_max_value` | value | — | max, no count twin |
| `attacks.defended_count` | count | `attacks.defended_value` | §3.7 |
| `attacks.pinned_count` | count | `attacks.pinned_value` | §3.7 |
| — | — | `attacks.attacked_count` / `attacked_value` | §3.7, both new |
| `king.ring_attackers` | count | `king.ring_attack_weight` | present |
| `king.ring_defended` (squares) | count | `king.ring_defenders` / `ring_defence_weight` | §3.5 |
| `mobility.*` | squares | — | squares have no value twin |
| `pieces.knights_on_outpost` | count | — | roles, not values |
| `tactics.fork_count` | count | `tactics.fork_max_value` | present |
| `tactics.capture_count` | count | `exchange.see_positive_total` | §3.8 |
| `pawns.passer_protected` | count | — | pawns are one value |
| — | — | `pieces.trapped_pieces` / `trapped_value` | §3.4, both new |
| — | — | `threats.threatened_count` / `threatened_value` | §3.9, both new |

---

## 3. Candidate position groups

### 3.1 `placement` — piece planes (new group)

| Feature | Definition | Sd | W | Enc | Cost | Var | Twin | Verdict |
|---|---|---|---|---|---|---|---|---|
| `piece_planes` | 12 planes, role × side, in the mover's view | us/them | 768 | plane | A | all | — | keep — the standard baseline input, and the thing every ablation is measured against |

### 3.2 `material` additions

| Feature | Definition | Sd | W | Enc | Cost | Var | Twin | Verdict |
|---|---|---|---|---|---|---|---|---|
| `bishop_pair_imbalance` | our bishop pair minus theirs | shared | 1 | diff/1 | A | all | — | keep — the imbalance a player names, not derivable from two bits by a linear head |
| `minor_kind_diff` | (our B − our N) − (their B − their N) | shared | 1 | diff/4 | A | all | — | maybe — a linear function of `piece_count_diff` |
| `non_pawn_material_diff` | value sum of N,B,R,Q, us − them | shared | 1 | diff/20 | A | all | value of piece counts | keep — the phase-relevant balance, currently only per side |
| `material_value_ratio` | our value sum / (ours + theirs) | shared | 1 | ratio | A | all | — | maybe — `material_balance` covers it except at low material |

### 3.3 `pawns` additions

| Feature | Definition | Sd | W | Enc | Cost | Var | Twin | Verdict |
|---|---|---|---|---|---|---|---|---|
| `chain_count` | pawn chains of length ≥ 2 | us/them | 2 | count/4 | B | all | — | maybe — length matters more than how many |
| `chain_max_length` | longest chain | us/them | 2 | count/5 | B | all | — | keep — names the structure the position is about |
| `chain_base_attacked` | a chain base attacked by an enemy unit | us/them | 2 | bit | B | all | — | keep — the standard target, and cheap once chains exist |
| `duos` | pawn duos | us/them | 2 | count/4 | A | all | — | maybe — correlated with `defended_pawns` |
| `majority_by_wing` | majority on the queen-side, on the king-side | wings | 4 | bits | A | all | — | keep — decides where a passer will come from |
| `pawn_diff_by_wing` | our pawns − theirs, per wing | wings | 2 | diff/4 | A | all | — | maybe — the majority bits already say the sign |
| `holes` | holes for the side (§1.3) | us/them | 2 | count/16 | A | all | — | keep — the square-weakness a player reads first |
| `holes_occupied` | enemy knights and bishops standing on our holes | us/them | 2 | count/4 | B | all | — | keep — a hole only hurts when it is used |
| `tension` | mutually attacking pawn pairs | shared | 1 | count/4 | A | all | — | maybe — `levers` counts almost the same set |
| `fixed_pawns` | pawns whose stop square is occupied | us/them | 2 | count/8 | A | all | — | keep — separates a static structure from a mobile one |
| `blocked_passers` | passers blockaded by an enemy unit | us/them | 2 | count/2 | A | all | — | keep — a blockaded passer is a different fact from a passer |
| `passer_distance` | promotion distance of the most advanced passer | us/them | 2 | count/6 | A | all | — | keep — what `passer_lead_rank` says, as a number the net can subtract |
| `passer_king_distance` | Chebyshev distance from own king, and from the enemy king, to that passer's promotion square | us/them | 4 | count/8 | A | all | — | keep — the endgame's central quantity |
| `passer_in_square` | the defending king is in the square of the lead passer | us/them | 2 | bit | A | all | — | keep — the rule of the square as its own fact, not buried in `passer_unstoppable` |
| `passers_by_rank` | passers per relative rank | us/them | 16 | count/2 | A | all | — | maybe — 16 values for what `passer_lead_rank` mostly says |
| `connected_passers_count` | passers on adjacent files | us/them | 2 | count/2 | A | all | count twin of `passers_connected` | maybe — the v0 bit is nearly always the same value |
| `passer_free_path` | passers whose whole front span is empty | us/them | 2 | count/2 | A | all | — | keep — distinguishes a running passer from a stopped one |
| `passer_path_attacked` | passers whose front span the enemy attacks | us/them | 2 | count/2 | B | all | — | maybe — overlaps `passer_free_path` and king distance |
| `half_open_at_enemy_king` | our semi-open files among the enemy king files | us/them | 2 | count/3 | A | all | — | keep — the attacking plan in one number |
| `open_file_at_enemy_king` | open files among the enemy king files | us/them | 2 | count/3 | A | all | — | maybe — `king_file_openness` already carries it per file |
| `backward_on_semi_open` | backward pawns on a file semi-open for the enemy | us/them | 2 | count/4 | A | all | — | keep — the backward pawns that are actually weak |
| `isolated_on_semi_open` | isolated pawns likewise | us/them | 2 | count/4 | A | all | — | maybe — weaker signal than the backward case |
| `doubled_count` | doubled pawns | us/them | 2 | count/4 | A | all | count twin of `doubled_files` | maybe — the mask carries the same information |

### 3.4 `pieces` additions

| Feature | Definition | Sd | W | Enc | Cost | Var | Twin | Verdict |
|---|---|---|---|---|---|---|---|---|
| `fixed_pawns_on_bishop_colour` | own fixed pawns on the colour of own bishops | us/them | 2 | count/8 | B | all | — | keep — the bad-bishop refinement: a blocked pawn is what makes it bad |
| `bishops_blocked` | own bishops attacking ≤ 3 squares | us/them | 2 | count/2 | B | all | — | maybe — `mobility_by_type` says this for the whole role |
| `bishop_pair_vs_knight_pair` | we have the bishop pair and they two knights, minus the reverse | shared | 1 | diff/1 | A | all | — | keep — the named imbalance |
| `fianchetto` | per wing: fianchetto held, fianchetto without its bishop | us/them | 8 | bits | B | all | — | maybe — 8 values for a pattern the shelter features half-cover |
| `rook_on_7th_with_king_on_8th` | a rook on relative rank 7 with the enemy king on relative rank 8 | us/them | 2 | bit | A | all | — | keep — the version of "rook on the 7th" that is actually worth material |
| `rooks_doubled_on_open_file` | two connected rooks on one open or semi-open file | us/them | 2 | bit | A | all | — | maybe — `rooks_connected_file` plus `rooks_on_open_file` nearly implies it |
| `rooks_lifted` | lifted rooks (§1.4) | us/them | 2 | count/2 | A | all | — | maybe — the pattern is common but its value is position-specific |
| `trapped_pieces` | non-pawn, non-king units with no safe destination | us/them | 2 | count/4 | C | all | — | keep — "the piece has no moves" is a first-rank fact and `immobile_pieces` misses safety |
| `trapped_value` | value sum of those | us/them | 2 | count/20 | C | all | value of `trapped_pieces` | keep — a trapped queen and a trapped knight are not one fact |
| `minors_on_outpost` | knights and bishops on an outpost square | us/them | 2 | count/2 | B | all | — | keep — generalises `knights_on_outpost`, which it should replace |
| `outpost_occupant_defended` | outpost occupants defended by an own pawn | us/them | 2 | count/2 | B | all | — | maybe — the outpost definition already requires a pawn attacks the square |

### 3.5 `king` additions

| Feature | Definition | Sd | W | Enc | Cost | Var | Twin | Verdict |
|---|---|---|---|---|---|---|---|---|
| `ring_defenders` | own N,B,R,Q attacking own king-ring squares | us/them | 2 | count/6 | B | all | count twin of `ring_defence_weight` | keep — attack without the defence half is not readable |
| `ring_defence_weight` | Σ over those of N,B = 1, R = 2, Q = 4 | us/them | 2 | count/16 | B | all | value twin | keep — mirrors `ring_attack_weight` exactly |
| `ring_attacker_surplus` | ring attack weight minus ring defence weight | us/them | 2 | diff/16 | B | all | — | keep — the quantity a player judges, and a difference a linear head cannot form from ratios |
| `open_rays_to_king` | open rays from the king (§1.4) | us/them | 2 | count/8 | A | all | — | keep — direct exposure, distinct from `virtual_mobility` in what it counts |
| `luft` | the king has luft | us/them | 2 | bit | A | all | — | keep — one bit, and the whole back-rank question |
| `castled_side` | one-hot short / long / none, read statically | us/them | 6 | one-hot | A | classic | — | maybe — `king_castled_zone` says nearly the same in 4 values |
| `opposite_side_castling` | the kings stand on opposite wings | shared | 1 | bit | A | all | — | keep — governs whether a pawn storm is a plan at all |

### 3.6 `mobility` additions

| Feature | Definition | Sd | W | Enc | Cost | Var | Twin | Verdict |
|---|---|---|---|---|---|---|---|---|
| `safe_mobility_diff_by_type` | safe mobility us − them, per type | shared | 5 | diff/16 | A | all | — | keep — the unsafe difference exists; the safe one is the one that predicts |
| `mobility_value_weighted` | Σ over units of destinations × unit value | us/them | 2 | count/200 | B | all | value twin of `total_mobility` | maybe — mixes two scales into one number |

### 3.7 `attacks` additions

| Feature | Definition | Sd | W | Enc | Cost | Var | Twin | Verdict |
|---|---|---|---|---|---|---|---|---|
| `attacked_count` | own units the opponent attacks, defended or not | us/them | 2 | count/8 | A | all | count twin of `attacked_value` | keep — the base the hanging and en-prise subsets are read against |
| `attacked_value` | value sum of those | us/them | 2 | count/20 | B | all | value twin | keep — pairs the count |
| `en_prise_value` | value sum of en prise units | us/them | 2 | count/20 | B | all | value of `en_prise_count` | keep — the missing twin |
| `defended_value` | value sum of defended own units | us/them | 2 | count/30 | B | all | value of `defended_count` | maybe — the count is already a weak feature |
| `pinned_value` | value sum of absolutely pinned units | us/them | 2 | count/20 | B | all | value of `pinned_count` | keep — a pinned queen and a pinned pawn are not one fact |
| `relative_pin_count` | relative pins (v0 glossary) | us/them | 2 | count/4 | B | all | — | keep — the glossary defines it and no feature emits it |
| `hanging_max_value` | largest hanging unit | us/them | 2 | count/9 | B | all | — | keep — matches `en_prise_max_value` |
| `xray_through_enemy` | slider x-rays onto an enemy unit through one enemy unit | us/them | 2 | count/4 | B | all | — | keep — pin and skewer geometry before either is a pin or a skewer |
| `xray_through_own` | the same through one own unit | us/them | 2 | count/4 | B | all | — | maybe — largely the battery count seen from the other end |
| `battery_count` | batteries (§1.2) | us/them | 2 | count/4 | B | all | — | keep — named, cheap, and invisible to per-square attack maps |
| `battery_at_king` | a battery whose ray meets the enemy king ring | us/them | 2 | bit | B | all | — | keep — the attacking version, which is the one that matters |

### 3.8 `exchange` — SEE (new group)

Two blocks, `us` then `them`, the second after a null move, as `tactics` is.

| Feature | Definition | Sd | W | Enc | Cost | Var | Twin | Verdict |
|---|---|---|---|---|---|---|---|---|
| `see_best_capture` | largest SEE over the side's captures | us/them | 2 | diff/9 | C·E | all | — | keep — the exact answer the `winning_capture_*` pair approximates |
| `see_positive_capture_count` | captures with SEE > 0 | us/them | 2 | count/8 | C·E | all | — | keep — the count a move-ordering scheme and a reader both want |
| `see_equal_capture_count` | captures with SEE = 0 | us/them | 2 | count/8 | C·E | all | — | keep — the exchange-exact replacement for `equal_capture_count` |
| `see_negative_capture_count` | captures with SEE < 0 | us/them | 2 | count/8 | C·E | all | — | maybe — the complement of the other two given `capture_count` |
| `see_positive_total` | Σ of positive SEEs | us/them | 2 | count/20 | C·E | all | value of `see_positive_capture_count` | maybe — only one of them can be taken |

### 3.9 `threats` (new group)

| Feature | Definition | Sd | W | Enc | Cost | Var | Twin | Verdict |
|---|---|---|---|---|---|---|---|---|
| `threatened_count` | own units with SEE of a unit > 0 | us/them | 2 | count/4 | B·E | all | count twin | keep — "is something hanging" answered exactly rather than by the 1-ply proxy |
| `threatened_value` | value sum of those | us/them | 2 | count/20 | B·E | all | value twin | keep — pairs the count |
| `threat_max_gain` | largest SEE the opponent has against us | us/them | 2 | count/9 | B·E | all | — | keep — one number for "how much am I about to lose" |
| `attacked_by_lesser_count` | own units attacked by a strictly lower-valued enemy unit | us/them | 2 | count/4 | B | all | count twin | keep — the classic threat, and cheap without SEE |
| `attacked_by_lesser_value` | value sum of those | us/them | 2 | count/20 | B | all | value twin | maybe — `threatened_value` covers the cases that matter |
| `queen_attacked_by_lesser` | our queen attacked by a lower-valued unit | us/them | 2 | bit | A | all | — | keep — the single most consequential instance, worth its own bit |
| `overloaded_defenders` | overloaded defenders (§1.2) | us/them | 2 | count/4 | B | all | — | keep — a named motif with an exact definition |
| `removable_defenders` | removable defenders (§1.2) | us/them | 2 | count/4 | B·E | all | — | keep — the other half of "the defence is not real" |
| `loose_units` | own units nothing defends | us/them | 2 | count/8 | A | all | count twin | keep — loose pieces drop pieces; hanging misses the undefended-but-unattacked ones |
| `loose_max_value` | largest loose unit | us/them | 2 | count/9 | B | all | — | maybe — correlated with the count at the values that matter |
| `attacker_surplus_count` | own units with attacker surplus > 0 | us/them | 2 | count/4 | B | all | — | keep — the multi-attacker case that a per-unit view misses |
| `attacker_surplus_max` | largest such surplus | us/them | 2 | count/4 | B | all | — | maybe — rarely above 1 in real positions |

### 3.10 `tactics` additions

Per block, emitted twice (`us`, `them`); W below is the pair.

| Feature | Definition | Sd | W | Enc | Cost | Var | Twin | Verdict |
|---|---|---|---|---|---|---|---|---|
| `safe_check_capturing` | a safe check that also captures | us/them | 2 | bit | C | all | — | keep — check plus material is a different move from either |
| `discovered_attack_on_queen` | a move uncovering a slider's attack on the enemy queen | us/them | 2 | bit | C | all | — | keep — the discovered attack that decides games |
| `skewer_creation_count` | moves creating a skewer | us/them | 2 | count/4 | C | all | count twin of `skewer_creation_available` | maybe — the bit rarely under-reports |
| `back_rank_mate_threat` | a rook or queen can reach the enemy relative rank 8 giving check while that king's three forward squares are blocked by its own units | us/them | 2 | bit | C | all | — | keep — an exactly definable mating pattern below the cost of `mate_in_1` |
| `quiet_threat_available` | a quiet move after which our best SEE exceeds our current best | us/them | 2 | bit | C·E | all | — | keep — the whole class of "threat" moves is invisible to v0 |
| `quiet_threat_max_gain` | the largest such new best SEE | us/them | 2 | count/9 | C·E | all | — | maybe — the bit is most of the signal at a fraction of the cost |
| `mate_threat_after_quiet` | a quiet move after which we mate in 1 whatever they do to stop it | us/them | 2 | bit | F | all | — | maybe — two plies, cost-capped at best |
| `no_piece_moves` | no legal move by a non-pawn, non-king unit | us/them | 2 | bit | C | all | — | maybe — a zugzwang proxy that fires mostly in endings |
| `no_safe_moves` | no legal move has a safe destination | us/them | 2 | bit | C | all | — | keep — the readable half of zugzwang, and free once safety is computed |
| `capture_of_defender_available` | a capture whose victim is a removable defender | us/them | 2 | bit | C·E | all | — | maybe — `removable_defenders` says it from the defender's side |
| `promotion_see_positive` | a promotion with SEE > 0 | us/them | 2 | bit | C·E | all | — | keep — `safe_promotion_available` is the approximation of exactly this |

### 3.11 `endgame` (new group)

Always emitted; meaningful mostly at low `phase`.

| Feature | Definition | Sd | W | Enc | Cost | Var | Twin | Verdict |
|---|---|---|---|---|---|---|---|---|
| `king_centralisation` | centralisation distance of the king | us/them | 2 | count/6 | A | all | — | keep — the endgame king fact, and one lookup |
| `race_plies` | race plies (§1.5) | us/them | 2 | count/8 | A | all | — | keep — states the race in the unit it is counted in |
| `race_plies_diff` | ours minus theirs | shared | 1 | diff/8 | A | all | — | keep — the sign is the whole answer |
| `opposition` | one-hot: we hold it / they hold it / nobody | shared | 3 | one-hot | A | all | — | maybe — exact but only decisive in king-and-pawn endings |
| `key_square_occupied` | own king on a key square of an own passer | us/them | 2 | bit | B | all | — | maybe — textbook, but narrow outside K+P |
| `wrong_colour_bishop` | wrong-colour bishop (§1.4) | us/them | 2 | bit | A | all | — | keep — turns an extra piece into a draw, and nothing else says so |
| `rook_pawns_only` | the side's only pawns are on files a or h | us/them | 2 | bit | A | all | — | maybe — a component of the wrong-bishop fact |
| `drawish_material` | one-hot: KNN v K / K+wrong bishop+rook pawn / opposite bishops and nothing else | shared | 3 | one-hot | A | all | — | keep — the refinements `insufficient_material` deliberately excludes |

### 3.12 `history` (new group, `Game` only)

Zero, with `history_known` = 0, when the caller supplies no history.

| Feature | Definition | Sd | W | Enc | Cost | Var | Twin | Verdict |
|---|---|---|---|---|---|---|---|---|
| `captures_in_last_8` | captures in the last 8 plies | shared | 1 | count/8 | A | all | — | keep — the quietness a reader assumes and no feature states |
| `checks_in_last_8` | checks in the last 8 plies | shared | 1 | count/8 | A | all | — | keep — pairs the above; forcing sequences look different |
| `quiet_plies` | plies since the last capture or check | shared | 1 | count/16 | A | all | — | keep — the sharpness of the moment in one value |
| `material_trend` | material balance now minus 8 plies ago | shared | 1 | diff/9 | A | all | — | keep — says who has been winning material, which the balance alone does not |
| `last_move_victim` | one-hot P,N,B,R,Q; zeros for a quiet move | shared | 5 | one-hot | A | all | — | keep — recaptures are read from it |
| `last_move_mover` | one-hot P,N,B,R,Q,K | shared | 6 | one-hot | A | all | — | keep — pairs the victim |
| `last_move_was_check` | the last move gave check | shared | 1 | bit | A | all | — | keep — `in_check` says we are in check, not that they just gave it |
| `last_move_was_capture` | the last move captured | shared | 1 | bit | A | all | — | maybe — implied by `last_move_victim` |
| `repetition_count` | one-hot: 0 / 1 / 2+ prior occurrences | shared | 3 | one-hot | A | all | count twin of `repetition_seen` | maybe — refines a bit that is already rarely 1 |
| `plies_played` | plies since the start position | shared | 1 | count/100 | A | all | — | maybe — `phase` proxies it and the dump has no move number |

### 3.13 `planes` additions

| Plane | Definition | Sd | W | Enc | Cost | Var | Twin | Verdict |
|---|---|---|---|---|---|---|---|---|
| `their_threatened` | their units with SEE of a unit > 0 | them | 64 | plane | B·E | all | — | keep — where the material is, not how much |
| `our_threatened` | ours likewise | us | 64 | plane | B·E | all | — | maybe — `our_hanging` already covers most of it |
| `our_see_loss` | per own unit, its SEE of a unit / 9 | us | 64 | plane | B·E | all | value twin of `our_threatened` | maybe — a graded plane is a lot of width for a rare signal |
| `their_see_loss` | theirs likewise | them | 64 | plane | B·E | all | value twin | maybe — as above |
| `holes_us` | our holes | us | 64 | plane | A | all | — | maybe — the count may be enough |
| `holes_them` | theirs | them | 64 | plane | A | all | — | maybe — as above |

### 3.14 Totals

| Group | Candidates | keep | maybe | Width keep | Width maybe |
|---|---|---|---|---|---|
| `placement` | 1 | 1 | 0 | 768 | 0 |
| `material` | 4 | 2 | 2 | 2 | 2 |
| `pawns` | 23 | 13 | 10 | 30 | 33 |
| `pieces` | 11 | 6 | 5 | 11 | 16 |
| `king` | 7 | 6 | 1 | 11 | 6 |
| `mobility` | 2 | 1 | 1 | 5 | 2 |
| `attacks` | 11 | 9 | 2 | 18 | 4 |
| `exchange` | 5 | 3 | 2 | 6 | 4 |
| `threats` | 12 | 9 | 3 | 18 | 6 |
| `tactics` | 11 | 6 | 5 | 12 | 10 |
| `endgame` | 8 | 5 | 3 | 10 | 7 |
| `history` | 10 | 7 | 3 | 16 | 5 |
| `planes` | 6 | 1 | 5 | 64 | 320 |
| **total** | **111** | **69** | **42** | **971** | **415** |

v0 after the drops of §6 is 1060. A v1 of the `keep` set is **2031**
values, of which 768 are `placement`; with every `maybe` it is **2446**.
Without `placement` and `planes` the `keep` set is 139 new values over v0.

---

## 4. Per-move candidates (`move` schema)

v0 is 24 values, all class A per move. The deltas below need a make-move and
a partial rescan per move (class F): they are the reason a move schema might
have two tiers.

| Feature | Definition | W | Enc | Cost | Var | Verdict |
|---|---|---|---|---|---|---|
| `see` | SEE of the move | 1 | diff/9 | E | all | keep — the single most useful move number, and what `is_safe` approximates |
| `see_positive` | SEE > 0 | 1 | bit | E | all | maybe — a threshold on `see` |
| `threat_created_max` | largest SEE we would then have | 1 | count/9 | F | all | keep — separates a threat from a mere developing move |
| `moves_attacked_unit` | the origin square is attacked by them | 1 | bit | A | all | keep — `escapes_attack` also demands a safe destination, so it misses half the cases |
| `moves_hanging_unit` | the moved unit was hanging | 1 | bit | A | all | maybe — a conjunction of two facts already present |
| `blocks_check` | the destination interposes on the checking ray | 1 | bit | A | all | keep — in check, it is the move's whole character |
| `develops` | a minor leaves its home square | 1 | bit | A | classic | maybe — opening-only, and undefined off classic |
| `advances_passer` | the moved unit is a passed pawn | 1 | bit | A | all | keep — the move class a policy head must rank |
| `passer_new_distance` | promotion distance after the move | 1 | count/6 | A | all | maybe — derivable from the destination |
| `creates_passer` | the mover's side has a passer it did not have | 1 | bit | B | all | keep — a decisive, named consequence |
| `creates_weakness` | bits: creates an isolated / doubled / backward own pawn | 3 | bits | B | all | keep — the cost side of a pawn move, absent from v0 |
| `opens_file_at_enemy_king` | a king file of theirs becomes open or semi-open for us | 1 | bit | B | all | keep — the attacking pawn move, named |
| `closes_file_at_own_king` | a king file of ours stops being open for them | 1 | bit | B | all | maybe — the defensive mirror is rarer and weaker |
| `ring_attack_delta` | change in our attackers of their ring, and in theirs of ours | 2 | diff/4 | F | all | keep — states the attacking or weakening intent directly |
| `mobility_delta` | change in our total mobility | 1 | diff/16 | F | all | maybe — expensive for a diffuse signal |
| `own_hanging_delta` | change in the count of our hanging units | 1 | diff/4 | F | all | keep — the safety consequence of the move |
| `their_hanging_delta` | change in the count of theirs | 1 | diff/4 | F | all | keep — pairs the above |
| `leaves_unit_hanging` | after the move some own unit of value ≥ 3 hangs that did not | 1 | bit | F | all | keep — the blunder flag; the label a human reader asks for |
| `gives_discovered_attack` | a unit that did not move gains an attack on a unit of value ≥ 3 | 1 | bit | B | all | keep — the position-level fact exists; the per-move one does not |
| `is_recapture` | the destination is the previous move's destination | 1 | bit | A | all | maybe — `Game` only, so zero on dump rows |

20 candidates: 13 `keep` (16 values, move schema 40), 7 `maybe` (7 values,
move schema 47).

---

## 5. Encoding and cost notes

Two cost classes are added to `features.md` §1:

| Class | Meaning | Rough budget |
|---|---|---|
| **E** | One exchange loop on one square: least valuable attacker, x-rays refreshed, at most about 32 steps. | ~50–500 ns |
| **F** | Make-move plus a rescan of the affected sets, per move. | ~1–5 µs |

`B·E` is an exchange loop per unit of a side, `C·E` one per capture in the
move list. Class F over a whole move list is the most expensive thing
proposed here and is why every F feature is per-move, never per-position.

Encoding tokens are unchanged: everything above is `bit`, `bits`, `one-hot`,
`plane`, `ratio`, `count/S` or `diff/S`. Two conventions are new:

| Convention | |
|---|---|
| `diff/1` | A difference of two bits, values −1, 0, 1. |
| Value aggregates | Always `count/20` for a side's sum, `count/9` for a single largest unit, `diff/20` for a difference. |

Variant sensitivity: `castled_side` and the per-move `develops` read classic
starting squares and are `classic` only, written as zeros elsewhere, as v0
does for its four such features. Everything else here is defined from the
position alone and holds under every variant.

---

## 6. Schema v1 shape

### Drops

| v0 feature | Reason |
|---|---|
| `king.king_distance` indices 1 and 8 | Chebyshev distance between kings is never 1 and never 8; two dead columns. Width 8 → 6. |
| `state.repetition_available_them` | Near-constant; and zero across the whole dump, where `history_known` is 0. |
| `pieces.knights_on_outpost` | Superseded by `pieces.minors_on_outpost`, which counts the same knights. |

### Merges and redefinitions

| Change | |
|---|---|
| `tactics.winning_capture_available` | Redefine as "a capture with SEE > 0"; the victim-versus-capturer test becomes exact and the feature keeps its name, width and encoding. |
| `tactics.winning_capture_max_gain` | Redefine as `exchange.see_best_capture` clamped at 0, or drop in favour of it. |
| `tactics.equal_capture_count`, `losing_capture_count` | Redefine as SEE = 0 and SEE < 0; they then partition the captures with `see_positive_capture_count`. |
| `pawns.passers_connected` | Drop the bit if `connected_passers_count` is taken; keep the bit otherwise. |
| `state.halfmove_*`, `state.repetition_*`, `history_known` | Move into the new `history` group, so that the whole history-dependent block is one group a training set without clocks can omit by group. |

### Group order

Placement first, then the position groups roughly cheapest-first, then the
history block, then the planes:

```
placement, state, material, pawns, pieces, king, mobility,
attacks, exchange, threats, tactics, endgame, history, planes
```

`exchange` and `threats` sit next to `attacks` because they read the same
scratch; `endgame` and `history` sit last among the scalar groups because
they are the two most likely to be omitted from a run. `placement` and
`planes` bracket the schema so that a net trained on scalars alone selects a
contiguous middle.

Group versions: `state`, `pawns`, `pieces`, `king`, `mobility`, `attacks`
and `tactics` all gain features, so each bumps to version 2; `material`
likewise. `placement`, `exchange`, `threats`, `endgame` and `history` are
new groups at version 1. `planes` bumps to version 2. Under the v0 evolution
rules that is a new `schema_id` and, because of the drops, a `schema_semver`
major.
