//! The `pawns` group, fact by fact.
//!
//! Every expectation is worked out from the definitions in `docs/features.md`
//! §1 for the named position above it.

mod common;

use common::{facts_of, facts_under, files, squares};
use esca::{CHESS960, Side};
use rstest::rstest;

/// The untouched array: one island a side and nothing else true of it.
const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// Black to move: doubled f-pawns against doubled c-pawns, over two open files.
const WINGS: &str = "4k3/p5pp/5p2/5p2/2P5/2P5/PP5P/4K3 b - - 0 1";

/// Three pawns on one file against two on another, both files free of enemies.
const TRIPLED: &str = "4k3/1p6/1p6/8/3P4/3P4/3P4/4K3 w - - 0 1";

/// e3 and e6 each stand still: their stop square is covered by an enemy pawn.
const BACKWARD: &str = "4k3/8/4p3/3p1p2/3P1P2/4P3/8/4K3 w - - 0 1";

/// Scattered pawns: two backward ones a side, and a three-island black wing.
const SPLIT: &str = "4k3/1p6/8/P6p/5p2/7P/6P1/4K3 w - - 0 1";

/// c4 and f5 each head a majority on a file the enemy has left.
const CANDIDATES: &str = "4k3/1p6/8/5pp1/1PP5/8/6P1/4K3 w - - 0 1";

/// A locked centre: four rams, with pawns in contact on both sides of it.
const LOCKED: &str = "4k3/8/4p3/1pppPp2/2PP1P2/8/8/4K3 w - - 0 1";

/// Doubled c-pawns against doubled f-pawns; d4 and e6 each head a majority.
const MAJORITIES: &str = "4k3/pp3p1p/4pp2/8/3P4/2P5/PPP2PPP/4K3 w - - 0 1";

/// A wedge of three rams, every pawn of it in contact with an enemy pawn.
const WEDGE: &str = "4k3/8/2pp4/3Ppp2/4PP2/8/8/4K3 w - - 0 1";

/// One pawn against two that both attack it, and it both of them.
const CONTACT: &str = "4k3/8/8/3p1p2/4P3/8/8/4K3 w - - 0 1";

/// Two connected passers a side, the rear pawn of each pair defending the front.
const PASSERS: &str = "4k3/5p2/6p1/2P5/1P6/8/8/4K3 w - - 0 1";

/// Two chains of three connected passers; d6 is out of the black king's square.
const PHALANX: &str = "k7/8/3P4/2P3p1/1P3p2/4p3/8/7K w - - 0 1";

/// a7 queens: the black king is outside the square and nothing else can help.
const RUNAWAY: &str = "8/P6k/8/8/8/8/6K1/8 w - - 0 1";

/// The same runaway one tempo later: it is theirs, and the tempo saves nothing.
const RUNAWAY_THEIRS: &str = "8/P6k/8/8/8/8/6K1/8 b - - 0 1";

/// The same locked centre with Black to move: every side-paired value swaps.
const LOCKED_THEIRS: &str = "4k3/8/4p3/1pppPp2/2PP1P2/8/8/4K3 b - - 0 1";

/// A knight and a bishop each hold a square the other side's pawns have left.
const HOLES: &str = "4k3/pp3ppp/2pB4/8/2PP4/3n4/PP3PPP/6K1 w - - 0 1";

/// Chains of two against a chain of three, both bases under attack.
const CHAINS: &str = "4k3/b4p2/4p3/3pP1N1/3P1P2/8/8/4K3 w - - 0 1";

/// Passers blockaded by minor pieces that also stand on holes.
const BLOCKADE: &str = "4k3/8/1n6/1P1b1p2/3P1N2/8/8/4K3 w - - 0 1";

/// Both kings castled short, with files left open in front of each.
const CASTLED: &str = "6k1/pp3pp1/8/8/4P3/8/PPP4P/6K1 w - - 0 1";

/// Three backward pawns, each on a file its opponent has left.
const WEAK: &str = "4k3/8/8/8/1p1p4/8/2P3P1/4K3 w - - 0 1";

