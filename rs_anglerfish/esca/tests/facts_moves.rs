//! The `move` schema, fact by fact.
//!
//! Every expectation is worked out from the definitions in `docs/features.md`
//! §1 and §3 for the named position above it. A case names its move in UCI,
//! castling written king-to-rook the way `Move` prints it.

mod common;

use common::{move_facts, move_facts_under};
use esca::{CHESS960, CLASSIC, MoveFacts, Role, Variant};
use rstest::rstest;

/// A knight on e4 reaching an enemy unit of every role; only d6 is undefended.
const WHEEL: &str = "k7/8/3n1b2/2r3q1/4N3/2p5/8/K7 w - - 0 1";

/// The same wheel a rank flip and a colour swap later, for Black to turn.
const WHEEL_BLACK: &str = "k7/8/2P5/4n3/2R3Q1/3N1B2/8/K7 b - - 0 1";

/// A symmetric middlegame: either side can move a unit of every role, and castle.
const EVERY_ROLE: &str = "r3k2r/ppp1qppp/2npbn2/8/8/2NPBN2/PPP1QPPP/R3K2R w KQkq - 0 1";

/// The same middlegame with Black to move.
const EVERY_ROLE_BLACK: &str = "r3k2r/ppp1qppp/2npbn2/8/8/2NPBN2/PPP1QPPP/R3K2R b KQkq - 0 1";

/// A pawn on e7 that can queen straight ahead or by taking either back-rank unit.
const PROMOTION: &str = "k2r1n2/4P3/8/8/8/8/8/6K1 w - - 0 1";

/// The same promotion a rank flip and a colour swap later.
const PROMOTION_BLACK: &str = "6k1/8/8/8/8/8/4p3/K2R1N2 b - - 0 1";

/// b8 is attacked by a rook and defended by one: the role that lands decides.
const NEW_QUEEN: &str = "7r/1P6/8/8/7k/8/8/1R2K3 w - - 0 1";

/// A lone queen with three ways to check a bare king, only one of them free.
const CHECKS: &str = "4k3/8/8/3p4/1Q6/8/8/4K3 w - - 0 1";

/// The same three checks a rank flip and a colour swap later.
const CHECKS_BLACK: &str = "4k3/8/8/1q6/3P4/8/8/4K3 b - - 0 1";

/// An open d-file: the rook's squares on it are defended, cheaply attacked, or bare.
const EXCHANGE: &str = "3rk3/8/1n6/8/8/2P5/8/3RK1B1 w - - 0 1";

/// The same file a rank flip and a colour swap later.
const EXCHANGE_BLACK: &str = "3rk1b1/8/2p5/8/8/1N6/8/3RK3 b - - 0 1";

/// Black has just played d7-d5; the pawn it leaves on d5 is the rook's already.
const EN_PASSANT: &str = "4k3/8/8/3pP3/8/8/8/3RK3 w - d6 0 1";

/// The same capture a rank flip and a colour swap later.
const EN_PASSANT_BLACK: &str = "3rk3/8/8/8/3Pp3/8/8/4K3 b - d3 0 1";

/// A Chess960 array: the king on b1 between its rooks on a1 and e1.
const NINE_SIXTY: &str = "rk2r3/pppqbppp/2n2n2/8/8/2N2N2/PPPQBPPP/RK2R3 w AEae - 0 1";

/// The same array with Black to move.
const NINE_SIXTY_BLACK: &str = "rk2r3/pppqbppp/2n2n2/8/8/2N2N2/PPPQBPPP/RK2R3 b AEae - 0 1";

