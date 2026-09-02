//! The `tactics` group, fact by fact.
//!
//! Every expectation is worked out from the definitions in `docs/features.md`
//! §1 for the named position above it. The `them` block of a position is the
//! `us` block of the same placement with the other side to move, which is what
//! the null move of `features.md` §1 makes it.

mod common;

use common::{facts_of, facts_under, files};
use esca::{CHESS960, Side, TacticsFacts};
use rstest::rstest;

/// The untouched array: twenty moves a side, and not a tactic among them.
const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// White's king has one square left, and Black has a move that stalemates him.
const ONLY_MOVE: &str = "8/8/8/8/8/p1k5/P7/K7 w - - 0 1";

/// White stands in check, so the null move their block needs does not exist.
const IN_CHECK: &str = "4k3/8/8/8/8/8/4r3/4K3 w - - 0 1";

/// Black is already without a move; most of White's moves leave it that way.
const STALEMATE: &str = "7k/5K2/8/8/8/8/8/1B6 w - - 0 1";

/// Ra8 mates: the eighth rank is sealed and the black rook cannot come back.
const MATE: &str = "6k1/5ppp/8/8/8/7r/5PPP/R5K1 w - - 0 1";

/// The same board a tempo later, so the mate is the one they have.
const MATE_THEIRS: &str = "6k1/5ppp/8/8/8/7r/5PPP/R5K1 b - - 0 1";

/// Knight and rook checks a side; two of White's fork the king and a rook.
const CHECKS: &str = "4k3/8/8/5N1r/1n5R/8/8/4K3 w - - 0 1";

/// The same board with Black to move, so each block changes sides.
const CHECKS_THEIRS: &str = "4k3/8/8/5N1r/1n5R/8/8/4K3 b - - 0 1";

/// Two promotions a side, one of each guarded by the enemy knight.
const PROMOTIONS: &str = "8/1P3P2/n6k/8/7K/4N3/1p3p2/8 w - - 0 1";

/// Each side may promote by pushing or by taking the rook that stands in reach.
const PROMOTION_CAPTURES: &str = "r7/1P6/8/8/2K4k/8/6p1/5R2 w - - 0 1";

/// A knight fork for White and a rook fork for Black, both on loose pieces.
const FORKS: &str = "2r3k1/5r2/8/8/4N3/8/7K/1R3N2 w - - 0 1";

/// The rook's file pins the knight to the queen; taking it skewers her instead.
const PINS: &str = "3r4/3q4/7k/3n4/8/8/1B6/3R3K w - - 0 1";

/// The same board with Black to move.
const PINS_THEIRS: &str = "3r4/3q4/7k/3n4/8/8/1B6/3R3K b - - 0 1";

/// Every knight move uncovers the rook's check, and two of them check twice.
const DISCOVERY: &str = "r3k3/7p/8/6n1/4N3/8/3N4/2B1R2K w - - 0 1";

/// The same board with Black to move.
const DISCOVERY_THEIRS: &str = "r3k3/7p/8/6n1/4N3/8/3N4/2B1R2K b - - 0 1";

/// Chess960: castling long lands the king on c1 and the rook on d1, in check.
const NINE_SIXTY: &str = "3k3r/8/8/8/8/8/8/RK6 w A - 0 1";

/// The two blocks of `fen` under classic chess, us first.
fn blocks(fen: &str) -> [TacticsFacts; 2] {
    facts_of(fen).tactics
}

/// The five counted roles, named by their letters in the order the schema
/// writes them: `roles("nr")` is a knight and a rook.
fn roles(letters: &str) -> [bool; 5] {
    let mut set = [false; 5];
    for letter in letters.chars() {
        set["pnbrq".find(letter).expect("a counted role letter")] = true;
    }
    set
}

