"""The `endings` layer: which named ending a position is, what theory says the
result is, and the method it is played by.

Every expectation is read off the diagram above the named position and the
definitions in `docs/esca-api.md` §13. The cases mirror `tests/endings.rs`.
"""

from __future__ import annotations

import esca
import pytest
from esca import endings

#: Kings only.
BARE_KINGS = "4k3/8/8/8/8/8/8/4K3 w - - 0 1"

#: Queen against a lone king.
LONE_QUEEN = "4k3/8/8/8/8/8/8/3QK3 w - - 0 1"

#: Rook against a lone king.
LONE_ROOK = "4k3/8/8/8/8/8/8/3RK3 w - - 0 1"

#: Two bishops against a lone king, one on each square colour.
TWO_BISHOPS = "4k3/8/8/8/8/8/8/2BBK3 w - - 0 1"

#: Two bishops against a lone king, both on dark squares: promotions can put
#: them there, and then nothing can be forced.
TWO_DARK_BISHOPS = "4k3/8/8/8/8/B7/8/2B1K3 w - - 0 1"

#: Bishop and knight against a lone king.
BISHOP_AND_KNIGHT = "4k3/8/8/8/8/8/8/2BNK3 w - - 0 1"

#: Two knights against a lone king.
TWO_KNIGHTS = "4k3/8/8/8/8/8/8/2NNK3 w - - 0 1"

#: One bishop against a lone king.
LONE_BISHOP = "4k3/8/8/8/8/8/8/3BK3 w - - 0 1"

#: One knight against a lone king.
LONE_KNIGHT = "4k3/8/8/8/8/8/8/3NK3 w - - 0 1"

#: A centre pawn on e3, both kings far from it.
CENTRE_PAWN = "4k3/8/8/8/8/4P3/8/4K3 w - - 0 1"

#: The h-pawn's promotion corner, held by the black king on g8.
H_PAWN_CORNER = "6k1/8/8/8/8/8/7P/6K1 w - - 0 1"

#: The a-pawn's promotion corner, held by the black king on b8.
A_PAWN_CORNER = "1k6/8/8/8/8/8/P7/1K6 w - - 0 1"

#: The black king on h1 is seven king moves from a8 and the pawn five: it
#: never catches the pawn.
RUNNING_PAWN = "8/8/8/8/8/8/P7/K6k w - - 0 1"

#: The kings face each other on e6 and e8 with e7 empty between them.
KINGS_IN_OPPOSITION = "4k3/8/4K3/4P3/8/8/8/8 w - - 0 1"

#: Bishop and a-pawn: the c1 bishop is dark and a8 is light.
WRONG_BISHOP = "4k3/8/8/8/8/8/P7/2B1K3 w - - 0 1"

#: Bishop and h-pawn: the f1 bishop is light and h8 is dark.
WRONG_BISHOP_OTHER_CORNER = "4k3/8/8/8/8/8/7P/4KB2 w - - 0 1"

#: Bishop and a-pawn: the d1 bishop is light, as a8 is.
RIGHT_BISHOP = "4k3/8/8/8/8/8/P7/3BK3 w - - 0 1"

#: A queen each.
QUEEN_V_QUEEN = "3qk3/8/8/8/8/8/8/3QK3 w - - 0 1"

#: Queen against rook.
QUEEN_V_ROOK = "3rk3/8/8/8/8/8/8/3QK3 w - - 0 1"

#: Queen against bishop.
QUEEN_V_BISHOP = "2b1k3/8/8/8/8/8/8/3QK3 w - - 0 1"

#: Queen against knight.
QUEEN_V_KNIGHT = "1n2k3/8/8/8/8/8/8/3QK3 w - - 0 1"

#: Queen against one pawn.
QUEEN_V_PAWN = "4k3/8/8/8/4p3/8/8/3QK3 w - - 0 1"

#: Queen against bishop and knight together.
QUEEN_V_TWO_MINORS = "1bn1k3/8/8/8/8/8/8/3QK3 w - - 0 1"

