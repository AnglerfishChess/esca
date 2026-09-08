//! The `tactics` group, fact by fact.
//!
//! Every expectation is worked out from the definitions in `docs/features.md`
//! §1 for the named position above it. The `them` block of a position is the
//! `us` block of the same placement with the other side to move, which is what
//! the null move of `features.md` §1 makes it.

mod common;

use common::{by_colour, facts_of, facts_under, files};
use esca::{CHESS960, Colour, TacticsFacts};
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

/// Knight for knight, each defended by a pawn on neither side: an even trade.
const TRADE: &str = "4k3/8/2p5/3n4/8/2N5/8/4K3 w - - 0 1";

/// The same board with Black to move, whose knight has nothing behind it.
const TRADE_THEIRS: &str = "4k3/8/2p5/3n4/8/2N5/8/4K3 b - - 0 1";

/// The only capture is a rook taking a pawn a pawn defends.
const LOSING_TAKE: &str = "4k3/8/2p5/3p4/8/8/3R4/4K3 w - - 0 1";

/// The same board with Black to move, who has no capture at all.
const LOSING_TAKE_THEIRS: &str = "4k3/8/2p5/3p4/8/8/3R4/4K3 b - - 0 1";

/// Chess960: castling long lands the king on c1 and the rook on d1, in check.
const NINE_SIXTY: &str = "3k3r/8/8/8/8/8/8/RK6 w A - 0 1";

/// Rxe8 takes the rook and checks in one, onto a square nothing covers.
const SAFE_CHECK_CAPTURE: &str = "k3r3/8/8/8/8/8/4R3/4R2K w - - 0 1";

/// The same board with Black to move.
const SAFE_CHECK_CAPTURE_THEIRS: &str = "k3r3/8/8/8/8/8/4R3/4R2K b - - 0 1";

/// Rxe7 checks and captures, and the king takes the rook straight back.
const UNSAFE_CHECK_CAPTURE: &str = "4k3/4r3/8/8/8/8/8/4R2K w - - 0 1";

/// Every knight move uncovers the rook's attack on the queen behind it.
const DISCOVERED_QUEEN: &str = "3q2k1/8/8/8/3N4/8/8/3R3K w - - 0 1";

/// The same board with Black to move.
const DISCOVERED_QUEEN_THEIRS: &str = "3q2k1/8/8/8/3N4/8/8/3R3K b - - 0 1";

/// The same discovery with the colours exchanged, Black to move.
const MIRRORED_QUEEN: &str = "3r3k/8/8/8/3n4/8/8/3Q2K1 b - - 0 1";

/// The same board with White to move.
const MIRRORED_QUEEN_THEIRS: &str = "3r3k/8/8/8/3n4/8/8/3Q2K1 w - - 0 1";

/// The same discovery onto a rook: uncovered, but not onto the queen.
const DISCOVERED_ROOK: &str = "3r2k1/8/8/8/3N4/8/8/3R3K w - - 0 1";

/// The mating board with the h-pawn a rank on, so the black king has luft.
const BACK_RANK_LUFT: &str = "6k1/5pp1/7p/8/8/7r/5PPP/R5K1 w - - 0 1";

/// Ra8 mates; White's own king stands a rank up, on no back rank of its own.
const BACK_RANK_ONE_SIDED: &str = "6k1/5ppp/8/8/8/4r2P/5PPK/R7 w - - 0 1";

/// A rook of Black's own seals the eighth rank, so Ra8 arrives without check.
const BACK_RANK_BLOCKED: &str = "1r4k1/5ppp/8/8/8/7r/5PPP/R5K1 w - - 0 1";

/// Rd1 attacks the knight the rook reaches nothing from where it stands.
const QUIET_THREAT: &str = "6k1/8/8/3n4/8/8/8/R5K1 w - - 0 1";

