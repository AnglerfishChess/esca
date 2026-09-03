//! The `placement` group, plane by plane.
//!
//! Every expectation is worked out from the definitions in `docs/features.md`
//! §2.1 for the named position above it.

mod common;

use common::{facts_of, squares};
use esca::{Role, Schema, Side, SquareSet};
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

/// Where the plane of `role` for `side` starts in the 768-wide row.
fn plane_at(side: Side, role: Role) -> usize {
    64 * (6 * side.index() + role.index())
}

/// The encoded `placement` row of `fen`.
fn placement_row(fen: &str) -> Vec<f32> {
    let schema = Schema::v1();
    let groups = schema
        .group_set(&["placement"])
        .expect("the placement group");
    facts_of(fen).encode(schema, groups)
}

#[rstest]
#[case::start(START, "a2 b2 c2 d2 e2 f2 g2 h2", "a7 b7 c7 d7 e7 f7 g7 h7")]
#[case::start_black(START_BLACK, "a7 b7 c7 d7 e7 f7 g7 h7", "a2 b2 c2 d2 e2 f2 g2 h2")]
#[case::kiwipete(KIWIPETE, "a2 b2 c2 d5 e4 f2 g2 h2", "a7 b4 c7 d7 e6 f7 g6 h3")]
#[case::endgame(ENDGAME, "f7", "f2")]
#[case::one_each(ONE_EACH, "", "")]
fn the_pawn_planes_hold_the_pawns_of_their_own_side(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let placement = facts_of(fen).placement;
    assert_eq!(placement.of(Side::Us, Role::Pawn), squares(us));
    assert_eq!(placement.of(Side::Them, Role::Pawn), squares(them));
}

#[rstest]
#[case::start(START, "b1 g1", "b8 g8")]
#[case::start_black(START_BLACK, "b8 g8", "b1 g1")]
#[case::kiwipete(KIWIPETE, "c3 e5", "b6 f6")]
#[case::endgame(ENDGAME, "", "")]
#[case::one_each(ONE_EACH, "g2", "b7")]
fn the_knight_planes_hold_the_knights_of_their_own_side(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let placement = facts_of(fen).placement;
    assert_eq!(placement.of(Side::Us, Role::Knight), squares(us));
    assert_eq!(placement.of(Side::Them, Role::Knight), squares(them));
}

#[rstest]
#[case::start(START, "c1 f1", "c8 f8")]
#[case::start_black(START_BLACK, "c8 f8", "c1 f1")]
#[case::kiwipete(KIWIPETE, "d2 e2", "a6 g7")]
#[case::endgame(ENDGAME, "", "e3")]
#[case::one_each(ONE_EACH, "", "c6")]
fn the_bishop_planes_hold_the_bishops_of_their_own_side(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let placement = facts_of(fen).placement;
    assert_eq!(placement.of(Side::Us, Role::Bishop), squares(us));
    assert_eq!(placement.of(Side::Them, Role::Bishop), squares(them));
}

#[rstest]
#[case::start(START, "a1 h1", "a8 h8")]
#[case::start_black(START_BLACK, "a8 h8", "a1 h1")]
#[case::kiwipete(KIWIPETE, "a1 h1", "a8 h8")]
#[case::endgame(ENDGAME, "", "")]
#[case::one_each(ONE_EACH, "f3", "")]
fn the_rook_planes_hold_the_rooks_of_their_own_side(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let placement = facts_of(fen).placement;
    assert_eq!(placement.of(Side::Us, Role::Rook), squares(us));
    assert_eq!(placement.of(Side::Them, Role::Rook), squares(them));
}

#[rstest]
#[case::start(START, "d1", "d8")]
#[case::start_black(START_BLACK, "d8", "d1")]
#[case::kiwipete(KIWIPETE, "f3", "e7")]
#[case::endgame(ENDGAME, "", "")]
#[case::one_each(ONE_EACH, "h1", "d8")]
fn the_queen_planes_hold_the_queens_of_their_own_side(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let placement = facts_of(fen).placement;
    assert_eq!(placement.of(Side::Us, Role::Queen), squares(us));
    assert_eq!(placement.of(Side::Them, Role::Queen), squares(them));
}

