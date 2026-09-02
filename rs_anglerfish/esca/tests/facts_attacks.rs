//! The `attacks` group, fact by fact.
//!
//! Every expectation is worked out from the definitions in `docs/features.md`
//! §1 for the named position above it.

mod common;

use common::{facts_of, facts_under, squares};
use esca::{CHESS960, Role, Side, SquareSet};
use rstest::rstest;

/// The untouched array: every unit but the two rooks stands on a defended square.
const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// The checked king stops the rook's ray on it, and the pawn covers two squares.
const RAYS: &str = "4k3/8/8/8/8/1N6/1p6/r2K4 w - - 0 1";

/// Rooks face each other down an open file; a knight and a bishop stand before pawns.
const LOOSE: &str = "3r2k1/7p/6p1/p4N2/1b2P3/2P5/1P4PP/3R2K1 w - - 0 1";

/// The checking rook is defended, the rook that defends the checked king is not.
const CHECKED: &str = "4k3/8/6b1/8/8/8/4r3/1R2K1n1 w - - 0 1";

/// Two white units may not leave the line to their king, and one black unit may not.
const PINS: &str = "4k3/5p2/2n5/1B2q3/1b6/2N1N2Q/8/4K3 w - - 0 1";

/// The same placement with Black to move: every fact of it changes sides.
const PINS_THEIRS: &str = "4k3/5p2/2n5/1B2q3/1b6/2N1N2Q/8/4K3 b - - 0 1";

/// A rook and a bishop each look through a piece at a cheaper one behind it.
const SKEWERS: &str = "1r1r2k1/6b1/8/1N1qN3/8/8/1P6/3R2K1 w - - 0 1";

/// Sliders looking through four units at what stands behind, of every value.
const BEHIND: &str = "2n3k1/Rrr5/b6b/8/q7/8/3K4/2Q5 w - - 0 1";

/// A castled middlegame: the pinned f7-pawn is neither hanging nor en prise.
const CASTLED: &str = "3q1rk1/5ppp/3p1n2/8/1bBP4/2N1P3/5PPP/3Q1RK1 w - - 0 1";

/// A Chess960 middlegame: three loose pawns a side, and a bishop loose on b3.
const NINE_SIXTY: &str = "nnqrkr1b/1p1pp3/3p4/p4ppp/PP5P/1b2PP2/2PPK1P1/N1QR1RBB w fd - 0 10";

#[rstest]
#[case::start(
    START,
    "a2 a3 b1 b2 b3 c1 c2 c3 d1 d2 d3 e1 e2 e3 f1 f2 f3 g1 g2 g3 h2 h3",
    "a6 a7 b6 b7 b8 c6 c7 c8 d6 d7 d8 e6 e7 e8 f6 f7 f8 g6 g7 g8 h6 h7"
)]
#[case::rays(
    RAYS,
    "a1 a5 c1 c2 c5 d2 d4 e1 e2",
    "a1 a2 a3 a4 a5 a6 a7 a8 b1 c1 d1 d7 d8 e7 f7 f8"
)]
#[case::checked(
    CHECKED,
    "a1 b2 b3 b4 b5 b6 b7 b8 c1 d1 d2 e1 e2 f1 f2",
    "a2 b1 b2 c2 d2 d3 d7 d8 e1 e2 e3 e4 e5 e6 e7 e8 f2 f3 f5 f7 f8 g2 h2 h3 h5 h7"
)]
fn a_side_attacks_every_square_one_of_its_units_could_capture_on(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let facts = facts_of(fen);
    assert_eq!(facts.attacks.by[Side::Us.index()], squares(us));
    assert_eq!(facts.attacks.by[Side::Them.index()], squares(them));
}