#[rstest]
#[case::takes_pawn(WHEEL, "e4c3", Some(Role::Pawn))]
#[case::takes_knight(WHEEL, "e4d6", Some(Role::Knight))]
#[case::takes_bishop(WHEEL, "e4f6", Some(Role::Bishop))]
#[case::takes_rook(WHEEL, "e4c5", Some(Role::Rook))]
#[case::takes_queen(WHEEL, "e4g5", Some(Role::Queen))]
#[case::quiet_move(WHEEL, "e4f2", None)]
#[case::black_takes_pawn(WHEEL_BLACK, "e5c6", Some(Role::Pawn))]
#[case::black_takes_knight(WHEEL_BLACK, "e5d3", Some(Role::Knight))]
#[case::black_takes_bishop(WHEEL_BLACK, "e5f3", Some(Role::Bishop))]
#[case::black_takes_rook(WHEEL_BLACK, "e5c4", Some(Role::Rook))]
#[case::black_takes_queen(WHEEL_BLACK, "e5g4", Some(Role::Queen))]
#[case::black_quiet_move(WHEEL_BLACK, "e5g6", None)]
#[case::en_passant(EN_PASSANT, "e5d6", Some(Role::Pawn))]
#[case::black_en_passant(EN_PASSANT_BLACK, "e4d3", Some(Role::Pawn))]
fn a_capture_names_the_role_it_removes(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] victim: Option<Role>,
) {
    assert_eq!(move_facts(fen, uci).victim, victim);
}

#[rstest]
#[case::pawn(EVERY_ROLE, "a2a3", Role::Pawn)]
#[case::knight(EVERY_ROLE, "c3b5", Role::Knight)]
#[case::bishop(EVERY_ROLE, "e3d4", Role::Bishop)]
#[case::rook(EVERY_ROLE, "a1b1", Role::Rook)]
#[case::queen(EVERY_ROLE, "e2d2", Role::Queen)]
#[case::king(EVERY_ROLE, "e1f1", Role::King)]
#[case::castling_king(EVERY_ROLE, "e1h1", Role::King)]
#[case::black_pawn(EVERY_ROLE_BLACK, "a7a6", Role::Pawn)]
#[case::black_knight(EVERY_ROLE_BLACK, "c6b4", Role::Knight)]
#[case::black_bishop(EVERY_ROLE_BLACK, "e6d5", Role::Bishop)]
#[case::black_rook(EVERY_ROLE_BLACK, "a8b8", Role::Rook)]
#[case::black_queen(EVERY_ROLE_BLACK, "e7d7", Role::Queen)]
#[case::black_king(EVERY_ROLE_BLACK, "e8f8", Role::King)]
#[case::black_castling_king(EVERY_ROLE_BLACK, "e8h8", Role::King)]
fn every_move_names_the_role_that_makes_it(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] mover: Role,
) {
    assert_eq!(move_facts(fen, uci).mover, mover);
}

#[rstest]
#[case::to_queen(PROMOTION, "e7e8q", Some(Role::Queen))]
#[case::to_rook(PROMOTION, "e7d8r", Some(Role::Rook))]
#[case::to_bishop(PROMOTION, "e7f8b", Some(Role::Bishop))]
#[case::to_knight(PROMOTION, "e7e8n", Some(Role::Knight))]
#[case::no_promotion(PROMOTION, "g1g2", None)]
#[case::black_to_queen(PROMOTION_BLACK, "e2e1q", Some(Role::Queen))]
#[case::black_to_rook(PROMOTION_BLACK, "e2f1r", Some(Role::Rook))]
#[case::black_to_bishop(PROMOTION_BLACK, "e2e1b", Some(Role::Bishop))]
#[case::black_to_knight(PROMOTION_BLACK, "e2d1n", Some(Role::Knight))]
#[case::black_no_promotion(PROMOTION_BLACK, "g8g7", None)]
fn a_promotion_names_the_role_the_pawn_becomes(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] promotion: Option<Role>,
) {
    assert_eq!(move_facts(fen, uci).promotion, promotion);
}

#[rstest]
#[case::queen_to_the_back_rank(CHECKS, "b4b8", true)]
#[case::queen_beside_the_king(CHECKS, "b4e7", true)]
#[case::queen_onto_the_king_file(CHECKS, "b4e4", true)]
#[case::queen_off_every_line(CHECKS, "b4d6", false)]
#[case::queen_backwards(CHECKS, "b4b3", false)]
#[case::black_queen_to_the_back_rank(CHECKS_BLACK, "b5b1", true)]
#[case::black_queen_beside_the_king(CHECKS_BLACK, "b5e2", true)]
#[case::black_queen_onto_the_king_file(CHECKS_BLACK, "b5e5", true)]
#[case::black_queen_off_every_line(CHECKS_BLACK, "b5d3", false)]
#[case::rook_takes_rook(EXCHANGE, "d1d8", true)]
#[case::new_queen_on_the_rank(PROMOTION, "e7d8q", true)]
#[case::new_queen_behind_a_rook(PROMOTION, "e7e8q", false)]
#[case::new_knight(PROMOTION, "e7d8n", false)]
fn a_move_gives_check_when_it_leaves_the_enemy_king_attacked(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] gives_check: bool,
) {
    assert_eq!(move_facts(fen, uci).gives_check, gives_check);
}

