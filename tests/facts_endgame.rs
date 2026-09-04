//! The `endgame` group, fact by fact.
//!
//! Every expectation is worked out from the definitions in `docs/features.md`
//! §1 and §2.12 for the named position above it.

mod common;

use common::facts_of;
use esca::{DrawishMaterial, Opposition, Schema, Side};
use rstest::rstest;

/// The untouched array: no passer, no opposition, no ending.
const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// Kings on d3 and d5 with d4 between them: the direct opposition.
const BARE_KINGS: &str = "8/8/8/3k4/8/3K4/8/8 w - - 0 1";

/// The same with Black to move, so the opposition changes hands and the
/// one-hot, which says only which kind stands, does not.
const BARE_KINGS_BLACK: &str = "8/8/8/3k4/8/3K4/8/8 b - - 0 1";

/// Kings on e2 and e8: five empty squares between them.
const DISTANT: &str = "4k3/8/8/8/8/8/4K3/8 w - - 0 1";

/// Kings on e2 and e7: four, so neither side has the opposition.
const NO_OPPOSITION: &str = "8/4k3/8/8/8/8/4K3/8 w - - 0 1";

/// A pawn on e5 stands between the kings, so the file is no corridor.
const BLOCKED_FILE: &str = "8/8/4k3/4p3/4K3/8/8/8 w - - 0 1";

/// Kings on c3 and e5: the opposition holds on a diagonal too.
const DIAGONAL: &str = "8/8/8/4k3/8/2K5/8/8 w - - 0 1";

/// A pawn each on opposite wings, the black one a move from queening with its
/// king already on the squares it promotes through.
const PAWN_RACE: &str = "8/8/8/P7/8/8/6p1/K6k w - - 0 1";

/// The same with Black to move: the tempo takes a ply off Black's race.
const PAWN_RACE_BLACK: &str = "8/8/8/P7/8/8/6p1/K6k b - - 0 1";

/// Pawns that block each other head on: neither side has a passer.
const BLOCKED_PAWNS: &str = "8/8/8/3p4/3P4/8/8/K6k w - - 0 1";

/// The white king two ranks ahead of its passer on e4, on a key square.
const KEY_SQUARE: &str = "8/8/4K3/8/4P3/8/8/4k3 w - - 0 1";

/// The same with Black to move, so the key square is theirs.
const KEY_SQUARE_BLACK: &str = "8/8/4K3/8/4P3/8/8/4k3 b - - 0 1";

/// A passer on the fifth: its key squares are the three squares in front.
const KEY_SQUARE_HIGH: &str = "8/8/3K4/4P3/8/8/8/4k3 w - - 0 1";

/// The king on b6 escorts an a-pawn, which has no key squares at all.
const ROOK_PAWN_KING: &str = "8/8/1K6/P7/8/8/8/4k3 w - - 0 1";

/// The king right in front of its passer, a rank short of a key square.
const SHORT_OF_KEY: &str = "8/8/8/4K3/4P3/8/8/4k3 w - - 0 1";

/// The king on e6 with the e4 pawn no longer passed: the d6 pawn stops it.
const KEY_SQUARE_NOT_PASSED: &str = "8/8/3pK3/8/4P3/8/8/k7 w - - 0 1";

/// A light-squared bishop with an h-pawn, which promotes on a dark square.
const WRONG_BISHOP: &str = "7k/8/8/8/8/7P/6B1/6K1 w - - 0 1";

/// The same with Black to move: the wrong bishop is theirs.
const WRONG_BISHOP_BLACK: &str = "7k/8/8/8/8/7P/6B1/6K1 b - - 0 1";

/// The same bishop on h2, the colour its own h-pawn promotes on.
const RIGHT_BISHOP: &str = "7k/8/8/8/8/7P/7B/6K1 w - - 0 1";

/// Rook pawns on both wings: the a-pawn promotes on the bishop's colour.
const BOTH_ROOK_PAWNS: &str = "7k/8/8/8/8/P6P/6B1/6K1 w - - 0 1";

/// A bishop whose only pawn stands on e3: not a rook pawn.
const CENTRE_PAWN_BISHOP: &str = "7k/8/8/8/8/4P3/6B1/6K1 w - - 0 1";

/// Black's dark-squared bishop against its own h-pawn, promoting on h1.
const WRONG_BISHOP_THEM: &str = "6k1/6b1/7p/8/8/8/8/7K w - - 0 1";

/// Two knights against a bare king: no forced mate.
const TWO_KNIGHTS: &str = "8/8/8/3k4/8/8/1NN5/3K4 w - - 0 1";

/// The same two knights on the other side.
const TWO_KNIGHTS_THEM: &str = "3k4/8/1nn5/8/8/8/8/3K4 w - - 0 1";