#[rstest]
#[case::start(START, 22, 22)]
#[case::rays(RAYS, 9, 16)]
#[case::loose(LOOSE, 29, 26)]
#[case::checked(CHECKED, 15, 26)]
#[case::skewers(SKEWERS, 26, 34)]
#[case::behind(BEHIND, 23, 43)]
#[case::castled(CASTLED, 34, 29)]
fn the_attacked_square_count_is_the_size_of_that_map(
    #[case] fen: &str,
    #[case] us: u32,
    #[case] them: u32,
) {
    let facts = facts_of(fen);
    assert_eq!(facts.attacks.by[Side::Us.index()].len(), us);
    assert_eq!(facts.attacks.by[Side::Them.index()].len(), them);
}

#[rstest]
#[case::start(START, "a3 b3 c3 d3 e3 f3 g3 h3", "a6 b6 c6 d6 e6 f6 g6 h6")]
#[case::rays(RAYS, "", "a1 c1")]
#[case::loose(LOOSE, "a3 b4 c3 d4 d5 f3 f5 g3 h3", "b4 f5 g6 h5")]
#[case::castled(CASTLED, "c5 d4 e3 e5 f3 f4 g3 h3", "c5 e5 e6 f6 g6 h6")]
fn a_pawn_attacks_the_two_squares_diagonally_ahead_of_it_and_no_other(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let facts = facts_of(fen);
    assert_eq!(facts.attacks.by_pawns[Side::Us.index()], squares(us));
    assert_eq!(facts.attacks.by_pawns[Side::Them.index()], squares(them));
}

/// The whole map is the union of the six role maps, so a role a side has none
/// of contributes nothing.
#[test]
fn the_map_is_kept_per_role_as_well() {
    let facts = facts_of(LOOSE);
    let attacks = &facts.attacks;
    let (us, them) = (Side::Us.index(), Side::Them.index());

    assert_eq!(
        attacks.by_role[us][Role::Knight.index()],
        squares("d4 d6 e3 e7 g3 g7 h4 h6")
    );
    assert_eq!(
        attacks.by_role[us][Role::Rook.index()],
        squares("a1 b1 c1 d2 d3 d4 d5 d6 d7 d8 e1 f1 g1")
    );
    assert_eq!(
        attacks.by_role[us][Role::King.index()],
        squares("f1 f2 g2 h1 h2")
    );
    assert_eq!(
        attacks.by_role[them][Role::Bishop.index()],
        squares("a3 a5 c3 c5 d6 e7 f8")
    );
    assert!(attacks.by_role[them][Role::Queen.index()].is_empty());

    for side in [us, them] {
        let union = Role::ALL.into_iter().fold(SquareSet::EMPTY, |all, role| {
            all | attacks.by_role[side][role.index()]
        });
        assert_eq!(union, attacks.by[side]);
        assert_eq!(
            attacks.by_pawns[side],
            attacks.by_role[side][Role::Pawn.index()]
        );
    }
}

#[rstest]
#[case::start(START, "", "")]
#[case::rays(RAYS, "", "")]
#[case::loose(LOOSE, "d1", "d8")]
#[case::checked(CHECKED, "b1", "")]
#[case::pins(PINS, "c3", "c6")]
#[case::pins_theirs(PINS_THEIRS, "c6", "c3")]
#[case::skewers(SKEWERS, "b5 d1 e5", "")]
#[case::behind(BEHIND, "a7", "")]
#[case::castled(CASTLED, "c3", "")]
fn a_hanging_unit_is_attacked_and_undefended_and_is_never_a_king(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let facts = facts_of(fen);
    assert_eq!(facts.attacks.hanging[Side::Us.index()], squares(us));
    assert_eq!(facts.attacks.hanging[Side::Them.index()], squares(them));
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::rays(RAYS, [0, 0])]
#[case::loose(LOOSE, [5, 5])]
#[case::checked(CHECKED, [5, 0])]
#[case::pins(PINS, [3, 3])]
#[case::skewers(SKEWERS, [11, 0])]
#[case::behind(BEHIND, [5, 0])]
#[case::castled(CASTLED, [3, 0])]
fn the_hanging_value_adds_up_what_the_hanging_units_are_worth(
    #[case] fen: &str,
    #[case] value: [i32; 2],
) {
    assert_eq!(facts_of(fen).attacks.hanging_value, value);
}

