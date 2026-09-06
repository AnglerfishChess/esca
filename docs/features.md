# The facts

`Position::facts()` answers what is true about a position. Every fact belongs
to a group, and every fact names the colour it is about.

- `ByColour<T>` is one `T` for White and one for Black: `.white`, `.black`, or
  `.of(colour)`. In Python it is an `esca.ByColour`, read the same way, and
  `to_dict()` writes it as `{"white": …, "black": …}`.
- `SquareSet` is a set of squares, `FileSet` a set of files. Both are objects
  in Python too; `to_dict()` writes them as lists of names.
- `Option<T>` is a value that may be absent — `None` in Python.
- `[T; n]` is a fixed run of `n` values, a Python list.
- `Role` is a piece kind, written as its lower-case letter: `p n b r q k`.
- The named values are `Opposition` (`direct`, `distant`), `DrawishMaterial`
  (`two_knights`, `wrong_bishop`, `opposite_bishops`) and `CastledSide`
  (`short`, `long`). In Python each is that name as text.
- Where a fact is about the side to move rather than about a colour — that it
  stands in check, say — it is a plain value, and its meaning says so.
- A rank "counted from a side's own back rank" is rank 1 at that side's
  first rank and rank 8 at the promotion rank.

Every group is also a Python attribute of `Facts`, listed by `Facts.groups()`
and read by `facts.group(name)`; `fields()` and `to_dict()` walk one group in
the order below. This file is generated from the group definitions themselves:
`cargo run --example catalogue`.

## `placement`

Where the units stand.

| field | type | meaning |
| --- | --- | --- |
| `units` | `ByColour<SquareSet>` | Every unit of the side. |
| `pawns` | `ByColour<SquareSet>` | The side's pawns. |
| `knights` | `ByColour<SquareSet>` | The side's knights. |
| `bishops` | `ByColour<SquareSet>` | The side's bishops. |
| `rooks` | `ByColour<SquareSet>` | The side's rooks. |
| `queens` | `ByColour<SquareSet>` | The side's queens. |
| `kings` | `ByColour<SquareSet>` | The side's king. |

## `state`

Check, castling rights and the en-passant square.

| field | type | meaning |
| --- | --- | --- |
| `in_check` | `bool` | The side to move stands in check. |
| `double_check` | `bool` | Two or more units give check to the side to move. |
| `can_castle_short` | `ByColour<bool>` | The side may still castle short. |
| `can_castle_long` | `ByColour<bool>` | The side may still castle long. |
| `en_passant` | `Option<File>` | The file the position names as the en-passant target, if any. |
| `en_passant_capture_legal` | `bool` | Some legal move captures en passant. |

## `history`

What the plies before this position say: the clock, repetition, and how forcing the recent play was. Everything but the halfmove clock, which the position itself carries, is zero unless `known` is true.

| field | type | meaning |
| --- | --- | --- |
| `known` | `bool` | A position history was supplied. |
| `halfmove_clock` | `u32` | Plies since the last capture or pawn move. |
| `halfmove_known` | `bool` | The position carried a halfmove clock. |
| `repetition_seen` | `bool` | This position occurred before in the supplied history. |
| `repetition_available` | `bool` | Some legal move reaches a position of the supplied history. |
| `captures_in_last_8` | `u8` | Captures among the last eight plies of the supplied history. |
| `checks_in_last_8` | `u8` | Plies among the last eight that gave check. |
| `quiet_plies` | `u32` | Plies since the last one that captured or gave check; the whole history when it holds none. |
| `material_trend` | `i32` | The material balance now, less the balance eight plies ago, White less Black. |
| `last_move_victim` | `Option<Role>` | The role the last move captured, the pawn for an en-passant capture. |
| `last_move_mover` | `Option<Role>` | The role that made the last move; for castling, the king. |

## `material`

Material and phase.