#: A rook each.
ROOK_V_ROOK = "3rk3/8/8/8/8/8/8/3RK3 w - - 0 1"

#: Rook against bishop.
ROOK_V_BISHOP = "2b1k3/8/8/8/8/8/8/3RK3 w - - 0 1"

#: Rook against knight.
ROOK_V_KNIGHT = "1n2k3/8/8/8/8/8/8/3RK3 w - - 0 1"

#: Rook against one pawn.
ROOK_V_PAWN = "4k3/8/8/8/4p3/8/8/3RK3 w - - 0 1"

#: Rook against bishop and knight together: the two minors are the stronger
#: side, so the signature writes Black first.
ROOK_V_TWO_MINORS = "1bn1k3/8/8/8/8/8/8/3RK3 w - - 0 1"

#: Rook and pawn against rook, the black king off the pawn's file.
ROOK_AND_PAWN = "3rk3/8/8/8/8/8/3P4/3RK3 w - - 0 1"

#: The same material with the black king on d8, in front of the pawn.
ROOK_AND_PAWN_HELD = "3k4/3r4/8/8/8/8/3P4/3RK3 w - - 0 1"

#: Two bishops against a knight.
TWO_BISHOPS_V_KNIGHT = "1n2k3/8/8/8/8/8/8/2BBK3 w - - 0 1"

#: Bishop against knight: the bishop side is written first on the tie.
BISHOP_V_KNIGHT = "1n2k3/8/8/8/8/8/8/3BK3 w - - 0 1"

#: A bishop each, c8 and d1, both light.
SAME_COLOUR_BISHOPS = "2b1k3/8/8/8/8/8/8/3BK3 w - - 0 1"

#: A bishop each, d8 dark and d1 light.
OPPOSITE_BISHOPS = "3bk3/8/8/8/8/8/8/3BK3 w - - 0 1"

#: A knight each.
KNIGHT_V_KNIGHT = "1n2k3/8/8/8/8/8/8/3NK3 w - - 0 1"

#: Bishop against one pawn.
BISHOP_V_PAWN = "4k3/8/8/8/4p3/8/8/3BK3 w - - 0 1"

#: Knight against one pawn.
KNIGHT_V_PAWN = "4k3/8/8/8/4p3/8/8/3NK3 w - - 0 1"

#: Two pawns a side and nothing else.
PAWN_ENDING = "4k3/pp6/8/8/8/8/PP6/4K3 w - - 0 1"

#: Two pawns against a lone king: still a pawn ending, not `KPvK`.
TWO_PAWNS = "4k3/8/8/8/8/8/PP6/4K3 w - - 0 1"

#: Black holds the queen and White the rook, so the signature writes Black
#: first.
BLACK_IS_STRONGER = "3qk3/8/8/8/8/8/8/3RK3 w - - 0 1"

#: Queen and rook against a lone king: two pieces, so an ending, and one the
#: catalogue does not name.
QUEEN_AND_ROOK = "4k3/8/8/8/8/8/8/R2QK3 w - - 0 1"

#: Queen, rook and knight against a lone king: three pieces, one too many.
THREE_PIECES = "4k3/8/8/8/8/8/8/RN1QK3 w - - 0 1"

#: Two pieces a side, which is still an ending.
TWO_PIECES_EACH = "1r1qk3/8/8/8/8/8/8/1R1QK3 w - - 0 1"

#: The untouched array.
START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"


def ending(fen: str) -> endings.Ending:
    """The ending the position `fen` describes is."""
    return endings.classify(esca.Position.from_fen(fen))


# ------------------------------------------------------------------ signature


