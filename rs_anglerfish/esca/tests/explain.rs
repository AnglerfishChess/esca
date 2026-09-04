//! The `explain` layer: the evidence behind a rules answer.
//!
//! Every expectation is read off the diagram above the named position, from
//! the definitions in `docs/esca-api.md` §12.

mod common;

use common::squares;
use esca::explain::{
    AutomaticDraw, ClaimableDraw, Difference, DrawStatus, EpCapture, EpObstacle, MaterialConfig,
    NearMiss, Repetition, ResetKind, StalemateDetail, Stuck, Wing,
};
use esca::{
    CHESS960, CLASSIC, Colour, Game, MoveList, Position, Square, SquareSet, Variant, classic,
};
use rstest::rstest;

// ---------------------------------------------------------------- positions

/// Both sides may castle either way and the back ranks are otherwise bare.
const CLEAR_BACK_RANK: &str = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";

/// The same, with White's own queen on d1 and knight on g1 in the way.
const QUEEN_AND_KNIGHT: &str = "r3k2r/8/8/8/8/8/8/R2QK1NR w KQkq - 0 1";

/// A bishop on b5 covers f1, which the king would cross.
const BISHOP_COVERS_F1: &str = "4k3/8/8/1b6/8/8/8/4K2R w K - 0 1";

/// King and rooks stand ready, but no right survives.
const NO_RIGHTS: &str = "4k3/8/8/8/8/8/8/R3K2R w - - 0 1";

/// Every reason at once: the king is checked from e8, a6 covers f1, and
/// White's own knight sits on g1.
const EVERY_REASON: &str = "k3r3/8/b7/8/8/8/8/4K1NR w K - 0 1";

/// Chess960 with the kings on b1 and b8 and the rooks on the corners.
const NINE_SIXTY_CLEAR: &str = "rk5r/8/8/8/8/8/8/RK5R w AHah - 0 1";

/// The same with a third black rook on e8, which the short castling crosses
/// and the long one does not.
const NINE_SIXTY_E_FILE: &str = "rk2r2r/8/8/8/8/8/8/RK5R w AHah - 0 1";

/// Chess960 with the king already on g1: castling short moves only the rook.
const NINE_SIXTY_KING_STAYS: &str = "r5kr/8/8/8/8/8/8/R5KR w AHah - 0 1";

/// An untouched Chess960 array, king on b1: five of its own units stand on
/// the short castling's path.
const NINE_SIXTY_ARRAY: &str = "rkbbnqnr/pppppppp/8/8/8/8/PPPPPPPP/RKBBNQNR w AHah - 0 1";

/// The d-pawn has just run past e5 and nothing forbids the capture.
const EP_PLAIN: &str = "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1";

/// No pawn has just moved two squares.
const EP_NONE: &str = "4k3/8/8/3pP3/8/8/8/4K3 w - - 0 1";

/// A target with no pawn of the moving side beside it.
const EP_NO_TAKER: &str = "4k3/8/8/3p4/8/8/8/4K3 w - d6 0 1";

/// The rank pin: both pawns leave rank 5 at once and a5 checks h5.
const EP_RANK_PIN: &str = "4k3/8/8/r2pP2K/8/8/8/8 w - d6 0 1";

/// The e5 pawn is pinned on the b2–g7 diagonal, which d6 is not on.
const EP_PINNED: &str = "4k3/6K1/8/3pP3/8/8/1b6/8 w - d6 0 1";

/// The e5 pawn is pinned on the c7–h2 diagonal, which d6 is on.
const EP_PIN_ALONG_RAY: &str = "4k3/2b5/8/3pP3/8/8/7K/8 w - d6 0 1";

/// The pawn that ran past uncovered the a7 rook, and taking on d6 leaves the
/// check standing.
const EP_IN_CHECK: &str = "8/r6K/8/3pP3/8/8/8/k7 w - d6 0 1";

/// The pawn that ran past gives check, and the capture takes it off.
const EP_ANSWERS_CHECK: &str = "7k/8/8/2Pp4/4K3/8/8/8 w - d6 0 1";

/// Two pawns may take: c5 freely, e5 only off its pin.
const EP_TWO_TAKERS: &str = "4k3/6K1/8/2PpP3/8/8/1b6/8 w - d6 0 1";

/// A rook on e2 and a knight on f3 check e1 together.
const DOUBLE_CHECK: &str = "4k3/8/8/8/8/5n2/4r3/4K3 w - - 0 1";

/// Three white units bear on e5 and two black ones defend it.
const CROWD: &str = "4rk2/8/3p4/4p3/3P4/5N2/8/4RK2 w - - 0 1";

/// b4 pins the d2 knight and e8 pins the e4 bishop, both against e1.
const TWO_PINS: &str = "4r2k/8/8/8/1b2B3/8/3N4/4K3 w - - 0 1";

/// a1 attacks the black king on a5 with the a8 rook behind it.
const SKEWERED_KING: &str = "r7/8/8/k7/8/8/8/R3K3 b - - 0 1";

/// The a1 bishop attacks the d4 queen with the f6 rook behind it.
const SKEWERED_QUEEN: &str = "4k3/8/5r2/8/3q4/8/8/B3K3 w - - 0 1";

