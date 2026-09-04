//! The `threats` group: what each side stands to lose, and the slider geometry
//! a threat is made of.
//!
//! Every expectation is worked out from the definitions in `docs/features.md`
//! §1 and §2.10 for the named position above it.

mod common;

use common::{facts_of, squares};
use esca::Side;
use rstest::rstest;

/// The untouched array: nothing attacked, and a loose rook in each corner.
const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// A rook bears on a knight nothing defends.
const HANGING_KNIGHT: &str = "4k3/8/8/3n4/8/8/3R4/4K3 w - - 0 1";

/// The same knight, defended by a pawn: taking it costs the rook.
const DEFENDED_KNIGHT: &str = "4k3/8/2p5/3n4/8/8/3R4/4K3 w - - 0 1";

/// A pawn attacks our rook, which nothing defends.
const PAWN_ATTACKS_ROOK: &str = "7k/8/8/3p4/4R3/8/8/4K3 w - - 0 1";

/// A knight attacks our queen: the costliest thing a lesser unit can attack.
const QUEEN_UNDER_KNIGHT: &str = "7k/8/5n2/8/4Q3/8/8/4K3 w - - 0 1";

/// The same with Black to move: the queen under a knight is theirs.
const QUEEN_UNDER_KNIGHT_BLACK: &str = "7k/8/5n2/8/4Q3/8/8/4K3 b - - 0 1";

/// Our pawn and their queen attack each other, both undefended.
const CROSS_THREATS: &str = "4k3/8/8/3q4/4P3/8/3R4/4K3 w - - 0 1";

/// The same with Black to move: the two blocks read the other way round.
const CROSS_THREATS_BLACK: &str = "4k3/8/8/3q4/4P3/8/3R4/4K3 b - - 0 1";

/// The d7 rook alone defends the two pawns two rooks attack, and is safe itself.
const OVERLOADED_ROOK: &str = "6k1/p2r4/8/3p4/8/8/8/R2RK3 w - - 0 1";

/// The same with Black to move.
const OVERLOADED_ROOK_BLACK: &str = "6k1/p2r4/8/3p4/8/8/8/R2RK3 b - - 0 1";

/// The same overloaded rook, now hanging to a bishop: the defence can be taken.
const REMOVABLE_ROOK: &str = "6k1/p2r4/8/3p4/8/7B/8/R2RK3 w - - 0 1";

/// The same with Black to move.
const REMOVABLE_ROOK_BLACK: &str = "6k1/p2r4/8/3p4/8/7B/8/R2RK3 b - - 0 1";

/// A knight alone defends the two pawns the rooks attack; nothing attacks it.
const OVERLOADED_KNIGHT: &str = "7k/p3p3/2n5/8/8/8/8/R3R1K1 w - - 0 1";

/// The same with Black to move.
const OVERLOADED_KNIGHT_BLACK: &str = "7k/p3p3/2n5/8/8/8/8/R3R1K1 b - - 0 1";

/// The same, with a knight that attacks the defender and hangs to it in turn.
const REMOVABLE_KNIGHT: &str = "7k/p3p3/2n5/8/1N6/8/8/R3R1K1 w - - 0 1";

/// The same with Black to move.
const REMOVABLE_KNIGHT_BLACK: &str = "7k/p3p3/2n5/8/1N6/8/8/R3R1K1 b - - 0 1";

/// The defender is now defended by a pawn: taking it is an even trade, which is
/// enough to remove it.
const REMOVABLE_TRADE: &str = "7k/pp2p3/2n5/8/1N6/8/8/R3R1K1 w - - 0 1";

/// The same with Black to move.
const REMOVABLE_TRADE_BLACK: &str = "7k/pp2p3/2n5/8/1N6/8/8/R3R1K1 b - - 0 1";

/// The b7 pawn alone defends the two attacked pawns; the queen that attacks it
/// would lose itself to the rook behind.
const OVERLOADED_PAWN: &str = "1r5k/1p6/p1p5/8/8/1Q6/8/R1R3K1 w - - 0 1";