/// The same board with Black to move.
const QUIET_THREAT_THEIRS: &str = "6k1/8/8/3n4/8/8/8/R5K1 b - - 0 1";

/// The rook attacks the knight already, and no move of White's wins more.
const THREAT_STANDS: &str = "6k1/8/8/3n4/8/8/8/3R2K1 w - - 0 1";

/// Five legal moves for White, every one of them onto a square a pawn covers.
const BOXED_IN: &str = "k7/8/8/8/1p4pp/4p3/5bPP/1N5K w - - 0 1";

/// The same board with Black to move.
const BOXED_IN_THEIRS: &str = "k7/8/8/8/1p4pp/4p3/5bPP/1N5K b - - 0 1";

/// The same position with the colours exchanged, so Black is the boxed side.
const BOXED_IN_MIRRORED: &str = "1n5k/5Bpp/4P3/1P4PP/8/8/8/K7 b - - 0 1";

/// The knight covers b8, so every promotion there loses the new piece.
const GUARDED_PROMOTION: &str = "8/1P6/n6k/8/7K/8/8/8 w - - 0 1";

/// Nothing covers f8: the push alone wins a queen.
const FREE_PROMOTION: &str = "8/5P2/7k/8/7K/8/8/8 w - - 0 1";

/// The same board with Black to move.
const FREE_PROMOTION_THEIRS: &str = "8/5P2/7k/8/7K/8/8/8 b - - 0 1";

/// The `tactics` group of `fen` under classic chess.
fn blocks(fen: &str) -> TacticsFacts {
    facts_of(fen).tactics
}

/// The five checking flags of `colour`, in the order `roles` writes them.
fn check_roles(tactics: &TacticsFacts, colour: Colour) -> [bool; 5] {
    [
        tactics.check_by_pawn,
        tactics.check_by_knight,
        tactics.check_by_bishop,
        tactics.check_by_rook,
        tactics.check_by_queen,
    ]
    .map(|flag| *flag.of(colour))
}

/// The same for the checks whose destination is safe.
fn safe_check_roles(tactics: &TacticsFacts, colour: Colour) -> [bool; 5] {
    [
        tactics.safe_check_by_pawn,
        tactics.safe_check_by_knight,
        tactics.safe_check_by_bishop,
        tactics.safe_check_by_rook,
        tactics.safe_check_by_queen,
    ]
    .map(|flag| *flag.of(colour))
}

/// The four promotion flags of `colour`, queen first.
fn promotion_roles(tactics: &TacticsFacts, colour: Colour) -> [bool; 4] {
    [
        tactics.promotes_to_queen,
        tactics.promotes_to_rook,
        tactics.promotes_to_bishop,
        tactics.promotes_to_knight,
    ]
    .map(|flag| *flag.of(colour))
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
#[case::mate_theirs(MATE_THEIRS, [true, false])]
#[case::checks(CHECKS, [true, true])]
#[case::discovery(DISCOVERY, [true, false])]
fn a_check_is_available_when_a_legal_move_leaves_the_enemy_king_attacked(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.check_available().white, available[0]);
    assert_eq!(tactics.check_available().black, available[1]);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::mate(MATE, [1, 0])]
#[case::checks(CHECKS, [3, 2])]
#[case::checks_theirs(CHECKS_THEIRS, [3, 2])]
#[case::promotions(PROMOTIONS, [4, 0])]
#[case::discovery(DISCOVERY, [7, 0])]
fn every_checking_move_is_counted_once(#[case] fen: &str, #[case] checks: [u16; 2]) {
    let tactics = blocks(fen);
    assert_eq!(tactics.check_count.white, checks[0]);
    assert_eq!(tactics.check_count.black, checks[1]);
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
    #[case] white: &str,
    #[case] black: &str,
) {
    let tactics = blocks(fen);
    assert_eq!(check_roles(&tactics, Colour::White), roles(white));
    assert_eq!(check_roles(&tactics, Colour::Black), roles(black));
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
    assert_eq!(tactics.safe_check_available().white, available[0]);
    assert_eq!(tactics.safe_check_available().black, available[1]);
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
    assert_eq!(tactics.safe_check_count.white, checks[0]);
    assert_eq!(tactics.safe_check_count.black, checks[1]);
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
    #[case] white: &str,
    #[case] black: &str,
) {
    let tactics = blocks(fen);
    assert_eq!(safe_check_roles(&tactics, Colour::White), roles(white));
    assert_eq!(safe_check_roles(&tactics, Colour::Black), roles(black));
}