| field | type | meaning |
| --- | --- | --- |
| `pawns` | `ByColour<u8>` | The side's pawns. |
| `knights` | `ByColour<u8>` | The side's knights. |
| `bishops` | `ByColour<u8>` | The side's bishops. |
| `rooks` | `ByColour<u8>` | The side's rooks. |
| `queens` | `ByColour<u8>` | The side's queens. |
| `non_pawn_value` | `ByColour<i32>` | The value of the side's knights, bishops, rooks and queens. |
| `value` | `ByColour<i32>` | The value of every unit of the side but the king. |
| `phase_points` | `u8` | `4·Q + 2·R + B + N` over both sides: 24 at the full opening array, 0 with nothing but kings and pawns left. |
| `both_queens` | `bool` | Both sides have at least one queen. |
| `pawns_only` | `bool` | Neither side has a unit other than kings and pawns. |
| `insufficient_material` | `ByColour<bool>` | The side's own material could never deliver mate. |

## `pawns`

Pawn structure.

| field | type | meaning |
| --- | --- | --- |
| `pawns` | `ByColour<SquareSet>` | The side's pawns. |
| `passed` | `ByColour<SquareSet>` | The side's pawns with no enemy pawn ahead on their own or an adjacent file. |
| `candidates` | `ByColour<SquareSet>` | The side's candidate passers: pawns with no enemy pawn ahead on their own file, backed by at least as many friendly pawns behind on the adjacent files as there are enemy pawns ahead on them. |
| `doubled` | `ByColour<SquareSet>` | The side's pawns sharing a file with another pawn of their own colour. |
| `isolated` | `ByColour<SquareSet>` | The side's pawns with no friendly pawn on either adjacent file. |
| `backward` | `ByColour<SquareSet>` | The side's pawns that cannot advance without being taken and cannot be defended by a pawn. |
| `defended` | `ByColour<SquareSet>` | The side's pawns defended by a pawn of their own colour. |
| `count_by_file` | `ByColour<[u8; 8]>` | The side's pawns per file, file a first. |
| `count_by_rank` | `ByColour<[u8; 8]>` | The side's pawns per rank counted from its own back rank, rank 1 first. |
| `open_files` | `FileSet` | Files carrying no pawn of either colour. |
| `semi_open_files` | `ByColour<FileSet>` | Files carrying no pawn of the side and at least one of the other. |
| `islands` | `ByColour<u8>` | The side's maximal runs of adjacent files carrying a pawn. |
| `levers` | `ByColour<u8>` | The side's pawns that can capture an enemy pawn. |
| `rams` | `u8` | Pawn pairs blocking each other head on. |
| `passer_lead_rank` | `ByColour<Option<u8>>` | The rank, counted from the side's own back rank, of its most advanced passer. |
| `passer_protected` | `ByColour<u8>` | The side's passers defended by a friendly pawn. |
| `passers_connected` | `ByColour<bool>` | The side has two passers on adjacent files. |
| `passer_unstoppable` | `ByColour<bool>` | The side has a passer the enemy king cannot catch. |
| `chain_max_length` | `ByColour<u8>` | The side's longest run of pawns each defending the next. |
| `chain_base_attacked` | `ByColour<bool>` | An enemy unit attacks the base of one of the side's chains of two or more. |
| `queenside_majority` | `ByColour<bool>` | The side has more pawns than the enemy on files a to d. |
| `kingside_majority` | `ByColour<bool>` | The side has more pawns than the enemy on files e to h. |
| `holes` | `ByColour<SquareSet>` | Squares on the side's ranks 3 to 6 that no pawn of the side can ever attack. |
| `holes_occupied` | `ByColour<u8>` | Enemy knights and bishops standing on those squares. |
| `fixed_pawns` | `ByColour<u8>` | The side's pawns whose stop square holds a unit of either colour. |
| `blocked_passers` | `ByColour<u8>` | The side's passers whose stop square holds an enemy unit. |
| `passer_distance` | `ByColour<Option<u8>>` | The pushes the side's lead passer still needs. |
| `passer_own_king_distance` | `ByColour<Option<u8>>` | The distance from the side's own king to its lead passer's promotion square. |
| `passer_enemy_king_distance` | `ByColour<Option<u8>>` | The distance from the enemy king to that same square. |
| `passer_in_square` | `ByColour<bool>` | The enemy king stands in the square of the side's lead passer. |
| `passer_free_path` | `ByColour<u8>` | The side's passers whose whole front span is empty. |
| `half_open_at_enemy_king` | `ByColour<u8>` | Files semi-open for the side among the enemy king's three shield files. |
| `backward_on_semi_open` | `ByColour<u8>` | The side's backward pawns on a file semi-open for the enemy. |

