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

/// A rook with three ways to threaten something: a free rook, a defended
/// queen, and an undefended knight.
const THREAT: &str = "4k3/8/8/3q4/7r/2n5/8/R3K3 w - - 0 1";

/// The same three threats a rank flip and a colour swap later.
const THREAT_BLACK: &str = "r3k3/8/2N5/7R/3Q4/8/8/4K3 b - - 0 1";

/// A rook check down the d-file: a knight and a bishop can interpose, another
/// rook can take the checker, and the king can step aside.
const CHECKED: &str = "R2rk3/8/8/8/8/8/1N4B1/3K4 w - - 0 1";

/// The same check a rank flip and a colour swap later.
const CHECKED_BLACK: &str = "3k4/1n4b1/8/8/8/8/8/r2RK3 b - - 0 1";

/// A passed pawn on c6 that can push or take, beside a b-pawn a b-pawn holds up.
const PASSERS: &str = "4k3/3n4/2P5/8/1p3p2/8/1P6/4K3 w - - 0 1";

/// The same pair a rank flip and a colour swap later.
const PASSERS_BLACK: &str = "4k3/1p6/8/1P3P2/8/2p5/3N4/4K3 b - - 0 1";

/// A b-pawn the c-pawn holds up: pushing past it or taking it makes a passer.
const CREATES: &str = "4k3/8/2p5/1P6/8/8/8/4K3 w - - 0 1";

/// The same pawn a rank flip and a colour swap later.
const CREATES_BLACK: &str = "4k3/8/8/8/1p6/2P5/8/4K3 b - - 0 1";

/// Two pawns that stay healthy unless the b-pawn takes onto the c-file.
const WEAK: &str = "4k3/8/8/8/8/2p5/1PP5/4K3 w - - 0 1";

/// The same pair a rank flip and a colour swap later.
const WEAK_BLACK: &str = "4k3/1pp5/2P5/8/8/8/8/4K3 b - - 0 1";

/// The d-pawn covers c3, so the b-pawn running ahead leaves c2 backward.
const WEAK2: &str = "4k3/8/8/8/3p4/8/1PP5/4K3 w - - 0 1";

/// The same pair a rank flip and a colour swap later.
const WEAK2_BLACK: &str = "4k3/1pp5/8/3P4/8/8/8/4K3 b - - 0 1";

/// A g-pawn beside the enemy king: either capture empties the g-file for us.
const OPENK: &str = "6k1/8/5p1p/6P1/8/8/8/6K1 w - - 0 1";

/// The same pawn a rank flip and a colour swap later.
const OPENK_BLACK: &str = "6k1/8/8/8/6p1/5P1P/8/6K1 b - - 0 1";

/// A rook that can reach their king's ring, facing one that holds ours.
const RING: &str = "6k1/5ppp/8/8/8/8/r4PPP/1R4K1 w - - 0 1";

/// The same pair of rooks a rank flip and a colour swap later.
const RING_BLACK: &str = "1r4k1/R4ppp/8/8/8/8/5PPP/6K1 b - - 0 1";

/// A knight standing between a rook and the enemy queen.
const DISC: &str = "3q3k/8/8/8/8/8/3N4/3RK3 w - - 0 1";

/// The same knight a rank flip and a colour swap later.
const DISC_BLACK: &str = "3rk3/3n4/8/8/8/8/8/3Q3K b - - 0 1";

/// The same battery uncovering a pawn instead of a queen.
const DISC_PAWN: &str = "7k/3p4/8/8/8/8/3N4/3RK3 w - - 0 1";

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
#[case::takes_a_free_knight(WHEEL, "e4d6", 3)]
#[case::takes_a_queen_a_bishop_holds(WHEEL, "e4g5", 6)]
#[case::takes_a_rook_a_queen_holds(WHEEL, "e4c5", 2)]
#[case::takes_a_pawn_a_bishop_holds(WHEEL, "e4c3", -2)]
#[case::quiet_move_onto_a_free_square(WHEEL, "e4f2", 0)]
#[case::quiet_move_onto_a_covered_square(WHEEL_BLACK, "e5g6", -3)]
#[case::black_takes_a_free_knight(WHEEL_BLACK, "e5d3", 3)]
#[case::black_takes_a_queen_a_bishop_holds(WHEEL_BLACK, "e5g4", 6)]
#[case::black_takes_a_rook_a_queen_holds(WHEEL_BLACK, "e5c4", 2)]
#[case::black_takes_a_pawn_a_bishop_holds(WHEEL_BLACK, "e5c6", -2)]
#[case::promotion_capture_nothing_answers(PROMOTION, "e7d8q", 13)]
#[case::promotion_a_rook_answers(PROMOTION, "e7e8q", -1)]
#[case::black_promotion_capture(PROMOTION_BLACK, "e2d1q", 13)]
#[case::rook_walks_onto_a_knight(EXCHANGE, "d1d5", -5)]
#[case::castling_wins_nothing(EVERY_ROLE, "e1h1", 0)]
fn see_is_what_the_move_wins_once_both_sides_stop_taking(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] see: i32,
) {
    assert_eq!(move_facts(fen, uci).see, see);
}

