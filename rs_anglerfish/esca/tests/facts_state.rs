//! The `state` group, fact by fact.
//!
//! Every expectation is worked out from the definitions in `docs/features.md`
//! §2.1 for the named position above it. Repetition and history are facts of a
//! game, so those cases play the moves that make them true.

mod common;

use common::{facts_of, facts_under};
use esca::{CHESS960, CLASSIC, File, Game, Schema, Side, Variant, classic};
use rstest::rstest;

/// The untouched array: every right, a fresh clock, nothing to repeat.
const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// The same array as the evaluation dump writes it: four fields, no clocks.
const START_NO_CLOCKS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -";

/// A queen down the open e-file: one checker, and nothing else to say.
const QUEEN_CHECK: &str = "4k3/8/8/4q3/8/8/8/4K3 w - - 6 40";

/// The same queen with a knight in the way: aimed at the king, not checking.
const BLOCKED: &str = "4k3/8/8/4q3/8/4N3/8/4K3 w - - 6 40";

/// A pawn on d2 checks e1 the only way a pawn can, diagonally.
const PAWN_CHECK: &str = "4k3/8/8/8/8/8/3p4/4K3 w - - 0 40";

/// A rook down the e-file and a knight on d3: two checkers at once.
const ROOK_AND_KNIGHT: &str = "k3r3/8/8/8/8/3n4/8/4K3 w - - 8 40";

/// A bishop bearing from a5 down to e1 and a knight on f3: two checkers again.
const BISHOP_AND_KNIGHT: &str = "4k3/8/8/b7/8/5n2/8/4K3 w - - 12 45";

/// Kings and rooks at home, but only White's short and Black's long right left.
const RIGHTS_SPLIT: &str = "r3k2r/8/8/8/8/8/8/R3K2R w Kq - 4 12";

/// The same array with Black to move and Black's long right gone.
const RIGHTS_BLACK_TO_MOVE: &str = "r3k2r/8/8/8/8/8/8/R3K2R b KQk - 2 9";

/// Kings and rooks at home with every right spent.
const RIGHTS_NONE: &str = "r3k2r/8/8/8/8/8/8/R3K2R b - - 10 30";

/// After 1.e4 c5 2.e5 d5: e5 stands beside the pawn that has just run past it.
const EP_TAKEABLE: &str = "rnbqkbnr/pp2pppp/8/2ppP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3";

/// The FEN names e3, and the one black pawn stands two files away from it.
const EP_UNREACHED: &str = "4k3/8/8/8/2p1P3/8/8/4K3 b - e3 0 1";

/// c4xd3 e.p. would empty the fourth rank between h4 and the black king.
const EP_PINNED: &str = "8/8/8/8/k1pP3R/8/8/4K3 b - d3 0 1";

/// Pawns on either side of d4: two legal moves take d3 en passant.
const EP_TWO_TAKERS: &str = "4k3/8/8/8/2pPp3/8/8/4K3 b - d3 0 1";

/// A rook endgame stripped of its clocks; the clock cases append their own.
const NO_CLOCKS: &str = "4k3/8/8/8/8/8/R7/4K3 w - -";

/// The same endgame 45 plies into a shuffle.
const CLOCK_45: &str = "4k3/8/8/8/8/8/R7/4K3 w - - 45 60";

/// Kings and a rook with room to walk: the repetition cases play from here.
const SHUFFLE: &str = "k6r/8/8/8/8/8/8/K7 w - - 0 1";

/// Chess960: kings on g between rooks on f and h, and a pawn just past b5.
const NINE_SIXTY: &str = "bqnb1rkr/pp1ppppp/8/1Pp5/8/8/P1PPPPPP/BQNB1RKR w HFhf c6 0 3";

/// The same rights in the `KQkq` dialect: the outermost rook of each wing.
const NINE_SIXTY_OUTERMOST: &str = "bqnb1rkr/pp1ppppp/8/1Pp5/8/8/P1PPPPPP/BQNB1RKR w KQkq c6 0 3";

/// Only White's h-rook and Black's f-rook are still free, and Black is to move.
const NINE_SIXTY_SPLIT: &str = "bqnb1rkr/pp1ppppp/8/1Pp5/8/8/P1PPPPPP/BQNB1RKR b Hf - 4 5";

/// Kings on b between rooks on a and c: only the c-rook may still be castled with.
const NINE_SIXTY_INNER: &str = "rkrbbnnq/pppppppp/8/8/8/8/PPPPPPPP/RKRBBNNQ w Cc - 0 1";

/// Where the `halfmove_bucket` one-hot starts in the 29-wide `state` row: after
/// `in_check`, `double_check`, the four castling bits, `ep_available`, the eight
/// `ep_file` bits and `ep_capture_legal` (`features.md` §2.1).
const HALFMOVE_BUCKET_AT: usize = 16;

