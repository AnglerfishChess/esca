//! The `material` group, fact by fact.
//!
//! Every expectation is worked out from the definitions in `docs/features.md`
//! for the named position above it. A per-side case writes White first.

mod common;

use common::{by_colour, facts_of, facts_under};
use esca::CHESS960;
use rstest::rstest;

/// The untouched array: a full set a side, and a phase of exactly 1.
const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// A queen too many a side: 28 phase points, capped back to a full opening.
const TWO_QUEENS_EACH: &str = "r1bqk2r/ppp1qppp/2n5/8/8/2N5/PPP1QPPP/R1BQK2R w KQkq - 0 1";

/// Queens, all four rooks and three minors: nineteen points, still an opening.
const HEAVY_AND_A_KNIGHT: &str = "r3k2r/pppq1ppp/2n5/8/8/8/PPPQ1PPP/RNB1K2R w KQkq - 0 1";

/// One minor fewer: eighteen points, the top of the middlegame bucket.
const HEAVY: &str = "r3k2r/pppq1ppp/2n5/8/8/8/PPPQ1PPP/R1B1K2R w KQkq - 0 1";

/// The array with both queens gone: two thirds of a full set.
const QUEENLESS: &str = "rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNB1KBNR w KQkq - 0 1";

/// A whole army against a bare king: both differences run past their scale.
const ARMY_AGAINST_A_KING: &str = "4k3/8/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ - 0 1";

/// A queen and five pawns against a rook and three: six points, the bottom of
/// the middlegame bucket.
const QUEEN_FOR_A_ROOK: &str = "r3k3/5ppp/8/8/8/8/PP3PPP/3QK3 w - - 0 1";

/// The same placement read from the other side: every difference changes sign.
const QUEEN_FOR_A_ROOK_THEIRS: &str = "r3k3/5ppp/8/8/8/8/PP3PPP/3QK3 b - - 0 1";

/// A knight and a pawn up in a rook ending: five points is already an endgame.
const ROOK_AND_KNIGHT: &str = "r3k3/pp3ppp/8/8/8/8/PPP2PPP/4K1NR w Kq - 0 1";

/// Two dark-squared bishops against two knights: only the bishops cannot mate.
const SAME_COLOUR_BISHOPS: &str = "4k3/8/8/1n1n4/8/2B1B3/8/4K3 w - - 0 1";

/// Bishops of both colours against a lone knight: only the knight cannot mate.
const BISHOP_PAIR: &str = "4k3/6n1/8/8/8/8/8/2B1KB2 w - - 0 1";

/// A bare knight against a rook and a pawn.
const LONE_KNIGHT: &str = "r3k3/5p2/8/8/8/5N2/8/4K3 w - - 0 1";

/// Kings and pawns, one pawn apart, and nothing to count phase with.
const PAWN_ENDING: &str = "4k3/pp4p1/8/8/8/8/P4PPP/4K3 w - - 0 1";

/// Nothing but the kings: every count zero, and neither side able to mate.
const BARE_KINGS: &str = "8/8/4k3/8/8/4K3/8/8 w - - 0 1";

/// A Chess960 middlegame; no material fact reads the back rank.
const NINE_SIXTY: &str = "nnqrkr1b/1p1pp3/3p4/p4ppp/PP5P/1b2PP2/2PPK1P1/N1QR1RBB w fd - 0 10";

