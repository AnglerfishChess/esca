//! The `mobility` group, fact by fact.
//!
//! Every expectation is worked out from the definitions in `docs/features.md`
//! §1 and §2.6 for the named position above it. `mobility_ratio`,
//! `mobility_diff_by_type` and the control difference are derived at encoding
//! time and are read off the group's own row.

mod common;

use common::{facts_of, facts_under};
use esca::{CHESS960, Schema, Side};
use rstest::rstest;

/// The untouched array: pawn attacks and two knight leaps a side, the rest of
/// the back rank walled in by its own units.
const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// One knight out: it frees the rook behind it and reaches into the far half.
const ONE_KNIGHT_OUT: &str = "rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQKB1R b KQkq - 0 1";

/// Two queens on touching diagonals: each ray towards the other stops on it.
const QUEEN_DUEL: &str = "7k/8/8/3q4/4Q3/8/8/4K3 w - - 0 1";

/// Two rooks against a bare king: all the mobility on the board is ours.
const OPEN_ROOKS: &str = "4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1";

/// The same board with the other side to move: every count changes hands.
const OPEN_ROOKS_THEIRS: &str = "4k3/8/8/8/8/8/8/R3K2R b KQ - 0 1";

/// A rook on an open board, two enemy pawns covering two squares it reaches.
const PAWN_SCREEN: &str = "5k2/8/8/3p1p2/4R3/8/8/4K3 w - - 0 1";

/// The same screen read from the pawns' side.
const PAWN_SCREEN_THEIRS: &str = "5k2/8/8/3p1p2/4R3/8/8/4K3 b - - 0 1";

/// Rooks blocking each other on the fourth rank, each side's pawns covering
/// squares the other's rook reaches.
const CROSSFIRE: &str = "4k3/8/8/2p1pp2/3R1r2/8/2P1PP2/4K3 w - - 0 1";

/// Rooks nose to nose on the e-file, each side's pawns denying the other one
/// square of it.
const TRENCHES: &str = "4k3/8/3p2p1/4r3/4R3/3P1P2/8/4K3 w - - 0 1";

/// A knight with not one legal move, pinned against its king by a rook.
const PINNED_KNIGHT: &str = "4r2k/8/8/8/8/8/4N3/4K3 w - - 0 1";

/// A knight and a bishop shut in by their own units, against a shut-in knight.
const BOXED_IN: &str = "n3k3/2p5/1p6/8/8/1P6/2P3P1/N3K2B w - - 0 1";

/// Two knights and a bishop covering all four central squares, against a wall
/// of three pawns that covers two of them.
const CENTRE_GRIP: &str = "4k3/8/8/3ppp2/8/2N1BN2/8/4K3 w - - 0 1";

/// A pawn phalanx and a knight camped in the enemy half.
const SPACE_GRAB: &str = "4k3/8/4N3/3PPP2/8/8/8/4K3 w - - 0 1";

/// Two kings alone: they control squares, but no king's squares are mobility.
const BARE_KINGS: &str = "8/8/4k3/8/8/4K3/8/8 w - - 0 1";

/// A Chess960 starting array: the same twelve squares a side as the classic
/// one, reached from other homes.
const NINE_SIXTY: &str = "bbqnnrkr/pppppppp/8/8/8/8/PPPPPPPP/BBQNNRKR w HFhf - 0 1";

/// The values `mobility.<feature>` encodes to for `fen`, taken from the group's
/// own row at the offset and width the schema gives the feature.
fn encoded(fen: &str, feature: &str) -> Vec<f32> {
    let schema = Schema::v1();
    let spec = schema
        .group("mobility")
        .and_then(|group| group.features.iter().find(|spec| spec.name == feature))
        .unwrap_or_else(|| panic!("the mobility group names {feature}"));
    let group = schema
        .group_set(&["mobility"])
        .expect("the schema has a mobility group");
    let values = facts_of(fen).encode(schema, group);
    values[spec.offset..spec.offset + spec.width].to_vec()
}

#[rstest]
#[case::start(START, 0.5)]
#[case::one_knight_out(ONE_KNIGHT_OUT, 12.0 / 27.0)]
#[case::queen_duel(QUEEN_DUEL, 23.0 / 47.0)]
#[case::open_rooks(OPEN_ROOKS, 1.0)]
#[case::open_rooks_theirs(OPEN_ROOKS_THEIRS, 0.0)]
#[case::pawn_screen(PAWN_SCREEN, 13.0 / 16.0)]
#[case::crossfire(CROSSFIRE, 17.0 / 27.0)]
#[case::space_grab(SPACE_GRAB, 1.0)]
#[case::bare_kings(BARE_KINGS, 0.0)]
fn the_ratio_is_our_share_of_the_mobility_on_the_board(#[case] fen: &str, #[case] ratio: f32) {
    assert_eq!(encoded(fen, "mobility_ratio"), [ratio]);
}

