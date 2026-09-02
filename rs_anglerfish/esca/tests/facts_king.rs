//! The `king` group, fact by fact.
//!
//! Every expectation is worked out from the definitions in `docs/features.md`
//! §1 for the named position above it.

mod common;

use common::{facts_of, facts_under};
use esca::{CHESS960, Rank, Schema, Side};
use rstest::rstest;

/// The untouched array: both kings home, walled in by their own first rank.
const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// Both sides castled short with the shelter intact and the pieces still on.
const DEVELOPED: &str = "r1bq1rk1/pp3ppp/2n1pn2/2pp4/3P4/2NBPN2/PPP2PPP/R1BQ1RK1 w - - 0 9";

/// Our king still on e1 with the centre pawns gone; theirs already on g8.
const UNCASTLED: &str = "r4rk1/ppp2ppp/8/8/8/8/PPP2PPP/R3K2R w KQ - 0 1";

/// The same placement the other way round: ours on g8's mirror, theirs on e8.
const UNCASTLED_THEIRS: &str = "r3k2r/ppp2ppp/8/8/8/8/PPP2PPP/R4RK1 w kq - 0 1";

/// A king in each corner: the king files are read off the clamped centre.
const CORNERS: &str = "k7/1p1p4/8/8/8/8/4P1P1/7K w - - 0 1";

/// Enemy pawns two, three and four ranks off our king, four and five off theirs.
const STORM: &str = "1k6/p7/1p6/2p4p/1P4pP/P4p2/5P2/6K1 w - - 0 1";

/// One open file and one the enemy has left beside each king.
const OPEN_FILES: &str = "2k5/3p3p/8/8/2P5/8/6P1/6K1 w - - 0 1";

/// Four pieces bearing on our ring against three on theirs, shelters intact.
const SIEGE: &str = "1k6/ppp1R3/8/2b5/5B1q/1Q1n4/5PPP/r4RK1 w - - 0 1";

/// A queen and a rook cover squares next to a king nothing of ours guards.
const HOLES: &str = "8/8/1k6/8/7q/8/5P2/R5K1 w - - 0 1";

/// Black to move, its own g-pawn one rank further on than the rest of the shield.
const BOXED: &str = "6k1/5p1p/6p1/8/8/8/5PPP/6K1 b - - 0 1";

/// Black to move with nothing but kings: one in the open, one on the back rank.
const BARE_KINGS: &str = "8/8/8/3k4/8/8/8/6K1 b - - 0 1";

/// A pawn ending: each king with an enemy pawn one and two ranks ahead of it.
const ENDGAME: &str = "8/8/8/3kp3/3P4/4K3/8/8 w - - 0 1";

/// A Chess960 array whose kings start on e1 and e8 without ever having moved.
const NINE_SIXTY_HOME: &str = "nnqrkrbb/pppppppp/8/8/8/8/PPPPPPPP/NNQRKRBB w FDfd - 0 1";

/// A Chess960 array whose kings start on g1 and g8, castled zone and all.
const NINE_SIXTY_WING: &str = "bbqnnrkr/pppppppp/8/8/8/8/PPPPPPPP/BBQNNRKR w HFhf - 0 1";

#[rstest]
#[case::start(START, 'e', 'e')]
#[case::corners(CORNERS, 'h', 'a')]
#[case::holes(HOLES, 'g', 'b')]
#[case::boxed(BOXED, 'g', 'g')]
#[case::bare_kings(BARE_KINGS, 'd', 'g')]
#[case::endgame(ENDGAME, 'e', 'd')]
fn a_kings_file_is_the_file_of_the_square_it_stands_on(
    #[case] fen: &str,
    #[case] us: char,
    #[case] them: char,
) {
    let facts = facts_of(fen);
    assert_eq!(facts.king.square[Side::Us.index()].file().to_char(), us);
    assert_eq!(facts.king.square[Side::Them.index()].file().to_char(), them);
}