/// Two knights and a pawn: the pawn takes the material out of the drawn set.
const TWO_KNIGHTS_AND_PAWN: &str = "8/8/8/3k4/8/8/1NN2P2/3K4 w - - 0 1";

/// One bishop each on opposite colours, with a pawn each and no other piece.
const OPPOSITE_BISHOPS: &str = "8/3k4/4p3/4b3/3P4/3B4/8/3K4 w - - 0 1";

/// Both bishops on light squares.
const SAME_COLOUR_BISHOPS: &str = "8/3k4/2b1p3/8/8/3B4/8/3K4 w - - 0 1";

/// The opposite bishops with a knight still on: a piece too many.
const OPPOSITE_BISHOPS_AND_KNIGHT: &str = "8/3k4/4p3/4b3/3P4/3B4/5N2/3K4 w - - 0 1";

/// The 15 values of the `endgame` row of `fen`.
fn endgame_row(fen: &str) -> Vec<f32> {
    let schema = Schema::v1();
    let groups = schema.group_set(&["endgame"]).expect("the endgame group");
    facts_of(fen).encode(schema, groups)
}

#[rstest]
#[case::start(START, [3, 3])]
#[case::bare_kings(BARE_KINGS, [1, 0])]
#[case::bare_kings_black(BARE_KINGS_BLACK, [0, 1])]
#[case::distant(DISTANT, [2, 3])]
#[case::no_opposition(NO_OPPOSITION, [2, 2])]
#[case::blocked_file(BLOCKED_FILE, [0, 1])]
#[case::key_square(KEY_SQUARE, [1, 3])]
#[case::key_square_black(KEY_SQUARE_BLACK, [3, 1])]
#[case::two_knights(TWO_KNIGHTS, [3, 0])]
#[case::opposite_bishops(OPPOSITE_BISHOPS, [3, 2])]
fn the_king_centralisation_is_the_distance_to_the_nearest_central_square(
    #[case] fen: &str,
    #[case] distance: [u8; 2],
) {
    let endgame = facts_of(fen).endgame;
    assert_eq!(endgame.king_centralisation[Side::Us.index()], distance[0]);
    assert_eq!(endgame.king_centralisation[Side::Them.index()], distance[1]);
}

#[rstest]
#[case::start(START, [8, 8])]
#[case::blocked_pawns(BLOCKED_PAWNS, [8, 8])]
#[case::pawn_race(PAWN_RACE, [2, 1])]
#[case::pawn_race_black(PAWN_RACE_BLACK, [0, 3])]
#[case::blocked_file(BLOCKED_FILE, [8, 4])]
#[case::key_square(KEY_SQUARE, [3, 8])]
#[case::key_square_black(KEY_SQUARE_BLACK, [8, 4])]
#[case::key_square_high(KEY_SQUARE_HIGH, [2, 8])]
#[case::wrong_bishop(WRONG_BISHOP, [4, 8])]
#[case::wrong_bishop_them(WRONG_BISHOP_THEM, [8, 5])]
fn race_plies_are_what_the_leading_passer_still_needs(#[case] fen: &str, #[case] plies: [u8; 2]) {
    let endgame = facts_of(fen).endgame;
    assert_eq!(endgame.race_plies[Side::Us.index()], plies[0]);
    assert_eq!(endgame.race_plies[Side::Them.index()], plies[1]);
}

#[rstest]
#[case::start(START, 0)]
#[case::pawn_race(PAWN_RACE, 1)]
#[case::pawn_race_black(PAWN_RACE_BLACK, -3)]
#[case::blocked_file(BLOCKED_FILE, 4)]
#[case::key_square(KEY_SQUARE, -5)]
#[case::key_square_high(KEY_SQUARE_HIGH, -6)]
#[case::wrong_bishop_them(WRONG_BISHOP_THEM, 3)]
fn the_race_difference_is_ours_less_theirs(#[case] fen: &str, #[case] difference: i32) {
    assert_eq!(facts_of(fen).endgame.race_plies_diff(), difference);
}

#[rstest]
#[case::start(START, None)]
#[case::bare_kings(BARE_KINGS, Some(Opposition::Direct))]
#[case::bare_kings_black(BARE_KINGS_BLACK, Some(Opposition::Direct))]
#[case::diagonal(DIAGONAL, Some(Opposition::Direct))]
#[case::distant(DISTANT, Some(Opposition::Distant))]
#[case::two_knights(TWO_KNIGHTS, Some(Opposition::Distant))]
#[case::no_opposition(NO_OPPOSITION, None)]
#[case::blocked_file(BLOCKED_FILE, None)]
#[case::pawn_race(PAWN_RACE, None)]
fn the_opposition_needs_an_odd_number_of_empty_squares_between_the_kings(
    #[case] fen: &str,
    #[case] opposition: Option<Opposition>,
) {
    assert_eq!(facts_of(fen).endgame.opposition, opposition);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::key_square(KEY_SQUARE, [true, false])]
