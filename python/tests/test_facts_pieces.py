"""The `pieces` group, fact by fact.

Every expectation is worked out from the definitions in `docs/features.md` §1
and §2.4 for the named position above it. The cases mirror `tests/facts_pieces.rs`.
"""

from __future__ import annotations

from collections.abc import Callable

import esca
import pytest

#: The untouched array: a bishop of each colour a side, every minor at home.
START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

#: The Italian after 4…Nf6: both queens are out, Black's knights both are.
ITALIAN = "r1b1k2r/ppppqppp/2n2n2/2b1p3/2B1P3/5N2/PPPPQPPP/RNB1K2R w KQkq - 8 6"

#: One bishop each, on unlike colours, with pawns on both colours a side.
OPPOSITE = "4k3/pp3p2/3b4/8/2B5/8/PP2P3/4K3 w - - 0 1"

#: Black to move, so the flip makes b7 a dark square and e3 a light one.
FLIPPED = "6k1/1b6/4p3/3n4/8/4B3/5P2/4K3 b - - 0 1"

#: White's two bishops share a colour and Black's do not.
SAME_COLOUR = "4k3/1b3p2/2p5/4p1b1/1P6/1B3P2/6B1/4K3 w - - 0 1"

#: Both back ranks cleared between the rooks; the f-file is Black's alone to use.
LINED = "r4rk1/6pp/8/8/8/5PP1/7P/3R1RK1 w - - 0 1"

#: Two white rooks on the enemy pawn rank; Black's own two are split by a knight.
SEVENTH = "3r2k1/R3R1pp/8/3N4/8/8/3r2PP/6K1 w - - 0 1"

#: Black's h8 rook has nowhere to go and no castling left; White may still castle.
CORNERED = "r2q2kr/pp4pp/8/4Q3/8/8/PP4PP/R3K2R w KQ - 0 1"

#: The mirror image: White's h1 rook is the boxed one, Black's f8 rook is free.
BOXED = "5rk1/1b4pp/8/8/3B4/8/6PP/6KR w - - 0 1"

#: A rook outside its king on the king's own wing, with the whole a-file to itself.
OPEN_CORNER = "4k3/8/8/8/8/8/1PP5/R1K5 w - - 0 1"

#: One passer a side, each with a friendly rook behind it and an enemy rook too.
PASSER_ROOKS = "6k1/5R2/2P2r2/8/5p2/2r5/2R5/6K1 w - - 0 1"

#: The same placement with Black to move: the rook on the seventh is theirs.
PASSER_ROOKS_BLACK = "6k1/5R2/2P2r2/8/5p2/2r5/2R5/6K1 b - - 0 1"

#: Both sides double their rooks behind their own passed pawn.
BATTERY = "6k1/8/6r1/2P3r1/8/6p1/2R5/2R3K1 w - - 0 1"

#: Three outpost squares a side, two of White's held by knights and one of Black's.
OUTPOSTS = "6k1/8/3p1p2/1N1Nn3/2P1P3/8/8/6K1 w - - 0 1"

#: The same three White outpost squares, two of them held by a knight and a
#: bishop; Black has minors on none of its own.
MINOR_OUTPOSTS = "7k/8/3p1p2/1N1B4/2P1P3/8/8/6K1 w - - 0 1"

#: The same placement with Black to move: the two occupied outposts are theirs.
MINOR_OUTPOSTS_BLACK = "7k/8/3p1p2/1N1B4/2P1P3/8/8/6K1 b - - 0 1"

#: The mirror image: a black knight and a black bishop on black outpost
#: squares, and White with three free ones.
MINOR_OUTPOSTS_MIRROR = "6k1/8/8/2p1p3/1n1b4/3P1P2/7K/8 w - - 0 1"

#: The a7 and h7 pawns veto b5 and g5; the knights stand on no outpost at all.
HOLES = "6k1/p6p/8/1N4n1/2P1P3/8/8/6K1 w - - 0 1"

