//! The `exchange` group, and the static exchange evaluation under it.
//!
//! Every expectation is worked out from the definitions in `docs/features.md`
//! §1 and §2.9 for the named position above it.

mod common;

use common::facts_of;
use esca::{CLASSIC, Position, Side, Variant};
use rstest::rstest;

/// A rook takes an undefended pawn.
const FREE_PAWN: &str = "4k3/8/8/3p4/8/8/3R4/4K3 w - - 0 1";

/// The same pawn, now defended by one of its own.
const DEFENDED_PAWN: &str = "4k3/8/2p5/3p4/8/8/3R4/4K3 w - - 0 1";

/// The same, with Black to move and nothing of its own to take.
const DEFENDED_PAWN_BLACK: &str = "4k3/8/2p5/3p4/8/8/3R4/4K3 b - - 0 1";

/// A pawn takes a pawn a pawn defends: the even trade.
const PAWN_TRADE: &str = "4k3/8/2p5/3p4/4P3/8/8/4K3 w - - 0 1";

/// The same, with Black to move: taking on e4 wins a pawn nothing guards.
const PAWN_TRADE_BLACK: &str = "4k3/8/2p5/3p4/4P3/8/8/4K3 b - - 0 1";

/// A queen has to take the pawn the c6 pawn defends.
const QUEEN_TAKES_PAWN: &str = "4k3/8/2p5/3p4/8/8/3Q4/4K3 w - - 0 1";

/// Knight for knight, each defended by a pawn on neither side.
const KNIGHT_TRADE: &str = "4k3/8/2p5/3n4/8/2N5/8/4K3 w - - 0 1";

/// The same with Black to move: the c3 knight has nothing behind it.
const KNIGHT_TRADE_BLACK: &str = "4k3/8/2p5/3n4/8/2N5/8/4K3 b - - 0 1";

/// A knight wins a defended rook: the recapture is worth less than the prize.
const KNIGHT_TAKES_ROOK: &str = "4k3/8/2p5/3r4/8/2N5/8/4K3 w - - 0 1";

/// A rook takes a defended knight and loses the difference.
const ROOK_TAKES_KNIGHT: &str = "4k3/8/2p5/3n4/8/8/3R4/4K3 w - - 0 1";

/// Rook takes rook and the enemy king takes back: an even trade.
const KING_RECAPTURES: &str = "8/8/8/8/8/4k3/3r4/3R3K w - - 0 1";

/// The doubled rook covers d5, so the king may not take back at all.
const KING_REFUSED: &str = "8/8/4k3/3p4/8/8/3R4/3R3K w - - 0 1";

/// The same without the second rook: the king takes back and wins the rook.
const KING_TAKES_BACK: &str = "8/8/4k3/3p4/8/8/3R4/7K w - - 0 1";

/// Two rooks on the d-file: the second joins once the first has taken.
const XRAY_ROOKS: &str = "4k3/8/2p5/3p4/8/8/3R4/3R1K2 w - - 0 1";

/// The same with one rook, so nothing comes back after the recapture.
const ONE_ROOK: &str = "4k3/8/2p5/3p4/8/8/3R4/5K2 w - - 0 1";

/// Both sides double on the file; Black's rook has the last word.
const BOTH_BATTERIES: &str = "3rk3/8/2p5/3p4/8/8/3R4/3R1K2 w - - 0 1";

/// The bishop that defends d5 may not legally move: an exchange ignores pins.
const PINNED_DEFENDER: &str = "2k5/8/2b5/3p4/8/8/3R4/2R1K3 w - - 0 1";

/// A pawn takes a rook and promotes, with nothing to answer it.
const PROMOTION_FREE: &str = "r3k3/1P6/8/8/8/8/6K1/8 w - - 0 1";

/// The same, with the second rook ready to take the new queen.
const PROMOTION_DEFENDED: &str = "r3k3/1P6/8/8/8/8/6K1/r7 w - - 0 1";

/// The d-pawn has just run past e5, and nothing covers d6.
const EP_FREE: &str = "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1";

/// The same with the c7 pawn covering d6.
const EP_DEFENDED: &str = "4k3/2p5/8/3pP3/8/8/8/4K3 w - d6 0 1";