#[rstest]
#[case::start(START, [false, false])]
#[case::mate(MATE, [true, false])]
#[case::mate_theirs(MATE_THEIRS, [false, true])]
#[case::checks(CHECKS, [true, true])]
#[case::discovery(DISCOVERY, [true, false])]
fn a_check_is_available_when_a_legal_move_leaves_the_enemy_king_attacked(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].check_available(), available[0]);
    assert_eq!(tactics[Side::Them.index()].check_available(), available[1]);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::mate(MATE, [1, 0])]
#[case::checks(CHECKS, [3, 2])]
#[case::checks_theirs(CHECKS_THEIRS, [2, 3])]
#[case::promotions(PROMOTIONS, [4, 0])]
#[case::discovery(DISCOVERY, [7, 0])]
fn every_checking_move_is_counted_once(#[case] fen: &str, #[case] checks: [u16; 2]) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].check_count, checks[0]);
    assert_eq!(tactics[Side::Them.index()].check_count, checks[1]);
}

#[rstest]
#[case::start(START, "", "")]
#[case::mate(MATE, "r", "")]
#[case::checks(CHECKS, "nr", "n")]
#[case::promotions(PROMOTIONS, "pn", "")]
#[case::promotion_captures(PROMOTION_CAPTURES, "r", "pr")]
#[case::pins(PINS, "b", "q")]
fn a_checking_move_is_recorded_against_the_role_that_makes_it(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].check_by_role, roles(us));
    assert_eq!(tactics[Side::Them.index()].check_by_role, roles(them));
}

#[rstest]
#[case::start(START, [false, false])]
#[case::mate(MATE, [true, false])]
#[case::forks(FORKS, [false, true])]
#[case::checks(CHECKS, [true, true])]
#[case::promotion_captures(PROMOTION_CAPTURES, [true, true])]
fn a_safe_check_is_a_check_whose_destination_the_enemy_cannot_profitably_take(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(
        tactics[Side::Us.index()].safe_check_available(),
        available[0]
    );
    assert_eq!(
        tactics[Side::Them.index()].safe_check_available(),
        available[1]
    );
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::forks(FORKS, [0, 2])]
#[case::checks(CHECKS, [3, 2])]
#[case::promotion_captures(PROMOTION_CAPTURES, [1, 3])]
#[case::pins(PINS, [1, 1])]
#[case::discovery(DISCOVERY, [7, 0])]
fn only_the_checks_with_a_safe_destination_are_counted_safe(
    #[case] fen: &str,
    #[case] checks: [u16; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].safe_check_count, checks[0]);
    assert_eq!(tactics[Side::Them.index()].safe_check_count, checks[1]);
}

#[rstest]
#[case::start(START, "", "")]
#[case::forks(FORKS, "", "r")]
#[case::checks(CHECKS, "nr", "n")]
#[case::promotions(PROMOTIONS, "pn", "")]
#[case::promotion_captures(PROMOTION_CAPTURES, "r", "pr")]
#[case::pins(PINS, "b", "q")]
fn a_safe_check_is_recorded_against_the_role_that_makes_it(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].safe_check_by_role, roles(us));
    assert_eq!(tactics[Side::Them.index()].safe_check_by_role, roles(them));
}

#[rstest]
#[case::start(START, [false, false])]
#[case::checks(CHECKS, [false, false])]
#[case::discovery(DISCOVERY, [true, false])]
#[case::discovery_theirs(DISCOVERY_THEIRS, [false, true])]
fn a_double_check_is_a_move_that_leaves_two_units_giving_check(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(
        tactics[Side::Us.index()].double_check_available,
        available[0]
    );
    assert_eq!(
        tactics[Side::Them.index()].double_check_available,
        available[1]
    );
}

#[rstest]
#[case::start(START, [false, false])]
#[case::checks(CHECKS, [false, false])]
#[case::forks(FORKS, [false, false])]
#[case::discovery(DISCOVERY, [true, false])]
#[case::discovery_theirs(DISCOVERY_THEIRS, [false, true])]
fn a_discovered_check_comes_from_a_unit_that_did_not_move(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(
        tactics[Side::Us.index()].discovered_check_available,
        available[0]
    );
    assert_eq!(
        tactics[Side::Them.index()].discovered_check_available,
        available[1]
    );
}