/// Two passers on one rank: b5 leads, being the nearer to file a.
const TWIN_PASSERS: &str = "6k1/3p4/3K4/1P4P1/8/8/8/8 w - - 0 1";

/// A Chess960 middlegame; no pawn fact reads the back rank, so nothing moves.
const NINE_SIXTY: &str = "nnqrkr1b/1p1pp3/3p4/p4ppp/PP5P/1b2PP2/2PPK1P1/N1QR1RBB w fd - 0 10";

#[rstest]
#[case::start(START, "a2 b2 c2 d2 e2 f2 g2 h2", "a7 b7 c7 d7 e7 f7 g7 h7")]
#[case::wings(WINGS, "a7 f5 f6 g7 h7", "a2 b2 c3 c4 h2")]
#[case::tripled(TRIPLED, "d2 d3 d4", "b6 b7")]
#[case::locked(LOCKED, "c4 d4 e5 f4", "b5 c5 d5 e6 f5")]
#[case::majorities(MAJORITIES, "a2 b2 c2 c3 d4 f2 g2 h2", "a7 b7 e6 f6 f7 h7")]
#[case::runaway(RUNAWAY, "a7", "")]
#[case::runaway_theirs(RUNAWAY_THEIRS, "", "a7")]
fn the_pawns_of_each_side_are_listed_us_first(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let facts = facts_of(fen);
    assert_eq!(facts.pawns.pawns[Side::Us.index()], squares(us));
    assert_eq!(facts.pawns.pawns[Side::Them.index()], squares(them));
}

#[rstest]
#[case::start(START, "", "")]
#[case::wings(WINGS, "f5 f6", "c3 c4")]
#[case::tripled(TRIPLED, "d2 d3 d4", "b6 b7")]
#[case::locked(LOCKED, "", "")]
#[case::passers(PASSERS, "b4 c5", "f7 g6")]
#[case::phalanx(PHALANX, "b4 c5 d6", "e3 f4 g5")]
#[case::runaway(RUNAWAY, "a7", "")]
#[case::runaway_theirs(RUNAWAY_THEIRS, "", "a7")]
fn a_passer_has_no_enemy_pawn_ahead_on_its_own_or_a_neighbouring_file(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let facts = facts_of(fen);
    assert_eq!(facts.pawns.passed[Side::Us.index()], squares(us));
    assert_eq!(facts.pawns.passed[Side::Them.index()], squares(them));
}

#[rstest]
#[case::start(START, "", "")]
#[case::wings(WINGS, "g7", "b2")]
#[case::candidates(CANDIDATES, "c4", "f5")]
#[case::majorities(MAJORITIES, "c2 c3 d4", "e6")]
#[case::wedge(WEDGE, "", "c6")]
#[case::locked(LOCKED, "", "b5")]
#[case::passers(PASSERS, "", "")]
#[case::split(SPLIT, "", "")]
fn a_candidate_has_a_free_file_ahead_and_support_enough_to_use_it(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let facts = facts_of(fen);
    assert_eq!(facts.pawns.candidates[Side::Us.index()], squares(us));
    assert_eq!(facts.pawns.candidates[Side::Them.index()], squares(them));
}

#[rstest]
#[case::start(START, "", "")]
#[case::wings(WINGS, "f5 f6", "c3 c4")]
#[case::tripled(TRIPLED, "d2 d3 d4", "b6 b7")]
#[case::locked(LOCKED, "", "")]
#[case::split(SPLIT, "", "")]
#[case::majorities(MAJORITIES, "c2 c3", "f6 f7")]
fn every_pawn_of_a_shared_file_is_doubled(#[case] fen: &str, #[case] us: &str, #[case] them: &str) {
    let facts = facts_of(fen);
    assert_eq!(facts.pawns.doubled[Side::Us.index()], squares(us));
    assert_eq!(facts.pawns.doubled[Side::Them.index()], squares(them));
}