/// The classic game `fen` starts, after the UCI `moves`.
fn game_of(fen: &str, moves: &[&str]) -> Game {
    let mut game = Game::from_fen(classic(), fen).expect("a test FEN is a legal position");
    for uci in moves {
        game.play_uci(uci).expect("a case names only legal moves");
    }
    game
}

/// The bucket the encoded `state` row's one-hot sets for `fen`.
fn halfmove_bucket(fen: &str) -> usize {
    let schema = Schema::v0();
    let groups = schema.group_set(&["state"]).expect("the state group");
    let row = facts_of(fen).encode(schema, groups);
    row[HALFMOVE_BUCKET_AT..HALFMOVE_BUCKET_AT + 8]
        .iter()
        .position(|&value| value == 1.0)
        .expect("a one-hot sets exactly one value")
}

#[rstest]
#[case::start(START, false)]
#[case::blocked(BLOCKED, false)]
#[case::rights_none(RIGHTS_NONE, false)]
#[case::queen_check(QUEEN_CHECK, true)]
#[case::pawn_check(PAWN_CHECK, true)]
#[case::rook_and_knight(ROOK_AND_KNIGHT, true)]
#[case::bishop_and_knight(BISHOP_AND_KNIGHT, true)]
fn a_check_is_the_side_to_move_standing_under_attack(#[case] fen: &str, #[case] in_check: bool) {
    assert_eq!(facts_of(fen).state.in_check, in_check);
}

#[rstest]
#[case::start(START, false)]
#[case::blocked(BLOCKED, false)]
#[case::queen_check(QUEEN_CHECK, false)]
#[case::pawn_check(PAWN_CHECK, false)]
#[case::rook_and_knight(ROOK_AND_KNIGHT, true)]
#[case::bishop_and_knight(BISHOP_AND_KNIGHT, true)]
fn two_checkers_at_once_make_the_check_a_double_one(#[case] fen: &str, #[case] double_check: bool) {
    assert_eq!(facts_of(fen).state.double_check, double_check);
}

#[rstest]
#[case::start(&CLASSIC, START, [true, true], [true, true])]
#[case::rights_split(&CLASSIC, RIGHTS_SPLIT, [true, false], [false, true])]
#[case::rights_black_to_move(&CLASSIC, RIGHTS_BLACK_TO_MOVE, [true, true], [false, true])]
#[case::rights_none(&CLASSIC, RIGHTS_NONE, [false, false], [false, false])]
#[case::nine_sixty(&CHESS960, NINE_SIXTY, [true, true], [true, true])]
#[case::nine_sixty_outermost(&CHESS960, NINE_SIXTY_OUTERMOST, [true, true], [true, true])]
#[case::nine_sixty_split(&CHESS960, NINE_SIXTY_SPLIT, [false, true], [true, false])]
#[case::nine_sixty_inner(&CHESS960, NINE_SIXTY_INNER, [true, true], [false, false])]
fn a_castling_right_survives_for_the_side_and_wing_the_fen_still_names(
    #[case] variant: &dyn Variant,
    #[case] fen: &str,
    #[case] short: [bool; 2],
    #[case] long: [bool; 2],
) {
    let state = facts_under(variant, fen).state;
    assert_eq!(state.castle_short, short);
    assert_eq!(state.castle_long, long);
}

#[rstest]
#[case::start(START, false)]
#[case::clock_45(CLOCK_45, false)]
#[case::ep_takeable(EP_TAKEABLE, true)]
#[case::ep_unreached(EP_UNREACHED, true)]
#[case::ep_pinned(EP_PINNED, true)]
#[case::ep_two_takers(EP_TWO_TAKERS, true)]
fn the_en_passant_bit_says_only_that_the_fen_named_a_target(
    #[case] fen: &str,
    #[case] available: bool,
) {
    assert_eq!(facts_of(fen).state.en_passant.is_some(), available);
}

#[rstest]
#[case::start(START, None)]
#[case::clock_45(CLOCK_45, None)]
#[case::ep_takeable(EP_TAKEABLE, Some(File::D))]
#[case::ep_unreached(EP_UNREACHED, Some(File::E))]
#[case::ep_pinned(EP_PINNED, Some(File::D))]
#[case::ep_two_takers(EP_TWO_TAKERS, Some(File::D))]
fn the_en_passant_file_is_the_one_the_target_square_stands_on(
    #[case] fen: &str,
    #[case] file: Option<File>,
) {
    assert_eq!(facts_of(fen).state.en_passant, file);
}

#[rstest]
#[case::start(START, false)]
#[case::ep_unreached(EP_UNREACHED, false)]
#[case::ep_pinned(EP_PINNED, false)]
#[case::ep_takeable(EP_TAKEABLE, true)]
#[case::ep_two_takers(EP_TWO_TAKERS, true)]
fn an_en_passant_capture_counts_only_when_a_legal_move_makes_it(
    #[case] fen: &str,
    #[case] legal: bool,
) {
    assert_eq!(facts_of(fen).state.ep_capture_legal, legal);
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
    assert_eq!(facts_of(&fen).state.halfmove_clock, clock);
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
    let state = facts_of(fen).state;
    assert_eq!(state.halfmove_known, known);
    assert_eq!(state.halfmove_clock, clock);
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
    assert_eq!(game_of(fen, moves).facts().state.repetition_seen, seen);
}