#[rstest]
#[case::start(START, [8, 4, 0, 0, 0], [8, 4, 0, 0, 0])]
#[case::one_knight_out(ONE_KNIGHT_OUT, [8, 4, 0, 0, 0], [7, 7, 0, 1, 0])]
#[case::queen_duel(QUEEN_DUEL, [0, 0, 0, 0, 23], [0, 0, 0, 0, 24])]
#[case::open_rooks(OPEN_ROOKS, [0, 0, 0, 19, 0], [0; 5])]
#[case::open_rooks_theirs(OPEN_ROOKS_THEIRS, [0; 5], [0, 0, 0, 19, 0])]
#[case::pawn_screen(PAWN_SCREEN, [0, 0, 0, 13, 0], [3, 0, 0, 0, 0])]
#[case::pawn_screen_theirs(PAWN_SCREEN_THEIRS, [3, 0, 0, 0, 0], [0, 0, 0, 13, 0])]
#[case::crossfire(CROSSFIRE, [5, 0, 0, 12, 0], [4, 0, 0, 6, 0])]
#[case::boxed_in(BOXED_IN, [5, 0, 0, 0, 0], [3, 0, 0, 0, 0])]
#[case::centre_grip(CENTRE_GRIP, [0, 15, 11, 0, 0], [5, 0, 0, 0, 0])]
#[case::space_grab(SPACE_GRAB, [4, 8, 0, 0, 0], [0; 5])]
#[case::pinned_knight(PINNED_KNIGHT, [0, 6, 0, 0, 0], [0, 0, 0, 12, 0])]
#[case::bare_kings(BARE_KINGS, [0; 5], [0; 5])]
fn a_types_mobility_is_what_its_attacks_cover_beyond_its_own_units(
    #[case] fen: &str,
    #[case] us: [u16; 5],
    #[case] them: [u16; 5],
) {
    let mobility = facts_of(fen).mobility;
    assert_eq!(mobility.by_role[Side::Us.index()], us);
    assert_eq!(mobility.by_role[Side::Them.index()], them);
}

#[rstest]
#[case::start(START, [8, 4, 0, 0, 0], [8, 4, 0, 0, 0])]
#[case::one_knight_out(ONE_KNIGHT_OUT, [8, 4, 0, 0, 0], [7, 7, 0, 1, 0])]
#[case::pawn_screen(PAWN_SCREEN, [0, 0, 0, 11, 0], [3, 0, 0, 0, 0])]
#[case::pawn_screen_theirs(PAWN_SCREEN_THEIRS, [3, 0, 0, 0, 0], [0, 0, 0, 11, 0])]
#[case::crossfire(CROSSFIRE, [5, 0, 0, 9, 0], [4, 0, 0, 5, 0])]
#[case::trenches(TRENCHES, [2, 0, 0, 9, 0], [3, 0, 0, 9, 0])]
#[case::boxed_in(BOXED_IN, [5, 0, 0, 0, 0], [3, 0, 0, 0, 0])]
#[case::centre_grip(CENTRE_GRIP, [0, 13, 9, 0, 0], [5, 0, 0, 0, 0])]
fn safe_mobility_drops_the_squares_an_enemy_pawn_attacks(
    #[case] fen: &str,
    #[case] us: [u16; 5],
    #[case] them: [u16; 5],
) {
    let mobility = facts_of(fen).mobility;
    assert_eq!(mobility.safe_by_role[Side::Us.index()], us);
    assert_eq!(mobility.safe_by_role[Side::Them.index()], them);
}

#[rstest]
#[case::start(START, [0.0; 5])]
#[case::one_knight_out(ONE_KNIGHT_OUT, [1.0 / 16.0, -3.0 / 16.0, 0.0, -1.0 / 16.0, 0.0])]
#[case::queen_duel(QUEEN_DUEL, [0.0, 0.0, 0.0, 0.0, -1.0 / 16.0])]
#[case::open_rooks(OPEN_ROOKS, [0.0, 0.0, 0.0, 1.0, 0.0])]
#[case::open_rooks_theirs(OPEN_ROOKS_THEIRS, [0.0, 0.0, 0.0, -1.0, 0.0])]
#[case::pawn_screen(PAWN_SCREEN, [-3.0 / 16.0, 0.0, 0.0, 13.0 / 16.0, 0.0])]
#[case::crossfire(CROSSFIRE, [1.0 / 16.0, 0.0, 0.0, 6.0 / 16.0, 0.0])]
#[case::centre_grip(CENTRE_GRIP, [-5.0 / 16.0, 15.0 / 16.0, 11.0 / 16.0, 0.0, 0.0])]
#[case::space_grab(SPACE_GRAB, [4.0 / 16.0, 8.0 / 16.0, 0.0, 0.0, 0.0])]
fn the_mobility_difference_is_ours_less_theirs_by_type(#[case] fen: &str, #[case] diff: [f32; 5]) {
    assert_eq!(encoded(fen, "mobility_diff_by_type"), diff);
}