/// Kings and a blocked pawn each: a king can triangulate, a king cannot
/// return in one move, so five plies restore the placement.
const TRIANGULATION: &str = "3k4/p7/8/8/8/8/P7/3K4 w - - 0 1";

/// A white pawn one step from a double step, with a black pawn waiting to
/// take it en passant.
const EN_PASSANT_RIGHTS: &str = "4k3/8/8/8/3p4/8/4P3/4K3 w - - 0 1";

/// One ply from the fifty-move claim.
const CLOCK_AT_99: &str = "4k2r/8/8/8/8/8/8/R3K3 w - - 99 60";

/// One ply from the automatic draw.
const CLOCK_AT_149: &str = "4k2r/8/8/8/8/8/8/R3K3 w - - 149 90";

/// Queen and king shut the black king in without checking it.
const SMOTHERED_STALEMATE: &str = "7k/5Q2/6K1/8/8/8/8/8 b - - 0 1";

/// Stalemate with nothing left that could ever mate.
const BISHOP_STALEMATE: &str = "k7/B7/K7/8/8/8/8/8 b - - 0 1";

/// Stalemate where the a7 pawn is blocked and the b7 knight is pinned.
const PINNED_AND_BLOCKED: &str = "k7/pn6/N7/8/4B3/8/8/6K1 b - - 0 1";

/// Mate delivered on the hundred-and-fiftieth quiet ply: the game is over,
/// so no draw stands.
const MATE_ON_THE_CLOCK: &str = "7k/6Q1/6K1/8/8/8/8/8 b - - 150 90";

/// Kings only.
const BARE_KINGS: &str = "4k3/8/8/8/8/8/8/4K3 w - - 0 1";

/// One bishop besides the kings.
const ONE_BISHOP: &str = "4k3/8/8/8/8/8/8/3BK3 w - - 0 1";

/// One knight besides the kings.
const ONE_KNIGHT: &str = "4k3/8/8/8/8/8/8/3NK3 w - - 0 1";

/// A bishop each, both on light squares.
const SAME_COLOUR_BISHOPS: &str = "4k3/8/4b3/8/8/8/8/3BK3 w - - 0 1";

/// A bishop each, on opposite square colours: a helpmate exists.
const OPPOSITE_BISHOPS: &str = "4k3/8/3b4/8/8/8/8/3BK3 w - - 0 1";

/// Knights out and back: four plies return to the same position.
const SHUFFLE: &str = "Nf3 Nf6 Ng1 Ng8";

/// The untouched array.
const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// ------------------------------------------------------------------ helpers

/// The position `fen` describes.
fn position(fen: &str) -> Position {
    Position::from_fen(fen).expect("a test FEN is a legal position")
}

/// The square `name` names.
fn square(name: &str) -> Square {
    name.parse().expect("a square name")
}

/// The variant a case names: `chess` or `chess960`.
fn variant_named(name: &str) -> &'static dyn Variant {
    match name {
        "chess" => &CLASSIC,
        "chess960" => &CHESS960,
        other => panic!("not a variant: {other}"),
    }
}

/// A classic game from the standard array with `moves` played, in SAN.
fn game_of(moves: &str) -> Game {
    game_from(&CLASSIC.start_position(0).fen(), moves)
}

/// A classic game from `fen` with `moves` played, in SAN.
fn game_from(fen: &str, moves: &str) -> Game {
    let mut game = Game::from_fen(classic(), fen).expect("a test FEN is a legal position");
    for text in moves.split_whitespace() {
        game.play_san(text).expect("a test move is legal");
    }
    game
}

/// Square-and-attackers pairs written as `[("f1", "a6 b5")]`.
fn covers(pairs: &[(&str, &str)]) -> Vec<(Square, SquareSet)> {
    pairs
        .iter()
        .map(|(name, by)| (square(name), squares(by)))
        .collect()
}

/// One en-passant capture of the position `fen` describes.
fn ep_capture(fen: &str, from: &str) -> EpCapture {
    position(fen)
        .en_passant_status()
        .captures()
        .iter()
        .find(|capture| capture.from == square(from))
        .unwrap_or_else(|| panic!("{from} has no en-passant capture in {fen}"))
        .clone()
}

/// The origin and legality of every en-passant capture on offer.
fn ep_offers(fen: &str) -> Vec<(String, bool)> {
    position(fen)
        .en_passant_status()
        .captures()
        .iter()
        .map(|capture| (capture.from.to_string(), capture.legal))
        .collect()
}

/// The case an obstacle is, named as the Python surface names it.
fn obstacle_kind(capture: &EpCapture) -> Option<&'static str> {
    capture
        .forbidden_by
        .as_ref()
        .map(|obstacle| match obstacle {
            EpObstacle::Pinned { .. } => "pinned",
            EpObstacle::ExposesKing { .. } => "exposes_king",
            EpObstacle::InCheck { .. } => "in_check",
        })
}