#[rstest]
#[case::start(START, Rank::First, Rank::First)]
#[case::uncastled(UNCASTLED, Rank::First, Rank::First)]
#[case::holes(HOLES, Rank::First, Rank::Third)]
#[case::boxed(BOXED, Rank::First, Rank::First)]
#[case::bare_kings(BARE_KINGS, Rank::Fourth, Rank::First)]
#[case::endgame(ENDGAME, Rank::Third, Rank::Fourth)]
fn a_kings_rank_is_counted_from_its_own_back_rank(
    #[case] fen: &str,
    #[case] us: Rank,
    #[case] them: Rank,
) {
    let facts = facts_of(fen);
    let ours = facts.side_to_move();
    let square = facts.king.square;
    assert_eq!(square[Side::Us.index()].rank().relative_to(ours), us);
    assert_eq!(square[Side::Them.index()].rank().relative_to(!ours), them);
}

#[rstest]
#[case::start(START, [true, true])]
#[case::uncastled(UNCASTLED, [true, false])]
#[case::uncastled_theirs(UNCASTLED_THEIRS, [false, true])]
#[case::developed(DEVELOPED, [false, false])]
#[case::corners(CORNERS, [false, false])]
#[case::endgame(ENDGAME, [false, false])]
fn a_king_is_home_on_the_e_file_of_its_own_first_rank(#[case] fen: &str, #[case] home: [bool; 2]) {
    assert_eq!(facts_of(fen).king.on_home_square, home);
}

#[rstest]
#[case::start(START, [false, false], [false, false])]
#[case::developed(DEVELOPED, [false, false], [true, true])]
#[case::corners(CORNERS, [false, true], [true, false])]
#[case::storm(STORM, [false, true], [true, false])]
#[case::uncastled(UNCASTLED, [false, false], [false, true])]
#[case::uncastled_theirs(UNCASTLED_THEIRS, [false, false], [true, false])]
fn a_castled_zone_is_the_wing_of_the_board_the_king_stands_on(
    #[case] fen: &str,
    #[case] queenside: [bool; 2],
    #[case] kingside: [bool; 2],
) {
    let king = facts_of(fen).king;
    assert_eq!(king.castled_queenside, queenside);
    assert_eq!(king.castled_kingside, kingside);
}

#[rstest]
#[case::start(START, "def", "def")]
#[case::corners(CORNERS, "fgh", "abc")]
#[case::storm(STORM, "fgh", "abc")]
#[case::open_files(OPEN_FILES, "fgh", "bcd")]
#[case::bare_kings(BARE_KINGS, "cde", "fgh")]
#[case::endgame(ENDGAME, "def", "cde")]
fn the_king_files_are_the_kings_own_clamped_to_b_to_g_and_its_neighbours(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let facts = facts_of(fen);
    let letters = |side: Side| {
        facts.king.shield_files[side.index()]
            .iter()
            .map(|file| file.to_char())
            .collect::<String>()
    };
    assert_eq!(letters(Side::Us), us);
    assert_eq!(letters(Side::Them), them);
}

#[rstest]
#[case::start(START, [Some(1), Some(1), Some(1)], [Some(1), Some(1), Some(1)])]
#[case::storm(STORM, [Some(1), None, Some(3)], [Some(1), Some(2), Some(3)])]
#[case::boxed(BOXED, [Some(1), Some(2), Some(1)], [Some(1), Some(1), Some(1)])]
#[case::corners(CORNERS, [None, Some(1), None], [None, Some(1), None])]
#[case::uncastled(UNCASTLED, [None, None, Some(1)], [Some(1), Some(1), Some(1)])]
#[case::endgame(ENDGAME, [Some(1), None, None], [None, None, None])]
#[case::bare_kings(BARE_KINGS, [None, None, None], [None, None, None])]
fn a_pawn_shield_is_how_far_ahead_the_nearest_friendly_pawn_of_a_king_file_is(
    #[case] fen: &str,
    #[case] us: [Option<u8>; 3],
    #[case] them: [Option<u8>; 3],
) {
    let facts = facts_of(fen);
    assert_eq!(facts.king.shield[Side::Us.index()], us);
    assert_eq!(facts.king.shield[Side::Them.index()], them);
}