@pytest.mark.parametrize(
    ("fen", "text", "stronger"),
    [
        (BARE_KINGS, "KvK", "w"),
        (LONE_QUEEN, "KQvK", "w"),
        (ROOK_AND_PAWN, "KRPvKR", "w"),
        (BLACK_IS_STRONGER, "KQvKR", "b"),
        (ROOK_V_TWO_MINORS, "KBNvKR", "b"),
        (BISHOP_V_KNIGHT, "KBvKN", "w"),
        (PAWN_ENDING, "KPPvKPP", "w"),
        (START, "KQRRBBNNPPPPPPPPvKQRRBBNNPPPPPPPP", "w"),
    ],
    ids=[
        "bare_kings",
        "lone_queen",
        "rook_and_pawn",
        "black_is_stronger",
        "rook_v_two_minors",
        "bishop_v_knight",
        "pawn_ending",
        "start",
    ],
)
def test_a_signature_writes_the_stronger_side_first(fen: str, text: str, stronger: str) -> None:
    signature = ending(fen).signature
    assert signature.text == text
    assert signature.stronger == stronger


@pytest.mark.parametrize(
    ("fen", "side", "unit", "count"),
    [
        (ROOK_AND_PAWN, "w", "r", 1),
        (ROOK_AND_PAWN, "w", "p", 1),
        (ROOK_AND_PAWN, "b", "r", 1),
        (ROOK_AND_PAWN, "b", "p", 0),
        (ROOK_AND_PAWN, "w", "k", 1),
        (START, "b", "p", 8),
        (START, "w", "b", 2),
    ],
    ids=[
        "white_rook",
        "white_pawn",
        "black_rook",
        "black_pawn",
        "white_king",
        "start_black_pawns",
        "start_white_bishops",
    ],
)
def test_a_signature_counts_every_role_of_every_side(fen: str, side: str, unit: str, count: int) -> None:
    assert ending(fen).signature.count(side, unit) == count


@pytest.mark.parametrize(
    ("fen", "side", "pieces", "value"),
    [
        (LONE_QUEEN, "w", 1, 9),
        (ROOK_AND_PAWN, "w", 1, 6),
        (TWO_PIECES_EACH, "b", 2, 14),
    ],
    ids=["lone_queen", "rook_and_pawn", "two_pieces_each"],
)
def test_a_signature_counts_the_pieces_and_the_material_of_a_side(fen: str, side: str, pieces: int, value: int) -> None:
    signature = ending(fen).signature
    assert signature.pieces(side) == pieces
    assert signature.value(side) == value


def test_a_signature_refuses_a_colour_or_a_role_it_does_not_know() -> None:
    signature = ending(BARE_KINGS).signature
    with pytest.raises(ValueError, match="not a colour"):
        signature.count("white", "r")
    with pytest.raises(ValueError, match="not a role"):
        signature.count("w", "rook")


# -------------------------------------------------------------- the threshold


@pytest.mark.parametrize(
    ("fen", "class_"),
    [
        (QUEEN_AND_ROOK, "other"),
        (TWO_PIECES_EACH, "other"),
        (THREE_PIECES, "not_an_ending"),
        (START, "not_an_ending"),
    ],
    ids=["two_pieces_one_side", "two_pieces_each", "three_pieces_one_side", "start"],
)
def test_a_position_is_an_ending_while_neither_side_has_more_than_two_pieces(fen: str, class_: str) -> None:
    assert ending(fen).class_ == class_


def test_a_position_that_is_not_an_ending_still_has_a_signature() -> None:
    """The material is still answered for a position that is not an ending."""
    answer = ending(THREE_PIECES)
    assert answer.class_ == "not_an_ending"
    assert answer.signature.text == "KQRNvK"
    assert answer.verdict == "unknown"


# ------------------------------------------------------------------ the class