/// The cases of a draw status, in the order it lists them.
fn automatic_kinds(status: &DrawStatus) -> Vec<&'static str> {
    status
        .automatic
        .iter()
        .map(|draw| match draw {
            AutomaticDraw::Stalemate(_) => "stalemate",
            AutomaticDraw::InsufficientMaterial(_) => "insufficient_material",
            AutomaticDraw::Fivefold(_) => "fivefold",
            AutomaticDraw::SeventyFiveMoves(_) => "seventy_five_moves",
        })
        .collect()
}

fn claimable_kinds(claims: &[ClaimableDraw]) -> Vec<&'static str> {
    claims
        .iter()
        .map(|claim| match claim {
            ClaimableDraw::Threefold(_) => "threefold",
            ClaimableDraw::FiftyMoves(_) => "fifty_moves",
        })
        .collect()
}

/// The material configuration a draw status names, if it names one.
fn material_config(status: &DrawStatus) -> Option<MaterialConfig> {
    status.automatic.iter().find_map(|draw| match draw {
        AutomaticDraw::InsufficientMaterial(config) => Some(*config),
        _ => None,
    })
}

/// The stalemate detail a draw status carries.
fn stalemate_detail(status: &DrawStatus) -> StalemateDetail {
    status
        .automatic
        .iter()
        .find_map(|draw| match draw {
            AutomaticDraw::Stalemate(detail) => Some(detail.clone()),
            _ => None,
        })
        .expect("the position is a stalemate")
}

/// Every stuck unit and the case that holds it.
fn stuck_kinds(detail: &StalemateDetail) -> Vec<(String, &'static str)> {
    detail
        .stuck_units
        .iter()
        .map(|(square, stuck)| {
            let kind = match stuck {
                Stuck::Pinned { .. } => "pinned",
                Stuck::Blocked => "blocked",
                Stuck::NoMoves => "no_moves",
            };
            (square.to_string(), kind)
        })
        .collect()
}

/// A near miss as its ply and the names of what differs.
fn near_miss(miss: &NearMiss) -> (u32, Vec<&'static str>) {
    let differs = miss
        .differs
        .iter()
        .map(|difference| match difference {
            Difference::CastlingRights => "castling_rights",
            Difference::EnPassant => "en_passant",
            Difference::SideToMove => "side_to_move",
        })
        .collect();
    (miss.ply, differs)
}

fn near_misses(repetition: &Repetition) -> Vec<(u32, Vec<&'static str>)> {
    repetition.near_misses.iter().map(near_miss).collect()
}

// ----------------------------------------------------------------- castling

#[rstest]
#[case::clear_short(CLEAR_BACK_RANK, Colour::White, Wing::Short, true)]
#[case::clear_long(CLEAR_BACK_RANK, Colour::White, Wing::Long, true)]
#[case::clear_for_the_side_not_to_move(CLEAR_BACK_RANK, Colour::Black, Wing::Short, true)]
#[case::knight_on_g1(QUEEN_AND_KNIGHT, Colour::White, Wing::Short, false)]
#[case::queen_on_d1(QUEEN_AND_KNIGHT, Colour::White, Wing::Long, false)]
#[case::covered_f1(BISHOP_COVERS_F1, Colour::White, Wing::Short, false)]
#[case::no_right(NO_RIGHTS, Colour::White, Wing::Short, false)]
#[case::every_reason(EVERY_REASON, Colour::White, Wing::Short, false)]
#[case::nine_sixty_short(NINE_SIXTY_CLEAR, Colour::White, Wing::Short, true)]
#[case::nine_sixty_long(NINE_SIXTY_CLEAR, Colour::White, Wing::Long, true)]
#[case::nine_sixty_crosses_e1(NINE_SIXTY_E_FILE, Colour::White, Wing::Short, false)]
#[case::nine_sixty_long_avoids_e1(NINE_SIXTY_E_FILE, Colour::White, Wing::Long, true)]
#[case::nine_sixty_king_stays(NINE_SIXTY_KING_STAYS, Colour::White, Wing::Short, true)]
#[case::nine_sixty_array(NINE_SIXTY_ARRAY, Colour::White, Wing::Short, false)]
fn a_castling_is_allowed_when_nothing_stands_in_its_way(
    #[case] fen: &str,
    #[case] colour: Colour,
    #[case] wing: Wing,
    #[case] allowed: bool,
) {
    assert_eq!(position(fen).castling(colour, wing).allowed, allowed);
}

#[rstest]
#[case::clear_short(CLEAR_BACK_RANK, Colour::White, Wing::Short, "")]
#[case::knight_on_g1(QUEEN_AND_KNIGHT, Colour::White, Wing::Short, "g1")]
#[case::queen_on_d1(QUEEN_AND_KNIGHT, Colour::White, Wing::Long, "d1")]
#[case::black_long_is_clear(QUEEN_AND_KNIGHT, Colour::Black, Wing::Long, "")]
#[case::no_right(NO_RIGHTS, Colour::White, Wing::Short, "")]
#[case::every_reason(EVERY_REASON, Colour::White, Wing::Short, "g1")]
#[case::nine_sixty_short(NINE_SIXTY_CLEAR, Colour::White, Wing::Short, "")]
#[case::nine_sixty_king_stays(NINE_SIXTY_KING_STAYS, Colour::White, Wing::Short, "")]
#[case::nine_sixty_array(NINE_SIXTY_ARRAY, Colour::White, Wing::Short, "c1 d1 e1 f1 g1")]
#[case::nine_sixty_array_long(NINE_SIXTY_ARRAY, Colour::White, Wing::Long, "c1 d1")]
fn a_castling_names_the_units_standing_on_its_path(
    #[case] fen: &str,
    #[case] colour: Colour,
    #[case] wing: Wing,
    #[case] blocked: &str,
) {
    assert_eq!(
        position(fen).castling(colour, wing).path_blocked,
        squares(blocked)
    );
}

