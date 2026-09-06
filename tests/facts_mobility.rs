//! The `mobility` group, fact by fact.
//!
//! Every expectation is worked out from the definitions in `docs/features.md`
//! for the named position above it. A per-side case writes White first.

mod common;

use common::{by_colour, facts_of, facts_under};
use esca::CHESS960;
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

#[rstest]
#[case::start(START, [8, 4, 0, 0, 0], [8, 4, 0, 0, 0])]
#[case::one_knight_out(ONE_KNIGHT_OUT, [7, 7, 0, 1, 0], [8, 4, 0, 0, 0])]
#[case::queen_duel(QUEEN_DUEL, [0, 0, 0, 0, 23], [0, 0, 0, 0, 24])]
#[case::open_rooks(OPEN_ROOKS, [0, 0, 0, 19, 0], [0; 5])]
#[case::open_rooks_theirs(OPEN_ROOKS_THEIRS, [0, 0, 0, 19, 0], [0; 5])]
#[case::pawn_screen(PAWN_SCREEN, [0, 0, 0, 13, 0], [3, 0, 0, 0, 0])]
#[case::pawn_screen_theirs(PAWN_SCREEN_THEIRS, [0, 0, 0, 13, 0], [3, 0, 0, 0, 0])]
#[case::crossfire(CROSSFIRE, [5, 0, 0, 12, 0], [4, 0, 0, 6, 0])]
#[case::boxed_in(BOXED_IN, [5, 0, 0, 0, 0], [3, 0, 0, 0, 0])]
#[case::centre_grip(CENTRE_GRIP, [0, 15, 11, 0, 0], [5, 0, 0, 0, 0])]
#[case::space_grab(SPACE_GRAB, [4, 8, 0, 0, 0], [0; 5])]
#[case::pinned_knight(PINNED_KNIGHT, [0, 6, 0, 0, 0], [0, 0, 0, 12, 0])]
#[case::bare_kings(BARE_KINGS, [0; 5], [0; 5])]
fn a_roles_mobility_is_what_its_attacks_cover_beyond_its_own_units(
    #[case] fen: &str,
    #[case] white: [u16; 5],
    #[case] black: [u16; 5],
) {
    let mobility = facts_of(fen).mobility;
    assert_eq!(mobility.pawns, by_colour([white[0], black[0]]));
    assert_eq!(mobility.knights, by_colour([white[1], black[1]]));
    assert_eq!(mobility.bishops, by_colour([white[2], black[2]]));
    assert_eq!(mobility.rooks, by_colour([white[3], black[3]]));
    assert_eq!(mobility.queens, by_colour([white[4], black[4]]));
}

#[rstest]
#[case::start(START, [8, 4, 0, 0, 0], [8, 4, 0, 0, 0])]
#[case::one_knight_out(ONE_KNIGHT_OUT, [7, 7, 0, 1, 0], [8, 4, 0, 0, 0])]
#[case::pawn_screen(PAWN_SCREEN, [0, 0, 0, 11, 0], [3, 0, 0, 0, 0])]
#[case::pawn_screen_theirs(PAWN_SCREEN_THEIRS, [0, 0, 0, 11, 0], [3, 0, 0, 0, 0])]
#[case::crossfire(CROSSFIRE, [5, 0, 0, 9, 0], [4, 0, 0, 5, 0])]
#[case::trenches(TRENCHES, [2, 0, 0, 9, 0], [3, 0, 0, 9, 0])]
#[case::boxed_in(BOXED_IN, [5, 0, 0, 0, 0], [3, 0, 0, 0, 0])]
#[case::centre_grip(CENTRE_GRIP, [0, 13, 9, 0, 0], [5, 0, 0, 0, 0])]
fn safe_mobility_drops_the_squares_an_enemy_pawn_attacks(
    #[case] fen: &str,
    #[case] white: [u16; 5],
    #[case] black: [u16; 5],
) {
    let mobility = facts_of(fen).mobility;
    assert_eq!(mobility.safe_pawns, by_colour([white[0], black[0]]));
    assert_eq!(mobility.safe_knights, by_colour([white[1], black[1]]));
    assert_eq!(mobility.safe_bishops, by_colour([white[2], black[2]]));
    assert_eq!(mobility.safe_rooks, by_colour([white[3], black[3]]));
    assert_eq!(mobility.safe_queens, by_colour([white[4], black[4]]));
}

/// Neither side's pawns cover a square the other's pieces reach in
/// `QUEEN_DUEL`, `OPEN_ROOKS` or `BARE_KINGS`, so there the safe difference is
/// the whole difference.

#[rstest]
#[case::start(START, [0, 0])]
#[case::one_knight_out(ONE_KNIGHT_OUT, [2, 0])]
#[case::queen_duel(QUEEN_DUEL, [8, 8])]
#[case::open_rooks(OPEN_ROOKS, [8, 0])]
#[case::open_rooks_theirs(OPEN_ROOKS_THEIRS, [8, 0])]
#[case::pawn_screen(PAWN_SCREEN, [4, 3])]
#[case::crossfire(CROSSFIRE, [4, 8])]
#[case::trenches(TRENCHES, [1, 1])]
#[case::centre_grip(CENTRE_GRIP, [8, 5])]
#[case::space_grab(SPACE_GRAB, [11, 0])]
fn space_is_what_a_side_attacks_in_the_half_the_other_starts_on(
    #[case] fen: &str,
    #[case] space: [u16; 2],
) {
    assert_eq!(facts_of(fen).mobility.space, by_colour(space));
}