#[rstest]
#[case::start(START, "e1", "e8")]
#[case::start_black(START_BLACK, "e8", "e1")]
#[case::kiwipete(KIWIPETE, "e1", "e8")]
#[case::endgame(ENDGAME, "g7", "g2")]
#[case::one_each(ONE_EACH, "e1", "e8")]
fn the_king_planes_hold_one_king_each(#[case] fen: &str, #[case] us: &str, #[case] them: &str) {
    let placement = facts_of(fen).placement;
    assert_eq!(placement.of(Side::Us, Role::King), squares(us));
    assert_eq!(placement.of(Side::Them, Role::King), squares(them));
}

/// The plane order the row is written in: ours before theirs, and P, N, B, R,
/// Q, K within a side.
#[rstest]
#[case::our_pawns(Side::Us, Role::Pawn, 0)]
#[case::our_knights(Side::Us, Role::Knight, 64)]
#[case::our_bishops(Side::Us, Role::Bishop, 128)]
#[case::our_rooks(Side::Us, Role::Rook, 192)]
#[case::our_queens(Side::Us, Role::Queen, 256)]
#[case::our_king(Side::Us, Role::King, 320)]
#[case::their_pawns(Side::Them, Role::Pawn, 384)]
#[case::their_knights(Side::Them, Role::Knight, 448)]
#[case::their_bishops(Side::Them, Role::Bishop, 512)]
#[case::their_rooks(Side::Them, Role::Rook, 576)]
#[case::their_queens(Side::Them, Role::Queen, 640)]
#[case::their_king(Side::Them, Role::King, 704)]
fn each_plane_sits_where_the_schema_names_it(
    #[case] side: Side,
    #[case] role: Role,
    #[case] offset: usize,
) {
    let schema = Schema::v1();
    let group = schema.group("placement").expect("the placement group");
    let feature = group.features[6 * side.index() + role.index()];
    assert_eq!(feature.offset, offset);
    assert_eq!(feature.width, 64);
    assert_eq!(plane_at(side, role), offset);
}

/// A plane is read in the mover's view, so the untouched array writes the same
/// row whichever side is to move.
#[test]
fn the_movers_view_makes_the_two_starting_rows_the_same() {
    let white = placement_row(START);
    let black = placement_row(START_BLACK);
    assert_eq!(white, black);
    assert_eq!(white.len(), 768);

    // Our pawns stand on relative rank 2, which is plane index 8 to 15.
    let pawns = &white[plane_at(Side::Us, Role::Pawn)..][..64];
    assert_eq!(
        pawns
            .iter()
            .enumerate()
            .filter(|(_, v)| **v == 1.0)
            .map(|(i, _)| i)
            .collect::<Vec<_>>(),
        (8..16).collect::<Vec<_>>()
    );
}

/// Every unit of the position stands in exactly one plane, and nothing else is
/// set.
#[rstest]
#[case::start(START)]
#[case::start_black(START_BLACK)]
#[case::kiwipete(KIWIPETE)]
#[case::endgame(ENDGAME)]
#[case::one_each(ONE_EACH)]
fn the_planes_hold_every_unit_once(#[case] fen: &str) {
    let facts = facts_of(fen);
    let row = placement_row(fen);
    let units: u32 = Side::ALL
        .iter()
        .flat_map(|side| Role::ALL.map(|role| facts.placement.of(*side, role).len()))
        .sum();
    assert_eq!(
        row.iter().filter(|value| **value == 1.0).count() as u32,
        units
    );
    assert!(row.iter().all(|value| *value == 0.0 || *value == 1.0));

    let mut seen = SquareSet::EMPTY;
    for side in Side::ALL {
        for role in Role::ALL {
            let set = facts.placement.of(side, role);
            assert!(
                (seen & set).is_empty(),
                "{side:?} {role:?} overlaps an earlier plane"
            );
            seen |= set;
        }
    }
    assert_eq!(seen.len(), units);
}