/// The same with Black to move.
const OVERLOADED_PAWN_BLACK: &str = "1r5k/1p6/p1p5/8/8/1Q6/8/R1R3K1 b - - 0 1";

/// A bishop attacks the queen and x-rays the rook behind it.
const XRAY_BISHOP: &str = "4k1r1/8/8/3q4/8/1B6/8/4K3 w - - 0 1";

/// The same with Black to move.
const XRAY_BISHOP_BLACK: &str = "4k1r1/8/8/3q4/8/1B6/8/4K3 b - - 0 1";

/// Our rook x-rays their rook through their pawn; theirs looks through its own.
const XRAY_ROOK: &str = "4k3/3r4/8/8/3p4/8/8/3R2K1 w - - 0 1";

/// Both sides double on the d-file, each battery bearing on the other's king.
const DOUBLED_ROOKS: &str = "3rk3/3r4/8/8/8/8/3R4/3RK3 w - - 0 1";

/// Two rooks bear on one the enemy defends once: the exchange wins a rook, and
/// no attacker is worth less than its target.
const TWO_ON_ONE: &str = "3r3k/3r4/8/8/8/8/3R4/3RK3 w - - 0 1";

/// A queen and a rook on the d-file, with the enemy king's ring on it.
const BATTERY_AT_KING: &str = "2k5/8/8/8/8/8/3Q4/3R1K2 w - - 0 1";

/// The same with Black to move.
const BATTERY_AT_KING_BLACK: &str = "2k5/8/8/8/8/8/3Q4/3R1K2 b - - 0 1";

/// A queen behind a bishop on the long diagonal, which the enemy king's ring
/// stands on.
const BISHOP_BATTERY: &str = "8/7k/8/8/8/2B5/1Q6/6K1 w - - 0 1";

/// The same with Black to move.
const BISHOP_BATTERY_BLACK: &str = "8/7k/8/8/8/2B5/1Q6/6K1 b - - 0 1";

#[rstest]
#[case::start(START, "", "")]
#[case::hanging_knight(HANGING_KNIGHT, "", "d5")]
#[case::defended_knight(DEFENDED_KNIGHT, "", "")]
#[case::pawn_attacks_rook(PAWN_ATTACKS_ROOK, "e4", "")]
#[case::queen_under_knight(QUEEN_UNDER_KNIGHT, "e4", "")]
#[case::cross_threats(CROSS_THREATS, "e4", "d5")]
#[case::cross_threats_black(CROSS_THREATS_BLACK, "d5", "e4")]
#[case::overloaded_rook(OVERLOADED_ROOK, "", "")]
#[case::removable_rook(REMOVABLE_ROOK, "", "d7")]
#[case::removable_rook_black(REMOVABLE_ROOK_BLACK, "d7", "")]
#[case::removable_knight(REMOVABLE_KNIGHT, "b4", "c6")]
#[case::removable_trade(REMOVABLE_TRADE, "b4", "")]
#[case::xray_bishop(XRAY_BISHOP, "b3", "d5")]
#[case::xray_bishop_black(XRAY_BISHOP_BLACK, "d5", "b3")]
#[case::doubled_rooks(DOUBLED_ROOKS, "", "")]
#[case::two_on_one(TWO_ON_ONE, "", "d7")]
fn a_unit_is_threatened_when_the_exchange_on_its_square_wins_material(
    #[case] fen: &str,
    #[case] ours: &str,
    #[case] theirs: &str,
) {
    let threats = facts_of(fen).threats;
    assert_eq!(threats.threatened[Side::Us.index()], squares(ours));
    assert_eq!(threats.threatened[Side::Them.index()], squares(theirs));
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::hanging_knight(HANGING_KNIGHT, [0, 3])]
#[case::defended_knight(DEFENDED_KNIGHT, [0, 0])]
#[case::pawn_attacks_rook(PAWN_ATTACKS_ROOK, [5, 0])]
#[case::queen_under_knight(QUEEN_UNDER_KNIGHT, [9, 0])]
#[case::cross_threats(CROSS_THREATS, [1, 9])]
#[case::cross_threats_black(CROSS_THREATS_BLACK, [9, 1])]
#[case::removable_rook(REMOVABLE_ROOK, [0, 5])]
#[case::removable_rook_black(REMOVABLE_ROOK_BLACK, [5, 0])]
#[case::removable_knight(REMOVABLE_KNIGHT, [3, 3])]
#[case::removable_trade(REMOVABLE_TRADE, [3, 0])]
#[case::xray_bishop(XRAY_BISHOP, [3, 9])]
#[case::xray_bishop_black(XRAY_BISHOP_BLACK, [9, 3])]
fn the_threatened_value_adds_up_what_is_about_to_be_lost(
    #[case] fen: &str,
    #[case] value: [i32; 2],
) {
    let threats = facts_of(fen).threats;
    assert_eq!(threats.threatened_value[Side::Us.index()], value[0]);
    assert_eq!(threats.threatened_value[Side::Them.index()], value[1]);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::hanging_knight(HANGING_KNIGHT, [0, 3])]