#[rstest]
#[case::start(START, [false, false])]
#[case::stalemate(STALEMATE, [false, false])]
#[case::checks(CHECKS, [false, false])]
#[case::mate(MATE, [true, false])]
#[case::mate_theirs(MATE_THEIRS, [false, true])]
fn a_mate_in_1_is_a_legal_move_that_leaves_the_opponent_checkmated(
    #[case] fen: &str,
    #[case] mate: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].mate_in_1, mate[0]);
    assert_eq!(tactics[Side::Them.index()].mate_in_1, mate[1]);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::mate(MATE, [false, false])]
#[case::checks(CHECKS, [false, false])]
#[case::stalemate(STALEMATE, [true, false])]
#[case::only_move(ONLY_MOVE, [false, true])]
fn a_stalemate_in_1_is_a_legal_move_that_leaves_the_opponent_without_one(
    #[case] fen: &str,
    #[case] stalemate: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].stalemate_in_1, stalemate[0]);
    assert_eq!(tactics[Side::Them.index()].stalemate_in_1, stalemate[1]);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::checks(CHECKS, [false, false])]
#[case::promotions(PROMOTIONS, [true, true])]
#[case::promotion_captures(PROMOTION_CAPTURES, [true, true])]
fn a_promotion_is_available_when_a_legal_move_makes_one(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(
        tactics[Side::Us.index()].promotion_available(),
        available[0]
    );
    assert_eq!(
        tactics[Side::Them.index()].promotion_available(),
        available[1]
    );
}

#[rstest]
#[case::start(START, "", "")]
#[case::checks(CHECKS, "", "")]
#[case::promotions(PROMOTIONS, "bf", "bf")]
#[case::promotion_captures(PROMOTION_CAPTURES, "ab", "fg")]
fn a_promotion_is_filed_under_the_file_it_lands_on(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].promotion_files, files(us));
    assert_eq!(tactics[Side::Them.index()].promotion_files, files(them));
}

#[rstest]
#[case::start(START, [false; 4], [false; 4])]
#[case::checks(CHECKS, [false; 4], [false; 4])]
#[case::promotions(PROMOTIONS, [true; 4], [true; 4])]
#[case::promotion_captures(PROMOTION_CAPTURES, [true; 4], [true; 4])]
fn every_promotion_piece_is_obtainable_wherever_a_pawn_may_promote(
    #[case] fen: &str,
    #[case] us: [bool; 4],
    #[case] them: [bool; 4],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].promotion_roles, us);
    assert_eq!(tactics[Side::Them.index()].promotion_roles, them);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::checks(CHECKS, [false, false])]
#[case::promotions(PROMOTIONS, [true, true])]
#[case::promotion_captures(PROMOTION_CAPTURES, [true, true])]
fn a_safe_promotion_is_one_whose_destination_is_a_safe_destination(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(
        tactics[Side::Us.index()].safe_promotion_available(),
        available[0]
    );
    assert_eq!(
        tactics[Side::Them.index()].safe_promotion_available(),
        available[1]
    );
}

#[rstest]
#[case::start(START, "", "")]
#[case::checks(CHECKS, "", "")]
#[case::promotions(PROMOTIONS, "f", "b")]
#[case::promotion_captures(PROMOTION_CAPTURES, "a", "f")]
fn a_guarded_promotion_square_is_left_out_of_the_safe_files(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].safe_promotion_files, files(us));
    assert_eq!(
        tactics[Side::Them.index()].safe_promotion_files,
        files(them)
    );
}

