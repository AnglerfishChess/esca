//! The `pieces` group, fact by fact.
//!
//! Every expectation is worked out from the definitions in `docs/features.md`
//! §1 and §2.4 for the named position above it.

mod common;

use common::{facts_of, facts_under, squares};
use esca::{CHESS960, Schema, Side};
use rstest::rstest;

/// The untouched array: a bishop of each colour a side, every minor at home.
const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// The Italian after 4…Nf6: both queens are out, Black's knights both are.
const ITALIAN: &str = "r1b1k2r/ppppqppp/2n2n2/2b1p3/2B1P3/5N2/PPPPQPPP/RNB1K2R w KQkq - 8 6";

/// One bishop each, on unlike colours, with pawns on both colours a side.
const OPPOSITE: &str = "4k3/pp3p2/3b4/8/2B5/8/PP2P3/4K3 w - - 0 1";

/// Black to move, so the flip makes b7 a dark square and e3 a light one.
const FLIPPED: &str = "6k1/1b6/4p3/3n4/8/4B3/5P2/4K3 b - - 0 1";

/// White's two bishops share a colour and Black's do not.
const SAME_COLOUR: &str = "4k3/1b3p2/2p5/4p1b1/1P6/1B3P2/6B1/4K3 w - - 0 1";

/// Both back ranks cleared between the rooks; the f-file is Black's alone to use.
const LINED: &str = "r4rk1/6pp/8/8/8/5PP1/7P/3R1RK1 w - - 0 1";

/// Two white rooks on the enemy pawn rank; Black's own two are split by a knight.
const SEVENTH: &str = "3r2k1/R3R1pp/8/3N4/8/8/3r2PP/6K1 w - - 0 1";

/// Black's h8 rook has nowhere to go and no castling left; White may still castle.
const CORNERED: &str = "r2q2kr/pp4pp/8/4Q3/8/8/PP4PP/R3K2R w KQ - 0 1";

/// The mirror image: White's h1 rook is the boxed one, Black's f8 rook is free.
const BOXED: &str = "5rk1/1b4pp/8/8/3B4/8/6PP/6KR w - - 0 1";

/// A rook outside its king on the king's own wing, with the whole a-file to itself.
const OPEN_CORNER: &str = "4k3/8/8/8/8/8/1PP5/R1K5 w - - 0 1";

/// One passer a side, each with a friendly rook behind it and an enemy rook too.
const PASSER_ROOKS: &str = "6k1/5R2/2P2r2/8/5p2/2r5/2R5/6K1 w - - 0 1";

/// Both sides double their rooks behind their own passed pawn.
const BATTERY: &str = "6k1/8/6r1/2P3r1/8/6p1/2R5/2R3K1 w - - 0 1";

/// Three outpost squares a side, two of White's held by knights and one of Black's.
const OUTPOSTS: &str = "6k1/8/3p1p2/1N1Nn3/2P1P3/8/8/6K1 w - - 0 1";

/// The same three White outpost squares, two of them held by a knight and a
/// bishop; Black has minors on none of its own.
const MINOR_OUTPOSTS: &str = "7k/8/3p1p2/1N1B4/2P1P3/8/8/6K1 w - - 0 1";

/// The same placement with Black to move: the two occupied outposts are theirs.
const MINOR_OUTPOSTS_BLACK: &str = "7k/8/3p1p2/1N1B4/2P1P3/8/8/6K1 b - - 0 1";

/// The mirror image: a black knight and a black bishop on black outpost
/// squares, and White with three free ones.
const MINOR_OUTPOSTS_MIRROR: &str = "6k1/8/8/2p1p3/1n1b4/3P1P2/7K/8 w - - 0 1";

/// The a7 and h7 pawns veto b5 and g5; the knights stand on no outpost at all.
const HOLES: &str = "6k1/p6p/8/1N4n1/2P1P3/8/8/6K1 w - - 0 1";

/// Knights on the a- and h-files and on either back rank, and one in the centre.
const RIM: &str = "2N3k1/8/4n3/n6N/8/8/8/1n4K1 w - - 0 1";

/// A Chess960 middlegame: the rooks start on d and f, the king between them on e.
const NINE_SIXTY: &str = "nnqrkr1b/1p1pp3/3p4/p4ppp/PP5P/1b2PP2/2PPK1P1/N1QR1RBB w fd - 0 10";

