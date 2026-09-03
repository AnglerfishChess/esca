//! The `history` group, fact by fact.
//!
//! Every expectation is worked out from the definitions in `docs/features.md`
//! §2.13 for the named position above it. Everything but the halfmove clock is
//! a fact of a game, so those cases play the moves that make them true.

mod common;

use common::{facts_of, facts_under};
use esca::{CHESS960, Game, Schema, classic};
use rstest::rstest;

/// The untouched array: a fresh clock and nothing to repeat.
const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// The same array as the evaluation dump writes it: four fields, no clocks.
const START_NO_CLOCKS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -";

/// A rook endgame stripped of its clocks; the clock cases append their own.
const NO_CLOCKS: &str = "4k3/8/8/8/8/8/R7/4K3 w - -";

/// The same endgame 45 plies into a shuffle.
const CLOCK_45: &str = "4k3/8/8/8/8/8/R7/4K3 w - - 45 60";

/// Kings and rooks at home with every right spent, ten plies into a shuffle.
const RIGHTS_NONE: &str = "r3k2r/8/8/8/8/8/8/R3K2R b - - 10 30";

/// A queen down the open e-file: a check, six plies into the fifty-move count.
const QUEEN_CHECK: &str = "4k3/8/8/4q3/8/8/8/4K3 w - - 6 40";

/// Kings and a rook with room to walk: the repetition cases play from here.
const SHUFFLE: &str = "k6r/8/8/8/8/8/8/K7 w - - 0 1";

/// Chess960: kings on g between rooks on f and h, and a pawn just past b5.
const NINE_SIXTY: &str = "bqnb1rkr/pp1ppppp/8/1Pp5/8/8/P1PPPPPP/BQNB1RKR w HFhf c6 0 3";

/// The `halfmove_bucket` one-hot opens the `history` row (`features.md` §2.13).
const HALFMOVE_BUCKET_AT: usize = 0;

/// The classic game `fen` starts, after the UCI `moves`.
fn game_of(fen: &str, moves: &[&str]) -> Game {
    let mut game = Game::from_fen(classic(), fen).expect("a test FEN is a legal position");
    for uci in moves {
        game.play_uci(uci).expect("a case names only legal moves");
    }
    game
}

/// The bucket the encoded `history` row's one-hot sets for `fen`.
fn halfmove_bucket(fen: &str) -> usize {
    let schema = Schema::v1();
    let groups = schema.group_set(&["history"]).expect("the history group");
    let row = facts_of(fen).encode(schema, groups);
    row[HALFMOVE_BUCKET_AT..HALFMOVE_BUCKET_AT + 8]
        .iter()
        .position(|&value| value == 1.0)
        .expect("a one-hot sets exactly one value")
}

#[rstest]
#[case::clock_0(0, 0)]
#[case::clock_1(1, 1)]
#[case::clock_3(3, 1)]
#[case::clock_4(4, 2)]
#[case::clock_9(9, 2)]
#[case::clock_10(10, 3)]
#[case::clock_19(19, 3)]
#[case::clock_20(20, 4)]
#[case::clock_39(39, 4)]
#[case::clock_40(40, 5)]
#[case::clock_69(69, 5)]
#[case::clock_70(70, 6)]
#[case::clock_89(89, 6)]
#[case::clock_90(90, 7)]
#[case::clock_100(100, 7)]
fn the_halfmove_clock_falls_in_the_bucket_whose_range_holds_it(
    #[case] clock: u32,
    #[case] bucket: usize,
) {
    let fen = format!("{NO_CLOCKS} {clock} 60");
    assert_eq!(facts_of(&fen).history.halfmove_clock, clock);
    assert_eq!(halfmove_bucket(&fen), bucket);
}

#[rstest]
#[case::start(START, true, 0)]
#[case::start_no_clocks(START_NO_CLOCKS, false, 0)]
#[case::no_clocks(NO_CLOCKS, false, 0)]
#[case::clock_45(CLOCK_45, true, 45)]
#[case::rights_none(RIGHTS_NONE, true, 10)]
fn a_clock_is_known_only_from_a_fen_that_carries_one(
    #[case] fen: &str,
    #[case] known: bool,
    #[case] clock: u32,
) {
    let history = facts_of(fen).history;
    assert_eq!(history.halfmove_known, known);
    assert_eq!(history.halfmove_clock, clock);
}