#[rstest]
#[case::start(START, "", "")]
#[case::rays(RAYS, "", "a1")]
#[case::loose(LOOSE, "d1 f5", "b4 d8")]
#[case::checked(CHECKED, "b1", "")]
#[case::pins(PINS, "c3", "c6")]
#[case::skewers(SKEWERS, "b5 d1 e5", "d5")]
#[case::behind(BEHIND, "a7 c1", "")]
#[case::castled(CASTLED, "c3", "")]
fn a_unit_is_en_prise_when_it_hangs_or_a_cheaper_unit_attacks_it(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let facts = facts_of(fen);
    assert_eq!(facts.attacks.en_prise[Side::Us.index()], squares(us));
    assert_eq!(facts.attacks.en_prise[Side::Them.index()], squares(them));
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::rays(RAYS, [0, 5])]
#[case::loose(LOOSE, [5, 5])]
#[case::checked(CHECKED, [5, 0])]
#[case::pins(PINS, [3, 3])]
#[case::skewers(SKEWERS, [5, 9])]
#[case::behind(BEHIND, [9, 0])]
#[case::castled(CASTLED, [3, 0])]
fn the_en_prise_maximum_is_the_largest_value_standing_en_prise(
    #[case] fen: &str,
    #[case] value: [i32; 2],
) {
    assert_eq!(facts_of(fen).attacks.en_prise_max_value, value);
}

#[rstest]
#[case::start(START, "", "")]
#[case::rays(RAYS, "", "")]
#[case::loose(LOOSE, "", "")]
#[case::checked(CHECKED, "", "")]
#[case::pins(PINS, "c3 e3", "c6")]
#[case::pins_theirs(PINS_THEIRS, "c6", "c3 e3")]
#[case::behind(BEHIND, "", "")]
#[case::castled(CASTLED, "", "f7")]
fn a_pinned_unit_is_the_only_thing_between_a_slider_and_its_own_king(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let facts = facts_of(fen);
    assert_eq!(facts.attacks.pinned[Side::Us.index()], squares(us));
    assert_eq!(facts.attacks.pinned[Side::Them.index()], squares(them));
}

#[rstest]
#[case::start(START, [0, 0])]
#[case::loose(LOOSE, [0, 0])]
#[case::checked(CHECKED, [0, 0])]
#[case::pins(PINS, [0, 0])]
#[case::skewers(SKEWERS, [1, 2])]
#[case::behind(BEHIND, [2, 1])]
#[case::castled(CASTLED, [0, 0])]
fn a_skewer_is_counted_once_per_slider_front_unit_and_cheaper_unit_behind(
    #[case] fen: &str,
    #[case] skewers: [u8; 2],
) {
    assert_eq!(facts_of(fen).attacks.skewer_candidates, skewers);
}

#[rstest]
#[case::start(
    START,
    "a2 b1 b2 c1 c2 d1 d2 e1 e2 f1 f2 g1 g2 h2",
    "a7 b7 b8 c7 c8 d7 d8 e7 e8 f7 f8 g7 g8 h7"
)]
#[case::rays(RAYS, "", "a1")]
#[case::loose(LOOSE, "c3 f5 g1 g2 h2", "a5 b4 g6 g8 h7")]
#[case::checked(CHECKED, "e1", "e2 e8")]
#[case::pins(PINS, "b5 e3", "b4 e5 e8 f7")]
#[case::skewers(SKEWERS, "g1", "b8 d5 d8 g7 g8")]
#[case::behind(BEHIND, "c1 d2", "a6 b7 c7 c8")]
#[case::castled(CASTLED, "d1 d4 e3 f1 f2 g1 g2 h2", "d6 d8 f6 f7 f8 g7 g8 h7")]
fn a_defended_unit_stands_on_a_square_its_own_side_attacks(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let facts = facts_of(fen);
    assert_eq!(facts.attacks.defended[Side::Us.index()], squares(us));
    assert_eq!(facts.attacks.defended[Side::Them.index()], squares(them));
}

