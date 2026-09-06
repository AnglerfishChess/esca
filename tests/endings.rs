//! The `endings` layer: which named ending a position is, what theory says
//! the result is, and the method it is played by.
//!
//! Every expectation is read off the diagram above the named position and the
//! definitions in `docs/esca-api.md` §13.

use esca::endings::{self, Class, Technique, Verdict};
use esca::{Colour, Game, Position, Role, classic};
use rstest::rstest;

// ---------------------------------------------------------------- positions

/// Kings only.
const BARE_KINGS: &str = "4k3/8/8/8/8/8/8/4K3 w - - 0 1";

/// Queen against a lone king.
const LONE_QUEEN: &str = "4k3/8/8/8/8/8/8/3QK3 w - - 0 1";

/// Rook against a lone king.
const LONE_ROOK: &str = "4k3/8/8/8/8/8/8/3RK3 w - - 0 1";

/// Two bishops against a lone king, one on each square colour.
const TWO_BISHOPS: &str = "4k3/8/8/8/8/8/8/2BBK3 w - - 0 1";

/// Two bishops against a lone king, both on dark squares: promotions can put
/// them there, and then nothing can be forced.
const TWO_DARK_BISHOPS: &str = "4k3/8/8/8/8/B7/8/2B1K3 w - - 0 1";

/// Bishop and knight against a lone king.
const BISHOP_AND_KNIGHT: &str = "4k3/8/8/8/8/8/8/2BNK3 w - - 0 1";

/// Two knights against a lone king.
const TWO_KNIGHTS: &str = "4k3/8/8/8/8/8/8/2NNK3 w - - 0 1";

/// One bishop against a lone king.
const LONE_BISHOP: &str = "4k3/8/8/8/8/8/8/3BK3 w - - 0 1";

/// One knight against a lone king.
const LONE_KNIGHT: &str = "4k3/8/8/8/8/8/8/3NK3 w - - 0 1";

/// A centre pawn on e3, both kings far from it.
const CENTRE_PAWN: &str = "4k3/8/8/8/8/4P3/8/4K3 w - - 0 1";

/// The h-pawn's promotion corner, held by the black king on g8.
const H_PAWN_CORNER: &str = "6k1/8/8/8/8/8/7P/6K1 w - - 0 1";

/// The a-pawn's promotion corner, held by the black king on b8.
const A_PAWN_CORNER: &str = "1k6/8/8/8/8/8/P7/1K6 w - - 0 1";

/// The black king on h1 is seven king moves from a8 and the pawn five: it
/// never catches the pawn.
const RUNNING_PAWN: &str = "8/8/8/8/8/8/P7/K6k w - - 0 1";

/// The kings face each other on e6 and e8 with e7 empty between them.
const KINGS_IN_OPPOSITION: &str = "4k3/8/4K3/4P3/8/8/8/8 w - - 0 1";

/// Bishop and a-pawn: the c1 bishop is dark and a8 is light.
const WRONG_BISHOP: &str = "4k3/8/8/8/8/8/P7/2B1K3 w - - 0 1";

/// Bishop and h-pawn: the f1 bishop is light and h8 is dark.
const WRONG_BISHOP_OTHER_CORNER: &str = "4k3/8/8/8/8/8/7P/4KB2 w - - 0 1";

/// Bishop and a-pawn: the d1 bishop is light, as a8 is.
const RIGHT_BISHOP: &str = "4k3/8/8/8/8/8/P7/3BK3 w - - 0 1";

/// A queen each.
const QUEEN_V_QUEEN: &str = "3qk3/8/8/8/8/8/8/3QK3 w - - 0 1";

/// Queen against rook.
const QUEEN_V_ROOK: &str = "3rk3/8/8/8/8/8/8/3QK3 w - - 0 1";

/// Queen against bishop.
const QUEEN_V_BISHOP: &str = "2b1k3/8/8/8/8/8/8/3QK3 w - - 0 1";

/// Queen against knight.
const QUEEN_V_KNIGHT: &str = "1n2k3/8/8/8/8/8/8/3QK3 w - - 0 1";

/// Queen against one pawn.
const QUEEN_V_PAWN: &str = "4k3/8/8/8/4p3/8/8/3QK3 w - - 0 1";