#[rstest]
#[case::clear_short(CLEAR_BACK_RANK, Colour::White, Wing::Short, &[])]
#[case::covered_f1(BISHOP_COVERS_F1, Colour::White, Wing::Short, &[("f1", "b5")])]
#[case::every_reason(EVERY_REASON, Colour::White, Wing::Short, &[("f1", "a6")])]
#[case::nine_sixty_crosses_e1(NINE_SIXTY_E_FILE, Colour::White, Wing::Short, &[("e1", "e8")])]
#[case::nine_sixty_long_avoids_e1(NINE_SIXTY_E_FILE, Colour::White, Wing::Long, &[])]
#[case::nine_sixty_king_stays(NINE_SIXTY_KING_STAYS, Colour::White, Wing::Short, &[])]
fn a_castling_names_every_covered_square_the_king_would_cross(
    #[case] fen: &str,
    #[case] colour: Colour,
    #[case] wing: Wing,
    #[case] attacked: &[(&str, &str)],
) {
    assert_eq!(
        position(fen).castling(colour, wing).path_attacked,
        covers(attacked)
    );
}

#[rstest]
#[case::clear_short(CLEAR_BACK_RANK, Colour::White, Wing::Short, true, true, "")]
#[case::no_right(NO_RIGHTS, Colour::White, Wing::Short, false, false, "")]
#[case::black_has_no_right(BISHOP_COVERS_F1, Colour::Black, Wing::Short, false, false, "")]
#[case::every_reason(EVERY_REASON, Colour::White, Wing::Short, true, true, "e8")]
#[case::nine_sixty_short(NINE_SIXTY_CLEAR, Colour::White, Wing::Short, true, true, "")]
fn a_castling_names_the_right_the_rook_and_the_check(
    #[case] fen: &str,
    #[case] colour: Colour,
    #[case] wing: Wing,
    #[case] right: bool,
    #[case] rook_present: bool,
    #[case] check_by: &str,
) {
    let castling = position(fen).castling(colour, wing);
    assert_eq!(castling.right, right);
    assert_eq!(castling.rook_present, rook_present);
    assert_eq!(castling.king_in_check_by, squares(check_by));
}

/// Three reasons hold at once, and all three are answered.
#[test]
fn a_castling_reports_every_reason_that_applies() {
    let castling = position(EVERY_REASON).castling(Colour::White, Wing::Short);
    assert_eq!(castling.king_in_check_by, squares("e8"));
    assert_eq!(castling.path_attacked, covers(&[("f1", "a6")]));
    assert_eq!(castling.path_blocked, squares("g1"));
    assert!(castling.right);
    assert!(castling.rook_present);
    assert!(!castling.allowed);
}

#[rstest]
#[case::clear("chess", CLEAR_BACK_RANK)]
#[case::queen_and_knight("chess", QUEEN_AND_KNIGHT)]
#[case::covered_f1("chess", BISHOP_COVERS_F1)]
#[case::no_rights("chess", NO_RIGHTS)]
#[case::every_reason("chess", EVERY_REASON)]
#[case::nine_sixty_clear("chess960", NINE_SIXTY_CLEAR)]
#[case::nine_sixty_e_file("chess960", NINE_SIXTY_E_FILE)]
#[case::nine_sixty_king_stays("chess960", NINE_SIXTY_KING_STAYS)]
#[case::nine_sixty_array("chess960", NINE_SIXTY_ARRAY)]
fn allowed_is_what_the_move_generator_says_for_the_side_to_move(
    #[case] variant: &str,
    #[case] fen: &str,
) {
    let variant = variant_named(variant);
    let position = position(fen);
    let colour = position.side_to_move();
    let mut moves = MoveList::new();
    variant.legal_moves(&position, &mut moves);
    for wing in [Wing::Short, Wing::Long] {
        let generated = moves.iter().any(|mv| {
            mv.is_castling() && (mv.to().file() > mv.from().file()) == (wing == Wing::Short)
        });
        assert_eq!(
            position.castling(colour, wing).allowed,
            generated,
            "{fen} {wing:?}"
        );
    }
}

// -------------------------------------------------------------- en passant

#[rstest]
#[case::plain(EP_PLAIN, Some("d6"))]
#[case::none(EP_NONE, None)]
#[case::no_taker(EP_NO_TAKER, Some("d6"))]
#[case::rank_pin(EP_RANK_PIN, Some("d6"))]
fn en_passant_status_names_the_square_a_pawn_skipped(
    #[case] fen: &str,
    #[case] target: Option<&str>,
) {
    assert_eq!(
        position(fen).en_passant_status().target(),
        target.map(square)
    );
}