#[rstest]
#[case::start(START, [8, 2, 2, 2, 1], [8, 2, 2, 2, 1])]
#[case::two_queens_each(TWO_QUEENS_EACH, [6, 1, 1, 2, 2], [6, 1, 1, 2, 2])]
#[case::heavy(HEAVY, [6, 0, 1, 2, 1], [6, 1, 0, 2, 1])]
#[case::army_against_a_king(ARMY_AGAINST_A_KING, [8, 2, 2, 2, 1], [0; 5])]
#[case::queen_for_a_rook(QUEEN_FOR_A_ROOK, [5, 0, 0, 0, 1], [3, 0, 0, 1, 0])]
#[case::queen_for_a_rook_theirs(QUEEN_FOR_A_ROOK_THEIRS, [5, 0, 0, 0, 1], [3, 0, 0, 1, 0])]
#[case::rook_and_knight(ROOK_AND_KNIGHT, [6, 1, 0, 1, 0], [5, 0, 0, 1, 0])]
#[case::pawn_ending(PAWN_ENDING, [4, 0, 0, 0, 0], [3, 0, 0, 0, 0])]
#[case::bare_kings(BARE_KINGS, [0; 5], [0; 5])]
fn the_units_of_a_side_are_counted_by_role(
    #[case] fen: &str,
    #[case] white: [u8; 5],
    #[case] black: [u8; 5],
) {
    let material = facts_of(fen).material;
    assert_eq!(material.pawns, by_colour([white[0], black[0]]));
    assert_eq!(material.knights, by_colour([white[1], black[1]]));
    assert_eq!(material.bishops, by_colour([white[2], black[2]]));
    assert_eq!(material.rooks, by_colour([white[3], black[3]]));
    assert_eq!(material.queens, by_colour([white[4], black[4]]));
}

#[rstest]
#[case::start(START, [31, 31])]
#[case::two_queens_each(TWO_QUEENS_EACH, [34, 34])]
#[case::heavy_and_a_knight(HEAVY_AND_A_KNIGHT, [25, 22])]
#[case::queenless(QUEENLESS, [22, 22])]
#[case::army_against_a_king(ARMY_AGAINST_A_KING, [31, 0])]
#[case::queen_for_a_rook(QUEEN_FOR_A_ROOK, [9, 5])]
#[case::queen_for_a_rook_theirs(QUEEN_FOR_A_ROOK_THEIRS, [9, 5])]
#[case::rook_and_knight(ROOK_AND_KNIGHT, [8, 5])]
#[case::pawn_ending(PAWN_ENDING, [0, 0])]
fn non_pawn_material_leaves_out_the_pawns_and_the_king(
    #[case] fen: &str,
    #[case] non_pawn_value: [i32; 2],
) {
    assert_eq!(
        facts_of(fen).material.non_pawn_value,
        by_colour(non_pawn_value)
    );
}

#[rstest]
#[case::start(START, [39, 39])]
#[case::two_queens_each(TWO_QUEENS_EACH, [40, 40])]
#[case::heavy_and_a_knight(HEAVY_AND_A_KNIGHT, [31, 28])]
#[case::army_against_a_king(ARMY_AGAINST_A_KING, [39, 0])]
#[case::queen_for_a_rook(QUEEN_FOR_A_ROOK, [14, 8])]
#[case::queen_for_a_rook_theirs(QUEEN_FOR_A_ROOK_THEIRS, [14, 8])]
#[case::rook_and_knight(ROOK_AND_KNIGHT, [14, 10])]
#[case::lone_knight(LONE_KNIGHT, [3, 6])]
#[case::pawn_ending(PAWN_ENDING, [4, 3])]
#[case::bare_kings(BARE_KINGS, [0, 0])]
fn the_value_of_a_side_is_the_sum_of_its_units_but_the_king(
    #[case] fen: &str,
    #[case] value: [i32; 2],
) {
    assert_eq!(facts_of(fen).material.value, by_colour(value));
}

#[rstest]
#[case::start(START, 24)]
#[case::two_queens_each(TWO_QUEENS_EACH, 28)]
#[case::heavy_and_a_knight(HEAVY_AND_A_KNIGHT, 19)]
#[case::heavy(HEAVY, 18)]
#[case::queenless(QUEENLESS, 16)]
#[case::army_against_a_king(ARMY_AGAINST_A_KING, 12)]
#[case::queen_for_a_rook(QUEEN_FOR_A_ROOK, 6)]
#[case::rook_and_knight(ROOK_AND_KNIGHT, 5)]
#[case::same_colour_bishops(SAME_COLOUR_BISHOPS, 4)]
#[case::bishop_pair(BISHOP_PAIR, 3)]
#[case::pawn_ending(PAWN_ENDING, 0)]
fn the_phase_points_count_what_is_left_of_a_full_opening_set(
    #[case] fen: &str,
    #[case] phase_points: u8,
) {
    assert_eq!(facts_of(fen).material.phase_points, phase_points);
}