/// Queen against bishop and knight together.
const QUEEN_V_TWO_MINORS: &str = "1bn1k3/8/8/8/8/8/8/3QK3 w - - 0 1";

/// A rook each.
const ROOK_V_ROOK: &str = "3rk3/8/8/8/8/8/8/3RK3 w - - 0 1";

/// Rook against bishop.
const ROOK_V_BISHOP: &str = "2b1k3/8/8/8/8/8/8/3RK3 w - - 0 1";

/// Rook against knight.
const ROOK_V_KNIGHT: &str = "1n2k3/8/8/8/8/8/8/3RK3 w - - 0 1";

/// Rook against one pawn.
const ROOK_V_PAWN: &str = "4k3/8/8/8/4p3/8/8/3RK3 w - - 0 1";

/// Rook against bishop and knight together: the two minors are the stronger
/// side, so the signature writes Black first.
const ROOK_V_TWO_MINORS: &str = "1bn1k3/8/8/8/8/8/8/3RK3 w - - 0 1";

/// Rook and pawn against rook, the black king off the pawn's file.
const ROOK_AND_PAWN: &str = "3rk3/8/8/8/8/8/3P4/3RK3 w - - 0 1";

/// The same material with the black king on d8, in front of the pawn.
const ROOK_AND_PAWN_HELD: &str = "3k4/3r4/8/8/8/8/3P4/3RK3 w - - 0 1";

/// Two bishops against a knight.
const TWO_BISHOPS_V_KNIGHT: &str = "1n2k3/8/8/8/8/8/8/2BBK3 w - - 0 1";

/// Bishop against knight: the bishop side is written first on the tie.
const BISHOP_V_KNIGHT: &str = "1n2k3/8/8/8/8/8/8/3BK3 w - - 0 1";

/// A bishop each, c8 and d1, both light.
const SAME_COLOUR_BISHOPS: &str = "2b1k3/8/8/8/8/8/8/3BK3 w - - 0 1";

/// A bishop each, d8 dark and d1 light.
const OPPOSITE_BISHOPS: &str = "3bk3/8/8/8/8/8/8/3BK3 w - - 0 1";

/// A knight each.
const KNIGHT_V_KNIGHT: &str = "1n2k3/8/8/8/8/8/8/3NK3 w - - 0 1";

/// Bishop against one pawn.
const BISHOP_V_PAWN: &str = "4k3/8/8/8/4p3/8/8/3BK3 w - - 0 1";

/// Knight against one pawn.
const KNIGHT_V_PAWN: &str = "4k3/8/8/8/4p3/8/8/3NK3 w - - 0 1";

/// Two pawns a side and nothing else.
const PAWN_ENDING: &str = "4k3/pp6/8/8/8/8/PP6/4K3 w - - 0 1";

/// Two pawns against a lone king: still a pawn ending, not `KPvK`.
const TWO_PAWNS: &str = "4k3/8/8/8/8/8/PP6/4K3 w - - 0 1";

/// Black holds the queen and White the rook, so the signature writes Black
/// first.
const BLACK_IS_STRONGER: &str = "3qk3/8/8/8/8/8/8/3RK3 w - - 0 1";

/// Queen and rook against a lone king: two pieces, so an ending, and one the
/// catalogue does not name.
const QUEEN_AND_ROOK: &str = "4k3/8/8/8/8/8/8/R2QK3 w - - 0 1";

/// Queen, rook and knight against a lone king: three pieces, one too many.
const THREE_PIECES: &str = "4k3/8/8/8/8/8/8/RN1QK3 w - - 0 1";

/// Two pieces a side, which is still an ending.
const TWO_PIECES_EACH: &str = "1r1qk3/8/8/8/8/8/8/1R1QK3 w - - 0 1";

/// The untouched array.
const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// ------------------------------------------------------------------ helpers

/// The position `fen` describes.
fn position(fen: &str) -> Position {
    Position::from_fen(fen).expect("a test FEN is a legal position")
}

/// The colour a case names: `w` or `b`.
fn colour(name: &str) -> Colour {
    Colour::from_char(name.chars().next().expect("a colour letter")).expect("a colour letter")
}

/// The role a case names, by its lower-case letter.
fn role(name: &str) -> Role {
    Role::from_char(name.chars().next().expect("a role letter")).expect("a role letter")
}

// ---------------------------------------------------------------- signature