/// The same placement with the castling rights spent, which classic chess reads too.
const NINE_SIXTY_CLASSIC: &str =
    "nnqrkr1b/1p1pp3/3p4/p4ppp/PP5P/1b2PP2/2PPK1P1/N1QR1RBB w - - 0 10";

#[rstest]
#[case::start(START, [true, true])]
#[case::italian(ITALIAN, [true, true])]
#[case::same_colour(SAME_COLOUR, [false, true])]
#[case::opposite(OPPOSITE, [false, false])]
#[case::flipped(FLIPPED, [false, false])]
#[case::lined(LINED, [false, false])]
fn a_bishop_pair_needs_a_bishop_of_each_square_colour(#[case] fen: &str, #[case] pair: [bool; 2]) {
    assert_eq!(facts_of(fen).pieces.bishop_pair, pair);
}

#[rstest]
#[case::start(START, [1, 1], [1, 1])]
#[case::opposite(OPPOSITE, [1, 0], [0, 1])]
#[case::same_colour(SAME_COLOUR, [2, 1], [0, 1])]
#[case::flipped(FLIPPED, [0, 1], [1, 0])]
#[case::boxed(BOXED, [0, 1], [1, 0])]
#[case::lined(LINED, [0, 0], [0, 0])]
fn bishops_are_counted_by_the_square_colour_the_mover_sees(
    #[case] fen: &str,
    #[case] light: [u8; 2],
    #[case] dark: [u8; 2],
) {
    let pieces = facts_of(fen).pieces;
    assert_eq!(pieces.bishops_light, light);
    assert_eq!(pieces.bishops_dark, dark);
}

#[rstest]
#[case::opposite(OPPOSITE, true)]
#[case::flipped(FLIPPED, true)]
#[case::boxed(BOXED, true)]
#[case::start(START, false)]
#[case::same_colour(SAME_COLOUR, false)]
#[case::lined(LINED, false)]
fn bishops_are_opposite_coloured_when_one_each_stands_on_unlike_colours(
    #[case] fen: &str,
    #[case] opposite: bool,
) {
    assert_eq!(facts_of(fen).pieces.opposite_coloured_bishops, opposite);
}

#[rstest]
#[case::start(START, [8, 8])]
#[case::italian(ITALIAN, [8, 8])]
#[case::opposite(OPPOSITE, [2, 1])]
#[case::same_colour(SAME_COLOUR, [1, 3])]
#[case::flipped(FLIPPED, [1, 1])]
#[case::lined(LINED, [0, 0])]
fn a_pawn_counts_on_the_bishop_colour_when_an_own_bishop_shares_its_colour(
    #[case] fen: &str,
    #[case] pawns: [u8; 2],
) {
    assert_eq!(facts_of(fen).pieces.pawns_on_bishop_colour, pawns);
}

#[rstest]
#[case::lined(LINED, [true, true])]
#[case::seventh(SEVENTH, [true, false])]
#[case::cornered(CORNERED, [false, false])]
#[case::battery(BATTERY, [false, false])]
#[case::start(START, [false, false])]
fn rooks_are_connected_on_a_rank_when_nothing_stands_between_them(
    #[case] fen: &str,
    #[case] connected: [bool; 2],
) {
    assert_eq!(facts_of(fen).pieces.rooks_connected_rank, connected);
}

#[rstest]
#[case::battery(BATTERY, [true, true])]
#[case::seventh(SEVENTH, [false, false])]
#[case::lined(LINED, [false, false])]
#[case::cornered(CORNERED, [false, false])]
#[case::start(START, [false, false])]
fn rooks_are_connected_on_a_file_when_nothing_stands_between_them(
    #[case] fen: &str,
    #[case] connected: [bool; 2],
) {
    assert_eq!(facts_of(fen).pieces.rooks_connected_file, connected);
}

#[rstest]
#[case::seventh(SEVENTH, [2, 2])]
#[case::lined(LINED, [1, 1])]
#[case::open_corner(OPEN_CORNER, [1, 0])]
#[case::boxed(BOXED, [0, 1])]
#[case::passer_rooks(PASSER_ROOKS, [0, 0])]
#[case::start(START, [0, 0])]
fn a_rook_is_on_an_open_file_when_no_pawn_of_either_colour_holds_it(
    #[case] fen: &str,
    #[case] rooks: [u8; 2],
) {
    assert_eq!(facts_of(fen).pieces.rooks_on_open_file, rooks);
}