#[case::defended_knight(DEFENDED_KNIGHT, [0, 0])]
#[case::pawn_attacks_rook(PAWN_ATTACKS_ROOK, [5, 0])]
#[case::queen_under_knight(QUEEN_UNDER_KNIGHT, [9, 0])]
#[case::cross_threats(CROSS_THREATS, [1, 9])]
#[case::cross_threats_black(CROSS_THREATS_BLACK, [9, 1])]
#[case::removable_rook(REMOVABLE_ROOK, [0, 5])]
#[case::removable_rook_black(REMOVABLE_ROOK_BLACK, [5, 0])]
#[case::removable_knight(REMOVABLE_KNIGHT, [3, 3])]
#[case::removable_trade(REMOVABLE_TRADE, [3, 0])]
#[case::xray_bishop(XRAY_BISHOP, [3, 9])]
#[case::doubled_rooks(DOUBLED_ROOKS, [0, 0])]
#[case::two_on_one(TWO_ON_ONE, [0, 5])]
fn the_max_gain_is_the_largest_exchange_the_opponent_can_start(
    #[case] fen: &str,
    #[case] gain: [i32; 2],
) {
    let threats = facts_of(fen).threats;
    assert_eq!(threats.threat_max_gain[Side::Us.index()], gain[0]);
    assert_eq!(threats.threat_max_gain[Side::Them.index()], gain[1]);
}

#[rstest]
#[case::start(START, "", "")]
#[case::hanging_knight(HANGING_KNIGHT, "", "")]
#[case::pawn_attacks_rook(PAWN_ATTACKS_ROOK, "e4", "")]
#[case::queen_under_knight(QUEEN_UNDER_KNIGHT, "e4", "")]
#[case::cross_threats(CROSS_THREATS, "", "d5")]
#[case::cross_threats_black(CROSS_THREATS_BLACK, "d5", "")]
#[case::removable_rook(REMOVABLE_ROOK, "", "d7")]
#[case::removable_rook_black(REMOVABLE_ROOK_BLACK, "d7", "")]
#[case::removable_knight(REMOVABLE_KNIGHT, "", "")]
#[case::xray_bishop(XRAY_BISHOP, "", "d5")]
#[case::xray_bishop_black(XRAY_BISHOP_BLACK, "d5", "")]
#[case::doubled_rooks(DOUBLED_ROOKS, "", "")]
fn a_lesser_attacker_is_one_the_defender_would_be_glad_to_trade_with(
    #[case] fen: &str,
    #[case] ours: &str,
    #[case] theirs: &str,
) {
    let threats = facts_of(fen).threats;
    assert_eq!(threats.attacked_by_lesser[Side::Us.index()], squares(ours));
    assert_eq!(
        threats.attacked_by_lesser[Side::Them.index()],
        squares(theirs)
    );
}