## `pieces`

Bishops, rooks, knights and queens.

| field | type | meaning |
| --- | --- | --- |
| `bishop_pair` | `ByColour<bool>` | The side holds bishops on both square colours. |
| `knight_pair` | `ByColour<bool>` | The side holds two or more knights. |
| `bishops_light` | `ByColour<u8>` | The side's bishops on light squares. |
| `bishops_dark` | `ByColour<u8>` | The side's bishops on dark squares. |
| `opposite_coloured_bishops` | `bool` | Exactly one bishop each, on different square colours. |
| `pawns_on_bishop_colour` | `ByColour<u8>` | The side's pawns standing on a square colour of one of its own bishops. |
| `rooks_connected_rank` | `ByColour<bool>` | Two of the side's rooks share a rank with nothing between. |
| `rooks_connected_file` | `ByColour<bool>` | Two of the side's rooks share a file with nothing between. |
| `rooks_on_open_file` | `ByColour<u8>` | The side's rooks on an open file. |
| `rooks_on_semi_open_file` | `ByColour<u8>` | The side's rooks on a file semi-open for their own side. |
| `rooks_on_seventh_rank` | `ByColour<u8>` | The side's rooks on its seventh rank, counted from its own back rank. |
| `rook_behind_own_passer` | `ByColour<u8>` | The side's rooks behind a passer of their own side. |
| `rook_behind_enemy_passer` | `ByColour<u8>` | The side's rooks behind an enemy passer. |
| `trapped_rook` | `ByColour<bool>` | The side has a rook shut in by its own king with no file to leave by. |
| `outposts` | `ByColour<SquareSet>` | The side's outpost squares: squares on its ranks 4 to 6 that a friendly pawn defends and no enemy pawn can ever attack. |
| `minors_on_outpost` | `ByColour<u8>` | The side's knights and bishops standing on one of them. |
| `outpost_squares_free` | `ByColour<u8>` | Its unoccupied outpost squares. |
| `knights_on_rim` | `ByColour<u8>` | The side's knights on file a or h, or on its rank 1 or 8. |
| `minors_undeveloped` | `ByColour<u8>` | The side's knights and bishops still on their starting squares. |
| `queen_developed` | `ByColour<bool>` | A queen of the side stands off its starting square. |
| `fixed_pawns_on_bishop_colour` | `ByColour<u8>` | The side's fixed pawns standing on a square colour of one of its own bishops. |
| `rook_on_seventh_with_king_on_eighth` | `ByColour<bool>` | The side has a rook on its seventh rank with the enemy king on its eighth. |
| `trapped_pieces` | `ByColour<u8>` | The side's units that are neither pawns nor kings and have no safe destination. |
| `trapped_value` | `ByColour<u8>` | The value sum of those. |

## `king`

King safety and shelter.