#[rstest]
#[case::passer_rooks(PASSER_ROOKS, [1, 1])]
#[case::lined(LINED, [0, 1])]
#[case::seventh(SEVENTH, [0, 0])]
#[case::battery(BATTERY, [0, 0])]
#[case::start(START, [0, 0])]
fn a_rook_is_on_a_semi_open_file_when_only_its_own_side_has_left_it(
    #[case] fen: &str,
    #[case] rooks: [u8; 2],
) {
    assert_eq!(facts_of(fen).pieces.rooks_on_semi_open_file, rooks);
}

#[rstest]
#[case::seventh(SEVENTH, [2, 1])]
#[case::passer_rooks(PASSER_ROOKS, [1, 0])]
#[case::lined(LINED, [0, 0])]
#[case::battery(BATTERY, [0, 0])]
#[case::start(START, [0, 0])]
fn the_relative_seventh_is_counted_from_the_rooks_own_back_rank(
    #[case] fen: &str,
    #[case] rooks: [u8; 2],
) {
    assert_eq!(facts_of(fen).pieces.rooks_on_relative_7th, rooks);
}

#[rstest]
#[case::battery(BATTERY, [2, 2])]
#[case::passer_rooks(PASSER_ROOKS, [1, 1])]
#[case::seventh(SEVENTH, [0, 0])]
#[case::lined(LINED, [0, 0])]
#[case::start(START, [0, 0])]
fn a_rook_behind_an_own_passer_shares_its_file_at_a_lower_relative_rank(
    #[case] fen: &str,
    #[case] rooks: [u8; 2],
) {
    assert_eq!(facts_of(fen).pieces.rook_behind_own_passer, rooks);
}

#[rstest]
#[case::passer_rooks(PASSER_ROOKS, [1, 1])]
#[case::battery(BATTERY, [0, 0])]
#[case::seventh(SEVENTH, [0, 0])]
#[case::lined(LINED, [0, 0])]
#[case::start(START, [0, 0])]
fn behind_an_enemy_passer_is_read_in_the_passer_owners_frame(
    #[case] fen: &str,
    #[case] rooks: [u8; 2],
) {
    assert_eq!(facts_of(fen).pieces.rook_behind_enemy_passer, rooks);
}

#[rstest]
#[case::cornered(CORNERED, [false, true])]
#[case::boxed(BOXED, [true, false])]
#[case::open_corner(OPEN_CORNER, [false, false])]
#[case::lined(LINED, [false, false])]
#[case::start(START, [false, false])]
fn a_trapped_rook_is_boxed_in_beyond_its_own_king_with_the_castling_rights_gone(
    #[case] fen: &str,
    #[case] trapped: [bool; 2],
) {
    assert_eq!(facts_of(fen).pieces.trapped_rook, trapped);
}

#[rstest]
#[case::outposts(OUTPOSTS, "b5 d5 f5", "c5 e5 g5")]
#[case::same_colour(SAME_COLOUR, "a5 c5", "b5 d4 d5 f4")]
#[case::holes(HOLES, "d5 f5", "")]
#[case::flipped(FLIPPED, "d5 f5", "")]
#[case::battery(BATTERY, "b6 d6", "")]
#[case::passer_rooks(PASSER_ROOKS, "", "e3 g3")]
#[case::lined(LINED, "e4", "")]
#[case::start(START, "", "")]
fn an_outpost_square_is_pawn_held_ground_on_ranks_four_to_six(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let pieces = facts_of(fen).pieces;
    assert_eq!(pieces.outposts[Side::Us.index()], squares(us));
    assert_eq!(pieces.outposts[Side::Them.index()], squares(them));
}

#[rstest]
#[case::outposts(OUTPOSTS, [2, 1])]
#[case::minor_outposts(MINOR_OUTPOSTS, [2, 0])]
#[case::minor_outposts_black(MINOR_OUTPOSTS_BLACK, [0, 2])]
#[case::minor_outposts_mirror(MINOR_OUTPOSTS_MIRROR, [0, 2])]
#[case::flipped(FLIPPED, [1, 0])]
#[case::holes(HOLES, [0, 0])]
#[case::rim(RIM, [0, 0])]
#[case::start(START, [0, 0])]
fn a_minor_on_an_outpost_is_a_knight_or_a_bishop_on_an_own_outpost_square(
    #[case] fen: &str,
    #[case] minors: [u8; 2],
) {
    assert_eq!(facts_of(fen).pieces.minors_on_outpost, minors);
}