#[rstest]
#[case::start(START, [false, false])]
#[case::checks(CHECKS, [false, false])]
#[case::discovery(DISCOVERY, [true, false])]
#[case::discovery_theirs(DISCOVERY_THEIRS, [true, false])]
fn a_double_check_is_a_move_that_leaves_two_units_giving_check(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.double_check_available.white, available[0]);
    assert_eq!(tactics.double_check_available.black, available[1]);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::checks(CHECKS, [false, false])]
#[case::forks(FORKS, [false, false])]
#[case::discovery(DISCOVERY, [true, false])]
#[case::discovery_theirs(DISCOVERY_THEIRS, [true, false])]
fn a_discovered_check_comes_from_a_unit_that_did_not_move(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.discovered_check_available.white, available[0]);
    assert_eq!(tactics.discovered_check_available.black, available[1]);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::stalemate(STALEMATE, [false, false])]
#[case::checks(CHECKS, [false, false])]
#[case::mate(MATE, [true, false])]
#[case::mate_theirs(MATE_THEIRS, [true, false])]
fn a_mate_in_1_is_a_legal_move_that_leaves_the_opponent_checkmated(
    #[case] fen: &str,
    #[case] mate: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.mate_in_1.white, mate[0]);
    assert_eq!(tactics.mate_in_1.black, mate[1]);
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
    assert_eq!(tactics.stalemate_in_1.white, stalemate[0]);
    assert_eq!(tactics.stalemate_in_1.black, stalemate[1]);
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
    assert_eq!(tactics.promotion_available().white, available[0]);
    assert_eq!(tactics.promotion_available().black, available[1]);
}

#[rstest]
#[case::start(START, "", "")]
#[case::checks(CHECKS, "", "")]
#[case::promotions(PROMOTIONS, "bf", "bf")]
#[case::promotion_captures(PROMOTION_CAPTURES, "ab", "fg")]
fn a_promotion_is_filed_under_the_file_it_lands_on(
    #[case] fen: &str,
    #[case] white: &str,
    #[case] black: &str,
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.promotion_files.white, files(white));
    assert_eq!(tactics.promotion_files.black, files(black));
}

#[rstest]
#[case::start(START, [false; 4], [false; 4])]
#[case::checks(CHECKS, [false; 4], [false; 4])]
#[case::promotions(PROMOTIONS, [true; 4], [true; 4])]
#[case::promotion_captures(PROMOTION_CAPTURES, [true; 4], [true; 4])]
fn every_promotion_piece_is_obtainable_wherever_a_pawn_may_promote(
    #[case] fen: &str,
    #[case] white: [bool; 4],
    #[case] black: [bool; 4],
) {
    let tactics = blocks(fen);
    assert_eq!(promotion_roles(&tactics, Colour::White), white);
    assert_eq!(promotion_roles(&tactics, Colour::Black), black);
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
    assert_eq!(tactics.safe_promotion_available().white, available[0]);
    assert_eq!(tactics.safe_promotion_available().black, available[1]);
}