| field | type | meaning |
| --- | --- | --- |
| `square` | `ByColour<Square>` | Where the side's king stands. |
| `on_home_square` | `ByColour<bool>` | It stands on its own starting square. |
| `castled_queenside` | `ByColour<bool>` | It stands on files a to c. |
| `castled_kingside` | `ByColour<bool>` | It stands on files f to h. |
| `shield_files` | `ByColour<[File; 3]>` | The three files the king's shelter is read on, in ascending order. |
| `shield` | `ByColour<[Option<u8>; 3]>` | Ranks to the nearest friendly pawn ahead of the king, per shield file. |
| `shield_file_open` | `ByColour<[bool; 3]>` | Each shield file carries no pawn of either colour. |
| `shield_file_semi_open_for_enemy` | `ByColour<[bool; 3]>` | Each shield file is semi-open for the enemy of that king. |
| `storm` | `ByColour<[Option<u8>; 3]>` | Ranks to the nearest enemy pawn ahead of the king, per shield file. |
| `ring` | `ByColour<SquareSet>` | The squares adjacent to the king. |
| `ring_attackers` | `ByColour<u8>` | Enemy knights, bishops, rooks and queens attacking that ring. |
| `ring_attack_weight` | `ByColour<u8>` | Σ over those attackers of N, B = 1, R = 2, Q = 4. |
| `ring_defended` | `ByColour<u8>` | Ring squares the king's own side attacks, the king itself apart. |
| `ring_holes` | `ByColour<u8>` | Ring squares the enemy attacks and `ring_defended` does not count. |
| `escape_squares` | `ByColour<u8>` | Squares adjacent to the king that are empty or capturable and not attacked. |
| `back_rank_risk` | `ByColour<bool>` | The king stands on its rank 1 with every forward-adjacent square held by a friendly unit. |
| `distance` | `u8` | Chebyshev distance between the kings. |
| `tropism` | `ByColour<f32>` | Mean Chebyshev distance of the enemy's knights, bishops, rooks and queens to this king. |
| `virtual_mobility` | `ByColour<u8>` | Squares a queen on this king's square would attack. |
| `ring_defenders` | `ByColour<u8>` | The king's own knights, bishops, rooks and queens attacking its ring. |
| `ring_defence_weight` | `ByColour<u8>` | Σ over those defenders of N, B = 1, R = 2, Q = 4. |
| `open_rays` | `ByColour<u8>` | Directions from the king holding at least one square, all of them empty out to the edge. |
| `luft` | `ByColour<bool>` | The king stands on its rank 1 with a square ahead of it empty and unattacked. |
| `castled_side` | `ByColour<Option<CastledSide>>` | The side the king's square and its spent castling rights read as. |
| `opposite_side_castling` | `bool` | The two kings stand on opposite wings. |

## `mobility`

Mobility and space.

| field | type | meaning |
| --- | --- | --- |
| `pawns` | `ByColour<u16>` | Squares the side's pawns attack that its own units do not hold. |
| `knights` | `ByColour<u16>` | The same for its knights. |
| `bishops` | `ByColour<u16>` | The same for its bishops. |
| `rooks` | `ByColour<u16>` | The same for its rooks. |
| `queens` | `ByColour<u16>` | The same for its queens. |
| `safe_pawns` | `ByColour<u16>` | Of the pawn squares, those no enemy pawn attacks. |
| `safe_knights` | `ByColour<u16>` | The same for its knights. |
| `safe_bishops` | `ByColour<u16>` | The same for its bishops. |
| `safe_rooks` | `ByColour<u16>` | The same for its rooks. |
| `safe_queens` | `ByColour<u16>` | The same for its queens. |
| `total` | `ByColour<u16>` | The five counts added up. |
| `space` | `ByColour<u16>` | Squares the side attacks in the enemy's half. |
| `centre_control` | `ByColour<u8>` | The side's attacks on d4, e4, d5 and e5. |
| `extended_centre_control` | `ByColour<u8>` | The side's attacks on c3 to f6. |
| `immobile_pieces` | `ByColour<u8>` | The side's units other than pawns and kings with no destination. |

## `attacks`

The attack maps and what they say about the units standing on the board.