#[rstest]
#[case::bare_kings(BARE_KINGS, "KvK", "w")]
#[case::lone_queen(LONE_QUEEN, "KQvK", "w")]
#[case::rook_and_pawn(ROOK_AND_PAWN, "KRPvKR", "w")]
#[case::black_is_stronger(BLACK_IS_STRONGER, "KQvKR", "b")]
#[case::rook_v_two_minors(ROOK_V_TWO_MINORS, "KBNvKR", "b")]
#[case::bishop_v_knight(BISHOP_V_KNIGHT, "KBvKN", "w")]
#[case::pawn_ending(PAWN_ENDING, "KPPvKPP", "w")]
#[case::start(START, "KQRRBBNNPPPPPPPPvKQRRBBNNPPPPPPPP", "w")]
fn a_signature_writes_the_stronger_side_first(
    #[case] fen: &str,
    #[case] text: &str,
    #[case] stronger: &str,
) {
    let signature = endings::classify(&position(fen)).signature;
    assert_eq!(signature.text, text);
    assert_eq!(signature.stronger, colour(stronger));
}

#[rstest]
#[case::white_rook(ROOK_AND_PAWN, "w", "r", 1)]
#[case::white_pawn(ROOK_AND_PAWN, "w", "p", 1)]
#[case::black_rook(ROOK_AND_PAWN, "b", "r", 1)]
#[case::black_pawn(ROOK_AND_PAWN, "b", "p", 0)]
#[case::white_king(ROOK_AND_PAWN, "w", "k", 1)]
#[case::start_black_pawns(START, "b", "p", 8)]
#[case::start_white_bishops(START, "w", "b", 2)]
fn a_signature_counts_every_role_of_every_side(
    #[case] fen: &str,
    #[case] side: &str,
    #[case] unit: &str,
    #[case] count: u8,
) {
    let signature = endings::classify(&position(fen)).signature;
    assert_eq!(signature.count(colour(side), role(unit)), count);
}

#[rstest]
#[case::lone_queen(LONE_QUEEN, "w", 1, 9)]
#[case::rook_and_pawn(ROOK_AND_PAWN, "w", 1, 6)]
#[case::two_pieces_each(TWO_PIECES_EACH, "b", 2, 14)]
fn a_signature_counts_the_pieces_and_the_material_of_a_side(
    #[case] fen: &str,
    #[case] side: &str,
    #[case] pieces: u32,
    #[case] value: u32,
) {
    let signature = endings::classify(&position(fen)).signature;
    assert_eq!(signature.pieces(colour(side)), pieces);
    assert_eq!(signature.value[colour(side).index()], value);
}

// ------------------------------------------------------------- the threshold

#[rstest]
#[case::two_pieces_one_side(QUEEN_AND_ROOK, Class::Other)]
#[case::two_pieces_each(TWO_PIECES_EACH, Class::Other)]
#[case::three_pieces_one_side(THREE_PIECES, Class::NotAnEnding)]
#[case::start(START, Class::NotAnEnding)]
fn a_position_is_an_ending_while_neither_side_has_more_than_two_pieces(
    #[case] fen: &str,
    #[case] class: Class,
) {
    assert_eq!(endings::classify(&position(fen)).class, class);
}

/// The material is still answered for a position that is not an ending.
#[test]
fn a_position_that_is_not_an_ending_still_has_a_signature() {
    let ending = endings::classify(&position(THREE_PIECES));
    assert_eq!(ending.class, Class::NotAnEnding);
    assert_eq!(ending.signature.text, "KQRNvK");
    assert_eq!(ending.verdict, Verdict::Unknown);
}

// ---------------------------------------------------------------- the class