#[rstest]
#[case::start(START, "", "")]
#[case::checks(CHECKS, "", "")]
#[case::promotions(PROMOTIONS, "f", "b")]
#[case::promotion_captures(PROMOTION_CAPTURES, "a", "f")]
fn a_guarded_promotion_square_is_left_out_of_the_safe_files(
    #[case] fen: &str,
    #[case] white: &str,
    #[case] black: &str,
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.safe_promotion_files.white, files(white));
    assert_eq!(tactics.safe_promotion_files.black, files(black));
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
    assert_eq!(tactics.capture_available().white, available[0]);
    assert_eq!(tactics.capture_available().black, available[1]);
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
    assert_eq!(tactics.capture_count.white, captures[0]);
    assert_eq!(tactics.capture_count.black, captures[1]);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::pins(PINS, [false, false])]
#[case::forks(FORKS, [false, false])]
#[case::checks(CHECKS, [true, true])]
#[case::mate(MATE, [true, false])]
#[case::mate_theirs(MATE_THEIRS, [true, false])]
#[case::promotion_captures(PROMOTION_CAPTURES, [true, true])]
#[case::discovery(DISCOVERY, [true, false])]
fn a_capture_wins_when_the_exchange_it_starts_wins_material(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.winning_capture_available.white, available[0]);
    assert_eq!(tactics.winning_capture_available.black, available[1]);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::pins(PINS, [0, 0])]
#[case::checks(CHECKS, [5, 3])]
#[case::mate(MATE, [5, 0])]
#[case::mate_theirs(MATE_THEIRS, [5, 0])]
#[case::promotion_captures(PROMOTION_CAPTURES, [13, 13])]
#[case::in_check(IN_CHECK, [5, 0])]
#[case::discovery(DISCOVERY, [3, 0])]
fn the_max_gain_is_the_best_see_over_the_captures_and_never_below_zero(
    #[case] fen: &str,
    #[case] gain: [i32; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.winning_capture_max_gain.white, gain[0]);
    assert_eq!(tactics.winning_capture_max_gain.black, gain[1]);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::forks(FORKS, [false, false])]
#[case::checks(CHECKS, [true, true])]
#[case::discovery(DISCOVERY, [true, false])]
#[case::mate_theirs(MATE_THEIRS, [true, false])]
fn a_hanging_victim_is_one_the_owner_leaves_undefended_under_attack(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.captures_hanging.white, available[0]);
    assert_eq!(tactics.captures_hanging.black, available[1]);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::forks(FORKS, [0, 0])]
#[case::checks(CHECKS, [5, 3])]
#[case::checks_theirs(CHECKS_THEIRS, [5, 3])]
#[case::promotion_captures(PROMOTION_CAPTURES, [5, 5])]
#[case::discovery(DISCOVERY, [3, 0])]
fn the_hanging_victims_are_ranked_by_value_and_the_largest_is_kept(
    #[case] fen: &str,
    #[case] value: [i32; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.hanging_victim_max_value.white, value[0]);
    assert_eq!(tactics.hanging_victim_max_value.black, value[1]);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::promotion_captures(PROMOTION_CAPTURES, [0, 0])]
#[case::checks(CHECKS, [0, 1])]
#[case::checks_theirs(CHECKS_THEIRS, [0, 1])]
#[case::discovery(DISCOVERY, [0, 1])]
#[case::discovery_theirs(DISCOVERY_THEIRS, [0, 1])]
#[case::trade(TRADE, [1, 0])]
#[case::trade_theirs(TRADE_THEIRS, [1, 0])]
fn an_equal_capture_is_one_whose_exchange_comes_out_level(
    #[case] fen: &str,
    #[case] captures: [u16; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.equal_capture_count.white, captures[0]);
    assert_eq!(tactics.equal_capture_count.black, captures[1]);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::checks(CHECKS, [0, 0])]