#[rstest]
#[case::start(START, [false, false])]
#[case::promotions(PROMOTIONS, [false, false])]
#[case::in_check(IN_CHECK, [true, false])]
#[case::checks(CHECKS, [true, true])]
#[case::forks(FORKS, [false, true])]
#[case::pins(PINS, [true, false])]
fn a_capture_is_available_when_a_legal_move_takes_a_unit(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].capture_available(), available[0]);
    assert_eq!(
        tactics[Side::Them.index()].capture_available(),
        available[1]
    );
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::in_check(IN_CHECK, [1, 0])]
#[case::checks(CHECKS, [2, 2])]
#[case::forks(FORKS, [0, 1])]
#[case::promotion_captures(PROMOTION_CAPTURES, [4, 4])]
fn each_capturing_move_counts_for_itself_so_four_promotions_count_four(
    #[case] fen: &str,
    #[case] captures: [u16; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].capture_count, captures[0]);
    assert_eq!(tactics[Side::Them.index()].capture_count, captures[1]);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::pins(PINS, [false, false])]
#[case::checks(CHECKS, [true, true])]
#[case::mate(MATE, [true, false])]
#[case::mate_theirs(MATE_THEIRS, [false, true])]
fn a_capture_wins_when_the_victim_outvalues_the_capturer_or_stands_undefended(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(
        tactics[Side::Us.index()].winning_capture_available,
        available[0]
    );
    assert_eq!(
        tactics[Side::Them.index()].winning_capture_available,
        available[1]
    );
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::checks(CHECKS, [0, 0])]
#[case::mate(MATE, [4, 0])]
#[case::promotion_captures(PROMOTION_CAPTURES, [4, 4])]
#[case::in_check(IN_CHECK, [5, 0])]
fn the_gain_of_a_capture_is_the_victim_less_the_capturer_and_never_below_zero(
    #[case] fen: &str,
    #[case] gain: [i32; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].winning_capture_max_gain, gain[0]);
    assert_eq!(
        tactics[Side::Them.index()].winning_capture_max_gain,
        gain[1]
    );
}

#[rstest]
#[case::start(START, [false, false])]
#[case::forks(FORKS, [false, false])]
#[case::checks(CHECKS, [true, true])]
#[case::discovery(DISCOVERY, [true, false])]
#[case::mate_theirs(MATE_THEIRS, [false, true])]
fn a_hanging_victim_is_one_the_owner_leaves_undefended_under_attack(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].captures_hanging, available[0]);
    assert_eq!(tactics[Side::Them.index()].captures_hanging, available[1]);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::forks(FORKS, [0, 0])]
#[case::checks(CHECKS, [5, 3])]
#[case::checks_theirs(CHECKS_THEIRS, [3, 5])]
#[case::promotion_captures(PROMOTION_CAPTURES, [5, 5])]
#[case::discovery(DISCOVERY, [3, 0])]
fn the_hanging_victims_are_ranked_by_value_and_the_largest_is_kept(
    #[case] fen: &str,
    #[case] value: [i32; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].hanging_victim_max_value, value[0]);
    assert_eq!(
        tactics[Side::Them.index()].hanging_victim_max_value,
        value[1]
    );
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::promotion_captures(PROMOTION_CAPTURES, [0, 0])]
#[case::checks(CHECKS, [0, 1])]
#[case::checks_theirs(CHECKS_THEIRS, [1, 0])]
#[case::discovery(DISCOVERY, [0, 1])]
fn a_capture_of_a_defended_unit_of_equal_value_is_an_equal_one(
    #[case] fen: &str,
    #[case] captures: [u16; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].equal_capture_count, captures[0]);
    assert_eq!(tactics[Side::Them.index()].equal_capture_count, captures[1]);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::checks(CHECKS, [0, 0])]
#[case::mate(MATE, [0, 1])]
#[case::forks(FORKS, [0, 1])]
#[case::pins(PINS, [1, 0])]
#[case::pins_theirs(PINS_THEIRS, [0, 1])]
fn a_capture_of_a_defended_unit_of_lower_value_is_a_losing_one(
    #[case] fen: &str,
    #[case] captures: [u16; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].losing_capture_count, captures[0]);
    assert_eq!(
        tactics[Side::Them.index()].losing_capture_count,
        captures[1]
    );
}