#[rstest]
#[case::bare_kings(BARE_KINGS, Class::KvK)]
#[case::lone_queen(LONE_QUEEN, Class::KQvK)]
#[case::lone_rook(LONE_ROOK, Class::KRvK)]
#[case::two_bishops(TWO_BISHOPS, Class::KBBvK)]
#[case::two_dark_bishops(TWO_DARK_BISHOPS, Class::KBBvK)]
#[case::bishop_and_knight(BISHOP_AND_KNIGHT, Class::KBNvK)]
#[case::two_knights(TWO_KNIGHTS, Class::KNNvK)]
#[case::lone_bishop(LONE_BISHOP, Class::KBvK)]
#[case::lone_knight(LONE_KNIGHT, Class::KNvK)]
#[case::centre_pawn(CENTRE_PAWN, Class::KPvK)]
#[case::wrong_bishop(WRONG_BISHOP, Class::KBPvK)]
#[case::right_bishop(RIGHT_BISHOP, Class::KBPvK)]
#[case::queen_v_queen(QUEEN_V_QUEEN, Class::KQvKQ)]
#[case::queen_v_rook(QUEEN_V_ROOK, Class::KQvKR)]
#[case::queen_v_bishop(QUEEN_V_BISHOP, Class::KQvKB)]
#[case::queen_v_knight(QUEEN_V_KNIGHT, Class::KQvKN)]
#[case::queen_v_pawn(QUEEN_V_PAWN, Class::KQvKP)]
#[case::queen_v_two_minors(QUEEN_V_TWO_MINORS, Class::KQvTwoMinors)]
#[case::rook_v_rook(ROOK_V_ROOK, Class::KRvKR)]
#[case::rook_v_bishop(ROOK_V_BISHOP, Class::KRvKB)]
#[case::rook_v_knight(ROOK_V_KNIGHT, Class::KRvKN)]
#[case::rook_v_pawn(ROOK_V_PAWN, Class::KRvKP)]
#[case::rook_v_two_minors(ROOK_V_TWO_MINORS, Class::KRvTwoMinors)]
#[case::rook_and_pawn(ROOK_AND_PAWN, Class::KRPvKR)]
#[case::two_bishops_v_knight(TWO_BISHOPS_V_KNIGHT, Class::KBBvKN)]
#[case::bishop_v_knight(BISHOP_V_KNIGHT, Class::KBvKN)]
#[case::same_colour_bishops(SAME_COLOUR_BISHOPS, Class::KBvKBSameColour)]
#[case::opposite_bishops(OPPOSITE_BISHOPS, Class::KBvKBOppositeColour)]
#[case::knight_v_knight(KNIGHT_V_KNIGHT, Class::KNvKN)]
#[case::bishop_v_pawn(BISHOP_V_PAWN, Class::KBvKP)]
#[case::knight_v_pawn(KNIGHT_V_PAWN, Class::KNvKP)]
#[case::pawn_ending(PAWN_ENDING, Class::Pawns)]
#[case::two_pawns(TWO_PAWNS, Class::Pawns)]
#[case::queen_and_rook(QUEEN_AND_ROOK, Class::Other)]
fn an_ending_is_classified_by_the_material_alone(#[case] fen: &str, #[case] class: Class) {
    assert_eq!(endings::classify(&position(fen)).class, class);
}

/// Which side holds the material never changes the class, only the verdict.
#[test]
fn the_class_is_the_same_whichever_side_is_stronger() {
    let white = endings::classify(&position(QUEEN_V_ROOK));
    let black = endings::classify(&position(BLACK_IS_STRONGER));
    assert_eq!(white.class, black.class);
    assert_eq!(white.verdict, Verdict::Win(Colour::White));
    assert_eq!(black.verdict, Verdict::Win(Colour::Black));
}

// -------------------------------------------------------------- the verdict