#[rstest]
#[case::rook_eyes_a_free_rook(THREAT, "a1a4", 5)]
#[case::rook_eyes_a_defended_queen(THREAT, "a1d1", 4)]
#[case::rook_eyes_a_free_knight(THREAT, "a1c1", 3)]
#[case::check_threatens_nothing(THREAT, "a1a8", 0)]
#[case::king_step_threatens_nothing(THREAT, "e1f2", 0)]
#[case::knight_uncovers_a_queen(DISC, "d2b3", 9)]
#[case::black_rook_eyes_a_free_rook(THREAT_BLACK, "a8a5", 5)]
#[case::black_rook_eyes_a_defended_queen(THREAT_BLACK, "a8d8", 4)]
#[case::black_rook_eyes_a_free_knight(THREAT_BLACK, "a8c8", 3)]
#[case::black_check_threatens_nothing(THREAT_BLACK, "a8a1", 0)]
#[case::black_knight_uncovers_a_queen(DISC_BLACK, "d7b6", 9)]
fn a_threat_is_the_most_the_next_capture_would_win(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] threat: i32,
) {
    assert_eq!(move_facts(fen, uci).threat_created_max, threat);
}

#[rstest]
#[case::rook_a_rook_attacks(EXCHANGE, "d1d4", true)]
#[case::knight_a_knight_attacks(WHEEL, "e4f2", true)]
#[case::pawn_nothing_attacks(EXCHANGE, "c3c4", false)]
#[case::bishop_nothing_attacks(EXCHANGE, "g1b6", false)]
#[case::castling_out_of_a_quiet_corner(EVERY_ROLE, "e1h1", false)]
#[case::black_rook_a_rook_attacks(EXCHANGE_BLACK, "d8d5", true)]
#[case::black_knight_a_knight_attacks(WHEEL_BLACK, "e5g6", true)]
#[case::black_pawn_nothing_attacks(EXCHANGE_BLACK, "c6c5", false)]
#[case::black_bishop_nothing_attacks(EXCHANGE_BLACK, "g8b3", false)]
fn a_move_notes_when_the_square_it_leaves_is_under_attack(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] moves_attacked_unit: bool,
) {
    assert_eq!(
        move_facts(fen, uci).moves_attacked_unit,
        moves_attacked_unit
    );
}

#[rstest]
#[case::knight_interposes(CHECKED, "b2d3", true)]
#[case::bishop_interposes(CHECKED, "g2d5", true)]
#[case::rook_takes_the_checker(CHECKED, "a8d8", false)]
#[case::king_steps_aside(CHECKED, "d1c1", false)]
#[case::king_steps_forward(CHECKED, "d1e2", false)]
#[case::no_check_to_block(EXCHANGE, "d1d4", false)]
#[case::black_knight_interposes(CHECKED_BLACK, "b7d6", true)]
#[case::black_bishop_interposes(CHECKED_BLACK, "g7d4", true)]
#[case::black_rook_takes_the_checker(CHECKED_BLACK, "a1d1", false)]
#[case::black_king_steps_aside(CHECKED_BLACK, "d8c8", false)]
#[case::black_king_steps_forward(CHECKED_BLACK, "d8e7", false)]
fn a_move_blocks_check_when_it_lands_between_the_checker_and_the_king(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] blocks_check: bool,
) {
    assert_eq!(move_facts(fen, uci).blocks_check, blocks_check);
}