#[case::mate(MATE, [0, 1])]
#[case::mate_theirs(MATE_THEIRS, [0, 1])]
#[case::forks(FORKS, [0, 1])]
#[case::pins(PINS, [1, 0])]
#[case::pins_theirs(PINS_THEIRS, [1, 0])]
#[case::losing_take(LOSING_TAKE, [1, 0])]
#[case::losing_take_theirs(LOSING_TAKE_THEIRS, [1, 0])]
fn a_losing_capture_is_one_whose_exchange_costs_material(
    #[case] fen: &str,
    #[case] captures: [u16; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.losing_capture_count.white, captures[0]);
    assert_eq!(tactics.losing_capture_count.black, captures[1]);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::pins(PINS, [false, false])]
#[case::checks(CHECKS, [true, false])]
#[case::checks_theirs(CHECKS_THEIRS, [true, false])]
#[case::forks(FORKS, [true, true])]
fn a_fork_leaves_the_mover_attacking_two_units_it_may_take(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.fork_available().white, available[0]);
    assert_eq!(tactics.fork_available().black, available[1]);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::checks(CHECKS, [2, 0])]
#[case::checks_theirs(CHECKS_THEIRS, [2, 0])]
#[case::promotions(PROMOTIONS, [3, 0])]
#[case::forks(FORKS, [1, 1])]
fn every_forking_move_is_counted_once_however_many_units_it_forks(
    #[case] fen: &str,
    #[case] forks: [u16; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.fork_count.white, forks[0]);
    assert_eq!(tactics.fork_count.black, forks[1]);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::forks(FORKS, [5, 5])]
#[case::checks(CHECKS, [9, 0])]
#[case::checks_theirs(CHECKS_THEIRS, [9, 0])]
#[case::promotions(PROMOTIONS, [9, 0])]
fn the_forked_value_is_the_largest_single_target_a_forking_king_counting_nine(
    #[case] fen: &str,
    #[case] value: [i32; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.fork_max_value.white, value[0]);
    assert_eq!(tactics.fork_max_value.black, value[1]);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::pins(PINS, [false, false])]
#[case::checks(CHECKS, [true, false])]
#[case::checks_theirs(CHECKS_THEIRS, [true, false])]
#[case::forks(FORKS, [true, false])]
fn a_knight_fork_is_one_the_knight_itself_makes(#[case] fen: &str, #[case] available: [bool; 2]) {
    let tactics = blocks(fen);
    assert_eq!(tactics.knight_fork_available.white, available[0]);
    assert_eq!(tactics.knight_fork_available.black, available[1]);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::forks(FORKS, [false, false])]
#[case::checks(CHECKS, [true, false])]
#[case::checks_theirs(CHECKS_THEIRS, [true, false])]
#[case::promotions(PROMOTIONS, [true, false])]
fn a_royal_fork_is_a_fork_one_of_whose_targets_is_the_king(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.royal_fork_available.white, available[0]);
    assert_eq!(tactics.royal_fork_available.black, available[1]);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::checks(CHECKS, [false, false])]
#[case::forks(FORKS, [true, false])]
#[case::mate(MATE, [false, true])]
#[case::pins(PINS, [true, false])]
#[case::pins_theirs(PINS_THEIRS, [true, false])]
fn a_pin_is_created_when_the_mover_traps_a_unit_in_front_of_a_dearer_one(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.pin_creation_available().white, available[0]);
    assert_eq!(tactics.pin_creation_available().black, available[1]);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::forks(FORKS, [1, 0])]
#[case::mate(MATE, [0, 1])]
#[case::pins(PINS, [3, 0])]
#[case::pins_theirs(PINS_THEIRS, [3, 0])]
#[case::discovery(DISCOVERY, [0, 1])]
fn every_move_that_pins_is_counted_once_however_many_pins_it_makes(
    #[case] fen: &str,
    #[case] pins: [u16; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.pin_creation_count.white, pins[0]);
    assert_eq!(tactics.pin_creation_count.black, pins[1]);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::forks(FORKS, [false, false])]