#[rstest]
#[case::bare_kings(BARE_KINGS, Verdict::Draw)]
#[case::lone_queen(LONE_QUEEN, Verdict::Win(Colour::White))]
#[case::lone_rook(LONE_ROOK, Verdict::Win(Colour::White))]
#[case::two_bishops(TWO_BISHOPS, Verdict::Win(Colour::White))]
#[case::two_dark_bishops(TWO_DARK_BISHOPS, Verdict::Draw)]
#[case::bishop_and_knight(BISHOP_AND_KNIGHT, Verdict::Win(Colour::White))]
#[case::two_knights(TWO_KNIGHTS, Verdict::Draw)]
#[case::lone_bishop(LONE_BISHOP, Verdict::Draw)]
#[case::lone_knight(LONE_KNIGHT, Verdict::Draw)]
#[case::centre_pawn(CENTRE_PAWN, Verdict::UsuallyWin(Colour::White))]
#[case::running_pawn(RUNNING_PAWN, Verdict::Win(Colour::White))]
#[case::h_pawn_corner(H_PAWN_CORNER, Verdict::Draw)]
#[case::a_pawn_corner(A_PAWN_CORNER, Verdict::Draw)]
#[case::wrong_bishop(WRONG_BISHOP, Verdict::Draw)]
#[case::wrong_bishop_other_corner(WRONG_BISHOP_OTHER_CORNER, Verdict::Draw)]
#[case::right_bishop(RIGHT_BISHOP, Verdict::UsuallyWin(Colour::White))]
#[case::queen_v_queen(QUEEN_V_QUEEN, Verdict::Draw)]
#[case::queen_v_rook(QUEEN_V_ROOK, Verdict::Win(Colour::White))]
#[case::queen_v_pawn(QUEEN_V_PAWN, Verdict::UsuallyWin(Colour::White))]
#[case::queen_v_two_minors(QUEEN_V_TWO_MINORS, Verdict::Unknown)]
#[case::rook_v_rook(ROOK_V_ROOK, Verdict::Draw)]
#[case::rook_v_bishop(ROOK_V_BISHOP, Verdict::UsuallyDraw(Colour::White))]
#[case::rook_v_knight(ROOK_V_KNIGHT, Verdict::UsuallyDraw(Colour::White))]
#[case::rook_and_pawn(ROOK_AND_PAWN, Verdict::UsuallyWin(Colour::White))]
#[case::rook_and_pawn_held(ROOK_AND_PAWN_HELD, Verdict::UsuallyDraw(Colour::White))]
#[case::two_bishops_v_knight(TWO_BISHOPS_V_KNIGHT, Verdict::UsuallyWin(Colour::White))]
#[case::bishop_v_pawn(BISHOP_V_PAWN, Verdict::UsuallyDraw(Colour::Black))]
#[case::same_colour_bishops(SAME_COLOUR_BISHOPS, Verdict::Draw)]
#[case::pawn_ending(PAWN_ENDING, Verdict::Unknown)]
#[case::black_is_stronger(BLACK_IS_STRONGER, Verdict::Win(Colour::Black))]
fn the_verdict_is_theory_adjusted_by_what_the_position_shows(
    #[case] fen: &str,
    #[case] verdict: Verdict,
) {
    assert_eq!(endings::classify(&position(fen)).verdict, verdict);
}

// ------------------------------------------------------------ the technique

#[rstest]
#[case::bare_kings(BARE_KINGS, Technique::None)]
#[case::lone_queen(LONE_QUEEN, Technique::BoxMethod)]
#[case::lone_rook(LONE_ROOK, Technique::BoxMethod)]
#[case::two_bishops(TWO_BISHOPS, Technique::TwoBishopMate)]
#[case::two_dark_bishops(TWO_DARK_BISHOPS, Technique::None)]
#[case::bishop_and_knight(BISHOP_AND_KNIGHT, Technique::BishopKnightMate)]
#[case::centre_pawn(CENTRE_PAWN, Technique::KeySquares)]
#[case::running_pawn(RUNNING_PAWN, Technique::RuleOfTheSquare)]
#[case::h_pawn_corner(H_PAWN_CORNER, Technique::WrongRookPawn)]
#[case::a_pawn_corner(A_PAWN_CORNER, Technique::WrongRookPawn)]
#[case::wrong_bishop(WRONG_BISHOP, Technique::WrongBishop)]
#[case::right_bishop(RIGHT_BISHOP, Technique::None)]
#[case::rook_and_pawn(ROOK_AND_PAWN, Technique::Lucena)]
#[case::rook_and_pawn_held(ROOK_AND_PAWN_HELD, Technique::Philidor)]
#[case::pawn_ending(PAWN_ENDING, Technique::Opposition)]
#[case::queen_v_rook(QUEEN_V_ROOK, Technique::None)]
fn the_technique_is_the_method_the_ending_is_played_by(
    #[case] fen: &str,
    #[case] technique: Technique,
) {
    assert_eq!(endings::classify(&position(fen)).technique, technique);
}

// ------------------------------------------------------------- the evidence