#[rstest]
#[case::start(START, "", "")]
#[case::wings(WINGS, "a7", "h2")]
#[case::tripled(TRIPLED, "d2 d3 d4", "b6 b7")]
#[case::split(SPLIT, "a5", "b7 f4 h5")]
#[case::contact(CONTACT, "e4", "d5 f5")]
#[case::runaway(RUNAWAY, "a7", "")]
fn an_isolated_pawn_has_no_friendly_pawn_on_either_neighbouring_file(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let facts = facts_of(fen);
    assert_eq!(facts.pawns.isolated[Side::Us.index()], squares(us));
    assert_eq!(facts.pawns.isolated[Side::Them.index()], squares(them));
}

#[rstest]
#[case::start(START, "", "")]
#[case::backward(BACKWARD, "e3", "e6")]
#[case::split(SPLIT, "a5 g2", "b7 f4")]
#[case::locked(LOCKED, "f4", "e6")]
#[case::passers(PASSERS, "", "")]
fn a_backward_pawn_is_unsupported_and_its_stop_square_is_held(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let facts = facts_of(fen);
    assert_eq!(facts.pawns.backward[Side::Us.index()], squares(us));
    assert_eq!(facts.pawns.backward[Side::Them.index()], squares(them));
}

#[rstest]
#[case::start(START, "", "")]
#[case::wings(WINGS, "f6", "c3")]
#[case::backward(BACKWARD, "d4 f4", "d5 f5")]
#[case::split(SPLIT, "h3", "")]
#[case::phalanx(PHALANX, "c5 d6", "e3 f4")]
#[case::majorities(MAJORITIES, "c3 d4", "e6")]
#[case::wedge(WEDGE, "d5", "e5")]
fn a_defended_pawn_stands_where_its_own_pawns_attack(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let facts = facts_of(fen);
    assert_eq!(facts.pawns.defended[Side::Us.index()], squares(us));
    assert_eq!(facts.pawns.defended[Side::Them.index()], squares(them));
}

#[rstest]
#[case::start(START, [1; 8], [1; 8])]
#[case::wings(WINGS, [1, 0, 0, 0, 0, 2, 1, 1], [1, 1, 2, 0, 0, 0, 0, 1])]
#[case::tripled(TRIPLED, [0, 0, 0, 3, 0, 0, 0, 0], [0, 2, 0, 0, 0, 0, 0, 0])]
#[case::locked(LOCKED, [0, 0, 1, 1, 1, 1, 0, 0], [0, 1, 1, 1, 1, 1, 0, 0])]
#[case::majorities(MAJORITIES, [1, 1, 2, 1, 0, 1, 1, 1], [1, 1, 0, 0, 1, 2, 0, 1])]
#[case::runaway(RUNAWAY, [1, 0, 0, 0, 0, 0, 0, 0], [0; 8])]
fn pawns_are_counted_by_file_from_a(#[case] fen: &str, #[case] us: [u8; 8], #[case] them: [u8; 8]) {
    let facts = facts_of(fen);
    assert_eq!(facts.pawns.count_by_file[Side::Us.index()], us);
    assert_eq!(facts.pawns.count_by_file[Side::Them.index()], them);
}

#[rstest]
#[case::start(START, [0, 8, 0, 0, 0, 0, 0, 0], [0, 8, 0, 0, 0, 0, 0, 0])]
#[case::split(SPLIT, [0, 1, 1, 0, 1, 0, 0, 0], [0, 1, 0, 1, 1, 0, 0, 0])]
#[case::locked(LOCKED, [0, 0, 0, 3, 1, 0, 0, 0], [0, 0, 1, 4, 0, 0, 0, 0])]
#[case::passers(PASSERS, [0, 0, 0, 1, 1, 0, 0, 0], [0, 1, 1, 0, 0, 0, 0, 0])]
#[case::majorities(MAJORITIES, [0, 6, 1, 1, 0, 0, 0, 0], [0, 4, 2, 0, 0, 0, 0, 0])]
#[case::runaway(RUNAWAY, [0, 0, 0, 0, 0, 0, 1, 0], [0; 8])]
fn pawns_are_counted_by_the_rank_their_owner_reads(
    #[case] fen: &str,
    #[case] us: [u8; 8],
    #[case] them: [u8; 8],
) {
    let facts = facts_of(fen);
    assert_eq!(facts.pawns.count_by_rank[Side::Us.index()], us);
    assert_eq!(facts.pawns.count_by_rank[Side::Them.index()], them);
}