@pytest.mark.parametrize(
    ("fen", "class_"),
    [
        (BARE_KINGS, "k_v_k"),
        (LONE_QUEEN, "kq_v_k"),
        (LONE_ROOK, "kr_v_k"),
        (TWO_BISHOPS, "kbb_v_k"),
        (TWO_DARK_BISHOPS, "kbb_v_k"),
        (BISHOP_AND_KNIGHT, "kbn_v_k"),
        (TWO_KNIGHTS, "knn_v_k"),
        (LONE_BISHOP, "kb_v_k"),
        (LONE_KNIGHT, "kn_v_k"),
        (CENTRE_PAWN, "kp_v_k"),
        (WRONG_BISHOP, "kbp_v_k"),
        (RIGHT_BISHOP, "kbp_v_k"),
        (QUEEN_V_QUEEN, "kq_v_kq"),
        (QUEEN_V_ROOK, "kq_v_kr"),
        (QUEEN_V_BISHOP, "kq_v_kb"),
        (QUEEN_V_KNIGHT, "kq_v_kn"),
        (QUEEN_V_PAWN, "kq_v_kp"),
        (QUEEN_V_TWO_MINORS, "kq_v_two_minors"),
        (ROOK_V_ROOK, "kr_v_kr"),
        (ROOK_V_BISHOP, "kr_v_kb"),
        (ROOK_V_KNIGHT, "kr_v_kn"),
        (ROOK_V_PAWN, "kr_v_kp"),
        (ROOK_V_TWO_MINORS, "kr_v_two_minors"),
        (ROOK_AND_PAWN, "krp_v_kr"),
        (TWO_BISHOPS_V_KNIGHT, "kbb_v_kn"),
        (BISHOP_V_KNIGHT, "kb_v_kn"),
        (SAME_COLOUR_BISHOPS, "kb_v_kb_same_colour"),
        (OPPOSITE_BISHOPS, "kb_v_kb_opposite_colour"),
        (KNIGHT_V_KNIGHT, "kn_v_kn"),
        (BISHOP_V_PAWN, "kb_v_kp"),
        (KNIGHT_V_PAWN, "kn_v_kp"),
        (PAWN_ENDING, "pawns"),
        (TWO_PAWNS, "pawns"),
        (QUEEN_AND_ROOK, "other"),
    ],
    ids=[
        "bare_kings",
        "lone_queen",
        "lone_rook",
        "two_bishops",
        "two_dark_bishops",
        "bishop_and_knight",
        "two_knights",
        "lone_bishop",
        "lone_knight",
        "centre_pawn",
        "wrong_bishop",
        "right_bishop",
        "queen_v_queen",
        "queen_v_rook",
        "queen_v_bishop",
        "queen_v_knight",
        "queen_v_pawn",
        "queen_v_two_minors",
        "rook_v_rook",
        "rook_v_bishop",
        "rook_v_knight",
        "rook_v_pawn",
        "rook_v_two_minors",
        "rook_and_pawn",
        "two_bishops_v_knight",
        "bishop_v_knight",
        "same_colour_bishops",
        "opposite_bishops",
        "knight_v_knight",
        "bishop_v_pawn",
        "knight_v_pawn",
        "pawn_ending",
        "two_pawns",
        "queen_and_rook",
    ],
)
def test_an_ending_is_classified_by_the_material_alone(fen: str, class_: str) -> None:
    assert ending(fen).class_ == class_


def test_the_class_is_the_same_whichever_side_is_stronger() -> None:
    """Which side holds the material never changes the class, only the verdict."""
    white = ending(QUEEN_V_ROOK)
    black = ending(BLACK_IS_STRONGER)
    assert white.class_ == black.class_
    assert (white.verdict, white.verdict.winner) == ("win", "w")
    assert (black.verdict, black.verdict.winner) == ("win", "b")


# ---------------------------------------------------------------- the verdict