#[rstest]
#[case::passer_pushes(PASSERS, "c6c7", true)]
#[case::passer_takes(PASSERS, "c6d7", true)]
#[case::held_up_pawn_pushes(PASSERS, "b2b3", false)]
#[case::king_steps(PASSERS, "e1e2", false)]
#[case::rook_moves(EXCHANGE, "d1d4", false)]
#[case::black_passer_pushes(PASSERS_BLACK, "c3c2", true)]
#[case::black_passer_takes(PASSERS_BLACK, "c3d2", true)]
#[case::black_held_up_pawn_pushes(PASSERS_BLACK, "b7b6", false)]
#[case::black_king_steps(PASSERS_BLACK, "e8e7", false)]
fn a_pawn_advances_a_passer_when_the_pawn_it_moves_was_passed(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] advances_passer: bool,
) {
    assert_eq!(move_facts(fen, uci).advances_passer, advances_passer);
}

#[rstest]
#[case::takes_the_holder(CREATES, "b5c6", true)]
#[case::pushes_past_the_holder(CREATES, "b5b6", true)]
#[case::pushes_beside_a_holder(WEAK2, "c2c4", true)]
#[case::king_steps(CREATES, "e1e2", false)]
#[case::passer_only_advances(PASSERS, "c6c7", false)]
#[case::black_takes_the_holder(CREATES_BLACK, "b4c3", true)]
#[case::black_pushes_past_the_holder(CREATES_BLACK, "b4b3", true)]
#[case::black_pushes_beside_a_holder(WEAK2_BLACK, "c7c5", true)]
#[case::black_king_steps(CREATES_BLACK, "e8e7", false)]
#[case::black_passer_only_advances(PASSERS_BLACK, "c3c2", false)]
fn a_move_creates_a_passer_when_it_leaves_the_side_with_more_of_them(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] creates_passer: bool,
) {
    assert_eq!(move_facts(fen, uci).creates_passer, creates_passer);
}

#[rstest]
#[case::takes_onto_its_neighbours_file(WEAK, "b2c3", (true, true, false))]
#[case::pushes(WEAK, "b2b3", (false, false, false))]
#[case::double_pushes(WEAK, "b2b4", (false, false, false))]
#[case::runs_ahead_of_its_neighbour(WEAK2, "b2b4", (false, false, true))]
#[case::steps_ahead_of_its_neighbour(WEAK2, "b2b3", (false, false, true))]
#[case::neighbour_keeps_up(WEAK2, "c2c3", (false, false, false))]
#[case::king_steps(WEAK2, "e1d1", (false, false, false))]
#[case::black_takes_onto_its_neighbours_file(WEAK_BLACK, "b7c6", (true, true, false))]
#[case::black_pushes(WEAK_BLACK, "b7b6", (false, false, false))]
#[case::black_double_pushes(WEAK_BLACK, "b7b5", (false, false, false))]
#[case::black_runs_ahead_of_its_neighbour(WEAK2_BLACK, "b7b5", (false, false, true))]
#[case::black_steps_ahead_of_its_neighbour(WEAK2_BLACK, "b7b6", (false, false, true))]
#[case::black_neighbour_keeps_up(WEAK2_BLACK, "c7c6", (false, false, false))]
fn a_move_creates_a_weakness_when_it_leaves_the_side_with_more_weak_pawns(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] weaknesses: (bool, bool, bool),
) {
    let facts = move_facts(fen, uci);
    assert_eq!(
        (
            facts.creates_isolated,
            facts.creates_doubled,
            facts.creates_backward
        ),
        weaknesses
    );
}

#[rstest]
#[case::takes_to_the_left(OPENK, "g5f6", true)]
#[case::takes_to_the_right(OPENK, "g5h6", true)]
#[case::stays_on_the_file(OPENK, "g5g6", false)]
#[case::king_steps(OPENK, "g1f1", false)]
#[case::pawn_leaves_a_file_away_from_their_king(EN_PASSANT, "e5d6", true)]
#[case::black_takes_to_the_left(OPENK_BLACK, "g4f3", true)]
#[case::black_takes_to_the_right(OPENK_BLACK, "g4h3", true)]
#[case::black_stays_on_the_file(OPENK_BLACK, "g4g3", false)]
#[case::black_king_steps(OPENK_BLACK, "g8f8", false)]
fn a_move_opens_a_file_at_their_king_when_our_last_pawn_leaves_it(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] opens: bool,
) {
    assert_eq!(move_facts(fen, uci).opens_file_at_enemy_king, opens);
}