#: Knights on the a- and h-files and on either back rank, and one in the centre.
RIM = "2N3k1/8/4n3/n6N/8/8/8/1n4K1 w - - 0 1"

#: A white rook on the seventh with the black king off the eighth, and a black
#: rook on its own seventh with the white king on its eighth.
SEVENTH_KING_OFF = "3r4/R5pp/6k1/3N4/8/8/3r2PP/5K2 w - - 0 1"

#: The same placement with Black to move, so the two bits change places.
SEVENTH_KING_OFF_BLACK = "3r4/R5pp/6k1/3N4/8/8/3r2PP/5K2 b - - 0 1"

#: A locked centre: two fixed pawns a side, each side's bishop on their colour.
FIXED_CENTRE = "2b3k1/5p2/4p3/3pP3/3P4/8/5P2/2B3K1 w - - 0 1"

#: The same locked pawns with only White holding a bishop.
FIXED_ONE_BISHOP = "6k1/8/4p3/3pP3/3P4/8/8/2B3K1 w - - 0 1"

#: The same again with Black to move, so the bishop's side is *them*.
FIXED_ONE_BISHOP_BLACK = "6k1/8/4p3/3pP3/3P4/8/8/2B3K1 b - - 0 1"

#: Two bare bishops against two bare knights.
PAIR_VS_KNIGHTS = "1n2k1n1/8/8/8/8/8/8/2B1KB2 w - - 0 1"

#: The same, read from the knights' side.
PAIR_VS_KNIGHTS_BLACK = "1n2k1n1/8/8/8/8/8/8/2B1KB2 b - - 0 1"

#: The mirror image: the knights are White's and the bishops Black's.
KNIGHTS_VS_PAIR = "2b1kb2/8/8/8/8/8/8/1N2K1N1 w - - 0 1"

#: A single bishop a side, so neither holds the pair whatever the knights do.
ONE_BISHOP_EACH = "1nb1k1n1/8/8/8/8/8/8/1N2KB2 w - - 0 1"

#: A bishop shut in by an enemy pawn on one square and its king on the other,
#: a side each.
TRAPPED_BISHOPS = "2k5/B1p5/1p6/8/8/6P1/5P1b/5K2 w - - 0 1"

#: A rook with no square of its own at all, and a knight whose only two
#: squares a pawn covers.
TRAPPED_CORNER = "n5k1/8/1P6/P7/8/8/6PP/6KR w - - 0 1"

#: A Chess960 middlegame: the rooks start on d and f, the king between them on e.
NINE_SIXTY = "nnqrkr1b/1p1pp3/3p4/p4ppp/PP5P/1b2PP2/2PPK1P1/N1QR1RBB w fd - 0 10"

#: The same placement with the castling rights spent, which classic chess reads too.
NINE_SIXTY_CLASSIC = "nnqrkr1b/1p1pp3/3p4/p4ppp/PP5P/1b2PP2/2PPK1P1/N1QR1RBB w - - 0 10"

#: The helpers `conftest.py` hands over.
FactsOf = Callable[..., esca.Facts]
Squares = Callable[[str], set[str]]