#[rstest]
#[case::start(START, "")]
#[case::wings(WINGS, "de")]
#[case::tripled(TRIPLED, "acefgh")]
#[case::locked(LOCKED, "agh")]
#[case::majorities(MAJORITIES, "")]
#[case::wedge(WEDGE, "abgh")]
#[case::runaway(RUNAWAY, "bcdefgh")]
fn an_open_file_carries_no_pawn_of_either_colour(#[case] fen: &str, #[case] open: &str) {
    assert_eq!(facts_of(fen).pawns.open_files, files(open));
}

#[rstest]
#[case::start(START, "", "")]
#[case::wings(WINGS, "bc", "fg")]
#[case::split(SPLIT, "bf", "ag")]
#[case::contact(CONTACT, "df", "e")]
#[case::majorities(MAJORITIES, "e", "cdg")]
#[case::wedge(WEDGE, "c", "")]
#[case::runaway(RUNAWAY, "", "a")]
#[case::runaway_theirs(RUNAWAY_THEIRS, "a", "")]
fn a_file_is_semi_open_for_the_side_that_has_left_it(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let facts = facts_of(fen);
    assert_eq!(facts.pawns.semi_open_files[Side::Us.index()], files(us));
    assert_eq!(facts.pawns.semi_open_files[Side::Them.index()], files(them));
}

#[rstest]
#[case::start(START, [1, 1])]
#[case::wings(WINGS, [2, 2])]
#[case::split(SPLIT, [2, 3])]
#[case::contact(CONTACT, [1, 2])]
#[case::majorities(MAJORITIES, [2, 3])]
#[case::runaway(RUNAWAY, [1, 0])]
#[case::runaway_theirs(RUNAWAY_THEIRS, [0, 1])]
fn an_island_is_a_maximal_run_of_files_carrying_a_pawn(
    #[case] fen: &str,
    #[case] islands: [u8; 2],
) {
    assert_eq!(facts_of(fen).pawns.islands, islands);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::split(SPLIT, [0, 0])]
#[case::contact(CONTACT, [1, 2])]
#[case::locked(LOCKED, [2, 3])]
#[case::wedge(WEDGE, [3, 3])]
#[case::wings(WINGS, [0, 0])]
fn a_lever_is_a_pawn_whose_attacks_reach_an_enemy_pawn(#[case] fen: &str, #[case] levers: [u8; 2]) {
    assert_eq!(facts_of(fen).pawns.levers, levers);
}

#[rstest]
#[case::start(START, 0)]
#[case::contact(CONTACT, 0)]
#[case::majorities(MAJORITIES, 0)]
#[case::backward(BACKWARD, 2)]
#[case::wedge(WEDGE, 3)]
#[case::locked(LOCKED, 4)]
fn a_ram_is_a_pawn_pair_blocking_each_other_head_on(#[case] fen: &str, #[case] rams: u8) {
    assert_eq!(facts_of(fen).pawns.rams, rams);
}

#[rstest]
#[case::start(START, [None, None])]
#[case::wings(WINGS, [Some(4), Some(4)])]
#[case::tripled(TRIPLED, [Some(4), Some(3)])]
#[case::passers(PASSERS, [Some(5), Some(3)])]
#[case::phalanx(PHALANX, [Some(6), Some(6)])]
#[case::runaway(RUNAWAY, [Some(7), None])]
#[case::runaway_theirs(RUNAWAY_THEIRS, [None, Some(7)])]
fn the_lead_rank_is_how_far_the_furthest_passer_has_come(
    #[case] fen: &str,
    #[case] lead: [Option<u8>; 2],
) {
    assert_eq!(facts_of(fen).pawns.passer_lead_rank, lead);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::tripled(TRIPLED, [0, 0])]