#[rstest]
#[case::start(START, [22, 22])]
#[case::one_knight_out(ONE_KNIGHT_OUT, [26, 22])]
#[case::queen_duel(QUEEN_DUEL, [28, 26])]
#[case::open_rooks(OPEN_ROOKS, [23, 5])]
#[case::open_rooks_theirs(OPEN_ROOKS_THEIRS, [23, 5])]
#[case::pawn_screen(PAWN_SCREEN, [18, 8])]
#[case::crossfire(CROSSFIRE, [19, 14])]
#[case::boxed_in(BOXED_IN, [13, 10])]
#[case::bare_kings(BARE_KINGS, [8, 8])]
fn the_controlled_squares_are_a_sides_whole_attack_map_kings_included(
    #[case] fen: &str,
    #[case] controlled: [u16; 2],
) {
    let attacks = facts_of(fen).attacks;
    assert_eq!(attacks.controlled.white.len(), u32::from(controlled[0]));
    assert_eq!(attacks.controlled.black.len(), u32::from(controlled[1]));
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::one_knight_out(ONE_KNIGHT_OUT, [2, 0])]
#[case::queen_duel(QUEEN_DUEL, [3, 3])]
#[case::pawn_screen(PAWN_SCREEN, [2, 1])]
#[case::pawn_screen_theirs(PAWN_SCREEN_THEIRS, [2, 1])]
#[case::crossfire(CROSSFIRE, [2, 2])]
#[case::trenches(TRENCHES, [3, 3])]
#[case::centre_grip(CENTRE_GRIP, [4, 2])]
#[case::space_grab(SPACE_GRAB, [1, 0])]
#[case::bare_kings(BARE_KINGS, [2, 2])]
fn centre_control_counts_the_attacks_on_d4_e4_d5_and_e5(
    #[case] fen: &str,
    #[case] centre: [u8; 2],
) {
    assert_eq!(facts_of(fen).mobility.centre_control, by_colour(centre));
}

#[rstest]
#[case::start(START, [4, 4])]
#[case::one_knight_out(ONE_KNIGHT_OUT, [6, 4])]
#[case::queen_duel(QUEEN_DUEL, [10, 10])]
#[case::open_rooks(OPEN_ROOKS, [0, 0])]
#[case::pawn_screen(PAWN_SCREEN, [6, 2])]
#[case::pawn_screen_theirs(PAWN_SCREEN_THEIRS, [6, 2])]
#[case::crossfire(CROSSFIRE, [8, 5])]
#[case::boxed_in(BOXED_IN, [3, 2])]
#[case::centre_grip(CENTRE_GRIP, [6, 4])]
#[case::space_grab(SPACE_GRAB, [7, 0])]
#[case::bare_kings(BARE_KINGS, [5, 5])]
fn the_extended_centre_is_the_sixteen_squares_from_c3_to_f6(
    #[case] fen: &str,
    #[case] extended: [u8; 2],
) {
    assert_eq!(
        facts_of(fen).mobility.extended_centre_control,
        by_colour(extended)
    );
}

#[rstest]
#[case::start(START, [5, 5])]
#[case::one_knight_out(ONE_KNIGHT_OUT, [4, 5])]
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
    assert_eq!(facts_of(fen).mobility.immobile_pieces, by_colour(immobile));
}

#[rstest]
#[case::start(START, [12, 12])]
#[case::one_knight_out(ONE_KNIGHT_OUT, [15, 12])]
#[case::queen_duel(QUEEN_DUEL, [23, 24])]
#[case::open_rooks(OPEN_ROOKS, [19, 0])]
#[case::open_rooks_theirs(OPEN_ROOKS_THEIRS, [19, 0])]
#[case::pawn_screen(PAWN_SCREEN, [13, 3])]
#[case::crossfire(CROSSFIRE, [17, 10])]
#[case::trenches(TRENCHES, [12, 13])]
#[case::centre_grip(CENTRE_GRIP, [26, 5])]
#[case::bare_kings(BARE_KINGS, [0, 0])]
fn the_total_mobility_adds_the_five_types_up(#[case] fen: &str, #[case] total: [u16; 2]) {
    assert_eq!(facts_of(fen).mobility.total, by_colour(total));
}

/// Every `mobility` fact reads attack maps alone, so a Chess960 starting array
/// answers exactly as the same placement would.
#[test]
fn the_mobility_facts_of_a_chess960_position_are_the_classic_ones() {
    let facts = facts_under(&CHESS960, NINE_SIXTY);
    let mobility = facts.mobility;
    assert_eq!(mobility.pawns, by_colour([8, 8]));
    assert_eq!(mobility.knights, by_colour([4, 4]));
    assert_eq!(mobility.safe_pawns, by_colour([8, 8]));
    assert_eq!(mobility.safe_knights, by_colour([4, 4]));
    assert_eq!(mobility.total, by_colour([12, 12]));
    assert_eq!(mobility.space, by_colour([0, 0]));
    // The two knights leave e2 and e7 uncovered, so a square fewer than the
    // classic array's 22.
    assert_eq!(facts.attacks.controlled.white.len(), 21);
    assert_eq!(facts.attacks.controlled.black.len(), 21);
    assert_eq!(mobility.centre_control, by_colour([0, 0]));
    assert_eq!(mobility.extended_centre_control, by_colour([4, 4]));
    assert_eq!(mobility.immobile_pieces, by_colour([5, 5]));

    let classic = facts_of("bbqnnrkr/pppppppp/8/8/8/8/PPPPPPPP/BBQNNRKR w - - 0 1");
    assert_eq!(classic.mobility, mobility);
}