#[rstest]
#[case::rook_reaches_their_seventh(RING, "b1b7", (1, 0))]
#[case::rook_reaches_their_back_rank(RING, "b1b8", (1, 0))]
#[case::rook_steps_aside(RING, "b1a1", (0, 0))]
#[case::king_walks_out_of_a_rooks_reach(RING, "g1h1", (0, -1))]
#[case::pawn_push_changes_neither(RING, "f2f3", (0, 0))]
#[case::knight_takes_a_ring_attacker(EXCHANGE, "g1b6", (1, 0))]
#[case::black_rook_reaches_their_second(RING_BLACK, "b8b2", (1, 0))]
#[case::black_rook_reaches_their_back_rank(RING_BLACK, "b8b1", (1, 0))]
#[case::black_rook_steps_aside(RING_BLACK, "b8a8", (0, 0))]
#[case::black_king_walks_out_of_a_rooks_reach(RING_BLACK, "g8h8", (0, -1))]
#[case::black_pawn_push_changes_neither(RING_BLACK, "f7f6", (0, 0))]
fn a_move_states_what_it_does_to_both_king_rings(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] deltas: (i32, i32),
) {
    let facts = move_facts(fen, uci);
    assert_eq!(
        (
            facts.our_ring_attackers_delta,
            facts.their_ring_attackers_delta
        ),
        deltas
    );
}

#[rstest]
#[case::rook_to_a_defended_square(EXCHANGE, "d1d4", (0, -1))]
#[case::rook_to_a_bare_square(EXCHANGE, "d1d6", (1, 0))]
#[case::pawn_to_a_bare_square(EXCHANGE, "c3c4", (1, 0))]
#[case::bishop_takes_the_hanging_knight(EXCHANGE, "g1b6", (0, -1))]
#[case::knight_takes_the_knight_that_held_it(WHEEL, "e4d6", (-1, -1))]
#[case::knight_walks_into_a_bishop(WHEEL, "e4c3", (0, -1))]
#[case::black_rook_to_a_defended_square(EXCHANGE_BLACK, "d8d5", (0, -1))]
#[case::black_rook_to_a_bare_square(EXCHANGE_BLACK, "d8d3", (1, 0))]
#[case::black_pawn_to_a_bare_square(EXCHANGE_BLACK, "c6c5", (1, 0))]
#[case::black_bishop_takes_the_hanging_knight(EXCHANGE_BLACK, "g8b3", (0, -1))]
#[case::black_knight_takes_the_knight_that_held_it(WHEEL_BLACK, "e5d3", (-1, -1))]
fn a_move_states_how_many_units_of_each_side_it_leaves_hanging(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] deltas: (i32, i32),
) {
    let facts = move_facts(fen, uci);
    assert_eq!((facts.own_hanging_delta, facts.their_hanging_delta), deltas);
}

#[rstest]
#[case::rook_walks_onto_a_bare_square(EXCHANGE, "d1d6", true)]
#[case::rook_walks_onto_a_knights_square(EXCHANGE, "d1d5", true)]
#[case::knight_walks_into_a_bishop(WHEEL, "e4c3", true)]
#[case::only_a_pawn_is_left_hanging(EXCHANGE, "c3c4", false)]
#[case::rook_stays_defended(EXCHANGE, "d1d4", false)]
#[case::knight_takes_its_attacker(WHEEL, "e4d6", false)]
#[case::black_rook_walks_onto_a_bare_square(EXCHANGE_BLACK, "d8d3", true)]
#[case::black_rook_walks_onto_a_knights_square(EXCHANGE_BLACK, "d8d4", true)]
#[case::black_knight_walks_into_a_bishop(WHEEL_BLACK, "e5c6", true)]
#[case::black_only_a_pawn_is_left_hanging(EXCHANGE_BLACK, "c6c5", false)]
#[case::black_rook_stays_defended(EXCHANGE_BLACK, "d8d5", false)]
fn a_move_leaves_a_unit_hanging_when_a_square_carries_a_new_hanging_piece(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] leaves: bool,
) {
    assert_eq!(move_facts(fen, uci).leaves_unit_hanging, leaves);
}