#[rstest]
#[case::start(START, [false, false])]
#[case::pawn_attacks_rook(PAWN_ATTACKS_ROOK, [false, false])]
#[case::queen_under_knight(QUEEN_UNDER_KNIGHT, [true, false])]
#[case::queen_under_knight_black(QUEEN_UNDER_KNIGHT_BLACK, [false, true])]
#[case::cross_threats(CROSS_THREATS, [false, true])]
#[case::cross_threats_black(CROSS_THREATS_BLACK, [true, false])]
#[case::xray_bishop(XRAY_BISHOP, [false, true])]
#[case::xray_bishop_black(XRAY_BISHOP_BLACK, [true, false])]
#[case::doubled_rooks(DOUBLED_ROOKS, [false, false])]
fn a_queen_under_a_lesser_unit_is_its_own_fact(#[case] fen: &str, #[case] under: [bool; 2]) {
    let threats = facts_of(fen).threats;
    assert_eq!(threats.queen_attacked_by_lesser[Side::Us.index()], under[0]);
    assert_eq!(
        threats.queen_attacked_by_lesser[Side::Them.index()],
        under[1]
    );
}

#[rstest]
#[case::start(START, "", "")]
#[case::defended_knight(DEFENDED_KNIGHT, "", "")]
#[case::overloaded_rook(OVERLOADED_ROOK, "", "d7")]
#[case::overloaded_rook_black(OVERLOADED_ROOK_BLACK, "d7", "")]
#[case::removable_rook(REMOVABLE_ROOK, "", "d7")]
#[case::overloaded_knight(OVERLOADED_KNIGHT, "", "c6")]
#[case::overloaded_knight_black(OVERLOADED_KNIGHT_BLACK, "c6", "")]
#[case::removable_knight(REMOVABLE_KNIGHT, "", "c6")]
#[case::removable_trade(REMOVABLE_TRADE, "", "c6")]
#[case::removable_trade_black(REMOVABLE_TRADE_BLACK, "c6", "")]
#[case::overloaded_pawn(OVERLOADED_PAWN, "", "b7")]
#[case::overloaded_pawn_black(OVERLOADED_PAWN_BLACK, "b7", "")]
#[case::doubled_rooks(DOUBLED_ROOKS, "", "")]
fn a_defender_of_two_attacked_units_is_overloaded(
    #[case] fen: &str,
    #[case] ours: &str,
    #[case] theirs: &str,
) {
    let threats = facts_of(fen).threats;
    assert_eq!(
        threats.overloaded_defenders[Side::Us.index()],
        squares(ours)
    );
    assert_eq!(
        threats.overloaded_defenders[Side::Them.index()],
        squares(theirs)
    );
}

#[rstest]
#[case::start(START, "", "")]
#[case::overloaded_rook(OVERLOADED_ROOK, "", "")]
#[case::removable_rook(REMOVABLE_ROOK, "", "d7")]
#[case::removable_rook_black(REMOVABLE_ROOK_BLACK, "d7", "")]
#[case::overloaded_knight(OVERLOADED_KNIGHT, "", "")]
#[case::removable_knight(REMOVABLE_KNIGHT, "", "c6")]
#[case::removable_knight_black(REMOVABLE_KNIGHT_BLACK, "c6", "")]
#[case::removable_trade(REMOVABLE_TRADE, "", "c6")]
#[case::removable_trade_black(REMOVABLE_TRADE_BLACK, "c6", "")]
#[case::overloaded_pawn(OVERLOADED_PAWN, "", "")]
#[case::overloaded_pawn_black(OVERLOADED_PAWN_BLACK, "", "")]
fn a_defender_the_enemy_can_take_for_free_is_removable(
    #[case] fen: &str,
    #[case] ours: &str,
    #[case] theirs: &str,
) {
    let threats = facts_of(fen).threats;
    assert_eq!(threats.removable_defenders[Side::Us.index()], squares(ours));
    assert_eq!(
        threats.removable_defenders[Side::Them.index()],
        squares(theirs)
    );
}