#[rstest]
#[case::start_fresh(START, &[], false)]
#[case::start_knight_out(START, &["g1f3"], false)]
#[case::start_knights_home(START, &["g1f3", "g8f6", "f3g1", "f6g8"], true)]
#[case::shuffle_fresh(SHUFFLE, &[], false)]
#[case::shuffle_rook_h6(SHUFFLE, &["a1b1", "h8h7", "b1a1", "h7h6"], false)]
#[case::shuffle_rook_back(SHUFFLE, &["a1b1", "h8h7", "b1a1", "h7h8"], true)]
#[case::shuffle_rook_back_twice(
    SHUFFLE,
    &["a1b1", "h8h7", "b1a1", "h7h8", "a1b1", "h8h7", "b1a1", "h7h8"],
    true
)]
fn a_position_the_game_has_already_held_is_a_repetition(
    #[case] fen: &str,
    #[case] moves: &[&str],
    #[case] seen: bool,
) {
    assert_eq!(game_of(fen, moves).facts().history.repetition_seen, seen);
}

#[rstest]
#[case::start_fresh(START, &[], false)]
#[case::start_knight_out(START, &["g1f3"], false)]
#[case::start_three_of_four(START, &["g1f3", "g8f6", "f3g1"], true)]
#[case::start_knights_home(START, &["g1f3", "g8f6", "f3g1", "f6g8"], true)]
#[case::shuffle_three_plies(SHUFFLE, &["a1b1", "h8h7", "b1a1"], true)]
#[case::shuffle_rook_back(SHUFFLE, &["a1b1", "h8h7", "b1a1", "h7h8"], true)]
#[case::shuffle_rook_h6(SHUFFLE, &["a1b1", "h8h7", "b1a1", "h7h6"], false)]
fn a_repetition_is_available_when_one_of_our_moves_reaches_the_history(
    #[case] fen: &str,
    #[case] moves: &[&str],
    #[case] available: bool,
) {
    assert_eq!(
        game_of(fen, moves).facts().history.repetition_available,
        available
    );
}

#[rstest]
#[case::start(START)]
#[case::shuffle(SHUFFLE)]
#[case::clock_45(CLOCK_45)]
#[case::queen_check(QUEEN_CHECK)]
fn a_history_is_known_only_to_the_game_that_holds_it(#[case] fen: &str) {
    assert!(!facts_of(fen).history.known);
    assert!(game_of(fen, &[]).facts().history.known);
}

#[rstest]
#[case::start_knights_home(START, &["g1f3", "g8f6", "f3g1", "f6g8"])]
#[case::shuffle_rook_back(SHUFFLE, &["a1b1", "h8h7", "b1a1", "h7h8"])]
#[case::shuffle_three_plies(SHUFFLE, &["a1b1", "h8h7", "b1a1"])]
fn a_position_on_its_own_carries_none_of_the_repetition_facts(
    #[case] fen: &str,
    #[case] moves: &[&str],
) {
    let game = game_of(fen, moves);
    let played = game.facts().history;
    let bare = facts_of(&game.position().fen()).history;

    assert!(
        played.repetition_seen || played.repetition_available,
        "the game sees something to repeat here"
    );
    assert!(!bare.known);
    assert!(!bare.repetition_seen);
    assert!(!bare.repetition_available);
}

/// No `history` fact is among the four `features.md` §4 defines for classic
/// chess only, and a Chess960 position carries its clock like any other.
#[test]
fn the_history_facts_of_a_chess960_position_are_the_clock_it_carries() {
    let history = facts_under(&CHESS960, NINE_SIXTY).history;
    assert_eq!(history.halfmove_clock, 0);
    assert!(history.halfmove_known);
    assert!(!history.known);
    assert!(!history.repetition_seen);
    assert!(!history.repetition_available);
}