/// Neither side's pawns cover a square the other's pieces reach in
/// `QUEEN_DUEL`, `OPEN_ROOKS` or `BARE_KINGS`, so there the safe difference is
/// the whole difference.
#[rstest]
#[case::start(START, [0.0; 5])]
#[case::one_knight_out(ONE_KNIGHT_OUT, [1.0 / 16.0, -3.0 / 16.0, 0.0, -1.0 / 16.0, 0.0])]
#[case::queen_duel(QUEEN_DUEL, [0.0, 0.0, 0.0, 0.0, -1.0 / 16.0])]
#[case::open_rooks(OPEN_ROOKS, [0.0, 0.0, 0.0, 1.0, 0.0])]
#[case::open_rooks_theirs(OPEN_ROOKS_THEIRS, [0.0, 0.0, 0.0, -1.0, 0.0])]
#[case::pawn_screen(PAWN_SCREEN, [-3.0 / 16.0, 0.0, 0.0, 11.0 / 16.0, 0.0])]
#[case::pawn_screen_theirs(PAWN_SCREEN_THEIRS, [3.0 / 16.0, 0.0, 0.0, -11.0 / 16.0, 0.0])]
#[case::crossfire(CROSSFIRE, [1.0 / 16.0, 0.0, 0.0, 4.0 / 16.0, 0.0])]
#[case::trenches(TRENCHES, [-1.0 / 16.0, 0.0, 0.0, 0.0, 0.0])]
#[case::boxed_in(BOXED_IN, [2.0 / 16.0, 0.0, 0.0, 0.0, 0.0])]
#[case::centre_grip(CENTRE_GRIP, [-5.0 / 16.0, 13.0 / 16.0, 9.0 / 16.0, 0.0, 0.0])]
#[case::bare_kings(BARE_KINGS, [0.0; 5])]
fn the_safe_difference_is_our_safe_mobility_less_theirs_by_type(
    #[case] fen: &str,
    #[case] diff: [f32; 5],
) {
    assert_eq!(encoded(fen, "safe_mobility_diff_by_type"), diff);
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::one_knight_out(ONE_KNIGHT_OUT, [0, 2])]
#[case::queen_duel(QUEEN_DUEL, [8, 8])]
#[case::open_rooks(OPEN_ROOKS, [8, 0])]
#[case::open_rooks_theirs(OPEN_ROOKS_THEIRS, [0, 8])]
#[case::pawn_screen(PAWN_SCREEN, [4, 3])]
#[case::crossfire(CROSSFIRE, [4, 8])]
#[case::trenches(TRENCHES, [1, 1])]
#[case::centre_grip(CENTRE_GRIP, [8, 5])]
#[case::space_grab(SPACE_GRAB, [11, 0])]
fn space_is_what_a_side_attacks_in_the_half_the_other_starts_on(
    #[case] fen: &str,
    #[case] space: [u16; 2],
) {
    assert_eq!(facts_of(fen).mobility.space, space);
}

#[rstest]
#[case::start(START, [22, 22], 0.0)]
#[case::one_knight_out(ONE_KNIGHT_OUT, [22, 26], -4.0 / 48.0)]
#[case::queen_duel(QUEEN_DUEL, [28, 26], 2.0 / 48.0)]
#[case::open_rooks(OPEN_ROOKS, [23, 5], 18.0 / 48.0)]
#[case::open_rooks_theirs(OPEN_ROOKS_THEIRS, [5, 23], -18.0 / 48.0)]
#[case::pawn_screen(PAWN_SCREEN, [18, 8], 10.0 / 48.0)]
#[case::crossfire(CROSSFIRE, [19, 14], 5.0 / 48.0)]
#[case::boxed_in(BOXED_IN, [13, 10], 3.0 / 48.0)]
#[case::bare_kings(BARE_KINGS, [8, 8], 0.0)]
fn the_controlled_squares_are_a_sides_whole_attack_map_kings_included(
    #[case] fen: &str,
    #[case] controlled: [u16; 2],
    #[case] difference: f32,
) {
    assert_eq!(facts_of(fen).mobility.controlled, controlled);
    assert_eq!(
        encoded(fen, "controlled_squares"),
        [
            f32::from(controlled[0]) / 48.0,
            f32::from(controlled[1]) / 48.0,
            difference,
        ]
    );
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::one_knight_out(ONE_KNIGHT_OUT, [0, 2])]
#[case::queen_duel(QUEEN_DUEL, [3, 3])]
#[case::pawn_screen(PAWN_SCREEN, [2, 1])]
#[case::pawn_screen_theirs(PAWN_SCREEN_THEIRS, [1, 2])]
#[case::crossfire(CROSSFIRE, [2, 2])]
#[case::trenches(TRENCHES, [3, 3])]
#[case::centre_grip(CENTRE_GRIP, [4, 2])]
#[case::space_grab(SPACE_GRAB, [1, 0])]
#[case::bare_kings(BARE_KINGS, [2, 2])]
fn centre_control_counts_the_attacks_on_d4_e4_d5_and_e5(
    #[case] fen: &str,
    #[case] centre: [u8; 2],
) {
    assert_eq!(facts_of(fen).mobility.centre_control, centre);
}