#[rstest]
#[case::same_colour(SAME_COLOUR, [2, 4])]
#[case::outposts(OUTPOSTS, [1, 2])]
#[case::holes(HOLES, [2, 0])]
#[case::passer_rooks(PASSER_ROOKS, [0, 2])]
#[case::flipped(FLIPPED, [1, 0])]
#[case::start(START, [0, 0])]
fn a_free_outpost_square_is_one_no_unit_of_either_colour_occupies(
    #[case] fen: &str,
    #[case] free: [u8; 2],
) {
    assert_eq!(facts_of(fen).pieces.outpost_squares_free, free);
}

#[rstest]
#[case::start(START, [2, 2])]
#[case::rim(RIM, [2, 2])]
#[case::italian(ITALIAN, [1, 0])]
#[case::outposts(OUTPOSTS, [0, 0])]
#[case::holes(HOLES, [0, 0])]
fn a_knight_is_on_the_rim_on_file_a_or_h_or_on_relative_rank_one_or_eight(
    #[case] fen: &str,
    #[case] knights: [u8; 2],
) {
    assert_eq!(facts_of(fen).pieces.knights_on_rim, knights);
}

#[rstest]
#[case::start(START, [4, 4])]
#[case::italian(ITALIAN, [2, 1])]
#[case::nine_sixty_classic(NINE_SIXTY_CLASSIC, [1, 1])]
#[case::opposite(OPPOSITE, [0, 0])]
#[case::rim(RIM, [0, 0])]
fn an_undeveloped_minor_still_stands_on_a_classic_starting_square(
    #[case] fen: &str,
    #[case] minors: [u8; 2],
) {
    assert_eq!(facts_of(fen).pieces.minors_undeveloped, minors);
}

#[rstest]
#[case::italian(ITALIAN, [true, true])]
#[case::nine_sixty_classic(NINE_SIXTY_CLASSIC, [true, true])]
#[case::cornered(CORNERED, [true, false])]
#[case::start(START, [false, false])]
#[case::opposite(OPPOSITE, [false, false])]
fn a_queen_is_developed_once_it_stands_off_its_classic_starting_square(
    #[case] fen: &str,
    #[case] developed: [bool; 2],
) {
    assert_eq!(facts_of(fen).pieces.queen_developed, developed);
}

/// Only `minors_undeveloped` and `queen_developed` read the starting squares, so
/// the rest of the group answers for a Chess960 placement as for any other.
#[test]
fn the_piece_facts_of_a_chess960_position_read_the_placement_as_they_find_it() {
    let pieces = facts_under(&CHESS960, NINE_SIXTY).pieces;
    assert_eq!(pieces.bishop_pair, [true, true]);
    assert_eq!(pieces.bishops_light, [1, 1]);
    assert_eq!(pieces.bishops_dark, [1, 1]);
    assert_eq!(pieces.pawns_on_bishop_colour, [8, 8]);
    assert_eq!(pieces.rooks_connected_rank, [true, false]);
    assert_eq!(pieces.rooks_connected_file, [false, false]);
    assert_eq!(pieces.rooks_on_relative_7th, [0, 0]);
    assert_eq!(pieces.trapped_rook, [true, false]);
    assert_eq!(pieces.knights_on_rim, [1, 2]);
    assert!(pieces.outposts[Side::Us.index()].is_empty());
    assert!(pieces.outposts[Side::Them.index()].is_empty());
}

/// `features.md` §4 defines `minors_undeveloped` and `queen_developed` for
/// classic chess only; the group's last four values are those two facts.
#[test]
fn chess960_writes_the_two_facts_that_assume_the_starting_squares_as_zeros() {
    let schema = Schema::v1();
    let group = schema.group_set(&["pieces"]).expect("pieces is a group");
    let classic = facts_of(NINE_SIXTY_CLASSIC).encode(schema, group);
    let nine_sixty = facts_under(&CHESS960, NINE_SIXTY_CLASSIC).encode(schema, group);

    assert_eq!(classic[31..], [0.25f32, 0.25, 1.0, 1.0]);
    assert_eq!(nine_sixty[31..], [0.0f32; 4]);
    assert_eq!(classic[..31], nine_sixty[..31]);
}