#[rstest]
#[case::check_out_of_reach(CHECKS, "b4b8", true)]
#[case::check_the_king_can_take(CHECKS, "b4e7", false)]
#[case::check_a_pawn_covers(CHECKS, "b4e4", false)]
#[case::not_a_check(CHECKS, "b4d6", false)]
#[case::black_check_out_of_reach(CHECKS_BLACK, "b5b1", true)]
#[case::black_check_the_king_can_take(CHECKS_BLACK, "b5e2", false)]
#[case::black_check_a_pawn_covers(CHECKS_BLACK, "b5e5", false)]
#[case::check_the_king_recaptures(EXCHANGE, "d1d8", false)]
#[case::new_queen_out_of_reach(PROMOTION, "e7d8q", true)]
fn a_safe_check_is_a_check_whose_destination_is_safe(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] gives_safe_check: bool,
) {
    assert_eq!(move_facts(fen, uci).gives_safe_check, gives_safe_check);
}

#[rstest]
#[case::attacked_but_defended(EXCHANGE, "d1d4", true)]
#[case::a_knight_answers(EXCHANGE, "d1d5", false)]
#[case::attacked_and_alone(EXCHANGE, "d1d6", false)]
#[case::pawn_left_undefended(EXCHANGE, "c3c4", false)]
#[case::unattacked(EXCHANGE, "d1a1", true)]
#[case::takes_a_free_knight(EXCHANGE, "g1b6", true)]
#[case::takes_a_defended_rook(EXCHANGE, "d1d8", false)]
#[case::black_attacked_but_defended(EXCHANGE_BLACK, "d8d5", true)]
#[case::black_a_knight_answers(EXCHANGE_BLACK, "d8d4", false)]
#[case::black_attacked_and_alone(EXCHANGE_BLACK, "d8d3", false)]
#[case::black_pawn_left_undefended(EXCHANGE_BLACK, "c6c5", false)]
#[case::black_unattacked(EXCHANGE_BLACK, "d8a8", true)]
#[case::black_takes_a_free_knight(EXCHANGE_BLACK, "g8b3", true)]
#[case::black_takes_a_defended_rook(EXCHANGE_BLACK, "d8d1", false)]
#[case::new_queen_a_rook_answers(NEW_QUEEN, "b7b8q", false)]
#[case::new_rook_trades_evenly(NEW_QUEEN, "b7b8r", true)]
#[case::new_knight_costs_the_rook(NEW_QUEEN, "b7b8n", true)]
fn a_safe_destination_is_free_of_pawns_of_cheaper_attackers_and_of_free_captures(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] is_safe: bool,
) {
    assert_eq!(move_facts(fen, uci).is_safe, is_safe);
}

#[rstest]
#[case::takes_the_undefended_knight(WHEEL, "e4d6", true)]
#[case::takes_a_defended_queen(WHEEL, "e4g5", false)]
#[case::takes_a_defended_pawn(WHEEL, "e4c3", false)]
#[case::takes_a_defended_rook(WHEEL, "e4c5", false)]
#[case::quiet_move(WHEEL, "e4f2", false)]
#[case::bishop_takes_a_free_knight(EXCHANGE, "g1b6", true)]
#[case::rook_takes_a_defended_rook(EXCHANGE, "d1d8", false)]
#[case::en_passant_takes_a_free_pawn(EN_PASSANT, "e5d6", true)]
#[case::black_takes_the_undefended_knight(WHEEL_BLACK, "e5d3", true)]
#[case::black_takes_a_defended_queen(WHEEL_BLACK, "e5g4", false)]
#[case::black_bishop_takes_a_free_knight(EXCHANGE_BLACK, "g8b3", true)]
#[case::black_rook_takes_a_defended_rook(EXCHANGE_BLACK, "d8d1", false)]
fn a_move_captures_hanging_when_its_victim_is_attacked_and_undefended(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] captures_hanging: bool,
) {
    assert_eq!(move_facts(fen, uci).captures_hanging, captures_hanging);
}