#[rstest]
#[case::start(START, [false, false])]
#[case::pins(PINS, [false, false])]
#[case::checks(CHECKS, [true, false])]
#[case::checks_theirs(CHECKS_THEIRS, [false, true])]
#[case::forks(FORKS, [true, true])]
fn a_fork_leaves_the_mover_attacking_two_units_it_may_take(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].fork_available(), available[0]);
    assert_eq!(tactics[Side::Them.index()].fork_available(), available[1]);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::checks(CHECKS, [2, 0])]
#[case::checks_theirs(CHECKS_THEIRS, [0, 2])]
#[case::promotions(PROMOTIONS, [3, 0])]
#[case::forks(FORKS, [1, 1])]
fn every_forking_move_is_counted_once_however_many_units_it_forks(
    #[case] fen: &str,
    #[case] forks: [u16; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].fork_count, forks[0]);
    assert_eq!(tactics[Side::Them.index()].fork_count, forks[1]);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::forks(FORKS, [5, 5])]
#[case::checks(CHECKS, [9, 0])]
#[case::checks_theirs(CHECKS_THEIRS, [0, 9])]
#[case::promotions(PROMOTIONS, [9, 0])]
fn the_forked_value_is_the_largest_single_target_a_forking_king_counting_nine(
    #[case] fen: &str,
    #[case] value: [i32; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].fork_max_value, value[0]);
    assert_eq!(tactics[Side::Them.index()].fork_max_value, value[1]);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::pins(PINS, [false, false])]
#[case::checks(CHECKS, [true, false])]
#[case::checks_theirs(CHECKS_THEIRS, [false, true])]
#[case::forks(FORKS, [true, false])]
fn a_knight_fork_is_one_the_knight_itself_makes(#[case] fen: &str, #[case] available: [bool; 2]) {
    let tactics = blocks(fen);
    assert_eq!(
        tactics[Side::Us.index()].knight_fork_available,
        available[0]
    );
    assert_eq!(
        tactics[Side::Them.index()].knight_fork_available,
        available[1]
    );
}

#[rstest]
#[case::start(START, [false, false])]
#[case::forks(FORKS, [false, false])]
#[case::checks(CHECKS, [true, false])]
#[case::checks_theirs(CHECKS_THEIRS, [false, true])]
#[case::promotions(PROMOTIONS, [true, false])]
fn a_royal_fork_is_a_fork_one_of_whose_targets_is_the_king(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].royal_fork_available, available[0]);
    assert_eq!(
        tactics[Side::Them.index()].royal_fork_available,
        available[1]
    );
}

#[rstest]
#[case::start(START, [false, false])]
#[case::checks(CHECKS, [false, false])]
#[case::forks(FORKS, [true, false])]
#[case::mate(MATE, [false, true])]
#[case::pins(PINS, [true, false])]
#[case::pins_theirs(PINS_THEIRS, [false, true])]
fn a_pin_is_created_when_the_mover_traps_a_unit_in_front_of_a_dearer_one(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(
        tactics[Side::Us.index()].pin_creation_available(),
        available[0]
    );
    assert_eq!(
        tactics[Side::Them.index()].pin_creation_available(),
        available[1]
    );
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::forks(FORKS, [1, 0])]
#[case::mate(MATE, [0, 1])]
#[case::pins(PINS, [3, 0])]
#[case::pins_theirs(PINS_THEIRS, [0, 3])]
#[case::discovery(DISCOVERY, [0, 1])]
fn every_move_that_pins_is_counted_once_however_many_pins_it_makes(
    #[case] fen: &str,
    #[case] pins: [u16; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].pin_creation_count, pins[0]);
    assert_eq!(tactics[Side::Them.index()].pin_creation_count, pins[1]);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::forks(FORKS, [false, false])]