#[case::key_square_black(KEY_SQUARE_BLACK, [false, true])]
#[case::key_square_high(KEY_SQUARE_HIGH, [true, false])]
#[case::pawn_race(PAWN_RACE, [false, true])]
#[case::pawn_race_black(PAWN_RACE_BLACK, [true, false])]
#[case::rook_pawn_king(ROOK_PAWN_KING, [false, false])]
#[case::short_of_key(SHORT_OF_KEY, [false, false])]
#[case::not_passed(KEY_SQUARE_NOT_PASSED, [false, false])]
fn the_king_stands_on_a_key_square_of_a_passer_of_its_own(
    #[case] fen: &str,
    #[case] occupied: [bool; 2],
) {
    let endgame = facts_of(fen).endgame;
    assert_eq!(endgame.key_square_occupied[Side::Us.index()], occupied[0]);
    assert_eq!(endgame.key_square_occupied[Side::Them.index()], occupied[1]);
}

#[rstest]
#[case::start(START, [false, false])]
#[case::wrong_bishop(WRONG_BISHOP, [true, false])]
#[case::wrong_bishop_black(WRONG_BISHOP_BLACK, [false, true])]
#[case::wrong_bishop_them(WRONG_BISHOP_THEM, [false, true])]
#[case::right_bishop(RIGHT_BISHOP, [false, false])]
#[case::both_rook_pawns(BOTH_ROOK_PAWNS, [false, false])]
#[case::centre_pawn(CENTRE_PAWN_BISHOP, [false, false])]
#[case::opposite_bishops(OPPOSITE_BISHOPS, [false, false])]
fn a_bishop_is_the_wrong_colour_for_rook_pawns_promoting_on_the_other(
    #[case] fen: &str,
    #[case] wrong: [bool; 2],
) {
    let endgame = facts_of(fen).endgame;
    assert_eq!(endgame.wrong_colour_bishop[Side::Us.index()], wrong[0]);
    assert_eq!(endgame.wrong_colour_bishop[Side::Them.index()], wrong[1]);
}

#[rstest]
#[case::start(START, None)]
#[case::bare_kings(BARE_KINGS, None)]
#[case::two_knights(TWO_KNIGHTS, Some(DrawishMaterial::TwoKnights))]
#[case::two_knights_them(TWO_KNIGHTS_THEM, Some(DrawishMaterial::TwoKnights))]
#[case::two_knights_and_pawn(TWO_KNIGHTS_AND_PAWN, None)]
#[case::wrong_bishop(WRONG_BISHOP, Some(DrawishMaterial::WrongBishop))]
#[case::wrong_bishop_them(WRONG_BISHOP_THEM, Some(DrawishMaterial::WrongBishop))]
#[case::right_bishop(RIGHT_BISHOP, None)]
#[case::opposite_bishops(OPPOSITE_BISHOPS, Some(DrawishMaterial::OppositeBishops))]
#[case::same_colour_bishops(SAME_COLOUR_BISHOPS, None)]
#[case::opposite_bishops_and_knight(OPPOSITE_BISHOPS_AND_KNIGHT, None)]
fn drawish_material_names_the_three_configurations_that_still_draw(
    #[case] fen: &str,
    #[case] drawn: Option<DrawishMaterial>,
) {
    assert_eq!(facts_of(fen).endgame.drawish_material, drawn);
}

/// The row is the seven features in schema order: two centralisations, two
/// race counts, their difference, the opposition one-hot with its third slot
/// for none, the two bit pairs, and the drawn-material one-hot.
#[rstest]
#[case::bare_kings(BARE_KINGS, [
    1.0 / 3.0, 0.0,
    1.0, 1.0,
    0.0,
    1.0, 0.0, 0.0,
    0.0, 0.0,
    0.0, 0.0,
    0.0, 0.0, 0.0,
])]
#[case::pawn_race(PAWN_RACE, [
    1.0, 1.0,
    0.25, 0.125,
    0.125,
    0.0, 0.0, 1.0,
    0.0, 1.0,
    0.0, 0.0,
    0.0, 0.0, 0.0,
])]
#[case::wrong_bishop(WRONG_BISHOP, [
    1.0, 1.0,
    0.5, 1.0,
    -0.5,
    0.0, 0.0, 1.0,
    0.0, 0.0,
    1.0, 0.0,
    0.0, 1.0, 0.0,
])]
fn the_encoded_row_carries_the_group_in_the_schemas_order(
    #[case] fen: &str,
    #[case] row: [f32; 15],
) {
    assert_eq!(endgame_row(fen), row);
}