/// Three white units bear on e5, two black ones defend it.
const CROWD: &str = "4rk2/8/3p4/4p3/3P4/5N2/8/4RK2 w - - 0 1";

/// The same with Black to move: e5 takes d4 and the knight takes back.
const CROWD_BLACK: &str = "4rk2/8/3p4/4p3/3P4/5N2/8/4RK2 b - - 0 1";

/// A queen hangs on d5; the one on d2 stands beside its king.
const QUEEN_FREE: &str = "4k3/8/8/3q4/8/8/3Q4/4K3 w - - 0 1";

/// Both queens are defended by their kings.
const QUEEN_DEFENDED: &str = "8/8/4k3/3q4/8/8/3Q4/4K3 w - - 0 1";

/// A rook with an open board and one black pawn covering e5 and g5.
const ROOK_WALK: &str = "4k3/8/5p2/R7/8/8/8/4K3 w - - 0 1";

/// Nothing to capture: the short castling is the move to ask about.
const CASTLING: &str = "4k3/8/8/8/8/8/8/4K2R w K - 0 1";

/// The king itself takes the pawn on d2.
const KING_TAKES: &str = "4k3/8/8/8/8/8/3p4/3K4 w - - 0 1";

/// The untouched array: no capture for either side.
const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// A rook checks from e2 and the king may take it.
const IN_CHECK: &str = "4k3/8/8/8/8/8/4r3/4K3 w - - 0 1";

/// Pawns on c4 and e4 against d5 and f5, each side's pawns guarding the other's
/// targets.
const PAWN_CHAINS: &str = "4k3/8/2p3p1/3p1p2/2P1P3/8/8/4K3 w - - 0 1";

/// The same with Black to move.
const PAWN_CHAINS_BLACK: &str = "4k3/8/2p3p1/3p1p2/2P1P3/8/8/4K3 b - - 0 1";

/// The SEE of the move `uci` in `fen`, under classic chess.
fn see_of(fen: &str, uci: &str) -> i32 {
    let position = Position::from_fen(fen).expect("a test FEN is a legal position");
    let mv = CLASSIC
        .move_from_uci(&position, uci)
        .unwrap_or_else(|_| panic!("{uci} is a legal move of {fen}"));
    position.see_capture(mv)
}

/// The SEE of the unit on `square` in `fen`.
fn see_unit(fen: &str, square: &str) -> i32 {
    Position::from_fen(fen)
        .expect("a test FEN is a legal position")
        .see(square.parse().expect("a square name"))
}

#[rstest]
#[case::free_pawn(FREE_PAWN, "d2d5", 1)]
#[case::defended_pawn(DEFENDED_PAWN, "d2d5", -4)]
#[case::pawn_trade(PAWN_TRADE, "e4d5", 0)]
#[case::queen_takes_pawn(QUEEN_TAKES_PAWN, "d2d5", -8)]
#[case::knight_trade(KNIGHT_TRADE, "c3d5", 0)]
#[case::knight_takes_rook(KNIGHT_TAKES_ROOK, "c3d5", 2)]
#[case::rook_takes_knight(ROOK_TAKES_KNIGHT, "d2d5", -2)]
#[case::king_recaptures(KING_RECAPTURES, "d1d2", 0)]
#[case::king_refused(KING_REFUSED, "d2d5", 1)]
#[case::king_takes_back(KING_TAKES_BACK, "d2d5", -4)]
#[case::xray_rooks(XRAY_ROOKS, "d2d5", -3)]
#[case::one_rook(ONE_ROOK, "d2d5", -4)]
#[case::both_batteries(BOTH_BATTERIES, "d2d5", -4)]
#[case::pinned_defender(PINNED_DEFENDER, "d2d5", -4)]
#[case::promotion_free(PROMOTION_FREE, "b7a8q", 13)]
#[case::promotion_defended(PROMOTION_DEFENDED, "b7a8q", 4)]
#[case::en_passant_free(EP_FREE, "e5d6", 1)]
#[case::en_passant_defended(EP_DEFENDED, "e5d6", 0)]
#[case::crowd_pawn_first(CROWD, "d4e5", 1)]
#[case::crowd_knight_first(CROWD, "f3e5", -1)]
#[case::crowd_rook_first(CROWD, "e1e5", -3)]
#[case::crowd_black_pawn(CROWD_BLACK, "e5d4", 0)]
#[case::queen_free(QUEEN_FREE, "d2d5", 9)]
#[case::queen_defended(QUEEN_DEFENDED, "d2d5", 0)]
#[case::king_takes_a_pawn(KING_TAKES, "d1d2", 1)]
#[case::quiet_into_a_pawn(ROOK_WALK, "a5e5", -5)]
#[case::quiet_and_safe(ROOK_WALK, "a5a6", 0)]
#[case::castling(CASTLING, "e1h1", 0)]
fn an_exchange_is_played_out_with_the_least_valuable_attacker_each_time(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] see: i32,
) {
    assert_eq!(see_of(fen, uci), see);
}