#[rstest]
#[case::start(START, "a1 h1", "a8 h8")]
#[case::hanging_knight(HANGING_KNIGHT, "", "d5")]
#[case::defended_knight(DEFENDED_KNIGHT, "", "c6")]
#[case::pawn_attacks_rook(PAWN_ATTACKS_ROOK, "e4", "d5")]
#[case::queen_under_knight(QUEEN_UNDER_KNIGHT, "e4", "f6")]
#[case::cross_threats(CROSS_THREATS, "e4", "d5")]
#[case::overloaded_rook(OVERLOADED_ROOK, "", "d7")]
#[case::removable_rook(REMOVABLE_ROOK, "h3", "d7")]
#[case::removable_trade(REMOVABLE_TRADE, "b4", "b7")]
#[case::overloaded_pawn(OVERLOADED_PAWN, "b3", "b8")]
#[case::xray_bishop(XRAY_BISHOP, "b3", "d5")]
#[case::xray_rook(XRAY_ROOK, "d1", "")]
#[case::doubled_rooks(DOUBLED_ROOKS, "", "")]
fn a_loose_unit_is_one_its_own_side_does_not_defend(
    #[case] fen: &str,
    #[case] ours: &str,
    #[case] theirs: &str,
) {
    let threats = facts_of(fen).threats;
    assert_eq!(threats.loose[Side::Us.index()], squares(ours));
    assert_eq!(threats.loose[Side::Them.index()], squares(theirs));
}

#[rstest]
#[case::start(START, "", "")]
#[case::defended_knight(DEFENDED_KNIGHT, "", "")]
#[case::pawn_attacks_rook(PAWN_ATTACKS_ROOK, "e4", "")]
#[case::queen_under_knight(QUEEN_UNDER_KNIGHT, "e4", "")]
#[case::cross_threats(CROSS_THREATS, "", "d5")]
#[case::cross_threats_black(CROSS_THREATS_BLACK, "d5", "")]
#[case::removable_rook(REMOVABLE_ROOK, "", "d7")]
#[case::removable_knight(REMOVABLE_KNIGHT, "b4", "c6")]
#[case::removable_trade(REMOVABLE_TRADE, "b4", "")]
#[case::xray_bishop(XRAY_BISHOP, "", "d5")]
#[case::overloaded_pawn(OVERLOADED_PAWN, "", "")]
#[case::doubled_rooks(DOUBLED_ROOKS, "", "")]
fn a_surplus_counts_only_the_attackers_and_defenders_worth_at_most_the_unit(
    #[case] fen: &str,
    #[case] ours: &str,
    #[case] theirs: &str,
) {
    let threats = facts_of(fen).threats;
    assert_eq!(threats.attacker_surplus[Side::Us.index()], squares(ours));
    assert_eq!(
        threats.attacker_surplus[Side::Them.index()],
        squares(theirs)
    );
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::defended_knight(DEFENDED_KNIGHT, [0, 0])]
#[case::overloaded_rook(OVERLOADED_ROOK, [1, 0])]
#[case::overloaded_rook_black(OVERLOADED_ROOK_BLACK, [0, 1])]
#[case::removable_rook(REMOVABLE_ROOK, [1, 0])]
#[case::xray_bishop(XRAY_BISHOP, [1, 0])]
#[case::xray_bishop_black(XRAY_BISHOP_BLACK, [0, 1])]
#[case::xray_rook(XRAY_ROOK, [1, 0])]
#[case::doubled_rooks(DOUBLED_ROOKS, [1, 1])]
#[case::overloaded_pawn(OVERLOADED_PAWN, [1, 0])]
#[case::bishop_battery(BISHOP_BATTERY, [0, 0])]
fn an_x_ray_looks_through_one_enemy_unit_at_another(#[case] fen: &str, #[case] count: [u8; 2]) {
    let threats = facts_of(fen).threats;
    assert_eq!(threats.xray_through_enemy[Side::Us.index()], count[0]);
    assert_eq!(threats.xray_through_enemy[Side::Them.index()], count[1]);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::xray_rook(XRAY_ROOK, [0, 0])]