#[rstest]
#[case::knight_steps_off_the_file(DISC, "d2b3", true)]
#[case::knight_steps_off_the_other_way(DISC, "d2f3", true)]
#[case::the_rook_itself_moves(DISC, "d1c1", false)]
#[case::king_steps_aside(DISC, "e1e2", false)]
#[case::what_it_uncovers_is_only_a_pawn(DISC_PAWN, "d2b3", false)]
#[case::black_knight_steps_off_the_file(DISC_BLACK, "d7b6", true)]
#[case::black_knight_steps_off_the_other_way(DISC_BLACK, "d7f6", true)]
#[case::black_rook_itself_moves(DISC_BLACK, "d8c8", false)]
#[case::black_king_steps_aside(DISC_BLACK, "e8e7", false)]
fn a_move_gives_a_discovered_attack_when_a_slider_it_leaves_standing_gains_one(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] discovers: bool,
) {
    assert_eq!(move_facts(fen, uci).gives_discovered_attack, discovers);
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
        // see 13, threat 3, advances a passer, one more ring attacker
        1.0, 3.0 / 9.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.25, 0.0, 0.0, 0.0, 0.0, 0.0,
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
        // a symmetric position castling changes nothing about
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
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
        // see 1, a passer that leaves their king's e-file, one pawn fewer hanging
        1.0 / 9.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -0.25, 0.0, 0.0,
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
        // see 6, off an attacked square, and the knight is left hanging
        6.0 / 9.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.25, 1.0, 0.0,
    ]
)]
#[case::quiet_move_that_hangs_a_rook(
    THREAT,
    "a1a4",
    [
        0.0, // quiet
        0.0, 0.0, 0.0, 0.0, 0.0, // victim: none
        0.0, 0.0, 0.0, 1.0, 0.0, 0.0, // mover: rook
        0.0, 0.0, 0.0, 0.0, // promotion: none
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        // see −5, threatening a rook, and hanging one of each side's
        -5.0 / 9.0, 5.0 / 9.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.25, 0.25, 1.0,
        0.0,
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
        // see 3, threatening a rook, one more ring attacker, one fewer hanging
        3.0 / 9.0, 5.0 / 9.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.25, 0.0, 0.0, -0.25, 0.0,
        0.0,
    ]
)]
fn a_move_is_forty_values_in_the_order_the_catalogue_lists(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] row: [f32; 40],
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

/// Castling is read from the squares the move really uses: the king's landing
/// square and the rook's, wherever the array puts them. Castling long lands
/// the rook on d1, behind the queen, and the pair then wins a piece on d7;
/// stepping the king to c1 instead only moves its ring under the enemy queen.
#[rstest]
#[case::short(NINE_SIXTY, "b1e1", 0, 0)]
#[case::long(NINE_SIXTY, "b1a1", 3, 1)]
#[case::king_step(NINE_SIXTY, "b1c1", 0, 1)]
#[case::pawn_push(NINE_SIXTY, "a2a3", 0, 0)]
#[case::black_short(NINE_SIXTY_BLACK, "b8e8", 0, 0)]
#[case::black_long(NINE_SIXTY_BLACK, "b8a8", 3, 1)]
#[case::black_king_step(NINE_SIXTY_BLACK, "b8c8", 0, 1)]
#[case::black_pawn_push(NINE_SIXTY_BLACK, "a7a6", 0, 0)]
fn a_chess960_move_reads_its_own_castling_geometry(
    #[case] fen: &str,
    #[case] uci: &str,
    #[case] threat: i32,
    #[case] their_ring_delta: i32,
) {
    let facts = move_facts_under(&CHESS960, fen, uci);
    assert_eq!(facts.threat_created_max, threat);
    assert_eq!(facts.their_ring_attackers_delta, their_ring_delta);
    assert_eq!(facts.our_ring_attackers_delta, 0);
    assert!(!facts.blocks_check);
    assert!(!facts.moves_attacked_unit);
    assert!(!facts.leaves_unit_hanging);
    assert!(!facts.gives_discovered_attack);
    assert_eq!(facts.see, 0);
}

/// The queen that walks into three defenders hangs on the square it lands on.
#[rstest]
#[case::white(NINE_SIXTY, "d2d7")]
#[case::black(NINE_SIXTY_BLACK, "d7d2")]
fn a_chess960_capture_states_what_it_leaves_behind(#[case] fen: &str, #[case] uci: &str) {
    let facts = move_facts_under(&CHESS960, fen, uci);
    assert!(facts.moves_attacked_unit);
    assert_eq!(facts.own_hanging_delta, 1);
    assert!(facts.leaves_unit_hanging);
    assert_eq!(facts.our_ring_attackers_delta, 1);
    assert_eq!(facts.threat_created_max, 0);
}