#[rstest]
#[case::free_pawn(FREE_PAWN, "d5", 1)]
#[case::defended_pawn(DEFENDED_PAWN, "d5", 0)]
#[case::knight_takes_rook(KNIGHT_TAKES_ROOK, "d5", 2)]
#[case::queen_free(QUEEN_FREE, "d5", 9)]
#[case::queen_defended(QUEEN_DEFENDED, "d5", 0)]
#[case::crowd_pawn(CROWD, "e5", 1)]
#[case::our_own_rook(FREE_PAWN, "d2", 0)]
#[case::a_king(FREE_PAWN, "e1", 0)]
#[case::an_empty_square(FREE_PAWN, "a1", 0)]
fn the_see_of_a_unit_is_what_the_other_side_wins_by_taking_it(
    #[case] fen: &str,
    #[case] square: &str,
    #[case] see: i32,
) {
    assert_eq!(see_unit(fen, square), see);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::crowd(CROWD, [1, 0])]
#[case::crowd_black(CROWD_BLACK, [0, 1])]
#[case::queen_free(QUEEN_FREE, [9, 0])]
#[case::in_check(IN_CHECK, [5, 0])]
#[case::knight_trade(KNIGHT_TRADE, [0, 3])]
#[case::knight_trade_black(KNIGHT_TRADE_BLACK, [3, 0])]
#[case::defended_pawn(DEFENDED_PAWN, [-4, 0])]
#[case::defended_pawn_black(DEFENDED_PAWN_BLACK, [0, -4])]
#[case::pawn_chains(PAWN_CHAINS, [1, 1])]
#[case::pawn_chains_black(PAWN_CHAINS_BLACK, [1, 1])]
#[case::pawn_trade_black(PAWN_TRADE_BLACK, [1, 0])]
fn the_best_capture_is_the_largest_see_the_side_has(#[case] fen: &str, #[case] best: [i32; 2]) {
    let exchange = facts_of(fen).exchange;
    assert_eq!(exchange[Side::Us.index()].see_best_capture, best[0]);
    assert_eq!(exchange[Side::Them.index()].see_best_capture, best[1]);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::crowd(CROWD, [1, 0])]