@pytest.mark.parametrize(
    ("fen", "verdict", "winner"),
    [
        (BARE_KINGS, "draw", None),
        (LONE_QUEEN, "win", "w"),
        (LONE_ROOK, "win", "w"),
        (TWO_BISHOPS, "win", "w"),
        (TWO_DARK_BISHOPS, "draw", None),
        (BISHOP_AND_KNIGHT, "win", "w"),
        (TWO_KNIGHTS, "draw", None),
        (LONE_BISHOP, "draw", None),
        (LONE_KNIGHT, "draw", None),
        (CENTRE_PAWN, "usually_win", "w"),
        (RUNNING_PAWN, "win", "w"),
        (H_PAWN_CORNER, "draw", None),
        (A_PAWN_CORNER, "draw", None),
        (WRONG_BISHOP, "draw", None),
        (WRONG_BISHOP_OTHER_CORNER, "draw", None),
        (RIGHT_BISHOP, "usually_win", "w"),
        (QUEEN_V_QUEEN, "draw", None),
        (QUEEN_V_ROOK, "win", "w"),
        (QUEEN_V_PAWN, "usually_win", "w"),
        (QUEEN_V_TWO_MINORS, "unknown", None),
        (ROOK_V_ROOK, "draw", None),
        (ROOK_V_BISHOP, "usually_draw", "w"),
        (ROOK_V_KNIGHT, "usually_draw", "w"),
        (ROOK_AND_PAWN, "usually_win", "w"),
        (ROOK_AND_PAWN_HELD, "usually_draw", "w"),
        (TWO_BISHOPS_V_KNIGHT, "usually_win", "w"),
        (BISHOP_V_PAWN, "usually_draw", "b"),
        (SAME_COLOUR_BISHOPS, "draw", None),
        (PAWN_ENDING, "unknown", None),
        (BLACK_IS_STRONGER, "win", "b"),
    ],
    ids=[
        "bare_kings",
        "lone_queen",
        "lone_rook",
        "two_bishops",
        "two_dark_bishops",
        "bishop_and_knight",
        "two_knights",
        "lone_bishop",
        "lone_knight",
        "centre_pawn",
        "running_pawn",
        "h_pawn_corner",
        "a_pawn_corner",
        "wrong_bishop",
        "wrong_bishop_other_corner",
        "right_bishop",
        "queen_v_queen",
        "queen_v_rook",
        "queen_v_pawn",
        "queen_v_two_minors",
        "rook_v_rook",
        "rook_v_bishop",
        "rook_v_knight",
        "rook_and_pawn",
        "rook_and_pawn_held",
        "two_bishops_v_knight",
        "bishop_v_pawn",
        "same_colour_bishops",
        "pawn_ending",
        "black_is_stronger",
    ],
)
def test_the_verdict_is_theory_adjusted_by_what_the_position_shows(fen: str, verdict: str, winner: str | None) -> None:
    answer = ending(fen)
    assert answer.verdict == verdict
    assert answer.verdict.winner == winner


# -------------------------------------------------------------- the technique


@pytest.mark.parametrize(
    ("fen", "technique"),
    [
        (BARE_KINGS, "none"),
        (LONE_QUEEN, "box_method"),
        (LONE_ROOK, "box_method"),
        (TWO_BISHOPS, "two_bishop_mate"),
        (TWO_DARK_BISHOPS, "none"),
        (BISHOP_AND_KNIGHT, "bishop_knight_mate"),
        (CENTRE_PAWN, "key_squares"),
        (RUNNING_PAWN, "rule_of_the_square"),
        (H_PAWN_CORNER, "wrong_rook_pawn"),
        (A_PAWN_CORNER, "wrong_rook_pawn"),
        (WRONG_BISHOP, "wrong_bishop"),
        (RIGHT_BISHOP, "none"),
        (ROOK_AND_PAWN, "lucena"),
        (ROOK_AND_PAWN_HELD, "philidor"),
        (PAWN_ENDING, "opposition"),
        (QUEEN_V_ROOK, "none"),
    ],
    ids=[
        "bare_kings",
        "lone_queen",
        "lone_rook",
        "two_bishops",
        "two_dark_bishops",
        "bishop_and_knight",
        "centre_pawn",
        "running_pawn",
        "h_pawn_corner",
        "a_pawn_corner",
        "wrong_bishop",
        "right_bishop",
        "rook_and_pawn",
        "rook_and_pawn_held",
        "pawn_ending",
        "queen_v_rook",
    ],
)
def test_the_technique_is_the_method_the_ending_is_played_by(fen: str, technique: str) -> None:
    assert ending(fen).technique == technique


# --------------------------------------------------------------- the evidence