@pytest.mark.parametrize(
    ("fen", "pair"),
    [
        (START, (True, True)),
        (ITALIAN, (True, True)),
        (SAME_COLOUR, (False, True)),
        (OPPOSITE, (False, False)),
        (FLIPPED, (False, False)),
        (LINED, (False, False)),
    ],
    ids=["start", "italian", "same_colour", "opposite", "flipped", "lined"],
)
def test_a_bishop_pair_needs_a_bishop_of_each_square_colour(
    fen: str, pair: tuple[bool, bool], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pieces.bishop_pair == pair


@pytest.mark.parametrize(
    ("fen", "light", "dark"),
    [
        (START, (1, 1), (1, 1)),
        (OPPOSITE, (1, 0), (0, 1)),
        (SAME_COLOUR, (2, 1), (0, 1)),
        (FLIPPED, (0, 1), (1, 0)),
        (BOXED, (0, 1), (1, 0)),
        (LINED, (0, 0), (0, 0)),
    ],
    ids=["start", "opposite", "same_colour", "flipped", "boxed", "lined"],
)
def test_bishops_are_counted_by_the_square_colour_the_mover_sees(
    fen: str, light: tuple[int, int], dark: tuple[int, int], facts_of: FactsOf
) -> None:
    pieces = facts_of(fen).pieces
    assert pieces.bishops_light == light
    assert pieces.bishops_dark == dark


@pytest.mark.parametrize(
    ("fen", "opposite"),
    [
        (OPPOSITE, True),
        (FLIPPED, True),
        (BOXED, True),
        (START, False),
        (SAME_COLOUR, False),
        (LINED, False),
    ],
    ids=["opposite", "flipped", "boxed", "start", "same_colour", "lined"],
)
def test_bishops_are_opposite_coloured_when_one_each_stands_on_unlike_colours(
    fen: str, opposite: bool, facts_of: FactsOf
) -> None:
    assert facts_of(fen).pieces.opposite_coloured_bishops == opposite


@pytest.mark.parametrize(
    ("fen", "pawns"),
    [
        (START, (8, 8)),
        (ITALIAN, (8, 8)),
        (OPPOSITE, (2, 1)),
        (SAME_COLOUR, (1, 3)),
        (FLIPPED, (1, 1)),
        (LINED, (0, 0)),
    ],
    ids=["start", "italian", "opposite", "same_colour", "flipped", "lined"],
)
def test_a_pawn_counts_on_the_bishop_colour_when_an_own_bishop_shares_its_colour(
    fen: str, pawns: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pieces.pawns_on_bishop_colour == pawns


@pytest.mark.parametrize(
    ("fen", "connected"),
    [
        (LINED, (True, True)),
        (SEVENTH, (True, False)),
        (CORNERED, (False, False)),
        (BATTERY, (False, False)),
        (START, (False, False)),
    ],
    ids=["lined", "seventh", "cornered", "battery", "start"],
)
def test_rooks_are_connected_on_a_rank_when_nothing_stands_between_them(
    fen: str, connected: tuple[bool, bool], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pieces.rooks_connected_rank == connected


@pytest.mark.parametrize(
    ("fen", "connected"),
    [
        (BATTERY, (True, True)),
        (SEVENTH, (False, False)),
        (LINED, (False, False)),
        (CORNERED, (False, False)),
        (START, (False, False)),
    ],
    ids=["battery", "seventh", "lined", "cornered", "start"],
)
def test_rooks_are_connected_on_a_file_when_nothing_stands_between_them(
    fen: str, connected: tuple[bool, bool], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pieces.rooks_connected_file == connected


@pytest.mark.parametrize(
    ("fen", "rooks"),
    [
        (SEVENTH, (2, 2)),
        (LINED, (1, 1)),
        (OPEN_CORNER, (1, 0)),
        (BOXED, (0, 1)),
        (PASSER_ROOKS, (0, 0)),
        (START, (0, 0)),
    ],
    ids=["seventh", "lined", "open_corner", "boxed", "passer_rooks", "start"],
)
def test_a_rook_is_on_an_open_file_when_no_pawn_of_either_colour_holds_it(
    fen: str, rooks: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pieces.rooks_on_open_file == rooks


@pytest.mark.parametrize(
    ("fen", "rooks"),
    [
        (PASSER_ROOKS, (1, 1)),
        (LINED, (0, 1)),
        (SEVENTH, (0, 0)),
        (BATTERY, (0, 0)),
        (START, (0, 0)),
    ],
    ids=["passer_rooks", "lined", "seventh", "battery", "start"],
)
def test_a_rook_is_on_a_semi_open_file_when_only_its_own_side_has_left_it(
    fen: str, rooks: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pieces.rooks_on_semi_open_file == rooks


@pytest.mark.parametrize(
    ("fen", "rooks"),
    [
        (SEVENTH, (2, 1)),
        (PASSER_ROOKS, (1, 0)),
        (LINED, (0, 0)),
        (BATTERY, (0, 0)),
        (START, (0, 0)),
    ],
    ids=["seventh", "passer_rooks", "lined", "battery", "start"],
)
def test_the_relative_seventh_is_counted_from_the_rooks_own_back_rank(
    fen: str, rooks: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pieces.rooks_on_relative_7th == rooks


@pytest.mark.parametrize(
    ("fen", "rooks"),
    [
        (BATTERY, (2, 2)),
        (PASSER_ROOKS, (1, 1)),
        (SEVENTH, (0, 0)),
        (LINED, (0, 0)),
        (START, (0, 0)),
    ],
    ids=["battery", "passer_rooks", "seventh", "lined", "start"],
)
def test_a_rook_behind_an_own_passer_shares_its_file_at_a_lower_relative_rank(
    fen: str, rooks: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pieces.rook_behind_own_passer == rooks


@pytest.mark.parametrize(
    ("fen", "rooks"),
    [
        (PASSER_ROOKS, (1, 1)),
        (BATTERY, (0, 0)),
        (SEVENTH, (0, 0)),
        (LINED, (0, 0)),
        (START, (0, 0)),
    ],
    ids=["passer_rooks", "battery", "seventh", "lined", "start"],
)
def test_behind_an_enemy_passer_is_read_in_the_passer_owners_frame(
    fen: str, rooks: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pieces.rook_behind_enemy_passer == rooks


@pytest.mark.parametrize(
    ("fen", "trapped"),
    [
        (CORNERED, (False, True)),
        (BOXED, (True, False)),
        (OPEN_CORNER, (False, False)),
        (LINED, (False, False)),
        (START, (False, False)),
    ],
    ids=["cornered", "boxed", "open_corner", "lined", "start"],
)
def test_a_trapped_rook_is_boxed_in_beyond_its_own_king_with_the_castling_rights_gone(
    fen: str, trapped: tuple[bool, bool], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pieces.trapped_rook == trapped


@pytest.mark.parametrize(
    ("fen", "us", "them"),
    [
        (OUTPOSTS, "b5 d5 f5", "c5 e5 g5"),
        (SAME_COLOUR, "a5 c5", "b5 d4 d5 f4"),
        (HOLES, "d5 f5", ""),
        (FLIPPED, "d5 f5", ""),
        (BATTERY, "b6 d6", ""),
        (PASSER_ROOKS, "", "e3 g3"),
        (LINED, "e4", ""),
        (START, "", ""),
    ],
    ids=["outposts", "same_colour", "holes", "flipped", "battery", "passer_rooks", "lined", "start"],
)
def test_an_outpost_square_is_pawn_held_ground_on_ranks_four_to_six(
    fen: str, us: str, them: str, facts_of: FactsOf, squares: Squares
) -> None:
    pieces = facts_of(fen).pieces
    assert set(pieces.outposts[esca.US]) == squares(us)
    assert set(pieces.outposts[esca.THEM]) == squares(them)


@pytest.mark.parametrize(
    ("fen", "minors"),
    [
        (OUTPOSTS, (2, 1)),
        (MINOR_OUTPOSTS, (2, 0)),
        (MINOR_OUTPOSTS_BLACK, (0, 2)),
        (MINOR_OUTPOSTS_MIRROR, (0, 2)),
        (FLIPPED, (1, 0)),
        (HOLES, (0, 0)),
        (RIM, (0, 0)),
        (START, (0, 0)),
    ],
    ids=[
        "outposts",
        "minor_outposts",
        "minor_outposts_black",
        "minor_outposts_mirror",
        "flipped",
        "holes",
        "rim",
        "start",
    ],
)
def test_a_minor_on_an_outpost_is_a_knight_or_a_bishop_on_an_own_outpost_square(
    fen: str, minors: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pieces.minors_on_outpost == minors


@pytest.mark.parametrize(
    ("fen", "free"),
    [
        (SAME_COLOUR, (2, 4)),
        (OUTPOSTS, (1, 2)),
        (HOLES, (2, 0)),
        (PASSER_ROOKS, (0, 2)),
        (FLIPPED, (1, 0)),
        (START, (0, 0)),
    ],
    ids=["same_colour", "outposts", "holes", "passer_rooks", "flipped", "start"],
)
def test_a_free_outpost_square_is_one_no_unit_of_either_colour_occupies(
    fen: str, free: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pieces.outpost_squares_free == free


@pytest.mark.parametrize(
    ("fen", "knights"),
    [
        (START, (2, 2)),
        (RIM, (2, 2)),
        (ITALIAN, (1, 0)),
        (OUTPOSTS, (0, 0)),
        (HOLES, (0, 0)),
    ],
    ids=["start", "rim", "italian", "outposts", "holes"],
)
def test_a_knight_is_on_the_rim_on_file_a_or_h_or_on_relative_rank_one_or_eight(
    fen: str, knights: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pieces.knights_on_rim == knights


@pytest.mark.parametrize(
    ("fen", "minors"),
    [
        (START, (4, 4)),
        (ITALIAN, (2, 1)),
        (NINE_SIXTY_CLASSIC, (1, 1)),
        (OPPOSITE, (0, 0)),
        (RIM, (0, 0)),
    ],
    ids=["start", "italian", "nine_sixty_classic", "opposite", "rim"],
)
def test_an_undeveloped_minor_still_stands_on_a_classic_starting_square(
    fen: str, minors: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pieces.minors_undeveloped == minors


@pytest.mark.parametrize(
    ("fen", "developed"),
    [
        (ITALIAN, (True, True)),
        (NINE_SIXTY_CLASSIC, (True, True)),
        (CORNERED, (True, False)),
        (START, (False, False)),
        (OPPOSITE, (False, False)),
    ],
    ids=["italian", "nine_sixty_classic", "cornered", "start", "opposite"],
)
def test_a_queen_is_developed_once_it_stands_off_its_classic_starting_square(
    fen: str, developed: tuple[bool, bool], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pieces.queen_developed == developed


@pytest.mark.parametrize(
    ("fen", "pawns"),
    [
        (ITALIAN, (2, 3)),
        (FIXED_CENTRE, (2, 2)),
        (FIXED_ONE_BISHOP, (2, 0)),
        (FIXED_ONE_BISHOP_BLACK, (0, 2)),
        (OPPOSITE, (0, 0)),
        (START, (0, 0)),
    ],
    ids=["italian", "fixed_centre", "fixed_one_bishop", "fixed_one_bishop_black", "opposite", "start"],
)
def test_a_fixed_pawn_on_the_bishop_colour_is_one_whose_stop_square_is_held(
    fen: str, pawns: tuple[int, int], facts_of: FactsOf
) -> None:
    pieces = facts_of(fen).pieces
    assert pieces.fixed_pawns_on_bishop_colour == pawns
    for fixed, on_colour in zip(pieces.fixed_pawns_on_bishop_colour, pieces.pawns_on_bishop_colour, strict=True):
        assert fixed <= on_colour, "the fixed pawns are a subset of the pawns on the colour"


@pytest.mark.parametrize(
    ("fen", "imbalance"),
    [
        (PAIR_VS_KNIGHTS, 1),
        (PAIR_VS_KNIGHTS_BLACK, -1),
        (KNIGHTS_VS_PAIR, -1),
        (START, 0),
        (ITALIAN, 0),
        (ONE_BISHOP_EACH, 0),
        (LINED, 0),
    ],
    ids=["pair_vs_knights", "pair_vs_knights_black", "knights_vs_pair", "start", "italian", "one_bishop_each", "lined"],
)
def test_the_bishop_pair_counts_against_a_knight_pair_and_cancels_with_the_reverse(
    fen: str, imbalance: int, facts_of: FactsOf
) -> None:
    assert facts_of(fen).pieces.bishop_pair_vs_knight_pair == imbalance


@pytest.mark.parametrize(
    ("fen", "cut_off"),
    [
        (SEVENTH, (True, True)),
        (PASSER_ROOKS, (True, False)),
        (SEVENTH_KING_OFF_BLACK, (True, False)),
        (SEVENTH_KING_OFF, (False, True)),
        (PASSER_ROOKS_BLACK, (False, True)),
        (LINED, (False, False)),
        (START, (False, False)),
    ],
    ids=[
        "seventh",
        "passer_rooks",
        "seventh_king_off_black",
        "seventh_king_off",
        "passer_rooks_black",
        "lined",
        "start",
    ],
)
def test_a_rook_on_the_seventh_pays_when_the_enemy_king_stands_on_the_eighth(
    fen: str, cut_off: tuple[bool, bool], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pieces.rook_on_7th_with_king_on_8th == cut_off


@pytest.mark.parametrize(
    ("fen", "trapped"),
    [
        (START, (5, 5)),
        (ITALIAN, (2, 1)),
        (TRAPPED_BISHOPS, (1, 1)),
        (TRAPPED_CORNER, (1, 1)),
        (BOXED, (1, 0)),
        (CORNERED, (0, 1)),
        (LINED, (0, 0)),
        (OPEN_CORNER, (0, 0)),
    ],
    ids=["start", "italian", "trapped_bishops", "trapped_corner", "boxed", "cornered", "lined", "open_corner"],
)
def test_a_trapped_unit_has_no_safe_destination_among_the_squares_it_reaches(
    fen: str, trapped: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pieces.trapped_pieces == trapped


@pytest.mark.parametrize(
    ("fen", "value"),
    [
        (START, (25, 25)),
        (ITALIAN, (8, 3)),
        (TRAPPED_BISHOPS, (3, 3)),
        (TRAPPED_CORNER, (5, 3)),
        (BOXED, (5, 0)),
        (CORNERED, (0, 5)),
        (LINED, (0, 0)),
        (OPEN_CORNER, (0, 0)),
    ],
    ids=["start", "italian", "trapped_bishops", "trapped_corner", "boxed", "cornered", "lined", "open_corner"],
)
def test_the_trapped_value_is_the_value_sum_of_the_trapped_units(
    fen: str, value: tuple[int, int], facts_of: FactsOf
) -> None:
    assert facts_of(fen).pieces.trapped_value == value


def test_the_piece_facts_of_a_chess960_position_read_the_placement_as_they_find_it(
    facts_of: FactsOf,
) -> None:
    """Only `minors_undeveloped` and `queen_developed` read the starting squares,
    so the rest of the group answers for a Chess960 placement as for any other."""
    pieces = facts_of(NINE_SIXTY, esca.CHESS960).pieces
    assert pieces.bishop_pair == (True, True)
    assert pieces.bishops_light == (1, 1)
    assert pieces.bishops_dark == (1, 1)
    assert pieces.pawns_on_bishop_colour == (8, 8)
    assert pieces.rooks_connected_rank == (True, False)
    assert pieces.rooks_connected_file == (False, False)
    assert pieces.rooks_on_relative_7th == (0, 0)
    assert pieces.trapped_rook == (True, False)
    assert pieces.knights_on_rim == (1, 2)
    assert set(pieces.outposts[esca.US]) == set()
    assert set(pieces.outposts[esca.THEM]) == set()


def test_chess960_writes_the_two_facts_that_assume_the_starting_squares_as_zeros() -> None:
    """`features.md` §4 defines `minors_undeveloped` and `queen_developed` for
    classic chess only; they are the group's values 31 to 34."""
    classic = esca.encode([NINE_SIXTY_CLASSIC], groups=["pieces"])[0]
    nine_sixty = esca.encode([NINE_SIXTY_CLASSIC], variant=esca.CHESS960, groups=["pieces"])[0]

    assert list(classic[31:35]) == [0.25, 0.25, 1.0, 1.0]
    assert list(nine_sixty[31:35]) == [0.0, 0.0, 0.0, 0.0]
    assert list(classic[:31]) == list(nine_sixty[:31])
    assert list(classic[35:]) == list(nine_sixty[35:])