#[case::crowd_black(CROWD_BLACK, [0, 1])]
#[case::queen_free(QUEEN_FREE, [1, 0])]
#[case::in_check(IN_CHECK, [1, 0])]
#[case::knight_trade(KNIGHT_TRADE, [0, 1])]
#[case::knight_trade_black(KNIGHT_TRADE_BLACK, [1, 0])]
#[case::defended_pawn(DEFENDED_PAWN, [0, 0])]
#[case::defended_pawn_black(DEFENDED_PAWN_BLACK, [0, 0])]
#[case::pawn_chains(PAWN_CHAINS, [2, 3])]
#[case::pawn_chains_black(PAWN_CHAINS_BLACK, [3, 2])]
#[case::pawn_trade_black(PAWN_TRADE_BLACK, [1, 0])]
fn a_positive_capture_is_one_that_wins_material_outright(
    #[case] fen: &str,
    #[case] count: [u16; 2],
) {
    let exchange = facts_of(fen).exchange;
    assert_eq!(
        exchange[Side::Us.index()].see_positive_capture_count,
        count[0]
    );
    assert_eq!(
        exchange[Side::Them.index()].see_positive_capture_count,
        count[1]
    );
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::crowd(CROWD, [0, 1])]
#[case::crowd_black(CROWD_BLACK, [1, 0])]
#[case::queen_free(QUEEN_FREE, [0, 1])]
#[case::in_check(IN_CHECK, [0, 0])]
#[case::knight_trade(KNIGHT_TRADE, [1, 0])]
#[case::knight_trade_black(KNIGHT_TRADE_BLACK, [0, 1])]
#[case::defended_pawn(DEFENDED_PAWN, [0, 0])]
#[case::defended_pawn_black(DEFENDED_PAWN_BLACK, [0, 0])]
#[case::pawn_chains(PAWN_CHAINS, [1, 0])]
#[case::pawn_chains_black(PAWN_CHAINS_BLACK, [0, 1])]
#[case::pawn_trade_black(PAWN_TRADE_BLACK, [0, 1])]
fn an_equal_capture_is_one_the_exchange_leaves_level(#[case] fen: &str, #[case] count: [u16; 2]) {
    let exchange = facts_of(fen).exchange;
    assert_eq!(exchange[Side::Us.index()].see_equal_capture_count, count[0]);
    assert_eq!(
        exchange[Side::Them.index()].see_equal_capture_count,
        count[1]
    );
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::crowd(CROWD, [1, 0])]
#[case::crowd_black(CROWD_BLACK, [0, 1])]
#[case::queen_free(QUEEN_FREE, [9, 0])]
#[case::in_check(IN_CHECK, [5, 0])]
#[case::knight_trade(KNIGHT_TRADE, [0, 3])]
#[case::knight_trade_black(KNIGHT_TRADE_BLACK, [3, 0])]
#[case::defended_pawn(DEFENDED_PAWN, [0, 0])]
#[case::defended_pawn_black(DEFENDED_PAWN_BLACK, [0, 0])]
#[case::pawn_chains(PAWN_CHAINS, [2, 3])]
#[case::pawn_chains_black(PAWN_CHAINS_BLACK, [3, 2])]
#[case::pawn_trade_black(PAWN_TRADE_BLACK, [1, 0])]
fn the_positive_total_adds_up_the_captures_that_win_material(
    #[case] fen: &str,
    #[case] total: [i32; 2],
) {
    let exchange = facts_of(fen).exchange;
    assert_eq!(exchange[Side::Us.index()].see_positive_total, total[0]);
    assert_eq!(exchange[Side::Them.index()].see_positive_total, total[1]);
}

/// In check there is no null move, so the `them` block is zero and
/// `tactics.them` says why.
#[test]
fn the_them_block_is_empty_when_we_are_in_check() {
    let facts = facts_of(IN_CHECK);
    assert!(facts.state.in_check);
    assert!(!facts.tactics[Side::Them.index()].available);
    let theirs = facts.exchange[Side::Them.index()];
    assert_eq!(theirs.see_best_capture, 0);
    assert_eq!(theirs.see_positive_capture_count, 0);
    assert_eq!(theirs.see_equal_capture_count, 0);
    assert_eq!(theirs.see_positive_total, 0);
}

/// The block's counts and the move list agree: every capture falls in exactly
/// one of the three classes.
#[rstest]
#[case::crowd(CROWD)]
#[case::pawn_chains(PAWN_CHAINS)]
#[case::queen_free(QUEEN_FREE)]
#[case::start(START)]
fn the_three_counts_partition_the_captures(#[case] fen: &str) {
    let facts = facts_of(fen);
    let position = Position::from_fen(fen).expect("a test FEN is a legal position");
    let ours = facts.exchange[Side::Us.index()];
    let captures: Vec<i32> = facts
        .moves
        .iter()
        .filter(|annotated| annotated.facts.victim.is_some())
        .map(|annotated| position.see_capture(annotated.mv))
        .collect();

    let negative = captures.iter().filter(|see| **see < 0).count() as u16;
    assert_eq!(
        ours.see_positive_capture_count + ours.see_equal_capture_count + negative,
        captures.len() as u16
    );
    assert_eq!(
        ours.see_positive_total,
        captures.iter().filter(|see| **see > 0).sum::<i32>()
    );
    assert_eq!(
        ours.see_best_capture,
        captures.iter().copied().max().unwrap_or(0)
    );
}