#[rstest]
#[case::plain(EP_PLAIN, &[("e5", true)])]
#[case::none(EP_NONE, &[])]
#[case::no_taker(EP_NO_TAKER, &[])]
#[case::rank_pin(EP_RANK_PIN, &[("e5", false)])]
#[case::pinned_off_the_ray(EP_PINNED, &[("e5", false)])]
#[case::pinned_along_the_ray(EP_PIN_ALONG_RAY, &[("e5", true)])]
#[case::in_check(EP_IN_CHECK, &[("e5", false)])]
#[case::answers_the_check(EP_ANSWERS_CHECK, &[("c5", true)])]
#[case::two_takers(EP_TWO_TAKERS, &[("c5", true), ("e5", false)])]
fn en_passant_status_names_every_pawn_that_could_take(
    #[case] fen: &str,
    #[case] offers: &[(&str, bool)],
) {
    let expected: Vec<(String, bool)> = offers
        .iter()
        .map(|(from, legal)| (from.to_string(), *legal))
        .collect();
    assert_eq!(ep_offers(fen), expected);
}

#[rstest]
#[case::plain(EP_PLAIN, "e5", None)]
#[case::pinned_along_the_ray(EP_PIN_ALONG_RAY, "e5", None)]
#[case::rank_pin(EP_RANK_PIN, "e5", Some("exposes_king"))]
#[case::pinned_off_the_ray(EP_PINNED, "e5", Some("pinned"))]
#[case::in_check(EP_IN_CHECK, "e5", Some("in_check"))]
#[case::two_takers(EP_TWO_TAKERS, "e5", Some("pinned"))]
fn an_illegal_en_passant_names_what_forbids_it(
    #[case] fen: &str,
    #[case] from: &str,
    #[case] kind: Option<&str>,
) {
    assert_eq!(obstacle_kind(&ep_capture(fen, from)), kind);
}

/// The rank pin binds neither pawn on its own, so it names the slider.
#[test]
fn the_rank_pin_names_the_slider_the_two_pawns_hide() {
    let capture = ep_capture(EP_RANK_PIN, "e5");
    assert_eq!(
        capture.forbidden_by,
        Some(EpObstacle::ExposesKing {
            attacker: square("a5")
        })
    );
}

#[test]
fn a_pinned_pawn_names_its_pinner_and_the_ray_it_may_not_leave() {
    let capture = ep_capture(EP_PINNED, "e5");
    assert_eq!(
        capture.forbidden_by,
        Some(EpObstacle::Pinned {
            ray: squares("c3 d4 e5 f6"),
            pinner: square("b2"),
        })
    );
}

#[test]
fn an_en_passant_that_leaves_a_check_standing_names_the_checkers() {
    let capture = ep_capture(EP_IN_CHECK, "e5");
    assert_eq!(
        capture.forbidden_by,
        Some(EpObstacle::InCheck { by: squares("a7") })
    );
}

// ------------------------------------------------- checks, attacks and rays

#[rstest]
#[case::none(CLEAR_BACK_RANK, "")]
#[case::double_check(DOUBLE_CHECK, "e2 f3")]
#[case::a_pawn_that_ran_past(EP_ANSWERS_CHECK, "d5")]
#[case::a_rook_along_the_rank(EP_IN_CHECK, "a7")]
fn checkers_are_the_units_giving_check_to_the_side_to_move(
    #[case] fen: &str,
    #[case] checkers: &str,
) {
    assert_eq!(position(fen).checkers(), squares(checkers));
}

#[rstest]
#[case::white_on_e5(CROWD, "e5", Colour::White, "d4 e1 f3")]
#[case::black_on_e5(CROWD, "e5", Colour::Black, "d6 e8")]
#[case::white_on_d4(CROWD, "d4", Colour::White, "f3")]
#[case::black_on_d4(CROWD, "d4", Colour::Black, "e5")]
#[case::an_empty_square(CROWD, "a1", Colour::White, "e1")]
fn attackers_are_the_units_of_a_colour_that_bear_on_a_square(
    #[case] fen: &str,
    #[case] square_name: &str,
    #[case] colour: Colour,
    #[case] attackers: &str,
) {
    assert_eq!(
        position(fen).attackers(square(square_name), colour),
        squares(attackers)
    );
}

#[rstest]
#[case::a_diagonal("a1", "d4", "b2 c3")]
#[case::a_file("a1", "a4", "a2 a3")]
#[case::a_rank("a1", "d1", "b1 c1")]
#[case::the_long_diagonal("h1", "a8", "b7 c6 d5 e4 f3 g2")]
#[case::adjacent("e4", "e5", "")]
#[case::unaligned("a1", "b3", "")]
#[case::itself("e4", "e4", "")]
fn between_is_the_squares_two_squares_share_a_line_through(
    #[case] from: &str,
    #[case] to: &str,
    #[case] expected: &str,
) {
    assert_eq!(
        position(CLEAR_BACK_RANK).between(square(from), square(to)),
        squares(expected)
    );
}