#[case::overloaded_rook(OVERLOADED_ROOK, [1, 0])]
#[case::overloaded_rook_black(OVERLOADED_ROOK_BLACK, [0, 1])]
#[case::overloaded_knight(OVERLOADED_KNIGHT, [1, 0])]
#[case::overloaded_pawn(OVERLOADED_PAWN, [1, 0])]
#[case::doubled_rooks(DOUBLED_ROOKS, [1, 1])]
#[case::battery_at_king(BATTERY_AT_KING, [1, 0])]
#[case::battery_at_king_black(BATTERY_AT_KING_BLACK, [0, 1])]
#[case::bishop_battery(BISHOP_BATTERY, [1, 0])]
#[case::bishop_battery_black(BISHOP_BATTERY_BLACK, [0, 1])]
fn a_battery_is_two_sliders_on_one_line_they_both_move_along(
    #[case] fen: &str,
    #[case] count: [u8; 2],
) {
    let threats = facts_of(fen).threats;
    assert_eq!(threats.battery_count[Side::Us.index()], count[0]);
    assert_eq!(threats.battery_count[Side::Them.index()], count[1]);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::overloaded_rook(OVERLOADED_ROOK, [false, false])]
#[case::overloaded_knight(OVERLOADED_KNIGHT, [false, false])]
#[case::doubled_rooks(DOUBLED_ROOKS, [true, true])]
#[case::battery_at_king(BATTERY_AT_KING, [true, false])]
#[case::battery_at_king_black(BATTERY_AT_KING_BLACK, [false, true])]
#[case::bishop_battery(BISHOP_BATTERY, [true, false])]
#[case::bishop_battery_black(BISHOP_BATTERY_BLACK, [false, true])]
fn a_battery_at_the_king_is_one_whose_line_meets_the_enemy_ring(
    #[case] fen: &str,
    #[case] at_king: [bool; 2],
) {
    let threats = facts_of(fen).threats;
    assert_eq!(threats.battery_at_king[Side::Us.index()], at_king[0]);
    assert_eq!(threats.battery_at_king[Side::Them.index()], at_king[1]);
}

/// Nothing in the group depends on whose turn it is, so the same position with
/// the other side to move reads the two blocks the other way round.
#[rstest]
#[case::cross_threats(CROSS_THREATS, CROSS_THREATS_BLACK)]
#[case::removable_rook(REMOVABLE_ROOK, REMOVABLE_ROOK_BLACK)]
#[case::overloaded_pawn(OVERLOADED_PAWN, OVERLOADED_PAWN_BLACK)]
#[case::bishop_battery(BISHOP_BATTERY, BISHOP_BATTERY_BLACK)]
fn the_blocks_swap_with_the_side_to_move(#[case] white: &str, #[case] black: &str) {
    let ours = facts_of(white).threats;
    let theirs = facts_of(black).threats;
    let us = Side::Us.index();
    let them = Side::Them.index();
    assert_eq!(ours.threatened[us], theirs.threatened[them]);
    assert_eq!(ours.threatened_value[us], theirs.threatened_value[them]);
    assert_eq!(ours.threat_max_gain[us], theirs.threat_max_gain[them]);
    assert_eq!(
        ours.overloaded_defenders[us],
        theirs.overloaded_defenders[them]
    );
    assert_eq!(ours.loose[us], theirs.loose[them]);
    assert_eq!(ours.battery_count[us], theirs.battery_count[them]);
}

/// `en_prise` reads one attacker against one defender, `threatened` plays the
/// exchange out: two rooks against one win the defended rook that no cheaper
/// unit attacks.
#[test]
fn threatened_is_what_en_prise_approximates() {
    let facts = facts_of(TWO_ON_ONE);
    let them = Side::Them.index();
    assert_eq!(facts.attacks.en_prise[them], squares(""));
    assert_eq!(facts.threats.threatened[them], squares("d7"));

    // The other way round on the same board: a knight a pawn defends is en
    // prise to nothing cheaper, and the rook that attacks it wins nothing.
    let facts = facts_of(DEFENDED_KNIGHT);
    let them = Side::Them.index();
    assert_eq!(facts.attacks.en_prise[them], squares(""));
    assert_eq!(facts.threats.threatened[them], squares(""));
}