#[case::mate(MATE, [true, true])]
#[case::pins(PINS, [true, false])]
#[case::pins_theirs(PINS_THEIRS, [false, true])]
fn a_skewer_puts_the_dearer_unit_in_front_of_the_cheaper_one(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(
        tactics[Side::Us.index()].skewer_creation_available,
        available[0]
    );
    assert_eq!(
        tactics[Side::Them.index()].skewer_creation_available,
        available[1]
    );
}

#[rstest]
#[case::start(START, [false, false])]
#[case::forks(FORKS, [false, false])]
#[case::pins(PINS, [false, true])]
#[case::pins_theirs(PINS_THEIRS, [true, false])]
#[case::discovery(DISCOVERY, [true, false])]
fn a_discovered_attack_uncovers_a_slider_onto_a_piece_worth_three_or_more(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(
        tactics[Side::Us.index()].discovered_attack_available,
        available[0]
    );
    assert_eq!(
        tactics[Side::Them.index()].discovered_attack_available,
        available[1]
    );
}

#[rstest]
#[case::start(START, [20, 20])]
#[case::only_move(ONLY_MOVE, [1, 6])]
#[case::in_check(IN_CHECK, [3, 0])]
#[case::stalemate(STALEMATE, [13, 0])]
#[case::checks(CHECKS, [22, 16])]
#[case::promotion_captures(PROMOTION_CAPTURES, [30, 27])]
fn the_legal_moves_are_counted_for_us_and_for_them_after_the_null_move(
    #[case] fen: &str,
    #[case] moves: [u16; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].legal_move_count, moves[0]);
    assert_eq!(tactics[Side::Them.index()].legal_move_count, moves[1]);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::checks(CHECKS, [false, false])]
#[case::in_check(IN_CHECK, [false, false])]
#[case::only_move(ONLY_MOVE, [true, false])]
#[case::stalemate(STALEMATE, [false, true])]
fn a_side_is_down_to_only_moves_with_at_most_two_of_them_to_choose_from(
    #[case] fen: &str,
    #[case] only: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].only_moves(), only[0]);
    assert_eq!(tactics[Side::Them.index()].only_moves(), only[1]);
}

#[rstest]
#[case::start(START, [true, true])]
#[case::stalemate(STALEMATE, [true, true])]
#[case::discovery(DISCOVERY, [true, true])]
#[case::in_check(IN_CHECK, [true, false])]
fn their_block_is_unavailable_and_empty_when_the_null_move_does_not_exist(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics[Side::Us.index()].available, available[0]);
    assert_eq!(tactics[Side::Them.index()].available, available[1]);
    if !available[1] {
        assert_eq!(tactics[Side::Them.index()], TacticsFacts::default());
    }
}

/// No `tactics` fact is among the four `features.md` §4 keeps for classic chess
/// only, and the group reads a Chess960 castling by the square its king lands
/// on: Black is checked by the rook the castling brings to d1, and the mover is
/// a king, so no `check_by_role` bit is set.
#[test]
fn the_tactics_of_a_chess960_position_read_a_castling_by_the_kings_landing_square() {
    let facts = facts_under(&CHESS960, NINE_SIXTY);
    let ours = &facts.tactics[Side::Us.index()];
    let theirs = &facts.tactics[Side::Them.index()];

    assert_eq!(ours.legal_move_count, 12);
    assert_eq!(ours.check_count, 2);
    assert_eq!(ours.check_by_role, roles("r"));
    assert_eq!(ours.safe_check_count, 2);
    assert!(!ours.discovered_check_available);
    assert!(!ours.fork_available());
    assert!(ours.skewer_creation_available);
    assert!(!ours.pin_creation_available());

    assert_eq!(theirs.legal_move_count, 15);
    assert_eq!(theirs.check_count, 1);
    assert_eq!(theirs.check_by_role, roles("r"));
    assert!(theirs.skewer_creation_available);
}