#[rstest]
#[case::two_pins(TWO_PINS, Colour::White, &["d2 b4 e1", "e4 e8 e1"])]
#[case::nothing_pinned(TWO_PINS, Colour::Black, &[])]
#[case::a_bare_board(CLEAR_BACK_RANK, Colour::White, &[])]
fn a_pin_names_the_unit_the_pinner_and_the_king_behind_it(
    #[case] fen: &str,
    #[case] colour: Colour,
    #[case] expected: &[&str],
) {
    let pins: Vec<String> = position(fen)
        .pins(colour)
        .iter()
        .map(|pin| format!("{} {} {}", pin.pinned, pin.pinner, pin.king))
        .collect();
    assert_eq!(pins, expected);
}

#[test]
fn a_pin_carries_the_ray_the_pinned_unit_may_not_leave() {
    let pins = position(TWO_PINS).pins(Colour::White);
    assert_eq!(pins[0].ray, squares("c3 d2"));
    assert_eq!(pins[1].ray, squares("e2 e3 e4 e5 e6 e7"));
}

#[rstest]
#[case::a_king_in_front(SKEWERED_KING, Colour::Black, &["a1 a5 a8"])]
#[case::a_queen_in_front(SKEWERED_QUEEN, Colour::Black, &["a1 d4 f6"])]
#[case::the_attacking_side(SKEWERED_KING, Colour::White, &[])]
#[case::a_pin_is_not_a_skewer(TWO_PINS, Colour::White, &[])]
fn a_skewer_names_the_attacker_the_front_unit_and_what_stands_behind(
    #[case] fen: &str,
    #[case] colour: Colour,
    #[case] expected: &[&str],
) {
    let skewers: Vec<String> = position(fen)
        .skewers(colour)
        .iter()
        .map(|skewer| format!("{} {} {}", skewer.attacker, skewer.front, skewer.behind))
        .collect();
    assert_eq!(skewers, expected);
}

#[test]
fn a_skewer_carries_the_ray_its_two_units_stand_on() {
    let skewers = position(SKEWERED_QUEEN).skewers(Colour::Black);
    assert_eq!(skewers[0].ray, squares("b2 c3 d4 e5"));
}

// --------------------------------------------------------------- repetition

#[rstest]
#[case::a_fresh_game("", 1, &[0])]
#[case::one_shuffle("Nf3 Nf6 Ng1 Ng8", 2, &[0, 4])]
#[case::two_shuffles("Nf3 Nf6 Ng1 Ng8 Nf3 Nf6 Ng1 Ng8", 3, &[0, 4, 8])]
#[case::mid_shuffle("Nf3 Nf6 Ng1", 1, &[3])]
fn a_repetition_lists_every_ply_the_position_has_stood_at(
    #[case] moves: &str,
    #[case] count: u32,
    #[case] plies: &[u32],
) {
    let repetition = game_of(moves).repetition_status();
    assert_eq!(repetition.count, count);
    assert_eq!(repetition.plies, plies);
}

/// The rook leaves and comes back: the same placement, one right poorer.
#[test]
fn a_repetition_starts_over_when_a_castling_right_is_spent() {
    let moves = "Rhg1 Rhg8 Rh1 Rh8 Rhg1 Rhg8 Rh1 Rh8";
    let repetition = game_from(CLEAR_BACK_RANK, moves).repetition_status();
    assert_eq!(repetition.count, 2);
    assert_eq!(repetition.plies, [4, 8]);
    assert_eq!(near_misses(&repetition), [(0, vec!["castling_rights"])]);
}

#[rstest]
#[case::none(START, "Nf3 Nf6 Ng1 Ng8", &[])]
#[case::castling_rights(CLEAR_BACK_RANK, "Rhg1 Rhg8 Rh1 Rh8 Rhg1 Rhg8 Rh1 Rh8", &["castling_rights"])]
#[case::en_passant(EN_PASSANT_RIGHTS, "e4 Kd8 Kd2 Ke8 Ke1", &["en_passant"])]
#[case::side_to_move(TRIANGULATION, "Kc1 Ke8 Kc2 Kd8 Kd1", &["side_to_move"])]
fn a_near_miss_says_what_keeps_it_from_counting(
    #[case] fen: &str,
    #[case] moves: &str,
    #[case] differs: &[&str],
) {
    let repetition = game_from(fen, moves).repetition_status();
    let found: Vec<Vec<&'static str>> = repetition
        .near_misses
        .iter()
        .map(|miss| near_miss(miss).1)
        .collect();
    let expected: Vec<Vec<&str>> = if differs.is_empty() {
        Vec::new()
    } else {
        vec![differs.to_vec()]
    };
    assert_eq!(found, expected);
}

#[rstest]
#[case::en_passant(EN_PASSANT_RIGHTS, "e4 Kd8 Kd2 Ke8 Ke1", 1)]
#[case::side_to_move(TRIANGULATION, "Kc1 Ke8 Kc2 Kd8 Kd1", 0)]
fn a_near_miss_names_the_ply_it_stood_at(#[case] fen: &str, #[case] moves: &str, #[case] ply: u32) {
    let repetition = game_from(fen, moves).repetition_status();
    assert_eq!(repetition.near_misses[0].ply, ply);
}

