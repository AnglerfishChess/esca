//! The `state` group, fact by fact.
//!
//! Every expectation is worked out from the definitions in `docs/features.md`
//! §2.2 for the named position above it.

mod common;

use common::{facts_of, facts_under};
use esca::{CHESS960, CLASSIC, File, Variant};
use rstest::rstest;

/// The untouched array: every right, a fresh clock, nothing to repeat.
const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

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

/// The same endgame 45 plies into a shuffle.
const CLOCK_45: &str = "4k3/8/8/8/8/8/R7/4K3 w - - 45 60";

/// Chess960: kings on g between rooks on f and h, and a pawn just past b5.
const NINE_SIXTY: &str = "bqnb1rkr/pp1ppppp/8/1Pp5/8/8/P1PPPPPP/BQNB1RKR w HFhf c6 0 3";

/// The same rights in the `KQkq` dialect: the outermost rook of each wing.
const NINE_SIXTY_OUTERMOST: &str = "bqnb1rkr/pp1ppppp/8/1Pp5/8/8/P1PPPPPP/BQNB1RKR w KQkq c6 0 3";

/// Only White's h-rook and Black's f-rook are still free, and Black is to move.
const NINE_SIXTY_SPLIT: &str = "bqnb1rkr/pp1ppppp/8/1Pp5/8/8/P1PPPPPP/BQNB1RKR b Hf - 4 5";

/// Kings on b between rooks on a and c: only the c-rook may still be castled with.
const NINE_SIXTY_INNER: &str = "rkrbbnnq/pppppppp/8/8/8/8/PPPPPPPP/RKRBBNNQ w Cc - 0 1";

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

    assert_eq!(
        facts_under(&CHESS960, NINE_SIXTY_OUTERMOST).state,
        state,
        "`KQkq` names the same two rooks the file letters do"
    );
}
