//! The `history` group, fact by fact.
//!
//! Every expectation is worked out from the definitions in `docs/features.md`
//! §2.13 for the named position above it. Everything but the halfmove clock is
//! a fact of a game, so those cases play the moves that make them true.

mod common;

use common::{facts_of, facts_under};
use esca::{CHESS960, Game, Role, Schema, classic};
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

/// A rook and a knight develop: nothing is taken and no check is given.
const QUIET: [&str; 4] = ["e2e4", "e7e5", "g1f3", "b8c6"];

/// The Scandinavian up to the pawn capture: White is a pawn ahead.
const SCANDI3: [&str; 3] = ["e2e4", "d7d5", "e4d5"];

/// One ply further, the queen recaptures and the material is level again.
const SCANDI4: [&str; 4] = ["e2e4", "d7d5", "e4d5", "d8d5"];

/// The gambit line, eight plies in: White keeps the pawn and both develop.
const SCANDI8: [&str; 8] = [
    "e2e4", "d7d5", "e4d5", "g8f6", "b1c3", "b8c6", "g1f3", "c8f5",
];

/// Four plies further still, so the capture has left the eight-ply window.
const SCANDI12: [&str; 12] = [
    "e2e4", "d7d5", "e4d5", "g8f6", "b1c3", "b8c6", "g1f3", "c8f5", "f1b5", "e7e6", "e1h1", "f8e7",
];

/// A knight takes the pawn back: the last move is a knight's and a capture.
const KNIGHT_TAKES: [&str; 6] = ["e2e4", "d7d5", "e4d5", "g8f6", "d2d4", "f6d5"];

/// The rook swings to b8 and checks the king that has just stepped to b1.
const CHECK: [&str; 2] = ["a1b1", "h8b8"];

/// The same check, two plies back.
const CHECK_AGO: [&str; 4] = ["a1b1", "h8b8", "b1a1", "b8h8"];

/// A rook takes a rook and checks the king beside it.
const ROOK_TAKES: &str = "3rk3/8/8/8/8/8/3R4/4K3 w - - 0 1";

/// The one move of the rook game.
const ROOK_TAKES_MOVES: [&str; 1] = ["d2d8"];

#[rstest]
#[case::fresh(START, &[], 0)]
#[case::quiet(START, &QUIET, 0)]
#[case::scandi3(START, &SCANDI3, 1)]
#[case::scandi4(START, &SCANDI4, 2)]
#[case::scandi8(START, &SCANDI8, 1)]
#[case::scandi12(START, &SCANDI12, 0)]
#[case::knight_takes(START, &KNIGHT_TAKES, 2)]
#[case::rook_takes(ROOK_TAKES, &ROOK_TAKES_MOVES, 1)]
fn the_captures_counted_are_those_of_the_last_eight_plies(
    #[case] fen: &str,
    #[case] moves: &[&str],
    #[case] captures: u8,
) {
    assert_eq!(
        game_of(fen, moves).facts().history.captures_in_last_8,
        captures
    );
}

#[rstest]
#[case::fresh(START, &[], 0)]
#[case::quiet(START, &QUIET, 0)]
#[case::scandi8(START, &SCANDI8, 0)]
#[case::check(SHUFFLE, &CHECK, 1)]
#[case::check_ago(SHUFFLE, &CHECK_AGO, 1)]
#[case::rook_takes(ROOK_TAKES, &ROOK_TAKES_MOVES, 1)]
fn the_checks_counted_are_those_of_the_last_eight_plies(
    #[case] fen: &str,
    #[case] moves: &[&str],
    #[case] checks: u8,
) {
    assert_eq!(game_of(fen, moves).facts().history.checks_in_last_8, checks);
}

#[rstest]
#[case::fresh(START, &[], 0)]
#[case::quiet(START, &QUIET, 4)]
#[case::scandi3(START, &SCANDI3, 0)]
#[case::scandi8(START, &SCANDI8, 5)]
#[case::scandi12(START, &SCANDI12, 9)]
#[case::check(SHUFFLE, &CHECK, 0)]
#[case::check_ago(SHUFFLE, &CHECK_AGO, 2)]
fn the_quiet_plies_are_those_since_the_last_capture_or_check(
    #[case] fen: &str,
    #[case] moves: &[&str],
    #[case] plies: u32,
) {
    assert_eq!(game_of(fen, moves).facts().history.quiet_plies, plies);
}

#[rstest]
#[case::fresh(START, &[], 0)]
#[case::quiet(START, &QUIET, 0)]
#[case::scandi3(START, &SCANDI3, -1)]
#[case::scandi4(START, &SCANDI4, 0)]
#[case::scandi8(START, &SCANDI8, 1)]
#[case::scandi12(START, &SCANDI12, 0)]
#[case::rook_takes(ROOK_TAKES, &ROOK_TAKES_MOVES, -5)]
fn the_material_trend_is_what_the_last_eight_plies_have_won_or_lost(
    #[case] fen: &str,
    #[case] moves: &[&str],
    #[case] trend: i32,
) {
    assert_eq!(game_of(fen, moves).facts().history.material_trend, trend);
}

#[rstest]
#[case::fresh(START, &[], None)]
#[case::quiet(START, &QUIET, None)]
#[case::scandi3(START, &SCANDI3, Some(Role::Pawn))]
#[case::scandi4(START, &SCANDI4, Some(Role::Pawn))]
#[case::knight_takes(START, &KNIGHT_TAKES, Some(Role::Pawn))]
#[case::rook_takes(ROOK_TAKES, &ROOK_TAKES_MOVES, Some(Role::Rook))]
fn the_last_victim_is_the_role_the_last_move_took(
    #[case] fen: &str,
    #[case] moves: &[&str],
    #[case] victim: Option<Role>,
) {
    assert_eq!(game_of(fen, moves).facts().history.last_move_victim, victim);
}

#[rstest]
#[case::fresh(START, &[], None)]
#[case::quiet(START, &QUIET, Some(Role::Knight))]
#[case::scandi3(START, &SCANDI3, Some(Role::Pawn))]
#[case::scandi4(START, &SCANDI4, Some(Role::Queen))]
#[case::scandi8(START, &SCANDI8, Some(Role::Bishop))]
#[case::knight_takes(START, &KNIGHT_TAKES, Some(Role::Knight))]
#[case::check(SHUFFLE, &CHECK, Some(Role::Rook))]
fn the_last_mover_is_the_role_that_made_the_last_move(
    #[case] fen: &str,
    #[case] moves: &[&str],
    #[case] mover: Option<Role>,
) {
    assert_eq!(game_of(fen, moves).facts().history.last_move_mover, mover);
}

/// A position on its own knows its clock and nothing else about the plies
/// before it.
#[rstest]
#[case::scandi4(START, &SCANDI4)]
#[case::check(SHUFFLE, &CHECK)]
#[case::rook_takes(ROOK_TAKES, &ROOK_TAKES_MOVES)]
fn a_position_on_its_own_carries_none_of_the_recent_play(
    #[case] fen: &str,
    #[case] moves: &[&str],
) {
    let game = game_of(fen, moves);
    let bare = facts_of(&game.position().fen()).history;

    assert!(!bare.known);
    assert_eq!(bare.captures_in_last_8, 0);
    assert_eq!(bare.checks_in_last_8, 0);
    assert_eq!(bare.quiet_plies, 0);
    assert_eq!(bare.material_trend, 0);
    assert_eq!(bare.last_move_victim, None);
    assert_eq!(bare.last_move_mover, None);
    assert_eq!(bare.halfmove_clock, game.position().halfmove_clock());
}