#[rstest]
#[case::centre_pawn(CENTRE_PAWN, "e3", "e8", false, 5)]
#[case::running_pawn(RUNNING_PAWN, "a2", "a8", true, 5)]
#[case::rook_and_pawn(ROOK_AND_PAWN, "d2", "d8", false, 5)]
#[case::kings_in_opposition(KINGS_IN_OPPOSITION, "e5", "e8", false, 3)]
#[case::queen_v_pawn(QUEEN_V_PAWN, "e4", "e1", false, 3)]
fn a_pawn_race_names_the_pawn_and_the_run_it_has_left(
    #[case] fen: &str,
    #[case] pawn: &str,
    #[case] promotion: &str,
    #[case] rook_pawn: bool,
    #[case] steps: u32,
) {
    let race = endings::classify(&position(fen))
        .evidence
        .pawn
        .expect("one pawn stands on the board");
    assert_eq!(race.pawn.to_string(), pawn);
    assert_eq!(race.promotion.to_string(), promotion);
    assert_eq!(race.rook_pawn, rook_pawn);
    assert_eq!(race.steps, steps);
}

#[rstest]
#[case::centre_pawn(CENTRE_PAWN, true, false, true)]
#[case::running_pawn(RUNNING_PAWN, false, false, false)]
#[case::h_pawn_corner(H_PAWN_CORNER, true, false, false)]
#[case::kings_in_opposition(KINGS_IN_OPPOSITION, true, true, true)]
#[case::rook_and_pawn_held(ROOK_AND_PAWN_HELD, true, false, true)]
fn a_pawn_race_says_who_stands_where_in_the_race(
    #[case] fen: &str,
    #[case] inside_square: bool,
    #[case] attacker_in_front: bool,
    #[case] defender_in_front: bool,
) {
    let race = endings::classify(&position(fen))
        .evidence
        .pawn
        .expect("one pawn stands on the board");
    assert_eq!(race.defender_inside_square, inside_square);
    assert_eq!(race.attacker_in_front, attacker_in_front);
    assert_eq!(race.defender_in_front, defender_in_front);
}

/// More than one pawn, or none, leaves nothing one race can be read off.
#[rstest]
#[case::bare_kings(BARE_KINGS)]
#[case::pawn_ending(PAWN_ENDING)]
#[case::two_pawns(TWO_PAWNS)]
fn a_position_without_exactly_one_pawn_has_no_pawn_race(#[case] fen: &str) {
    assert_eq!(endings::classify(&position(fen)).evidence.pawn, None);
}

#[rstest]
#[case::two_bishops(TWO_BISHOPS, false, false, false)]
#[case::two_dark_bishops(TWO_DARK_BISHOPS, false, true, false)]
#[case::same_colour_bishops(SAME_COLOUR_BISHOPS, false, true, false)]
#[case::opposite_bishops(OPPOSITE_BISHOPS, true, false, false)]
#[case::wrong_bishop(WRONG_BISHOP, false, true, true)]
#[case::wrong_bishop_other_corner(WRONG_BISHOP_OTHER_CORNER, false, true, true)]
#[case::right_bishop(RIGHT_BISHOP, false, true, false)]
fn the_bishops_say_which_squares_they_can_reach(
    #[case] fen: &str,
    #[case] opposite_colours: bool,
    #[case] same_colour: bool,
    #[case] wrong_bishop: bool,
) {
    let bishops = endings::classify(&position(fen))
        .evidence
        .bishops
        .expect("a bishop stands on the board");
    assert_eq!(bishops.opposite_colours, opposite_colours);
    assert_eq!(bishops.same_colour, same_colour);
    assert_eq!(bishops.wrong_bishop, wrong_bishop);
}

#[rstest]
#[case::bare_kings(BARE_KINGS)]
#[case::lone_rook(LONE_ROOK)]
#[case::pawn_ending(PAWN_ENDING)]
fn a_position_without_a_bishop_says_nothing_about_bishops(#[case] fen: &str) {
    assert_eq!(endings::classify(&position(fen)).evidence.bishops, None);
}

#[rstest]
#[case::kings_in_opposition(KINGS_IN_OPPOSITION, true)]
#[case::centre_pawn(CENTRE_PAWN, false)]
#[case::bare_kings(BARE_KINGS, false)]
fn the_evidence_says_whether_the_kings_stand_in_opposition(
    #[case] fen: &str,
    #[case] opposition: bool,
) {
    assert_eq!(
        endings::classify(&position(fen)).evidence.opposition,
        opposition
    );
}

// ----------------------------------------------------------------- the prose