#[rstest]
#[case::takes_the_attacker(WHEEL, "e4d6", true)]
#[case::to_a_free_square(WHEEL, "e4f2", true)]
#[case::to_a_square_that_hangs(WHEEL, "e4g5", false)]
#[case::to_a_square_a_pawn_covers(WHEEL, "e4d2", false)]
#[case::rook_to_a_defended_square(EXCHANGE, "d1d4", true)]
#[case::bishop_was_not_attacked(EXCHANGE, "g1b6", false)]
#[case::pawn_was_not_attacked(EXCHANGE, "c3c4", false)]
#[case::black_rook_to_a_defended_square(EXCHANGE_BLACK, "d8d5", true)]
#[case::black_rook_to_a_square_a_knight_holds(EXCHANGE_BLACK, "d8d4", false)]
#[case::black_bishop_was_not_attacked(EXCHANGE_BLACK, "g8b3", false)]
fn a_move_escapes_attack_when_it_leaves_a_held_square_for_a_safe_one(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] escapes_attack: bool,
) {
    assert_eq!(move_facts(fen, uci).escapes_attack, escapes_attack);
}

#[rstest]
#[case::onto_the_pawns_right_diagonal(CHECKS, "b4e4", true)]
#[case::onto_the_pawns_left_diagonal(CHECKS, "b4c4", true)]
#[case::onto_the_back_rank(CHECKS, "b4b8", false)]
#[case::beside_the_king(CHECKS, "b4e7", false)]
#[case::knight_onto_a_covered_square(WHEEL, "e4d2", true)]
#[case::knight_onto_a_free_square(WHEEL, "e4f2", false)]
#[case::black_onto_the_pawns_right_diagonal(CHECKS_BLACK, "b5e5", true)]
#[case::black_onto_the_pawns_left_diagonal(CHECKS_BLACK, "b5c5", true)]
#[case::black_onto_the_back_rank(CHECKS_BLACK, "b5b1", false)]
#[case::en_passant_removes_the_last_pawn(EN_PASSANT, "e5d6", false)]
fn a_move_notes_when_an_enemy_pawn_covers_its_destination(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] to_attacked_by_pawn: bool,
) {
    assert_eq!(
        move_facts(fen, uci).to_attacked_by_pawn,
        to_attacked_by_pawn
    );
}

#[rstest]
#[case::short(&CLASSIC, EVERY_ROLE, "e1h1", true)]
#[case::long(&CLASSIC, EVERY_ROLE, "e1a1", true)]
#[case::king_step(&CLASSIC, EVERY_ROLE, "e1f1", false)]
#[case::black_short(&CLASSIC, EVERY_ROLE_BLACK, "e8h8", true)]
#[case::black_long(&CLASSIC, EVERY_ROLE_BLACK, "e8a8", true)]
#[case::black_king_step(&CLASSIC, EVERY_ROLE_BLACK, "e8f8", false)]
#[case::chess960_short(&CHESS960, NINE_SIXTY, "b1e1", true)]
#[case::chess960_long(&CHESS960, NINE_SIXTY, "b1a1", true)]
#[case::chess960_king_step(&CHESS960, NINE_SIXTY, "b1c1", false)]
#[case::chess960_black_short(&CHESS960, NINE_SIXTY_BLACK, "b8e8", true)]
#[case::chess960_black_long(&CHESS960, NINE_SIXTY_BLACK, "b8a8", true)]
#[case::chess960_black_king_step(&CHESS960, NINE_SIXTY_BLACK, "b8c8", false)]
fn castling_is_the_king_move_written_to_its_own_rook(
    #[case] variant: &dyn Variant,
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] is_castling: bool,
) {
    assert_eq!(move_facts_under(variant, fen, uci).is_castling, is_castling);
}