#[case::wings(WINGS, [1, 1])]
#[case::passers(PASSERS, [1, 1])]
#[case::phalanx(PHALANX, [2, 2])]
fn a_protected_passer_stands_on_a_square_a_friendly_pawn_attacks(
    #[case] fen: &str,
    #[case] protected: [u8; 2],
) {
    assert_eq!(facts_of(fen).pawns.passer_protected, protected);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::tripled(TRIPLED, [false, false])]
#[case::wings(WINGS, [false, false])]
#[case::passers(PASSERS, [true, true])]
#[case::phalanx(PHALANX, [true, true])]
fn passers_are_connected_when_two_of_them_stand_on_neighbouring_files(
    #[case] fen: &str,
    #[case] connected: [bool; 2],
) {
    assert_eq!(facts_of(fen).pawns.passers_connected, connected);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::tripled(TRIPLED, [false, false])]
#[case::passers(PASSERS, [false, false])]
#[case::phalanx(PHALANX, [true, false])]
#[case::runaway(RUNAWAY, [true, false])]
#[case::runaway_theirs(RUNAWAY_THEIRS, [false, true])]
fn an_unstoppable_passer_beats_the_defending_king_to_its_promotion_square(
    #[case] fen: &str,
    #[case] unstoppable: [bool; 2],
) {
    assert_eq!(facts_of(fen).pawns.passer_unstoppable, unstoppable);
}

#[rstest]
#[case::start(START, [1, 1])]
#[case::runaway(RUNAWAY, [1, 0])]
#[case::holes(HOLES, [1, 2])]
#[case::chains(CHAINS, [2, 3])]
#[case::majorities(MAJORITIES, [3, 2])]
#[case::phalanx(PHALANX, [3, 3])]
fn the_longest_chain_is_the_longest_run_of_pawns_each_defending_the_next(
    #[case] fen: &str,
    #[case] length: [u8; 2],
) {
    assert_eq!(facts_of(fen).pawns.chain_max_length, length);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::phalanx(PHALANX, [false, false])]
#[case::locked(LOCKED, [true, false])]
#[case::locked_theirs(LOCKED_THEIRS, [false, true])]
#[case::wedge(WEDGE, [true, false])]
#[case::chains(CHAINS, [true, true])]
fn a_chain_base_is_attacked_when_an_enemy_unit_bears_on_its_rearmost_pawn(
    #[case] fen: &str,
    #[case] attacked: [bool; 2],
) {
    assert_eq!(facts_of(fen).pawns.chain_base_attacked, attacked);
}

#[rstest]
#[case::start(START, [[false, false], [false, false]])]
#[case::tripled(TRIPLED, [[true, false], [false, false]])]
#[case::contact(CONTACT, [[false, false], [true, false]])]
#[case::majorities(MAJORITIES, [[true, false], [false, true]])]
#[case::wings(WINGS, [[false, true], [true, false]])]
#[case::weak(WEAK, [[false, true], [true, false]])]
fn a_majority_is_more_own_pawns_than_enemy_pawns_on_a_wing(
    #[case] fen: &str,
    #[case] majority: [[bool; 2]; 2],
) {
    assert_eq!(facts_of(fen).pawns.majority_by_wing, majority);
}