#[rstest]
#[case::classes(&Class::ALL.map(Class::describe))]
#[case::techniques(&Technique::ALL.map(Technique::describe))]
fn every_value_of_an_enum_has_a_sentence_of_its_own(#[case] sentences: &[&'static str]) {
    for sentence in sentences {
        assert!(!sentence.is_empty());
        assert!(sentence.ends_with('.'), "{sentence}");
    }
}

#[test]
fn every_verdict_has_a_sentence_of_its_own() {
    let verdicts = [
        Verdict::Win(Colour::White),
        Verdict::UsuallyWin(Colour::Black),
        Verdict::UsuallyDraw(Colour::White),
        Verdict::Draw,
        Verdict::Unknown,
    ];
    for verdict in verdicts {
        let sentence = verdict.describe();
        assert!(!sentence.is_empty());
        assert!(sentence.ends_with('.'), "{sentence}");
    }
}

#[rstest]
#[case::k_v_k(Class::KvK, "k_v_k")]
#[case::krp_v_kr(Class::KRPvKR, "krp_v_kr")]
#[case::kb_v_kb_opposite_colour(Class::KBvKBOppositeColour, "kb_v_kb_opposite_colour")]
#[case::kq_v_two_minors(Class::KQvTwoMinors, "kq_v_two_minors")]
#[case::not_an_ending(Class::NotAnEnding, "not_an_ending")]
fn a_class_is_named_as_the_python_surface_names_it(#[case] class: Class, #[case] name: &str) {
    assert_eq!(class.name(), name);
}

/// The catalogue holds each name once.
#[test]
fn no_two_classes_share_a_name() {
    let mut names: Vec<&str> = Class::ALL.iter().map(|class| class.name()).collect();
    names.sort_unstable();
    let count = names.len();
    names.dedup();
    assert_eq!(names.len(), count);
}

#[test]
fn a_class_says_in_one_sentence_what_the_ending_is() {
    assert_eq!(
        Class::KvK.describe(),
        "Only the two kings are left, so nothing can be won."
    );
}

#[test]
fn a_verdict_names_the_side_it_gives_the_ending_to() {
    assert_eq!(
        Verdict::Win(Colour::White).describe(),
        "White wins this ending by force, against any defence."
    );
    assert_eq!(Verdict::Win(Colour::Black).winner(), Some(Colour::Black));
    assert_eq!(Verdict::Draw.winner(), None);
    assert_eq!(Verdict::Unknown.name(), "unknown");
}

#[test]
fn a_signature_says_in_one_sentence_what_each_side_has() {
    assert_eq!(
        endings::classify(&position(ROOK_AND_PAWN))
            .signature
            .describe(),
        "The material is KRPvKR: White has a rook and a pawn, Black has a rook."
    );
    assert_eq!(
        endings::classify(&position(BARE_KINGS))
            .signature
            .describe(),
        "The material is KvK: White has nothing besides its king, Black has nothing besides \
         its king."
    );
}

#[test]
fn evidence_that_applies_to_nothing_says_so() {
    assert_eq!(
        endings::classify(&position(BARE_KINGS)).evidence.describe(),
        "Nothing in this position changes what theory says about the ending."
    );
}

/// The whole answer reads as the material, the ending, the result and the
/// method, in that order.
#[test]
fn an_ending_describes_itself_from_its_parts() {
    let ending = endings::classify(&position(LONE_ROOK));
    let prose = ending.describe();
    assert!(prose.starts_with(&ending.signature.describe()));
    assert!(prose.contains(ending.class.describe()));
    assert!(prose.contains(&ending.verdict.describe()));
    assert!(prose.ends_with(ending.technique.describe()));
}

/// A position that is not an ending is answered with its material and no
/// theory.
#[test]
fn a_position_that_is_not_an_ending_describes_only_its_material() {
    let ending = endings::classify(&position(START));
    assert_eq!(
        ending.describe(),
        format!(
            "{} Too much material is left for this to be an ending at all.",
            ending.signature.describe()
        )
    );
}

// ------------------------------------------------------------- the surfaces

#[test]
fn a_position_and_a_game_answer_the_same_ending() {
    let position = position(ROOK_AND_PAWN);
    let game = Game::from_fen(classic(), ROOK_AND_PAWN).expect("a test FEN is a legal position");
    assert_eq!(position.ending(), endings::classify(&position));
    assert_eq!(game.ending(), endings::classify(&position));
}