// --------------------------------------------------------------- fifty move

#[rstest]
#[case::a_fresh_game("", 0, 100, 150)]
#[case::a_pawn_move("e4", 0, 100, 150)]
#[case::one_quiet_ply("e4 e5 Nf3", 1, 99, 149)]
#[case::three_quiet_plies("e4 e5 Nf3 Nc6 Bb5", 3, 97, 147)]
fn the_clock_counts_down_to_the_claim_and_to_the_automatic_draw(
    #[case] moves: &str,
    #[case] clock: u32,
    #[case] to_claim: u32,
    #[case] to_automatic: u32,
) {
    let fifty = game_of(moves).fifty_move_status();
    assert_eq!(fifty.clock, clock);
    assert_eq!(fifty.plies_to_claim, to_claim);
    assert_eq!(fifty.plies_to_automatic, to_automatic);
}

#[rstest]
#[case::at_the_claim(CLOCK_AT_99, "Kd1", 100, 0, 50)]
#[case::at_the_automatic_draw(CLOCK_AT_149, "Kd1", 150, 0, 0)]
fn a_clock_past_a_threshold_counts_down_no_further(
    #[case] fen: &str,
    #[case] moves: &str,
    #[case] clock: u32,
    #[case] to_claim: u32,
    #[case] to_automatic: u32,
) {
    let fifty = game_from(fen, moves).fifty_move_status();
    assert_eq!(fifty.clock, clock);
    assert_eq!(fifty.plies_to_claim, to_claim);
    assert_eq!(fifty.plies_to_automatic, to_automatic);
}

#[rstest]
#[case::nothing_played("", None)]
#[case::a_pawn_move("e4", Some((1, "pawn_move")))]
#[case::a_pawn_move_answered("e4 e5 Nf3", Some((2, "pawn_move")))]
#[case::a_capture("e4 d5 exd5", Some((3, "capture")))]
#[case::a_capture_then_quiet_plies("e4 d5 exd5 Nf6 Nf3", Some((3, "capture")))]
fn the_last_reset_names_the_move_that_cleared_the_clock(
    #[case] moves: &str,
    #[case] reset: Option<(u32, &str)>,
) {
    let found = game_of(moves).fifty_move_status().last_reset.map(|reset| {
        let kind = match reset.kind {
            ResetKind::Capture => "capture",
            ResetKind::PawnMove => "pawn_move",
        };
        (reset.ply, kind)
    });
    assert_eq!(found, reset);
}

/// A game that starts mid-clock has no reset of its own to point at.
#[test]
fn a_clock_a_game_started_with_has_no_reset() {
    let fifty = game_from(CLOCK_AT_99, "").fifty_move_status();
    assert_eq!(fifty.clock, 99);
    assert!(fifty.last_reset.is_none());
}

// -------------------------------------------------------------- draw status

#[rstest]
#[case::a_fresh_game(START, "", &[], &[])]
#[case::a_playable_game(OPPOSITE_BISHOPS, "", &[], &[])]
#[case::bare_kings(BARE_KINGS, "", &["insufficient_material"], &[])]
#[case::one_bishop(ONE_BISHOP, "", &["insufficient_material"], &[])]
#[case::one_knight(ONE_KNIGHT, "", &["insufficient_material"], &[])]
#[case::same_colour_bishops(SAME_COLOUR_BISHOPS, "", &["insufficient_material"], &[])]
#[case::smothered_stalemate(SMOTHERED_STALEMATE, "", &["stalemate"], &[])]
#[case::stalemate_with_nothing_left(
    BISHOP_STALEMATE,
    "",
    &["stalemate", "insufficient_material"],
    &[]
)]
#[case::at_the_fifty_move_claim(CLOCK_AT_99, "Kd1", &[], &["fifty_moves"])]
#[case::at_the_automatic_draw(CLOCK_AT_149, "Kd1", &["seventy_five_moves"], &["fifty_moves"])]
#[case::checkmate_ends_it_first(MATE_ON_THE_CLOCK, "", &[], &[])]
fn a_draw_status_lists_every_reason_that_applies(
    #[case] fen: &str,
    #[case] moves: &str,
    #[case] automatic: &[&str],
    #[case] claimable: &[&str],
) {
    let status = game_from(fen, moves).draw_status();
    assert_eq!(automatic_kinds(&status), automatic);
    assert_eq!(claimable_kinds(&status.claimable), claimable);
}

#[rstest]
#[case::threefold(2, &[], &["threefold"])]
#[case::fivefold(4, &["fivefold"], &["threefold"])]
fn a_repeated_position_is_claimable_before_it_is_automatic(
    #[case] shuffles: usize,
    #[case] automatic: &[&str],
    #[case] claimable: &[&str],
) {
    let moves = [SHUFFLE; 4][..shuffles].join(" ");
    let status = game_of(&moves).draw_status();
    assert_eq!(automatic_kinds(&status), automatic);
    assert_eq!(claimable_kinds(&status.claimable), claimable);
}