#[rstest]
#[case::rays(RAYS, "b3 d1", "a1 b2 e8")]
#[case::loose(LOOSE, "b2 c3 d1 e4 f5 g1 g2 h2", "a5 b4 d8 g6 g8 h7")]
#[case::skewers(SKEWERS, "b2 b5 d1 e5 g1", "b8 d5 d8 g7 g8")]
#[case::behind(BEHIND, "a7 c1 d2", "a4 a6 b7 c7 c8 g8 h6")]
#[case::pins_theirs(PINS_THEIRS, "b4 c6 e5 e8 f7", "b5 c3 e1 e3 h3")]
fn the_units_of_each_side_are_listed_us_first(
    #[case] fen: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let facts = facts_of(fen);
    assert_eq!(facts.attacks.units(Side::Us), squares(us));
    assert_eq!(facts.attacks.units(Side::Them), squares(them));
}

#[rstest]
#[case::three_at_the_centre(LOOSE, "d4", "c3 d1 f5", "d8")]
#[case::pawn_against_pawn(LOOSE, "f5", "e4", "g6")]
#[case::a_king_is_an_attacker_too(BEHIND, "c1", "d2", "c7")]
#[case::two_on_one_square(BEHIND, "b7", "a7", "a6 c7")]
#[case::the_checked_square(CHECKED, "e1", "b1", "e2")]
#[case::through_no_one(PINS, "e3", "h3", "e5")]
#[case::down_the_open_file(SKEWERS, "d5", "d1", "d8")]
fn the_attackers_of_a_square_are_the_units_of_a_side_that_bear_on_it(
    #[case] fen: &str,
    #[case] square: &str,
    #[case] us: &str,
    #[case] them: &str,
) {
    let facts = facts_of(fen);
    let square = square.parse().expect("a square name");
    assert_eq!(facts.attacks.attackers_of(square, Side::Us), squares(us));
    assert_eq!(
        facts.attacks.attackers_of(square, Side::Them),
        squares(them)
    );
}

#[rstest]
#[case::ours(LOOSE, "d1", true)]
#[case::theirs(LOOSE, "d8", true)]
#[case::defended(LOOSE, "f5", false)]
#[case::unattacked(LOOSE, "g1", false)]
#[case::one_of_three(SKEWERS, "b5", true)]
#[case::en_prise_but_defended(SKEWERS, "d5", false)]
fn a_unit_of_either_colour_is_asked_whether_it_hangs(
    #[case] fen: &str,
    #[case] square: &str,
    #[case] hanging: bool,
) {
    let facts = facts_of(fen);
    let square = square.parse().expect("a square name");
    assert_eq!(facts.attacks.is_hanging(square), hanging);
}

/// No `attacks` fact is among the four `features.md` §4 defines for classic
/// chess only, so a Chess960 position answers exactly as the same placement
/// would.
#[test]
fn the_attack_facts_of_a_chess960_position_are_the_classic_ones() {
    let facts = facts_under(&CHESS960, NINE_SIXTY);
    let attacks = &facts.attacks;
    let (us, them) = (Side::Us.index(), Side::Them.index());

    assert_eq!(attacks.by[us].len(), 28);
    assert_eq!(attacks.by[them].len(), 36);
    assert_eq!(attacks.hanging[us], squares("a4 b4 h4"));
    assert_eq!(attacks.hanging[them], squares("a5 b3 g5"));
    assert_eq!(attacks.hanging_value, [3, 5]);
    assert_eq!(attacks.en_prise[them], squares("a5 b3 g5"));
    assert_eq!(attacks.en_prise_max_value, [1, 3]);
    assert_eq!(
        attacks.defended[us],
        squares("a1 c1 c2 d1 d2 e3 f1 f3 g1 g2")
    );
    assert!(attacks.pinned[us].is_empty());
    assert!(attacks.pinned[them].is_empty());
    assert_eq!(attacks.skewer_candidates, [0, 0]);

    let classic = facts_of("nnqrkr1b/1p1pp3/3p4/p4ppp/PP5P/1b2PP2/2PPK1P1/N1QR1RBB w - - 0 10");
    assert_eq!(classic.attacks, *attacks);
}