| field | type | meaning |
| --- | --- | --- |
| `controlled` | `ByColour<SquareSet>` | Every square the side attacks. |
| `by_pawns` | `ByColour<SquareSet>` | The squares the side's pawns attack. |
| `by_knights` | `ByColour<SquareSet>` | The squares its knights attack. |
| `by_bishops` | `ByColour<SquareSet>` | The squares its bishops attack. |
| `by_rooks` | `ByColour<SquareSet>` | The squares its rooks attack. |
| `by_queens` | `ByColour<SquareSet>` | The squares its queens attack. |
| `by_kings` | `ByColour<SquareSet>` | The squares its king attacks. |
| `attacked` | `ByColour<SquareSet>` | The side's units the enemy attacks, defended or not; never a king. |
| `hanging` | `ByColour<SquareSet>` | Those of them their own side does not defend. |
| `en_prise` | `ByColour<SquareSet>` | Those hanging or attacked by a cheaper enemy unit. |
| `pinned` | `ByColour<SquareSet>` | The side's units under an absolute pin. |
| `defended` | `ByColour<SquareSet>` | The side's units standing on a square their own side attacks. |
| `attacked_value` | `ByColour<i32>` | The value sum of the attacked units. |
| `hanging_value` | `ByColour<i32>` | The value sum of the hanging units. |
| `en_prise_value` | `ByColour<i32>` | The value sum of the units en prise. |
| `en_prise_max_value` | `ByColour<i32>` | The largest value en prise. |
| `pinned_value` | `ByColour<i32>` | The value sum of the absolutely pinned units. |
| `skewer_candidates` | `ByColour<u8>` | Enemy unit pairs the side's sliders skewer. |

## `exchange`

Each side's captures, judged by the exchange they start. The values for the side not to move are read after a null move; when the side to move is in check that move does not exist and they are zero.

| field | type | meaning |
| --- | --- | --- |
| `see_best_capture` | `ByColour<i32>` | The largest static exchange evaluation over the side's captures; 0 when it has none. |
| `see_positive_capture_count` | `ByColour<u16>` | Its captures whose evaluation is above 0. |
| `see_equal_capture_count` | `ByColour<u16>` | Its captures whose evaluation is 0. |
| `see_positive_total` | `ByColour<i32>` | The sum of the evaluations above 0. |

## `threats`

What each side stands to lose, and the slider geometry behind it. Every set is read on the units it is about, and holds no king.

| field | type | meaning |
| --- | --- | --- |
| `threatened` | `ByColour<SquareSet>` | The side's units the enemy wins material by taking. |
| `threatened_value` | `ByColour<i32>` | The value sum of those. |
| `threat_max_gain` | `ByColour<i32>` | The largest such gain over the side's own units. |
| `attacked_by_lesser` | `ByColour<SquareSet>` | The side's units an enemy unit of strictly lower value attacks. |
| `queen_attacked_by_lesser` | `ByColour<bool>` | One of those units is a queen. |
| `overloaded_defenders` | `ByColour<SquareSet>` | The side's sole defenders of two or more attacked friendly units. |
| `removable_defenders` | `ByColour<SquareSet>` | Its sole defenders the enemy can capture without loss. |
| `loose` | `ByColour<SquareSet>` | The side's units no unit of their own side defends. |
| `attacker_surplus` | `ByColour<SquareSet>` | The side's units more enemy units attack than its own defend. |
| `xray_through_enemy` | `ByColour<u8>` | The side's x-rays onto an enemy unit through one enemy unit. |
| `battery_count` | `ByColour<u8>` | The side's batteries: two of its own sliders on one line, agreeing in direction. |
| `battery_at_king` | `ByColour<bool>` | One of them bears on a square of the enemy king's ring. |

## `tactics`

Each side's one-ply tactical options. The values for the side not to move are read after a null move; when the side to move is in check that move does not exist and they are zero, which `available` reports.

