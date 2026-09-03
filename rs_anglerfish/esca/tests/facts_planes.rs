//! The `planes` group, fact by fact.
//!
//! Every expectation is worked out from the definitions in `docs/features.md`
//! §1 and §2.9 for the named position above it. A square set is read on the
//! board; the eight encoded planes are read in the mover's view.

mod common;

use common::{facts_of, facts_under, squares};
use esca::{CHESS960, Schema, Side, Square, SquareSet};
use rstest::rstest;

/// The untouched array: each side covers its own first three ranks, corners aside.
const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// Kings and pawns, Black to move: d3 is loose, and so are b5 and the c2 pawn
/// that d3 attacks.
const PAWN_DUEL: &str = "4k3/1p3p2/2p5/1P6/6P1/3p4/2P5/4K3 b - - 0 1";

/// The c3 knight and the d6 bishop are each attacked with nothing defending them.
const LOOSE_PIECES: &str = "4k3/1p6/2nb4/8/1p6/2N5/P7/3RK3 w - - 0 1";

/// The bishops on b4 and b5 pin the knight in front of each king; a pinned
/// knight still attacks everything a free one would.
const CROSS_PINS: &str = "4k3/8/2n5/1B6/1b6/2N5/8/4K3 w - - 0 1";

/// A pin along a file against a pin along a rank; both pinned knights hang.
const FILE_PINS: &str = "4k3/7p/4n3/4R3/8/8/6P1/r1N1K3 w - - 0 1";

/// Black to move, a pin and a loose pawn a side, and every plane occupied.
const COUNTERPLAY: &str = "r5k1/6p1/2n4p/3p4/1b1P4/1BN3P1/PP6/4K2R b K - 0 1";

/// The rook's file ends at the king it checks: d6 and d7 are not attacked.
const THROUGH_THE_KING: &str = "8/8/8/3k4/8/8/P7/3RK3 b - - 0 1";

/// A Chess960 middlegame, each king tucked behind its own rook; a knight of
/// each side stands where nothing defends it.
const TUCKED_KINGS: &str = "1kr5/pp3p2/4N3/6n1/2n5/3B4/PP6/1KR5 w Cc - 0 1";

/// Chess960 with the white king on d1: a knight pinned to it by the rook that
/// castling rights still name, against a bishop pinned to the black king.
const CROSSED_BISHOPS: &str = "3r2k1/6pp/6n1/8/2bN4/8/BP4P1/3K3R w Hd - 0 1";

/// Chess960, the rights spent: the queen and the bishop attacking her are both
/// undefended, and each king holds a pinned pawn in front of it.
const LOOSE_QUEEN: &str = "1r5k/6pp/8/2b5/3Q4/1P6/P1P5/1K6 w - - 0 1";

/// The squares set in the plane `feature`, read out of the row `fen` encodes to
/// at the offset and width the schema gives the feature, and named in the
/// mover's view.
fn plane(fen: &str, feature: &str) -> SquareSet {
    let schema = Schema::v1();
    let spec = schema
        .group("planes")
        .and_then(|group| group.features.iter().find(|spec| spec.name == feature))
        .unwrap_or_else(|| panic!("the planes group names {feature}"));
    let group = schema
        .group_set(&["planes"])
        .expect("the schema has a planes group");
    let values = facts_of(fen).encode(schema, group);
    let bits = &values[spec.offset..spec.offset + spec.width];
    assert!(
        bits.iter().all(|value| *value == 0.0 || *value == 1.0),
        "a plane holds one bit a square"
    );
    Square::ALL
        .into_iter()
        .filter(|square| bits[square.index()] == 1.0)
        .collect()
}