#[rstest]
#[case::start(START, "", "")]
#[case::majorities(MAJORITIES, "", "d6 f6 h3 h4 h5 h6")]
#[case::holes(HOLES, "d3 d4", "d6")]
#[case::castled(CASTLED, "e3 e4 e5 e6 f3 f4 h3 h4 h5 h6", "d3 d4 d5 d6")]
fn a_hole_is_a_square_no_pawn_of_the_side_can_ever_attack(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let facts = facts_of(fen);
    assert_eq!(facts.pawns.holes[Side::Us.index()], squares(us));
    assert_eq!(facts.pawns.holes[Side::Them.index()], squares(them));
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::majorities(MAJORITIES, [0, 6])]
#[case::castled(CASTLED, [10, 4])]
#[case::wings(WINGS, [13, 13])]
#[case::blockade(BLOCKADE, [27, 28])]
fn holes_are_counted_over_the_four_ranks_the_definition_names(
    #[case] fen: &str,
    #[case] holes: [u32; 2],
) {
    let facts = facts_of(fen);
    assert_eq!(
        facts.pawns.holes.map(|set| set.len()),
        holes,
        "the encoding counts what the sets hold"
    );
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::castled(CASTLED, [0, 0])]
#[case::majorities(MAJORITIES, [0, 0])]
#[case::holes(HOLES, [1, 1])]
#[case::blockade(BLOCKADE, [2, 1])]
fn a_hole_is_occupied_when_an_enemy_knight_or_bishop_stands_on_it(
    #[case] fen: &str,
    #[case] occupied: [u8; 2],
) {
    assert_eq!(facts_of(fen).pawns.holes_occupied, occupied);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::majorities(MAJORITIES, [1, 1])]
#[case::blockade(BLOCKADE, [2, 1])]
#[case::tripled(TRIPLED, [2, 1])]
#[case::wedge(WEDGE, [3, 3])]
#[case::locked(LOCKED, [4, 4])]
fn a_fixed_pawn_has_a_unit_of_either_colour_on_its_stop_square(
    #[case] fen: &str,
    #[case] fixed: [u8; 2],
) {
    assert_eq!(facts_of(fen).pawns.fixed_pawns, fixed);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::passers(PASSERS, [0, 0])]
#[case::phalanx(PHALANX, [0, 0])]
#[case::twin_passers(TWIN_PASSERS, [0, 1])]
#[case::blockade(BLOCKADE, [2, 1])]
fn a_blocked_passer_has_an_enemy_unit_on_its_stop_square(
    #[case] fen: &str,
    #[case] blocked: [u8; 2],
) {
    assert_eq!(facts_of(fen).pawns.blocked_passers, blocked);
}

#[rstest]
#[case::start(START, [None, None])]
#[case::runaway(RUNAWAY, [Some(1), None])]
#[case::phalanx(PHALANX, [Some(2), Some(2)])]
#[case::passers(PASSERS, [Some(3), Some(5)])]
#[case::twin_passers(TWIN_PASSERS, [Some(3), Some(6)])]
#[case::tripled(TRIPLED, [Some(4), Some(5)])]
fn the_passer_distance_is_what_the_lead_passer_still_has_to_push(
    #[case] fen: &str,
    #[case] distance: [Option<u8>; 2],
) {
    assert_eq!(facts_of(fen).pawns.passer_distance, distance);
}

#[rstest]
#[case::start(START, [[None, None], [None, None]])]
#[case::runaway(RUNAWAY, [[Some(6), Some(7)], [None, None]])]
#[case::passers(PASSERS, [[Some(7), Some(2)], [Some(7), Some(2)]])]
#[case::phalanx(PHALANX, [[Some(7), Some(3)], [Some(7), Some(3)]])]
#[case::blockade(BLOCKADE, [[Some(7), Some(3)], [Some(7), Some(1)]])]
#[case::twin_passers(TWIN_PASSERS, [[Some(2), Some(5)], [Some(7), Some(5)]])]
fn both_kings_are_measured_to_the_lead_passers_promotion_square(
    #[case] fen: &str,
    #[case] distance: [[Option<u8>; 2]; 2],
) {
    assert_eq!(facts_of(fen).pawns.passer_king_distance, distance);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::runaway(RUNAWAY, [false, false])]
#[case::runaway_theirs(RUNAWAY_THEIRS, [false, false])]
#[case::phalanx(PHALANX, [false, true])]
#[case::twin_passers(TWIN_PASSERS, [false, true])]
#[case::passers(PASSERS, [true, true])]
#[case::blockade(BLOCKADE, [true, true])]
fn a_defending_king_in_the_square_catches_the_lead_passer(
    #[case] fen: &str,
    #[case] caught: [bool; 2],
) {
    assert_eq!(facts_of(fen).pawns.passer_in_square, caught);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::blockade(BLOCKADE, [0, 0])]