#[rstest]
#[case::start(START, true)]
#[case::two_queens_each(TWO_QUEENS_EACH, true)]
#[case::heavy(HEAVY, true)]
#[case::queenless(QUEENLESS, false)]
#[case::army_against_a_king(ARMY_AGAINST_A_KING, false)]
#[case::queen_for_a_rook(QUEEN_FOR_A_ROOK, false)]
#[case::queen_for_a_rook_theirs(QUEEN_FOR_A_ROOK_THEIRS, false)]
#[case::bare_kings(BARE_KINGS, false)]
fn both_queens_asks_for_a_queen_on_each_side(#[case] fen: &str, #[case] both_queens: bool) {
    assert_eq!(facts_of(fen).material.both_queens, both_queens);
}

#[rstest]
#[case::start(START, false)]
#[case::queenless(QUEENLESS, false)]
#[case::army_against_a_king(ARMY_AGAINST_A_KING, false)]
#[case::rook_and_knight(ROOK_AND_KNIGHT, false)]
#[case::lone_knight(LONE_KNIGHT, false)]
#[case::pawn_ending(PAWN_ENDING, true)]
#[case::bare_kings(BARE_KINGS, true)]
fn pawns_only_leaves_the_board_to_the_kings_and_the_pawns(
    #[case] fen: &str,
    #[case] pawns_only: bool,
) {
    assert_eq!(facts_of(fen).material.pawns_only, pawns_only);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::queen_for_a_rook(QUEEN_FOR_A_ROOK, [false, false])]
#[case::army_against_a_king(ARMY_AGAINST_A_KING, [false, true])]
#[case::same_colour_bishops(SAME_COLOUR_BISHOPS, [true, false])]
#[case::bishop_pair(BISHOP_PAIR, [false, true])]
#[case::lone_knight(LONE_KNIGHT, [true, false])]
#[case::pawn_ending(PAWN_ENDING, [false, false])]
#[case::bare_kings(BARE_KINGS, [true, true])]
fn a_side_holding_at_most_a_minor_or_bishops_of_one_colour_cannot_mate(
    #[case] fen: &str,
    #[case] insufficient: [bool; 2],
) {
    assert_eq!(
        facts_of(fen).material.insufficient_material,
        by_colour(insufficient)
    );
}

/// No `material` fact reads the back rank, so a Chess960 position answers
/// exactly as the same placement would.
#[test]
fn the_material_facts_of_a_chess960_position_are_the_classic_ones() {
    let facts = facts_under(&CHESS960, NINE_SIXTY);
    let material = facts.material;
    assert_eq!(material.pawns, by_colour([8, 8]));
    assert_eq!(material.knights, by_colour([1, 2]));
    assert_eq!(material.bishops, by_colour([2, 2]));
    assert_eq!(material.rooks, by_colour([2, 2]));
    assert_eq!(material.queens, by_colour([1, 1]));
    assert_eq!(material.non_pawn_value, by_colour([28, 31]));
    assert_eq!(material.value, by_colour([36, 39]));
    assert_eq!(material.phase_points, 23);
    assert!(material.both_queens);
    assert!(!material.pawns_only);
    assert_eq!(material.insufficient_material, by_colour([false, false]));

    let classic = facts_of("nnqrkr1b/1p1pp3/3p4/p4ppp/PP5P/1b2PP2/2PPK1P1/N1QR1RBB w - - 0 10");
    assert_eq!(classic.material, material);
}