#[rstest]
#[case::start(
    START,
    "a2 a3 b1 b2 b3 c1 c2 c3 d1 d2 d3 e1 e2 e3 f1 f2 f3 g1 g2 g3 h2 h3",
    "a6 a7 b6 b7 b8 c6 c7 c8 d6 d7 d8 e6 e7 e8 f6 f7 f8 g6 g7 g8 h6 h7"
)]
#[case::pawn_duel(
    PAWN_DUEL,
    "a6 b5 c2 c6 d5 d7 d8 e2 e6 e7 f7 f8 g6",
    "a6 b3 c6 d1 d2 d3 e2 f1 f2 f5 h5"
)]
#[case::loose_pieces(
    LOOSE_PIECES,
    "a1 a2 a4 b1 b3 b5 c1 d1 d2 d3 d4 d5 d6 e1 e2 e4 f1 f2",
    "a3 a5 a6 a7 b4 b8 c3 c5 c6 c7 d4 d7 d8 e5 e7 f4 f7 f8 g3 h2"
)]
#[case::cross_pins(
    CROSS_PINS,
    "a2 a4 a6 b1 b5 c4 c6 d1 d2 d3 d5 e2 e4 f1 f2",
    "a3 a5 a7 b4 b8 c3 c5 d4 d6 d7 d8 e5 e7 f7 f8"
)]
#[case::file_pins(
    FILE_PINS,
    "a2 a5 b3 b5 c5 d1 d2 d3 d5 e1 e2 e3 e4 e6 f1 f2 f3 f5 g5 h3 h5",
    "a2 a3 a4 a5 a6 a7 a8 b1 c1 c5 c7 d4 d7 d8 e7 f4 f7 f8 g5 g6 g7"
)]
#[case::counterplay(
    COUNTERPLAY,
    "a2 a3 a4 a5 a6 a7 b4 b8 c3 c4 c5 c8 d4 d6 d8 e4 e5 e7 e8 f6 f7 f8 g5 g7 g8 h6 h7 h8",
    "a2 a3 a4 b1 b3 b5 c2 c3 c4 c5 d1 d2 d5 e1 e2 e4 e5 f1 f2 f4 g1 h2 h3 h4 h5 h6"
)]
#[case::through_the_king(
    THROUGH_THE_KING,
    "c4 c5 c6 d4 d6 e4 e5 e6",
    "a1 b1 b3 c1 d1 d2 d3 d4 d5 e1 e2 f1 f2"
)]
fn an_attack_map_holds_every_square_its_side_could_capture_on(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let planes = facts_of(fen).planes;
    assert_eq!(planes.attacked[Side::Us.index()], squares(us));
    assert_eq!(planes.attacked[Side::Them.index()], squares(them));
}

#[rstest]
#[case::start(START, "a3 b3 c3 d3 e3 f3 g3 h3", "a6 b6 c6 d6 e6 f6 g6 h6")]
#[case::pawn_duel(PAWN_DUEL, "a6 b5 c2 c6 d5 e2 e6 g6", "a6 b3 c6 d3 f5 h5")]
#[case::loose_pieces(LOOSE_PIECES, "b3", "a3 a6 c3 c6")]
#[case::cross_pins(CROSS_PINS, "", "")]
#[case::file_pins(FILE_PINS, "f3 h3", "g6")]
#[case::counterplay(COUNTERPLAY, "c4 e4 f6 g5 h6", "a3 b3 c3 c5 e5 f4 h4")]
#[case::through_the_king(THROUGH_THE_KING, "", "b3")]
fn a_pawn_attack_map_holds_the_diagonals_and_nothing_a_pawn_pushes_onto(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let planes = facts_of(fen).planes;
    assert_eq!(planes.attacked_by_pawns[Side::Us.index()], squares(us));
    assert_eq!(planes.attacked_by_pawns[Side::Them.index()], squares(them));
}

#[rstest]
#[case::start(START, "", "")]
#[case::pawn_duel(PAWN_DUEL, "d3", "b5 c2")]
#[case::loose_pieces(LOOSE_PIECES, "c3", "d6")]
#[case::cross_pins(CROSS_PINS, "c3", "c6")]
#[case::file_pins(FILE_PINS, "c1", "e6")]
#[case::counterplay(COUNTERPLAY, "d5", "d4")]
#[case::through_the_king(THROUGH_THE_KING, "", "")]
fn a_hanging_unit_is_attacked_by_the_opponent_and_defended_by_nobody(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let planes = facts_of(fen).planes;
    assert_eq!(planes.hanging[Side::Us.index()], squares(us));
    assert_eq!(planes.hanging[Side::Them.index()], squares(them));
}

#[rstest]
#[case::start(START, "", "")]
#[case::pawn_duel(PAWN_DUEL, "", "")]
#[case::loose_pieces(LOOSE_PIECES, "", "")]
#[case::cross_pins(CROSS_PINS, "c3", "c6")]
#[case::file_pins(FILE_PINS, "c1", "e6")]
#[case::counterplay(COUNTERPLAY, "d5", "c3")]
#[case::through_the_king(THROUGH_THE_KING, "", "")]
fn an_absolutely_pinned_unit_is_the_one_thing_between_a_slider_and_its_own_king(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let planes = facts_of(fen).planes;
    assert_eq!(planes.pinned[Side::Us.index()], squares(us));
    assert_eq!(planes.pinned[Side::Them.index()], squares(them));
}