#[case::mate(MATE, [true, true])]
#[case::pins(PINS, [true, false])]
#[case::pins_theirs(PINS_THEIRS, [true, false])]
fn a_skewer_puts_the_dearer_unit_in_front_of_the_cheaper_one(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.skewer_creation_available.white, available[0]);
    assert_eq!(tactics.skewer_creation_available.black, available[1]);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::forks(FORKS, [false, false])]
#[case::pins(PINS, [false, true])]
#[case::pins_theirs(PINS_THEIRS, [false, true])]
#[case::discovery(DISCOVERY, [true, false])]
fn a_discovered_attack_uncovers_a_slider_onto_a_piece_worth_three_or_more(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.discovered_attack_available.white, available[0]);
    assert_eq!(tactics.discovered_attack_available.black, available[1]);
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
    assert_eq!(tactics.legal_move_count.white, moves[0]);
    assert_eq!(tactics.legal_move_count.black, moves[1]);
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
    assert_eq!(tactics.only_moves().white, only[0]);
    assert_eq!(tactics.only_moves().black, only[1]);
}

#[rstest]
#[case::start(START, [true, true])]
#[case::stalemate(STALEMATE, [true, true])]
#[case::discovery(DISCOVERY, [true, true])]
#[case::in_check(IN_CHECK, [true, false])]
fn the_side_not_to_move_is_unread_when_the_null_move_does_not_exist(
    #[case] fen: &str,
    #[case] available: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.available, by_colour(available));
    if !available[1] {
        assert_eq!(tactics.legal_move_count.black, 0);
        assert_eq!(tactics.capture_count.black, 0);
        assert_eq!(tactics.check_count.black, 0);
    }
}

#[rstest]
#[case::start(START, [false, false])]
#[case::mate(MATE, [false, false])]
#[case::safe_check_capture(SAFE_CHECK_CAPTURE, [true, false])]
#[case::safe_check_capture_theirs(SAFE_CHECK_CAPTURE_THEIRS, [true, false])]
#[case::unsafe_check_capture(UNSAFE_CHECK_CAPTURE, [false, true])]
#[case::discovery(DISCOVERY, [true, false])]
fn a_capturing_check_is_reported_only_when_its_destination_is_safe(
    #[case] fen: &str,
    #[case] capturing: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.safe_check_capturing.white, capturing[0]);
    assert_eq!(tactics.safe_check_capturing.black, capturing[1]);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::discovered_queen(DISCOVERED_QUEEN, [true, false])]
#[case::discovered_queen_theirs(DISCOVERED_QUEEN_THEIRS, [true, false])]
#[case::mirrored_queen(MIRRORED_QUEEN, [false, true])]
#[case::mirrored_queen_theirs(MIRRORED_QUEEN_THEIRS, [false, true])]
#[case::discovered_rook(DISCOVERED_ROOK, [false, false])]
fn a_discovered_attack_on_the_queen_wants_the_queen_and_no_lesser_unit(
    #[case] fen: &str,
    #[case] uncovered: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.discovered_attack_on_queen.white, uncovered[0]);
    assert_eq!(tactics.discovered_attack_on_queen.black, uncovered[1]);
    for colour in Colour::ALL {
        assert!(
            *tactics.discovered_attack_available.of(colour)
                || !*tactics.discovered_attack_on_queen.of(colour)
        );
    }
}

#[rstest]
#[case::start(START, [false, false])]
#[case::mate(MATE, [true, false])]
#[case::mate_theirs(MATE_THEIRS, [true, false])]
#[case::back_rank_luft(BACK_RANK_LUFT, [false, false])]
#[case::back_rank_one_sided(BACK_RANK_ONE_SIDED, [true, false])]
#[case::back_rank_blocked(BACK_RANK_BLOCKED, [false, true])]
fn a_back_rank_mate_threat_is_a_rook_or_queen_check_on_a_sealed_back_rank(
    #[case] fen: &str,
    #[case] threat: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.back_rank_mate_threat.white, threat[0]);
    assert_eq!(tactics.back_rank_mate_threat.black, threat[1]);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::quiet_threat(QUIET_THREAT, [true, false])]