#[rstest]
#[case::start(START, [false; 3], [false; 3], [false; 3], [false; 3])]
#[case::corners(CORNERS, [true, false, true], [false, true, false], [true, false, true], [false, true, false])]
#[case::open_files(OPEN_FILES, [true, false, false], [false, true, false], [true, false, false], [false, false, true])]
#[case::uncastled(UNCASTLED, [true, true, false], [false; 3], [false; 3], [false; 3])]
#[case::holes(HOLES, [false, true, true], [true, false, false], [true; 3], [false; 3])]
#[case::endgame(ENDGAME, [false, false, true], [true, false, false], [true, false, false], [false, false, true])]
#[case::siege(SIEGE, [false; 3], [true; 3], [false; 3], [true; 3])]
fn a_king_file_is_open_when_bare_and_semi_open_when_only_the_enemy_has_left_it(
    #[case] fen: &str,
    #[case] us_open: [bool; 3],
    #[case] us_semi_open: [bool; 3],
    #[case] them_open: [bool; 3],
    #[case] them_semi_open: [bool; 3],
) {
    let king = facts_of(fen).king;
    assert_eq!(king.file_open[Side::Us.index()], us_open);
    assert_eq!(king.file_open[Side::Them.index()], them_open);
    assert_eq!(
        king.file_semi_open_for_enemy[Side::Us.index()],
        us_semi_open
    );
    assert_eq!(
        king.file_semi_open_for_enemy[Side::Them.index()],
        them_semi_open
    );
}

#[rstest]
#[case::start(START, [Some(6), Some(6), Some(6)], [Some(6), Some(6), Some(6)])]
#[case::storm(STORM, [Some(2), Some(3), Some(4)], [Some(5), Some(4), None])]
#[case::open_files(OPEN_FILES, [None, None, Some(6)], [None, Some(4), None])]
#[case::endgame(ENDGAME, [None, Some(2), None], [None, Some(1), None])]
#[case::boxed(BOXED, [Some(6), Some(6), Some(6)], [Some(6), Some(5), Some(6)])]
#[case::uncastled(UNCASTLED, [None, None, Some(6)], [Some(6), Some(6), Some(6)])]
#[case::corners(CORNERS, [None, None, None], [None, None, None])]
fn a_pawn_storm_is_how_far_ahead_the_nearest_enemy_pawn_of_a_king_file_is(
    #[case] fen: &str,
    #[case] us: [Option<u8>; 3],
    #[case] them: [Option<u8>; 3],
) {
    let facts = facts_of(fen);
    assert_eq!(facts.king.storm[Side::Us.index()], us);
    assert_eq!(facts.king.storm[Side::Them.index()], them);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::siege(SIEGE, [4, 3])]
#[case::holes(HOLES, [1, 1])]
#[case::developed(DEVELOPED, [0, 1])]
#[case::boxed(BOXED, [0, 0])]
#[case::endgame(ENDGAME, [0, 0])]
fn a_ring_attacker_is_an_enemy_piece_bearing_on_a_square_next_to_the_king(
    #[case] fen: &str,
    #[case] attackers: [u8; 2],
) {
    assert_eq!(facts_of(fen).king.ring_attackers, attackers);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::siege(SIEGE, [8, 7])]
#[case::holes(HOLES, [4, 2])]
#[case::developed(DEVELOPED, [0, 1])]
#[case::uncastled(UNCASTLED, [0, 0])]
#[case::endgame(ENDGAME, [0, 0])]
fn ring_attack_weight_counts_a_queen_four_a_rook_two_and_a_minor_one(
    #[case] fen: &str,
    #[case] weight: [u8; 2],
) {
    assert_eq!(facts_of(fen).king.ring_attack_weight, weight);
}

#[rstest]
#[case::start(START, [2, 2])]
#[case::developed(DEVELOPED, [3, 3])]
#[case::uncastled(UNCASTLED, [2, 2])]
#[case::siege(SIEGE, [2, 1])]
#[case::holes(HOLES, [1, 0])]
#[case::endgame(ENDGAME, [0, 1])]
#[case::bare_kings(BARE_KINGS, [0, 0])]
fn a_ring_square_is_defended_by_the_kings_own_side_but_never_by_the_king(
    #[case] fen: &str,
    #[case] defended: [u8; 2],
) {
    assert_eq!(facts_of(fen).king.ring_defended, defended);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::siege(SIEGE, [1, 2])]