@pytest.mark.parametrize(
    ("fen", "pawn", "promotion", "rook_pawn", "steps"),
    [
        (CENTRE_PAWN, "e3", "e8", False, 5),
        (RUNNING_PAWN, "a2", "a8", True, 5),
        (ROOK_AND_PAWN, "d2", "d8", False, 5),
        (KINGS_IN_OPPOSITION, "e5", "e8", False, 3),
        (QUEEN_V_PAWN, "e4", "e1", False, 3),
    ],
    ids=["centre_pawn", "running_pawn", "rook_and_pawn", "kings_in_opposition", "queen_v_pawn"],
)
def test_a_pawn_race_names_the_pawn_and_the_run_it_has_left(
    fen: str, pawn: str, promotion: str, rook_pawn: bool, steps: int
) -> None:
    race = ending(fen).evidence.pawn
    assert race is not None
    assert race.pawn == pawn
    assert race.promotion == promotion
    assert race.rook_pawn == rook_pawn
    assert race.steps == steps


@pytest.mark.parametrize(
    ("fen", "inside_square", "attacker_in_front", "defender_in_front"),
    [
        (CENTRE_PAWN, True, False, True),
        (RUNNING_PAWN, False, False, False),
        (H_PAWN_CORNER, True, False, False),
        (KINGS_IN_OPPOSITION, True, True, True),
        (ROOK_AND_PAWN_HELD, True, False, True),
    ],
    ids=["centre_pawn", "running_pawn", "h_pawn_corner", "kings_in_opposition", "rook_and_pawn_held"],
)
def test_a_pawn_race_says_who_stands_where_in_the_race(
    fen: str, inside_square: bool, attacker_in_front: bool, defender_in_front: bool
) -> None:
    race = ending(fen).evidence.pawn
    assert race is not None
    assert race.defender_inside_square == inside_square
    assert race.attacker_in_front == attacker_in_front
    assert race.defender_in_front == defender_in_front


@pytest.mark.parametrize(
    "fen",
    [BARE_KINGS, PAWN_ENDING, TWO_PAWNS],
    ids=["bare_kings", "pawn_ending", "two_pawns"],
)
def test_a_position_without_exactly_one_pawn_has_no_pawn_race(fen: str) -> None:
    """More than one pawn, or none, leaves nothing one race can be read off."""
    assert ending(fen).evidence.pawn is None


@pytest.mark.parametrize(
    ("fen", "opposite_colours", "same_colour", "wrong_bishop"),
    [
        (TWO_BISHOPS, False, False, False),
        (TWO_DARK_BISHOPS, False, True, False),
        (SAME_COLOUR_BISHOPS, False, True, False),
        (OPPOSITE_BISHOPS, True, False, False),
        (WRONG_BISHOP, False, True, True),
        (WRONG_BISHOP_OTHER_CORNER, False, True, True),
        (RIGHT_BISHOP, False, True, False),
    ],
    ids=[
        "two_bishops",
        "two_dark_bishops",
        "same_colour_bishops",
        "opposite_bishops",
        "wrong_bishop",
        "wrong_bishop_other_corner",
        "right_bishop",
    ],
)
def test_the_bishops_say_which_squares_they_can_reach(
    fen: str, opposite_colours: bool, same_colour: bool, wrong_bishop: bool
) -> None:
    bishops = ending(fen).evidence.bishops
    assert bishops is not None
    assert bishops.opposite_colours == opposite_colours
    assert bishops.same_colour == same_colour
    assert bishops.wrong_bishop == wrong_bishop


@pytest.mark.parametrize(
    "fen",
    [BARE_KINGS, LONE_ROOK, PAWN_ENDING],
    ids=["bare_kings", "lone_rook", "pawn_ending"],
)
def test_a_position_without_a_bishop_says_nothing_about_bishops(fen: str) -> None:
    assert ending(fen).evidence.bishops is None


@pytest.mark.parametrize(
    ("fen", "opposition"),
    [(KINGS_IN_OPPOSITION, True), (CENTRE_PAWN, False), (BARE_KINGS, False)],
    ids=["kings_in_opposition", "centre_pawn", "bare_kings"],
)
def test_the_evidence_says_whether_the_kings_stand_in_opposition(fen: str, opposition: bool) -> None:
    assert ending(fen).evidence.opposition == opposition