#[case::quiet_threat_theirs(QUIET_THREAT_THEIRS, [true, false])]
#[case::threat_stands(THREAT_STANDS, [false, true])]
#[case::promotion_captures(PROMOTION_CAPTURES, [false, false])]
#[case::mate(MATE, [false, true])]
#[case::discovered_queen(DISCOVERED_QUEEN, [true, false])]
fn a_quiet_threat_leaves_more_to_be_won_than_stands_to_be_won_now(
    #[case] fen: &str,
    #[case] threat: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.quiet_threat_available.white, threat[0]);
    assert_eq!(tactics.quiet_threat_available.black, threat[1]);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::mate(MATE, [false, false])]
#[case::boxed_in(BOXED_IN, [true, false])]
#[case::boxed_in_theirs(BOXED_IN_THEIRS, [true, false])]
#[case::boxed_in_mirrored(BOXED_IN_MIRRORED, [false, true])]
#[case::stalemate(STALEMATE, [false, true])]
fn a_side_has_no_safe_moves_when_every_legal_move_lands_where_it_can_be_taken(
    #[case] fen: &str,
    #[case] boxed: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.no_safe_moves.white, boxed[0]);
    assert_eq!(tactics.no_safe_moves.black, boxed[1]);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::promotions(PROMOTIONS, [true, true])]
#[case::guarded_promotion(GUARDED_PROMOTION, [false, false])]
#[case::promotion_captures(PROMOTION_CAPTURES, [true, true])]
#[case::free_promotion(FREE_PROMOTION, [true, false])]
#[case::free_promotion_theirs(FREE_PROMOTION_THEIRS, [true, false])]
fn a_promotion_is_see_positive_when_the_exchange_on_its_square_wins_material(
    #[case] fen: &str,
    #[case] positive: [bool; 2],
) {
    let tactics = blocks(fen);
    assert_eq!(tactics.promotion_see_positive.white, positive[0]);
    assert_eq!(tactics.promotion_see_positive.black, positive[1]);
}

/// A block with no side to move states no tactic, `no_safe_moves` included:
/// zero there says the block was not computed, not that a safe move exists.
#[test]
fn a_block_the_null_move_does_not_allow_states_no_tactic_of_its_own() {
    let tactics = blocks(IN_CHECK);
    assert!(!tactics.available.black);
    assert!(!tactics.safe_check_capturing.black);
    assert!(!tactics.discovered_attack_on_queen.black);
    assert!(!tactics.back_rank_mate_threat.black);
    assert!(!tactics.quiet_threat_available.black);
    assert!(!tactics.no_safe_moves.black);
    assert!(!tactics.promotion_see_positive.black);
}

/// The group reads a Chess960 castling by the square its king lands on: Black
/// is checked by the rook the castling brings to d1, and the mover is a king,
/// so no checking-role flag is set for it.
#[test]
fn the_tactics_of_a_chess960_position_read_a_castling_by_the_kings_landing_square() {
    let tactics = facts_under(&CHESS960, NINE_SIXTY).tactics;

    assert_eq!(tactics.legal_move_count.white, 12);
    assert_eq!(tactics.check_count.white, 2);
    assert_eq!(check_roles(&tactics, Colour::White), roles("r"));
    assert_eq!(tactics.safe_check_count.white, 2);
    assert!(!tactics.discovered_check_available.white);
    assert!(!tactics.fork_available().white);
    assert!(tactics.skewer_creation_available.white);
    assert!(!tactics.pin_creation_available().white);

    assert_eq!(tactics.legal_move_count.black, 15);
    assert_eq!(tactics.check_count.black, 1);
    assert_eq!(check_roles(&tactics, Colour::Black), roles("r"));
    assert!(tactics.skewer_creation_available.black);
}
