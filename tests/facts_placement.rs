//! The `placement` group, role by role.
//!
//! Every expectation is worked out from the definitions in `docs/features.md`
//! for the named position above it. A per-side case writes White first.

mod common;

use common::{facts_of, squares};
use esca::{Colour, Role, SquareSet};
use rstest::rstest;

/// The untouched array, White to move.
const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// The same array with Black to move: every plane changes hands.
const START_BLACK: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";

/// A busy middlegame with a unit of every role a side.
const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

/// A bishop endgame, Black to move: most planes are empty.
const ENDGAME: &str = "8/5pk1/8/8/8/4B3/5PK1/8 b - - 0 1";

/// One unit of each role between the two sides, and no pawn at all.
const ONE_EACH: &str = "3qk3/1n6/2b5/8/8/5R2/6N1/4K2Q w - - 0 1";

#[rstest]
#[case::start(START, "a2 b2 c2 d2 e2 f2 g2 h2", "a7 b7 c7 d7 e7 f7 g7 h7")]
#[case::start_black(START_BLACK, "a2 b2 c2 d2 e2 f2 g2 h2", "a7 b7 c7 d7 e7 f7 g7 h7")]
#[case::kiwipete(KIWIPETE, "a2 b2 c2 d5 e4 f2 g2 h2", "a7 b4 c7 d7 e6 f7 g6 h3")]
#[case::endgame(ENDGAME, "f2", "f7")]
#[case::one_each(ONE_EACH, "", "")]
fn the_pawns_of_each_side_are_where_they_stand(
    #[case] fen: &str,
    #[case] white: &str,
    #[case] black: &str,
) {
    let placement = facts_of(fen).placement;
    assert_eq!(placement.of(Colour::White, Role::Pawn), squares(white));
    assert_eq!(placement.of(Colour::Black, Role::Pawn), squares(black));
}

#[rstest]
#[case::start(START, "b1 g1", "b8 g8")]
#[case::start_black(START_BLACK, "b1 g1", "b8 g8")]
#[case::kiwipete(KIWIPETE, "c3 e5", "b6 f6")]
#[case::endgame(ENDGAME, "", "")]
#[case::one_each(ONE_EACH, "g2", "b7")]
fn the_knights_of_each_side_are_where_they_stand(
    #[case] fen: &str,
    #[case] white: &str,
    #[case] black: &str,
) {
    let placement = facts_of(fen).placement;
    assert_eq!(placement.of(Colour::White, Role::Knight), squares(white));
    assert_eq!(placement.of(Colour::Black, Role::Knight), squares(black));
}

#[rstest]
#[case::start(START, "c1 f1", "c8 f8")]
#[case::start_black(START_BLACK, "c1 f1", "c8 f8")]
#[case::kiwipete(KIWIPETE, "d2 e2", "a6 g7")]
#[case::endgame(ENDGAME, "e3", "")]
#[case::one_each(ONE_EACH, "", "c6")]
fn the_bishops_of_each_side_are_where_they_stand(
    #[case] fen: &str,
    #[case] white: &str,
    #[case] black: &str,
) {
    let placement = facts_of(fen).placement;
    assert_eq!(placement.of(Colour::White, Role::Bishop), squares(white));
    assert_eq!(placement.of(Colour::Black, Role::Bishop), squares(black));
}

#[rstest]
#[case::start(START, "a1 h1", "a8 h8")]
#[case::start_black(START_BLACK, "a1 h1", "a8 h8")]
#[case::kiwipete(KIWIPETE, "a1 h1", "a8 h8")]
#[case::endgame(ENDGAME, "", "")]
#[case::one_each(ONE_EACH, "f3", "")]
fn the_rooks_of_each_side_are_where_they_stand(
    #[case] fen: &str,
    #[case] white: &str,
    #[case] black: &str,
) {
    let placement = facts_of(fen).placement;
    assert_eq!(placement.of(Colour::White, Role::Rook), squares(white));
    assert_eq!(placement.of(Colour::Black, Role::Rook), squares(black));
}

#[rstest]
#[case::start(START, "d1", "d8")]
#[case::start_black(START_BLACK, "d1", "d8")]
#[case::kiwipete(KIWIPETE, "f3", "e7")]
#[case::endgame(ENDGAME, "", "")]
#[case::one_each(ONE_EACH, "h1", "d8")]
fn the_queens_of_each_side_are_where_they_stand(
    #[case] fen: &str,
    #[case] white: &str,
    #[case] black: &str,
) {
    let placement = facts_of(fen).placement;
    assert_eq!(placement.of(Colour::White, Role::Queen), squares(white));
    assert_eq!(placement.of(Colour::Black, Role::Queen), squares(black));
}

#[rstest]
#[case::start(START, "e1", "e8")]
#[case::start_black(START_BLACK, "e1", "e8")]
#[case::kiwipete(KIWIPETE, "e1", "e8")]
#[case::endgame(ENDGAME, "g2", "g7")]
#[case::one_each(ONE_EACH, "e1", "e8")]
fn each_side_has_one_king(#[case] fen: &str, #[case] white: &str, #[case] black: &str) {
    let placement = facts_of(fen).placement;
    assert_eq!(placement.of(Colour::White, Role::King), squares(white));
    assert_eq!(placement.of(Colour::Black, Role::King), squares(black));
}

/// Every unit of the position stands in exactly one of the sets, and the
/// `units` set holds them all.
#[rstest]
#[case::start(START)]
#[case::start_black(START_BLACK)]
#[case::kiwipete(KIWIPETE)]
#[case::endgame(ENDGAME)]
#[case::one_each(ONE_EACH)]
fn every_unit_stands_in_one_set(#[case] fen: &str) {
    let placement = facts_of(fen).placement;
    let units: u32 = Colour::ALL
        .iter()
        .flat_map(|colour| Role::ALL.map(|role| placement.of(*colour, role).len()))
        .sum();

    let mut seen = SquareSet::EMPTY;
    for colour in Colour::ALL {
        for role in Role::ALL {
            let set = placement.of(colour, role);
            assert!(
                (seen & set).is_empty(),
                "{colour:?} {role:?} overlaps an earlier set"
            );
            seen |= set;
        }
    }
    assert_eq!(seen.len(), units);
    assert_eq!(placement.units.white | placement.units.black, seen);
}