#[rstest]
#[case::start_fresh(START, &[], [false, false])]
#[case::start_knight_out(START, &["g1f3"], [false, false])]
#[case::start_three_of_four(START, &["g1f3", "g8f6", "f3g1"], [true, false])]
#[case::start_knights_home(START, &["g1f3", "g8f6", "f3g1", "f6g8"], [true, false])]
#[case::shuffle_three_plies(SHUFFLE, &["a1b1", "h8h7", "b1a1"], [true, false])]
#[case::shuffle_rook_back(SHUFFLE, &["a1b1", "h8h7", "b1a1", "h7h8"], [true, false])]
#[case::shuffle_rook_h6(SHUFFLE, &["a1b1", "h8h7", "b1a1", "h7h6"], [false, true])]
fn a_repetition_is_available_to_us_when_one_of_our_moves_reaches_the_history(
    #[case] fen: &str,
    #[case] moves: &[&str],
    #[case] available: [bool; 2],
) {
    let facts = game_of(fen, moves).facts();
    assert_eq!(facts.state.repetition_available, available);
    assert_eq!(
        facts.state.repetition_available[Side::Us.index()],
        available[0]
    );
}

#[rstest]
#[case::start_fresh(START, &[], [false, false])]
#[case::start_knights_home(START, &["g1f3", "g8f6", "f3g1", "f6g8"], [true, false])]
#[case::shuffle_rook_h6(SHUFFLE, &["a1b1", "h8h7", "b1a1", "h7h6"], [false, true])]
#[case::shuffle_rook_h5(
    SHUFFLE,
    &["a1b1", "h8h7", "b1a1", "h7h6", "a1b1", "h6h5"],
    [false, true]
)]
#[case::shuffle_checked(SHUFFLE, &["a1b1", "h8b8"], [false, false])]
fn their_repetition_is_read_after_a_null_move_and_a_check_leaves_none(
    #[case] fen: &str,
    #[case] moves: &[&str],
    #[case] available: [bool; 2],
) {
    let facts = game_of(fen, moves).facts();
    assert_eq!(facts.state.repetition_available, available);
    assert_eq!(
        facts.state.repetition_available[Side::Them.index()],
        available[1]
    );
    assert!(
        !facts.state.in_check || !available[1],
        "no null move exists"
    );
}

#[rstest]
#[case::start(START)]
#[case::shuffle(SHUFFLE)]
#[case::clock_45(CLOCK_45)]
#[case::queen_check(QUEEN_CHECK)]
fn a_history_is_known_only_to_the_game_that_holds_it(#[case] fen: &str) {
    assert!(!facts_of(fen).state.history_known);
    assert!(game_of(fen, &[]).facts().state.history_known);
}

#[rstest]
#[case::start_knights_home(START, &["g1f3", "g8f6", "f3g1", "f6g8"])]
#[case::shuffle_rook_back(SHUFFLE, &["a1b1", "h8h7", "b1a1", "h7h8"])]
#[case::shuffle_rook_h6(SHUFFLE, &["a1b1", "h8h7", "b1a1", "h7h6"])]
fn a_position_on_its_own_carries_none_of_the_repetition_facts(
    #[case] fen: &str,
    #[case] moves: &[&str],
) {
    let game = game_of(fen, moves);
    let played = game.facts().state;
    let bare = facts_of(&game.position().fen()).state;

    assert!(
        played.repetition_seen || played.repetition_available != [false, false],
        "the game sees something to repeat here"
    );
    assert!(!bare.history_known);
    assert!(!bare.repetition_seen);
    assert_eq!(bare.repetition_available, [false, false]);
}

/// No `state` fact is among the four `features.md` §4 defines for classic chess
/// only, and castling rights are read from the rook files either dialect names.
#[test]
fn the_state_facts_of_a_chess960_position_read_the_rooks_the_rights_name() {
    let state = facts_under(&CHESS960, NINE_SIXTY).state;
    assert!(!state.in_check);
    assert!(!state.double_check);
    assert_eq!(state.castle_short, [true, true]);
    assert_eq!(state.castle_long, [true, true]);
    assert_eq!(state.en_passant, Some(File::C));
    assert!(state.ep_capture_legal, "b5 takes c6 en passant");
    assert_eq!(state.halfmove_clock, 0);
    assert!(state.halfmove_known);
    assert!(!state.history_known);
    assert!(!state.repetition_seen);
    assert_eq!(state.repetition_available, [false, false]);

    assert_eq!(
        facts_under(&CHESS960, NINE_SIXTY_OUTERMOST).state,
        state,
        "`KQkq` names the same two rooks the file letters do"
    );
}