#[rstest]
#[case::attacked_by_us(
    "attacked_by_us",
    "a2 a5 b3 b5 c5 d1 d2 d3 d5 e1 e2 e3 e4 e6 f1 f2 f3 f5 g5 h3 h5"
)]
#[case::attacked_by_them(
    "attacked_by_them",
    "a2 a3 a4 a5 a6 a7 a8 b1 c1 c5 c7 d4 d7 d8 e7 f4 f7 f8 g5 g6 g7"
)]
#[case::attacked_by_our_pawns("attacked_by_our_pawns", "f3 h3")]
#[case::attacked_by_their_pawns("attacked_by_their_pawns", "g6")]
#[case::our_hanging("our_hanging", "c1")]
#[case::their_hanging("their_hanging", "e6")]
#[case::our_pinned("our_pinned", "c1")]
#[case::their_pinned("their_pinned", "e6")]
fn white_to_move_writes_each_plane_as_the_board_stands(#[case] feature: &str, #[case] set: &str) {
    assert_eq!(plane(FILE_PINS, feature), squares(set));
}

/// The mover's view flips rank *r* onto rank 9−*r* and leaves the files alone,
/// so every plane of a Black-to-move position is written upside down.
#[rstest]
#[case::attacked_by_us(
    "attacked_by_us",
    "a2 a3 a4 a5 a6 a7 b1 b5 c1 c4 c5 c6 d1 d3 d5 e1 e2 e4 e5 f1 f2 f3 g1 g2 g4 h1 h2 h3"
)]
#[case::attacked_by_them(
    "attacked_by_them",
    "a5 a6 a7 b4 b6 b8 c4 c5 c6 c7 d4 d7 d8 e4 e5 e7 e8 f5 f7 f8 g8 h3 h4 h5 h6 h7"
)]
#[case::attacked_by_our_pawns("attacked_by_our_pawns", "c5 e5 f3 g4 h3")]
#[case::attacked_by_their_pawns("attacked_by_their_pawns", "a6 b6 c4 c6 e4 f5 h5")]
#[case::our_hanging("our_hanging", "d4")]
#[case::their_hanging("their_hanging", "d5")]
#[case::our_pinned("our_pinned", "d4")]
#[case::their_pinned("their_pinned", "c6")]
fn black_to_move_writes_each_plane_with_the_ranks_turned_around(
    #[case] feature: &str,
    #[case] set: &str,
) {
    assert_eq!(plane(COUNTERPLAY, feature), squares(set));
}

/// No plane reads the back rank as a starting square or the castling rights, so
/// a Chess960 placement answers exactly as any other placement would.
#[rstest]
#[case::tucked_kings(
    TUCKED_KINGS,
    ["a1 a2 a3 b1 b2 b3 c1 c2 c3 c4 c5 c7 d1 d4 d8 e1 e2 e4 f1 f4 f5 f8 g1 g5 g6 g7 h1 h7",
     "a3 a5 a6 a7 a8 b2 b6 b7 b8 c4 c5 c6 c7 c8 d2 d6 d8 e3 e4 e5 e6 e8 f3 f7 f8 g6 g8 h3 h7 h8"],
    ["a3 b3 c3", "a6 b6 c6 e6 g6"],
    ["e6", "g5"],
    ["", ""]
)]
#[case::crossed_bishops(
    CROSSED_BISHOPS,
    ["a3 b1 b3 b5 c1 c2 c3 c4 c6 d1 d2 e1 e2 e6 f1 f3 f5 g1 h2 h3 h4 h5 h6 h7",
     "a2 a6 a8 b3 b5 b8 c8 d3 d4 d5 d6 d7 e2 e5 e6 e7 e8 f1 f4 f6 f7 f8 g6 g7 g8 h4 h6 h7 h8"],
    ["a3 c3 f3 h3", "f6 g6 h6"],
    ["a2 d4", "c4"],
    ["d4", "c4"]
)]
#[case::loose_queen(
    LOOSE_QUEEN,
    ["a1 a2 a4 b2 b3 b4 c1 c2 c3 c4 c5 d1 d2 d3 d5 d6 d7 d8 e3 e4 e5 f2 f4 f6 g1 g4 g7 h4",
     "a3 a7 a8 b3 b4 b5 b6 b7 c8 d4 d6 d8 e7 e8 f6 f8 g6 g7 g8 h6 h7 h8"],
    ["a4 b3 c4 d3", "f6 g6 h6"],
    ["d4", "c5"],
    ["b3", "g7"]
)]
fn the_planes_of_a_chess960_position_read_the_placement_as_they_find_it(
    #[case] fen: &str,
    #[case] attacked: [&str; 2],
    #[case] by_pawns: [&str; 2],
    #[case] hanging: [&str; 2],
    #[case] pinned: [&str; 2],
) {
    let planes = facts_under(&CHESS960, fen).planes;
    for side in Side::ALL {
        let i = side.index();
        assert_eq!(planes.attacked[i], squares(attacked[i]));
        assert_eq!(planes.attacked_by_pawns[i], squares(by_pawns[i]));
        assert_eq!(planes.hanging[i], squares(hanging[i]));
        assert_eq!(planes.pinned[i], squares(pinned[i]));
    }
}