#[rstest]
#[case::takes_en_passant(EN_PASSANT, "e5d6", true)]
#[case::pushes_past(EN_PASSANT, "e5e6", false)]
#[case::rook_takes_the_pawn(EN_PASSANT, "d1d5", false)]
#[case::pawn_push(EVERY_ROLE, "a2a3", false)]
#[case::black_takes_en_passant(EN_PASSANT_BLACK, "e4d3", true)]
#[case::black_pushes_past(EN_PASSANT_BLACK, "e4e3", false)]
#[case::black_rook_takes_the_pawn(EN_PASSANT_BLACK, "d8d4", false)]
fn a_pawn_capture_onto_the_en_passant_square_is_marked_en_passant(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] is_en_passant: bool,
) {
    assert_eq!(move_facts(fen, uci).is_en_passant, is_en_passant);
}

#[rstest]
#[case::promotion_capture_with_check(
    PROMOTION,
    "e7d8q",
    [
        1.0, // capture
        0.0, 0.0, 0.0, 1.0, 0.0, // victim: rook
        1.0, 0.0, 0.0, 0.0, 0.0, 0.0, // mover: pawn
        1.0, 0.0, 0.0, 0.0, // promotion: queen
        1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0,
    ]
)]
#[case::castling(
    EVERY_ROLE,
    "e1h1",
    [
        0.0, // quiet
        0.0, 0.0, 0.0, 0.0, 0.0, // victim: none
        0.0, 0.0, 0.0, 0.0, 0.0, 1.0, // mover: king
        0.0, 0.0, 0.0, 0.0, // promotion: none
        0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0,
    ]
)]
#[case::en_passant(
    EN_PASSANT,
    "e5d6",
    [
        1.0, // capture
        1.0, 0.0, 0.0, 0.0, 0.0, // victim: pawn
        1.0, 0.0, 0.0, 0.0, 0.0, 0.0, // mover: pawn
        0.0, 0.0, 0.0, 0.0, // promotion: none
        0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0,
    ]
)]
#[case::capture_that_hangs(
    WHEEL,
    "e4g5",
    [
        1.0, // capture
        0.0, 0.0, 0.0, 0.0, 1.0, // victim: queen
        0.0, 1.0, 0.0, 0.0, 0.0, 0.0, // mover: knight
        0.0, 0.0, 0.0, 0.0, // promotion: none
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ]
)]
#[case::black_capture(
    EXCHANGE_BLACK,
    "g8b3",
    [
        1.0, // capture
        0.0, 1.0, 0.0, 0.0, 0.0, // victim: knight
        0.0, 0.0, 1.0, 0.0, 0.0, 0.0, // mover: bishop
        0.0, 0.0, 0.0, 0.0, // promotion: none
        0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0,
    ]
)]
fn a_move_is_twenty_four_values_in_the_order_the_catalogue_lists(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] row: [f32; 24],
) {
    let mut out = [0.0f32; MoveFacts::WIDTH];
    move_facts(fen, uci).encode_into(&mut out);
    assert_eq!(out, row);
}

/// No `move` fact is among the four `features.md` §4 defines for classic chess
/// only, so a Chess960 position is read the same way — except that castling
/// starts and ends where its own array puts the king and the rook.
#[test]
fn the_move_facts_of_a_chess960_position_follow_its_own_castling_geometry() {
    let short = move_facts_under(&CHESS960, NINE_SIXTY, "b1e1");
    assert!(short.is_castling);
    assert_eq!(short.mover, Role::King);
    assert_eq!(short.victim, None);
    assert!(short.is_safe);
    assert!(!short.gives_check);

    let long = move_facts_under(&CHESS960, NINE_SIXTY, "b1a1");
    assert!(long.is_castling);
    assert_eq!(long.mover, Role::King);
    assert!(long.is_safe);

    // The queen takes a queen a knight recaptures, so the destination is not safe.
    let takes_queen = move_facts_under(&CHESS960, NINE_SIXTY, "d2d7");
    assert_eq!(takes_queen.victim, Some(Role::Queen));
    assert!(!takes_queen.is_safe);
    assert!(!takes_queen.captures_hanging);
    assert!(!takes_queen.is_castling);

    // a3 is a bishop's square and the b-pawn's, which is defence enough.
    let pawn = move_facts_under(&CHESS960, NINE_SIXTY, "a2a3");
    assert_eq!(pawn.mover, Role::Pawn);
    assert!(pawn.is_safe);
    assert!(!pawn.escapes_attack);
}