#[case::runaway(RUNAWAY, [1, 0])]
#[case::twin_passers(TWIN_PASSERS, [1, 0])]
#[case::tripled(TRIPLED, [1, 1])]
#[case::passers(PASSERS, [2, 2])]
#[case::phalanx(PHALANX, [3, 3])]
fn a_free_path_is_a_passer_with_nothing_at_all_ahead_of_it(
    #[case] fen: &str,
    #[case] free: [u8; 2],
) {
    assert_eq!(facts_of(fen).pawns.passer_free_path, free);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::wings(WINGS, [0, 1])]
#[case::split(SPLIT, [1, 0])]
#[case::majorities(MAJORITIES, [1, 1])]
#[case::contact(CONTACT, [2, 1])]
#[case::castled(CASTLED, [2, 1])]
fn a_file_aimed_at_the_enemy_king_is_one_semi_open_for_us_among_its_three(
    #[case] fen: &str,
    #[case] aimed: [u8; 2],
) {
    assert_eq!(facts_of(fen).pawns.half_open_at_enemy_king, aimed);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::backward(BACKWARD, [0, 0])]
#[case::locked(LOCKED, [0, 0])]
#[case::weak(WEAK, [1, 2])]
#[case::split(SPLIT, [2, 2])]
fn a_backward_pawn_counts_again_on_a_file_the_enemy_has_left(
    #[case] fen: &str,
    #[case] weak: [u8; 2],
) {
    assert_eq!(facts_of(fen).pawns.backward_on_semi_open, weak);
}

/// No `pawns` fact is among the four `features.md` §4 defines for classic chess
/// only, so a Chess960 position answers exactly as the same placement would.
#[test]
fn the_pawn_facts_of_a_chess960_position_are_the_classic_ones() {
    let facts = facts_under(&CHESS960, NINE_SIXTY);
    let pawns = &facts.pawns;
    assert_eq!(
        pawns.pawns[Side::Us.index()],
        squares("a4 b4 c2 d2 e3 f3 g2 h4")
    );
    assert_eq!(
        pawns.pawns[Side::Them.index()],
        squares("a5 b7 d6 d7 e7 f5 g5 h5")
    );
    assert_eq!(pawns.doubled[Side::Them.index()], squares("d6 d7"));
    assert_eq!(pawns.defended[Side::Us.index()], squares("e3 f3"));
    assert_eq!(pawns.defended[Side::Them.index()], squares("d6"));
    assert_eq!(
        pawns.count_by_file[Side::Them.index()],
        [1, 1, 0, 2, 1, 1, 1, 1]
    );
    assert_eq!(
        pawns.count_by_rank[Side::Us.index()],
        [0, 3, 2, 3, 0, 0, 0, 0]
    );
    assert_eq!(pawns.islands, [1, 2]);
    assert_eq!(pawns.levers, [2, 2]);
    assert_eq!(pawns.rams, 2);
    assert_eq!(pawns.semi_open_files[Side::Them.index()], files("c"));
    assert!(pawns.open_files.is_empty());
    assert_eq!(pawns.chain_max_length, [2, 2]);
    assert_eq!(pawns.holes[Side::Us.index()], squares("a3 a4 g3"));
    assert_eq!(
        pawns.holes[Side::Them.index()],
        squares("b5 b6 g5 g6 h5 h6")
    );
    assert_eq!(pawns.fixed_pawns, [2, 3]);
    assert_eq!(pawns.majority_by_wing, [[false, false], [false, false]]);

    let classic = facts_of("nnqrkr1b/1p1pp3/3p4/p4ppp/PP5P/1b2PP2/2PPK1P1/N1QR1RBB w - - 0 10");
    assert_eq!(classic.pawns, *pawns);
}