# ------------------------------------------------------------------- the prose


@pytest.mark.parametrize("class_", endings.EndingClass.all(), ids=lambda value: value.name)
def test_every_class_has_a_sentence_of_its_own(class_: endings.EndingClass) -> None:
    assert class_.describe()
    assert class_.describe().endswith(".")


@pytest.mark.parametrize("technique", endings.EndingTechnique.all(), ids=lambda value: value.name)
def test_every_technique_has_a_sentence_of_its_own(technique: endings.EndingTechnique) -> None:
    assert technique.describe()
    assert technique.describe().endswith(".")


@pytest.mark.parametrize(
    "fen",
    [LONE_QUEEN, CENTRE_PAWN, ROOK_V_BISHOP, BARE_KINGS, PAWN_ENDING],
    ids=["win", "usually_win", "usually_draw", "draw", "unknown"],
)
def test_every_verdict_has_a_sentence_of_its_own(fen: str) -> None:
    verdict = ending(fen).verdict
    assert verdict.describe()
    assert verdict.describe().endswith(".")


def test_no_two_classes_share_a_name() -> None:
    """The catalogue holds each name once."""
    names = [class_.name for class_ in endings.EndingClass.all()]
    assert len(set(names)) == len(names)


def test_a_class_is_its_name_and_compares_equal_to_it() -> None:
    class_ = ending(ROOK_AND_PAWN).class_
    assert str(class_) == "krp_v_kr"
    assert class_.name == "krp_v_kr"
    assert class_ == "krp_v_kr"
    assert class_ != "kr_v_kr"
    assert {class_: "seen"}["krp_v_kr"] == "seen"


def test_a_class_says_in_one_sentence_what_the_ending_is() -> None:
    assert ending(BARE_KINGS).class_.describe() == "Only the two kings are left, so nothing can be won."


def test_a_verdict_names_the_side_it_gives_the_ending_to() -> None:
    verdict = ending(LONE_QUEEN).verdict
    assert verdict.describe() == "White wins this ending by force, against any defence."
    assert verdict.winner == "w"
    assert ending(BARE_KINGS).verdict.winner is None
    assert ending(PAWN_ENDING).verdict.name == "unknown"


def test_a_signature_says_in_one_sentence_what_each_side_has() -> None:
    assert ending(ROOK_AND_PAWN).signature.describe() == (
        "The material is KRPvKR: White has a rook and a pawn, Black has a rook."
    )
    assert ending(BARE_KINGS).signature.describe() == (
        "The material is KvK: White has nothing besides its king, Black has nothing besides its king."
    )


def test_evidence_that_applies_to_nothing_says_so() -> None:
    assert ending(BARE_KINGS).evidence.describe() == (
        "Nothing in this position changes what theory says about the ending."
    )


def test_an_ending_describes_itself_from_its_parts() -> None:
    """The whole answer reads as the material, the ending, the result and the method."""
    answer = ending(LONE_ROOK)
    prose = answer.describe()
    assert prose.startswith(answer.signature.describe())
    assert answer.class_.describe() in prose
    assert answer.verdict.describe() in prose
    assert prose.endswith(answer.technique.describe())


def test_a_position_that_is_not_an_ending_describes_only_its_material() -> None:
    """A position that is not an ending is answered with its material and no theory."""
    answer = ending(START)
    assert answer.describe() == (
        f"{answer.signature.describe()} Too much material is left for this to be an ending at all."
    )


# ------------------------------------------------------------------ surfaces


def test_a_position_and_a_game_answer_the_same_ending() -> None:
    position = esca.Position.from_fen(ROOK_AND_PAWN)
    game = esca.Game.from_fen(ROOK_AND_PAWN)
    assert position.ending().describe() == endings.classify(position).describe()
    assert game.ending().describe() == endings.classify(position).describe()