#[rstest]
#[case::start(START, [4, 4])]
#[case::one_knight_out(ONE_KNIGHT_OUT, [4, 6])]
#[case::queen_duel(QUEEN_DUEL, [10, 10])]
#[case::open_rooks(OPEN_ROOKS, [0, 0])]
#[case::pawn_screen(PAWN_SCREEN, [6, 2])]
#[case::pawn_screen_theirs(PAWN_SCREEN_THEIRS, [2, 6])]
#[case::crossfire(CROSSFIRE, [8, 5])]
#[case::boxed_in(BOXED_IN, [3, 2])]
#[case::centre_grip(CENTRE_GRIP, [6, 4])]
#[case::space_grab(SPACE_GRAB, [7, 0])]
#[case::bare_kings(BARE_KINGS, [5, 5])]
fn the_extended_centre_is_the_sixteen_squares_from_c3_to_f6(
    #[case] fen: &str,
    #[case] extended: [u8; 2],
) {
    assert_eq!(facts_of(fen).mobility.extended_centre_control, extended);
}

#[rstest]
#[case::start(START, [5, 5])]
#[case::one_knight_out(ONE_KNIGHT_OUT, [5, 4])]
#[case::boxed_in(BOXED_IN, [2, 1])]
#[case::queen_duel(QUEEN_DUEL, [0, 0])]
#[case::open_rooks(OPEN_ROOKS, [0, 0])]
#[case::crossfire(CROSSFIRE, [0, 0])]
#[case::centre_grip(CENTRE_GRIP, [0, 0])]
#[case::pinned_knight(PINNED_KNIGHT, [0, 0])]
fn an_immobile_piece_reaches_nothing_its_own_side_has_left_free(
    #[case] fen: &str,
    #[case] immobile: [u8; 2],
) {
    assert_eq!(facts_of(fen).mobility.immobile_pieces, immobile);
}

#[rstest]
#[case::start(START, [12, 12])]
#[case::one_knight_out(ONE_KNIGHT_OUT, [12, 15])]
#[case::queen_duel(QUEEN_DUEL, [23, 24])]
#[case::open_rooks(OPEN_ROOKS, [19, 0])]
#[case::open_rooks_theirs(OPEN_ROOKS_THEIRS, [0, 19])]
#[case::pawn_screen(PAWN_SCREEN, [13, 3])]
#[case::crossfire(CROSSFIRE, [17, 10])]
#[case::trenches(TRENCHES, [12, 13])]
#[case::centre_grip(CENTRE_GRIP, [26, 5])]
#[case::bare_kings(BARE_KINGS, [0, 0])]
fn the_total_mobility_adds_the_five_types_up(#[case] fen: &str, #[case] total: [u16; 2]) {
    assert_eq!(facts_of(fen).mobility.total, total);
}

/// No `mobility` fact is among the four `features.md` §4 defines for classic
/// chess only: every one of them reads attack maps alone, so a Chess960
/// starting array answers exactly as the same placement would.
#[test]
fn the_mobility_facts_of_a_chess960_position_are_the_classic_ones() {
    let facts = facts_under(&CHESS960, NINE_SIXTY);
    let mobility = facts.mobility;
    assert_eq!(mobility.by_role[Side::Us.index()], [8, 4, 0, 0, 0]);
    assert_eq!(mobility.by_role[Side::Them.index()], [8, 4, 0, 0, 0]);
    assert_eq!(mobility.safe_by_role[Side::Us.index()], [8, 4, 0, 0, 0]);
    assert_eq!(mobility.total, [12, 12]);
    assert_eq!(mobility.space, [0, 0]);
    // The two knights leave e2 and e7 uncovered, so a square fewer than the
    // classic array's 22.
    assert_eq!(mobility.controlled, [21, 21]);
    assert_eq!(mobility.centre_control, [0, 0]);
    assert_eq!(mobility.extended_centre_control, [4, 4]);
    assert_eq!(mobility.immobile_pieces, [5, 5]);

    let classic = facts_of("bbqnnrkr/pppppppp/8/8/8/8/PPPPPPPP/BBQNNRKR w - - 0 1");
    assert_eq!(classic.mobility, mobility);
}