| field | type | meaning |
| --- | --- | --- |
| `available` | `ByColour<bool>` | The side's values could be read. |
| `check_count` | `ByColour<u16>` | Its moves giving check. |
| `check_by_pawn` | `ByColour<bool>` | A pawn move of the side gives check. |
| `check_by_knight` | `ByColour<bool>` | A knight move gives check. |
| `check_by_bishop` | `ByColour<bool>` | A bishop move gives check. |
| `check_by_rook` | `ByColour<bool>` | A rook move gives check. |
| `check_by_queen` | `ByColour<bool>` | A queen move gives check. |
| `safe_check_count` | `ByColour<u16>` | Its checking moves whose destination is safe. |
| `safe_check_by_pawn` | `ByColour<bool>` | A pawn move of the side gives check from a safe square. |
| `safe_check_by_knight` | `ByColour<bool>` | A knight move does. |
| `safe_check_by_bishop` | `ByColour<bool>` | A bishop move does. |
| `safe_check_by_rook` | `ByColour<bool>` | A rook move does. |
| `safe_check_by_queen` | `ByColour<bool>` | A queen move does. |
| `double_check_available` | `ByColour<bool>` | A move gives check from two units at once. |
| `discovered_check_available` | `ByColour<bool>` | A move gives check with a unit that did not move. |
| `mate_in_1` | `ByColour<bool>` | A move leaves the opponent checkmated. |
| `stalemate_in_1` | `ByColour<bool>` | A move leaves the opponent stalemated. |
| `promotion_files` | `ByColour<FileSet>` | Files a legal move of the side promotes on. |
| `promotes_to_queen` | `ByColour<bool>` | A promotion to a queen is available. |
| `promotes_to_rook` | `ByColour<bool>` | One to a rook is. |
| `promotes_to_bishop` | `ByColour<bool>` | One to a bishop is. |
| `promotes_to_knight` | `ByColour<bool>` | One to a knight is. |
| `safe_promotion_files` | `ByColour<FileSet>` | Files a legal promotion with a safe destination lands on. |
| `capture_count` | `ByColour<u16>` | The side's capturing moves. |
| `winning_capture_available` | `ByColour<bool>` | One of them takes a unit that outvalues the capturer or is undefended. |
| `winning_capture_max_gain` | `ByColour<i32>` | The largest `victim − capturer` over the captures, at least 0. |
| `captures_hanging` | `ByColour<bool>` | A capture takes a hanging unit. |
| `hanging_victim_max_value` | `ByColour<i32>` | The largest value among the hanging units capturable now. |
| `equal_capture_count` | `ByColour<u16>` | Captures of a defended unit of equal value. |
| `losing_capture_count` | `ByColour<u16>` | Captures of a defended unit of lower value. |
| `fork_count` | `ByColour<u16>` | Moves after which the moved unit forks. |
| `fork_max_value` | `ByColour<i32>` | The largest single forked value. |
| `knight_fork_available` | `ByColour<bool>` | A fork by a knight. |
| `royal_fork_available` | `ByColour<bool>` | A fork one of whose targets is the king. |
| `pin_creation_count` | `ByColour<u16>` | Moves creating an absolute or a relative pin. |
| `skewer_creation_available` | `ByColour<bool>` | A move creating a skewer. |
| `discovered_attack_available` | `ByColour<bool>` | A move uncovering a slider's attack on a unit of value 3 or more. |
| `legal_move_count` | `ByColour<u16>` | The side's legal moves. |
| `safe_check_capturing` | `ByColour<bool>` | A checking move that captures and whose destination is safe. |
| `discovered_attack_on_queen` | `ByColour<bool>` | A move uncovering a slider's attack on the enemy queen. |
| `back_rank_mate_threat` | `ByColour<bool>` | A move after which a rook or queen of the side checks the enemy king along that king's own rank, its back rank being blocked. |
| `quiet_threat_available` | `ByColour<bool>` | A quiet move after which the largest gain the side threatens is greater than it is now. |
| `no_safe_moves` | `ByColour<bool>` | No legal move of the side has a safe destination. |
| `promotion_see_positive` | `ByColour<bool>` | A promotion of the side wins material. |

## `endgame`

Where the kings stand, how the pawn race runs, and which drawn configuration the material is.