#[case::holes(HOLES, [3, 3])]
#[case::endgame(ENDGAME, [3, 3])]
#[case::storm(STORM, [1, 0])]
#[case::developed(DEVELOPED, [0, 0])]
#[case::boxed(BOXED, [0, 0])]
fn a_ring_hole_is_a_ring_square_the_enemy_attacks_and_nothing_of_ours_covers(
    #[case] fen: &str,
    #[case] holes: [u8; 2],
) {
    assert_eq!(facts_of(fen).king.ring_holes, holes);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::siege(SIEGE, [1, 2])]
#[case::corners(CORNERS, [2, 2])]
#[case::boxed(BOXED, [3, 2])]
#[case::uncastled(UNCASTLED, [4, 1])]
#[case::endgame(ENDGAME, [5, 4])]
#[case::bare_kings(BARE_KINGS, [8, 5])]
fn an_escape_square_is_next_to_the_king_free_of_our_own_and_unattacked(
    #[case] fen: &str,
    #[case] escapes: [u8; 2],
) {
    assert_eq!(facts_of(fen).king.escape_squares, escapes);
}

#[rstest]
#[case::start(START, [true, true])]
#[case::siege(SIEGE, [true, true])]
#[case::uncastled(UNCASTLED, [false, true])]
#[case::uncastled_theirs(UNCASTLED_THEIRS, [true, false])]
#[case::boxed(BOXED, [false, true])]
#[case::endgame(ENDGAME, [false, false])]
fn back_rank_risk_is_a_first_rank_king_with_its_own_units_on_every_square_ahead(
    #[case] fen: &str,
    #[case] risk: [bool; 2],
) {
    assert_eq!(facts_of(fen).king.back_rank_risk, risk);
}

#[rstest]
#[case::start(START, 7)]
#[case::siege(SIEGE, 7)]
#[case::corners(CORNERS, 7)]
#[case::holes(HOLES, 5)]
#[case::bare_kings(BARE_KINGS, 4)]
#[case::endgame(ENDGAME, 2)]
fn the_kings_stand_a_chebyshev_distance_apart(#[case] fen: &str, #[case] distance: u8) {
    assert_eq!(facts_of(fen).king.distance, distance);
}

#[rstest]
#[case::start(START, [7.0, 7.0])]
#[case::uncastled(UNCASTLED, [7.0, 7.0])]
#[case::siege(SIEGE, [4.0, 4.75])]
#[case::holes(HOLES, [3.0, 5.0])]
#[case::corners(CORNERS, [0.0, 0.0])]
#[case::bare_kings(BARE_KINGS, [0.0, 0.0])]
fn tropism_is_the_mean_distance_of_the_enemy_pieces_to_the_king(
    #[case] fen: &str,
    #[case] tropism: [f32; 2],
) {
    assert_eq!(facts_of(fen).king.tropism, tropism);
}

#[rstest]
#[case::start(START, [5, 5])]
#[case::siege(SIEGE, [5, 10])]
#[case::storm(STORM, [12, 16])]
#[case::holes(HOLES, [16, 22])]
#[case::endgame(ENDGAME, [19, 21])]
#[case::bare_kings(BARE_KINGS, [27, 21])]
fn virtual_mobility_is_what_a_queen_on_the_kings_square_would_attack(
    #[case] fen: &str,
    #[case] mobility: [u8; 2],
) {
    assert_eq!(facts_of(fen).king.virtual_mobility, mobility);
}

/// `features.md` §4 keeps `king_on_home_square` to classic chess: a Chess960
/// array can start a king on e1 that has never moved, so the vector drops the
/// bit the facts still read off the geometry.
#[test]
fn the_home_square_bit_is_left_out_of_a_chess960_vector() {
    let facts = facts_under(&CHESS960, NINE_SIXTY_HOME);
    assert_eq!(facts.king.on_home_square, [true, true]);
    assert!(
        !Schema::v0()
            .features_for(&CHESS960)
            .contains("king", "king_on_home_square")
    );
}

/// The same for `king_castled_zone`: a Chess960 array can start both kings in
/// the kingside zone with no castling having happened.
#[test]
fn the_castled_zone_bits_are_left_out_of_a_chess960_vector() {
    let facts = facts_under(&CHESS960, NINE_SIXTY_WING);
    assert_eq!(facts.king.castled_kingside, [true, true]);
    assert_eq!(facts.king.castled_queenside, [false, false]);
    assert!(
        !Schema::v0()
            .features_for(&CHESS960)
            .contains("king", "king_castled_zone")
    );
}