#[rstest]
#[case::bare_kings(BARE_KINGS, Some(MaterialConfig::KvK))]
#[case::one_bishop(ONE_BISHOP, Some(MaterialConfig::KBvK))]
#[case::one_knight(ONE_KNIGHT, Some(MaterialConfig::KNvK))]
#[case::same_colour_bishops(SAME_COLOUR_BISHOPS, Some(MaterialConfig::KBvKBSameColour))]
#[case::opposite_bishops(OPPOSITE_BISHOPS, None)]
fn insufficient_material_names_the_configuration(
    #[case] fen: &str,
    #[case] config: Option<MaterialConfig>,
) {
    assert_eq!(material_config(&game_from(fen, "").draw_status()), config);
}

#[rstest]
#[case::smothered(SMOTHERED_STALEMATE, "h8", &[("g7", "f7 g6"), ("h7", "f7 g6"), ("g8", "f7")])]
#[case::bishop(BISHOP_STALEMATE, "a8", &[("a7", "a6"), ("b7", "a6"), ("b8", "a7")])]
#[case::pinned_and_blocked(PINNED_AND_BLOCKED, "a8", &[("b8", "a6")])]
fn a_stalemate_names_the_escape_squares_and_who_covers_them(
    #[case] fen: &str,
    #[case] king: &str,
    #[case] escapes: &[(&str, &str)],
) {
    let detail = stalemate_detail(&game_from(fen, "").draw_status());
    assert_eq!(detail.king, square(king));
    assert_eq!(detail.escape_squares, covers(escapes));
}

#[rstest]
#[case::nothing_else_left(SMOTHERED_STALEMATE, &[])]
#[case::a_pawn_and_a_knight(PINNED_AND_BLOCKED, &[("a7", "blocked"), ("b7", "pinned")])]
fn a_stalemate_says_what_holds_every_other_unit(#[case] fen: &str, #[case] stuck: &[(&str, &str)]) {
    let detail = stalemate_detail(&game_from(fen, "").draw_status());
    let expected: Vec<(String, &str)> = stuck
        .iter()
        .map(|(square, kind)| (square.to_string(), *kind))
        .collect();
    assert_eq!(stuck_kinds(&detail), expected);
}

#[test]
fn a_stuck_pinned_unit_names_its_pinner_and_the_ray() {
    let detail = stalemate_detail(&game_from(PINNED_AND_BLOCKED, "").draw_status());
    let (held, stuck) = detail.stuck_units[1];
    assert_eq!(held.to_string(), "b7");
    assert_eq!(
        stuck,
        Stuck::Pinned {
            ray: squares("b7 c6 d5"),
            pinner: square("e4"),
        }
    );
}

/// A claim carries the count or the clock that earns it.
#[test]
fn a_claimable_draw_carries_the_evidence_for_the_claim() {
    let threefold = game_of(&[SHUFFLE; 2].join(" ")).draw_status();
    match &threefold.claimable[0] {
        ClaimableDraw::Threefold(repetition) => {
            assert_eq!(repetition.count, 3);
            assert_eq!(repetition.plies, [0, 4, 8]);
        }
        other => panic!("expected a threefold claim, not {other:?}"),
    }

    let fifty = game_from(CLOCK_AT_99, "Kd1").draw_status();
    match &fifty.claimable[0] {
        ClaimableDraw::FiftyMoves(clock) => assert_eq!(clock.clock, 100),
        other => panic!("expected a fifty-move claim, not {other:?}"),
    }
}

// -------------------------------------------------------------- claims after

#[rstest]
#[case::a_third_occurrence("Nf3 Nf6 Ng1 Ng8 Nf3 Nf6 Ng1", "Ng8", &["threefold"])]
#[case::anything_else("Nf3 Nf6 Ng1 Ng8 Nf3 Nf6 Ng1", "e5", &[])]
#[case::a_second_occurrence("Nf3 Nf6 Ng1", "Ng8", &[])]
fn a_claim_after_a_move_is_what_that_move_would_earn(
    #[case] moves: &str,
    #[case] next: &str,
    #[case] claims: &[&str],
) {
    let game = game_of(moves);
    let mv = game
        .variant()
        .move_from_san(game.position(), next)
        .expect("the next move is legal");
    assert_eq!(claimable_kinds(&game.claims_after(mv)), claims);
}

#[test]
fn a_move_that_reaches_the_clock_earns_the_fifty_move_claim() {
    let game = game_from(CLOCK_AT_99, "");
    let mv = game
        .variant()
        .move_from_san(game.position(), "Kd1")
        .expect("the king move is legal");
    assert_eq!(claimable_kinds(&game.claims_after(mv)), ["fifty_moves"]);
}

#[test]
fn a_move_of_another_position_claims_nothing() {
    let mut game = game_of("");
    let opening = game
        .variant()
        .move_from_san(game.position(), "e4")
        .expect("the pawn move is legal");
    game.play_san("e4").expect("the pawn move is legal");
    assert!(game.claims_after(opening).is_empty());
}