| field | type | meaning |
| --- | --- | --- |
| `king_centralisation` | `ByColour<u8>` | Chebyshev distance from the side's king to the nearest central square. |
| `race_plies` | `ByColour<u8>` | Plies the side's most advanced passer needs to promote unopposed; 8 for a side with no passer. |
| `opposition` | `Option<Opposition>` | The opposition the kings stand in, if any. |
| `key_square_occupied` | `ByColour<bool>` | The side's king stands on a key square of one of its own passers. |
| `wrong_colour_bishop` | `ByColour<bool>` | The side's bishops all stand on the colour none of its pawns promotes on, every one of those pawns being a rook pawn. |
| `drawish_material` | `Option<DrawishMaterial>` | The drawn configuration the material is, if it is one of the three. |

## `maps`

The square sets a reader most often wants side by side: what each side attacks, and what of it stands hanging, pinned or threatened. Every one of them is also a field of the `attacks` or the `threats` group; this group is where they are read together.

| field | type | meaning |
| --- | --- | --- |
| `controlled` | `ByColour<SquareSet>` | Every square the side attacks. |
| `by_pawns` | `ByColour<SquareSet>` | The squares the side's pawns attack. |
| `hanging` | `ByColour<SquareSet>` | The side's units the enemy attacks and their own side does not defend. |
| `pinned` | `ByColour<SquareSet>` | The side's units under an absolute pin. |
| `threatened` | `ByColour<SquareSet>` | The side's units the enemy wins material by taking. |

## `move`

What one legal move does, beyond what the move itself says. The mover is the
side to move; the enemy is the other side. Every annotated move of
`facts.moves` carries these.

| field | type | meaning |
| --- | --- | --- |
| `victim` | `Option<Role>` | The role captured, the pawn for an en-passant capture. |
| `mover` | `Role` | The role that moves. |
| `promotion` | `Option<Role>` | The role a promoting pawn becomes. |
| `gives_check` | `bool` | The move gives check. |
| `gives_safe_check` | `bool` | The move gives check and its destination is safe. |
| `is_safe` | `bool` | The destination is a safe destination. |
| `captures_hanging` | `bool` | The move captures a hanging unit. |
| `escapes_attack` | `bool` | The origin is attacked by the enemy and the destination is safe. |
| `to_attacked_by_pawn` | `bool` | The destination is attacked by an enemy pawn. |
| `is_castling` | `bool` | The move is a castling. |
| `is_en_passant` | `bool` | The move is an en-passant capture. |
| `see` | `i32` | The static exchange evaluation of the move, signed. |
| `threat_created_max` | `i32` | The largest static exchange evaluation of one of the enemy's units after the move; 0 when the move threatens nothing. |
| `moves_attacked_unit` | `bool` | The origin is attacked by the enemy. |
| `blocks_check` | `bool` | The destination interposes on the ray of the one unit giving check. |
| `advances_passer` | `bool` | The moved unit is a passed pawn. |
| `creates_passer` | `bool` | The mover has more passers after the move than before. |
| `creates_isolated` | `bool` | The mover has more isolated pawns after the move than before. |
| `creates_doubled` | `bool` | The mover has more doubled pawns after the move than before. |
| `creates_backward` | `bool` | The mover has more backward pawns after the move than before. |
| `opens_file_at_enemy_king` | `bool` | One of the enemy king's shield files carried a pawn of the mover's and carries none after. |
| `enemy_king_attackers_delta` | `i32` | Change in the number of the mover's units attacking the enemy king ring. |
| `mover_king_attackers_delta` | `i32` | Change in the number of the enemy's units attacking the mover's king ring. |
| `mover_hanging_delta` | `i32` | Change in the number of the mover's hanging units. |
| `enemy_hanging_delta` | `i32` | Change in the number of the enemy's hanging units. |
| `leaves_unit_hanging` | `bool` | A square carries a hanging unit of the mover's worth 3 or more after the move and carried none before. |
| `gives_discovered_attack` | `bool` | A slider of the mover's that the move leaves standing gains an attack on an enemy unit worth 3 or more. |
