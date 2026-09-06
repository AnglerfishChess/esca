# Candidate facts

Facts that have been considered and are **not** implemented. What esca
answers today is [`features.md`](features.md), which is generated from the
groups themselves; this list is the review queue behind it.

Each row is the field a group would gain, the type it would carry, what it
would mean, and why it is still waiting. A candidate qualifies when a strong
player would name it, when it has one unambiguous definition, and when it
costs at most about one ply to compute.

Types read as in `features.md`: `ByColour<T>` is one `T` per colour, and a
plain type is one value for the position.

## `material`

| field | type | meaning | why not yet |
| --- | --- | --- | --- |
| `minor_kind_balance` | `i8` | White's bishops less its knights, less the same for Black. | A restatement of the four counts the group already carries. |

## `pawns`

| field | type | meaning | why not yet |
| --- | --- | --- | --- |
| `chain_count` | `ByColour<u8>` | The side's chains of two or more pawns, each defending the next. | `chain_max_length` names the structure; how many chains there are says less. |
| `duos` | `ByColour<u8>` | The side's pairs of pawns side by side on one rank. | Nearly the same set as `defended`. |
| `tension` | `u8` | Pairs of opposing pawns that each attack the other's square. | `levers` counts almost the same pairs. |
| `passers_by_rank` | `ByColour<[u8; 8]>` | The side's passers per rank, counted from its own back rank. | Eight values for what `passer_lead_rank` mostly says. |
| `passer_path_attacked` | `ByColour<u8>` | The side's passers whose front span the enemy attacks. | Overlaps `passer_free_path` and the king distances. |
| `open_file_at_enemy_king` | `ByColour<u8>` | Open files among the enemy king's three shield files. | `shield_file_open` already carries it file by file. |
| `isolated_on_semi_open` | `ByColour<u8>` | The side's isolated pawns on a file semi-open for the enemy. | A weaker signal than the backward case, which is carried. |

## `pieces`

| field | type | meaning | why not yet |
| --- | --- | --- | --- |
| `bishops_blocked` | `ByColour<u8>` | The side's bishops attacking three squares or fewer. | The `mobility` group says this for the whole role. |
| `fianchetto` | `ByColour<[bool; 2]>` | Per wing: a friendly pawn on b2 or g2 with the long diagonal from it clear, and a bishop standing there. | Four values for a pattern the shelter fields half-cover. |
| `rooks_doubled_on_open_file` | `ByColour<bool>` | Two connected rooks on one open or semi-open file. | `rooks_connected_file` and `rooks_on_open_file` nearly imply it. |
| `rooks_lifted` | `ByColour<u8>` | The side's rooks on its ranks 3 to 5 with no friendly pawn ahead on the file. | Common, but worth different amounts in different positions. |
| `outpost_occupant_defended` | `ByColour<u8>` | Outpost occupants a friendly pawn defends. | An outpost square is already one a friendly pawn attacks. |

## `mobility`

| field | type | meaning | why not yet |
| --- | --- | --- | --- |
| `value_weighted` | `ByColour<u16>` | Σ over the side's units of destinations × the unit's value. | Mixes two scales into one number. |

## `attacks`

| field | type | meaning | why not yet |
| --- | --- | --- | --- |
| `hanging_max_value` | `ByColour<i32>` | The largest value among the side's hanging units. | Would pair `en_prise_max_value`; the sets differ rarely. |
| `relative_pins` | `ByColour<u8>` | The side's units that may move but would expose a more valuable unit behind them. | Defined but not computed; the absolute pins are. |
| `defended_value` | `ByColour<i32>` | The value sum of the side's defended units. | The set is large and the number says little. |
| `xray_through_own` | `ByColour<u8>` | The side's x-rays onto an enemy unit through one of its own. | Largely `battery_count` seen from the other end. |

## `exchange`

| field | type | meaning | why not yet |
| --- | --- | --- | --- |
| `see_negative_capture_count` | `ByColour<u16>` | The side's captures that lose material. | The complement of the two counts already carried. |

## `threats`

| field | type | meaning | why not yet |
| --- | --- | --- | --- |
| `attacked_by_lesser_value` | `ByColour<i32>` | The value sum of the units a cheaper enemy unit attacks. | `threatened_value` covers the cases that matter. |
| `loose_max_value` | `ByColour<i32>` | The largest value among the side's undefended units. | Correlated with the count at the values that matter. |
| `attacker_surplus_max` | `ByColour<u8>` | The largest attacker surplus over the side's units. | Rarely above one in a real position. |

## `tactics`

| field | type | meaning | why not yet |
| --- | --- | --- | --- |
| `skewer_creation_count` | `ByColour<u16>` | Moves creating a skewer. | The flag rarely under-reports. |
| `quiet_threat_max_gain` | `ByColour<i32>` | The largest gain a quiet move would newly threaten. | The flag is most of the signal at a fraction of the cost. |
| `mate_threat_after_quiet` | `ByColour<bool>` | A quiet move after which the side mates next whatever the reply. | Two plies deep: past what a one-ply answer costs. |
| `no_piece_moves` | `ByColour<bool>` | The side has no legal move by a unit other than a pawn or the king. | A zugzwang proxy that fires mostly in endings. |
| `capture_of_defender_available` | `ByColour<bool>` | A capture whose victim is a sole defender the side can take without loss. | `removable_defenders` says it from the defender's side. |

## `endgame`

| field | type | meaning | why not yet |
| --- | --- | --- | --- |
| `rook_pawns_only` | `ByColour<bool>` | The side's only pawns stand on file a or h. | A component of `wrong_colour_bishop`, which is carried. |

## `history`

| field | type | meaning | why not yet |
| --- | --- | --- | --- |
| `last_move_was_check` | `bool` | The last move of the supplied history gave check. | `in_check` says the side to move stands in check, which is the same thing one ply on. |
| `last_move_was_capture` | `bool` | The last move captured. | Implied by `last_move_victim`. |
| `repetition_count` | `u8` | How often this position occurred before in the supplied history. | Refines a flag that is rarely set at all. |
| `plies_played` | `u32` | Plies since the start position. | `phase_points` proxies it, and a position read from a FEN has no history. |

## `maps`

| field | type | meaning | why not yet |
| --- | --- | --- | --- |
| `holes` | `ByColour<SquareSet>` | The squares no pawn of the side can ever attack. | The `pawns` group already carries the sets; the group would only gather them. |
| `see_loss` | — | Per unit, what the exchange on its square would cost its owner. | A value per square is a lot of data for a rare signal. |

## `move`

| field | type | meaning | why not yet |
| --- | --- | --- | --- |
| `see_positive` | `bool` | The move wins material. | A threshold on `see`, which the move already carries. |
| `moves_hanging_unit` | `bool` | The moved unit was hanging before the move. | A conjunction of two facts already present. |
| `develops` | `bool` | A knight or bishop leaves its starting square. | Opening-only, and undefined off classic chess. |
| `passer_new_distance` | `Option<u8>` | The pushes the moved passer would still need. | Read off the destination square. |
| `closes_file_at_own_king` | `bool` | A shield file of the mover's king stops being open for the enemy. | The defensive mirror of `opens_file_at_enemy_king`, and rarer. |
| `mobility_delta` | `i32` | The change in the mover's total mobility. | Expensive for a diffuse signal. |
| `is_recapture` | `bool` | The destination is the square the previous move ended on. | Needs a history, so a position read from a FEN could not answer it. |
