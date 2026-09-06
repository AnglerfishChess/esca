"""The `explain` layer: the evidence behind a rules answer.

Every expectation is read off the diagram above the named position, from the
definitions in `docs/esca-api.md` §12. The cases mirror `tests/explain.rs`.
"""

from __future__ import annotations

import esca
import pytest
from esca import explain

#: Both sides may castle either way and the back ranks are otherwise bare.
CLEAR_BACK_RANK = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1"

#: The same, with White's own queen on d1 and knight on g1 in the way.
QUEEN_AND_KNIGHT = "r3k2r/8/8/8/8/8/8/R2QK1NR w KQkq - 0 1"

#: A bishop on b5 covers f1, which the king would cross.
BISHOP_COVERS_F1 = "4k3/8/8/1b6/8/8/8/4K2R w K - 0 1"

#: King and rooks stand ready, but no right survives.
NO_RIGHTS = "4k3/8/8/8/8/8/8/R3K2R w - - 0 1"

#: Every reason at once: the king is checked from e8, a6 covers f1, and
#: White's own knight sits on g1.
EVERY_REASON = "k3r3/8/b7/8/8/8/8/4K1NR w K - 0 1"

#: Chess960 with the kings on b1 and b8 and the rooks on the corners.
NINE_SIXTY_CLEAR = "rk5r/8/8/8/8/8/8/RK5R w AHah - 0 1"

#: The same with a third black rook on e8, which the short castling crosses
#: and the long one does not.
NINE_SIXTY_E_FILE = "rk2r2r/8/8/8/8/8/8/RK5R w AHah - 0 1"

#: Chess960 with the king already on g1: castling short moves only the rook.
NINE_SIXTY_KING_STAYS = "r5kr/8/8/8/8/8/8/R5KR w AHah - 0 1"

#: An untouched Chess960 array, king on b1: five of its own units stand on
#: the short castling's path.
NINE_SIXTY_ARRAY = "rkbbnqnr/pppppppp/8/8/8/8/PPPPPPPP/RKBBNQNR w AHah - 0 1"

#: The d-pawn has just run past e5 and nothing forbids the capture.
EP_PLAIN = "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1"

#: No pawn has just moved two squares.
EP_NONE = "4k3/8/8/3pP3/8/8/8/4K3 w - - 0 1"

#: A target with no pawn of the moving side beside it.
EP_NO_TAKER = "4k3/8/8/3p4/8/8/8/4K3 w - d6 0 1"

#: The rank pin: both pawns leave rank 5 at once and a5 checks h5.
EP_RANK_PIN = "4k3/8/8/r2pP2K/8/8/8/8 w - d6 0 1"

#: The e5 pawn is pinned on the b2-g7 diagonal, which d6 is not on.
EP_PINNED = "4k3/6K1/8/3pP3/8/8/1b6/8 w - d6 0 1"

#: The e5 pawn is pinned on the c7-h2 diagonal, which d6 is on.
EP_PIN_ALONG_RAY = "4k3/2b5/8/3pP3/8/8/7K/8 w - d6 0 1"

#: The pawn that ran past uncovered the a7 rook, and taking on d6 leaves the
#: check standing.
EP_IN_CHECK = "8/r6K/8/3pP3/8/8/8/k7 w - d6 0 1"

#: The pawn that ran past gives check, and the capture takes it off.
EP_ANSWERS_CHECK = "7k/8/8/2Pp4/4K3/8/8/8 w - d6 0 1"

#: Two pawns may take: c5 freely, e5 only off its pin.
EP_TWO_TAKERS = "4k3/6K1/8/2PpP3/8/8/1b6/8 w - d6 0 1"

#: A rook on e2 and a knight on f3 check e1 together.
DOUBLE_CHECK = "4k3/8/8/8/8/5n2/4r3/4K3 w - - 0 1"

#: Three white units bear on e5 and two black ones defend it.
CROWD = "4rk2/8/3p4/4p3/3P4/5N2/8/4RK2 w - - 0 1"

#: b4 pins the d2 knight and e8 pins the e4 bishop, both against e1.
TWO_PINS = "4r2k/8/8/8/1b2B3/8/3N4/4K3 w - - 0 1"

#: a1 attacks the black king on a5 with the a8 rook behind it.
SKEWERED_KING = "r7/8/8/k7/8/8/8/R3K3 b - - 0 1"

#: The a1 bishop attacks the d4 queen with the f6 rook behind it.
SKEWERED_QUEEN = "4k3/8/5r2/8/3q4/8/8/B3K3 w - - 0 1"

#: Kings and a blocked pawn each: a king can triangulate, a king cannot
#: return in one move, so five plies restore the placement.
TRIANGULATION = "3k4/p7/8/8/8/8/P7/3K4 w - - 0 1"

#: A white pawn one step from a double step, with a black pawn waiting to
#: take it en passant.
EN_PASSANT_RIGHTS = "4k3/8/8/8/3p4/8/4P3/4K3 w - - 0 1"

#: One ply from the fifty-move claim.
CLOCK_AT_99 = "4k2r/8/8/8/8/8/8/R3K3 w - - 99 60"

#: One ply from the automatic draw.
CLOCK_AT_149 = "4k2r/8/8/8/8/8/8/R3K3 w - - 149 90"

#: Queen and king shut the black king in without checking it.
SMOTHERED_STALEMATE = "7k/5Q2/6K1/8/8/8/8/8 b - - 0 1"

#: Stalemate with nothing left that could ever mate.
BISHOP_STALEMATE = "k7/B7/K7/8/8/8/8/8 b - - 0 1"

#: Stalemate where the a7 pawn is blocked and the b7 knight is pinned.
PINNED_AND_BLOCKED = "k7/pn6/N7/8/4B3/8/8/6K1 b - - 0 1"

#: Mate delivered on the hundred-and-fiftieth quiet ply: the game is over,
#: so no draw stands.
MATE_ON_THE_CLOCK = "7k/6Q1/6K1/8/8/8/8/8 b - - 150 90"

#: Kings only.
BARE_KINGS = "4k3/8/8/8/8/8/8/4K3 w - - 0 1"

#: One bishop besides the kings.
ONE_BISHOP = "4k3/8/8/8/8/8/8/3BK3 w - - 0 1"

#: One knight besides the kings.
ONE_KNIGHT = "4k3/8/8/8/8/8/8/3NK3 w - - 0 1"

#: A bishop each, both on light squares.
SAME_COLOUR_BISHOPS = "4k3/8/4b3/8/8/8/8/3BK3 w - - 0 1"

#: A bishop each, on opposite square colours: a helpmate exists.
OPPOSITE_BISHOPS = "4k3/8/3b4/8/8/8/8/3BK3 w - - 0 1"

#: Knights out and back: four plies return to the same position.
SHUFFLE = "Nf3 Nf6 Ng1 Ng8"

#: The untouched array.
START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"


def position(fen: str) -> esca.Position:
    """The position `fen` describes."""
    return esca.Position.from_fen(fen)


def game_from(fen: str, moves: str) -> esca.Game:
    """A classic game from `fen` with `moves` played, in SAN."""
    game = esca.Game.from_fen(fen)
    for text in moves.split():
        game.play_san(text)
    return game


def game_of(moves: str) -> esca.Game:
    """A classic game from the standard array with `moves` played, in SAN."""
    return game_from(START, moves)


def move_named(game: esca.Game, san: str) -> esca.Move:
    """The legal move of `game` that SAN writes as `san`."""
    for mv in game.legal_moves():
        if game.move_to_san(mv) == san:
            return mv
    raise AssertionError(f"{san} is not a legal move of {game.position.fen}")


def covers(pairs: list[tuple[str, str]]) -> list[tuple[str, set[str]]]:
    """Square-and-attackers pairs written as `[("f1", "a6 b5")]`."""
    return [(square, set(by.split())) for square, by in pairs]


def named(items: list[tuple[str, esca.SquareSet]]) -> list[tuple[str, set[str]]]:
    """A list of square-and-set pairs, its sets as sets of square names."""
    return [(square, set(square_set)) for square, square_set in items]


def ep_capture(fen: str, origin: str) -> explain.EpCapture:
    """One en-passant capture of the position `fen` describes."""
    for capture in position(fen).en_passant_status().captures:
        if capture.origin == origin:
            return capture
    raise AssertionError(f"{origin} has no en-passant capture in {fen}")


def ep_offers(fen: str) -> list[tuple[str, bool]]:
    """The origin and legality of every en-passant capture on offer."""
    return [(capture.origin, capture.legal) for capture in position(fen).en_passant_status().captures]


def automatic_kinds(status: explain.DrawStatus) -> list[str]:
    """The cases of a draw status, in the order it lists them."""
    return [draw.kind for draw in status.automatic]


def claimable_kinds(claims: list[explain.ClaimableDraw]) -> list[str]:
    return [claim.kind for claim in claims]


def material_config(status: explain.DrawStatus) -> str | None:
    """The material configuration a draw status names, if it names one."""
    for draw in status.automatic:
        if draw.kind == "insufficient_material":
            return draw.material
    return None


def stalemate_detail(status: explain.DrawStatus) -> explain.StalemateDetail:
    """The stalemate detail a draw status carries."""
    for draw in status.automatic:
        if draw.kind == "stalemate":
            return draw.stalemate
    raise AssertionError("the position is not a stalemate")


def stuck_kinds(detail: explain.StalemateDetail) -> list[tuple[str, str]]:
    """Every stuck unit and the case that holds it."""
    return [(square, stuck.kind) for square, stuck in detail.stuck_units]


# ------------------------------------------------------------------ castling


@pytest.mark.parametrize(
    ("fen", "colour", "wing", "allowed"),
    [
        (CLEAR_BACK_RANK, "w", "short", True),
        (CLEAR_BACK_RANK, "w", "long", True),
        (CLEAR_BACK_RANK, "b", "short", True),
        (QUEEN_AND_KNIGHT, "w", "short", False),
        (QUEEN_AND_KNIGHT, "w", "long", False),
        (BISHOP_COVERS_F1, "w", "short", False),
        (NO_RIGHTS, "w", "short", False),
        (EVERY_REASON, "w", "short", False),
        (NINE_SIXTY_CLEAR, "w", "short", True),
        (NINE_SIXTY_CLEAR, "w", "long", True),
        (NINE_SIXTY_E_FILE, "w", "short", False),
        (NINE_SIXTY_E_FILE, "w", "long", True),
        (NINE_SIXTY_KING_STAYS, "w", "short", True),
        (NINE_SIXTY_ARRAY, "w", "short", False),
    ],
    ids=[
        "clear_short",
        "clear_long",
        "clear_for_the_side_not_to_move",
        "knight_on_g1",
        "queen_on_d1",
        "covered_f1",
        "no_right",
        "every_reason",
        "nine_sixty_short",
        "nine_sixty_long",
        "nine_sixty_crosses_e1",
        "nine_sixty_long_avoids_e1",
        "nine_sixty_king_stays",
        "nine_sixty_array",
    ],
)
def test_a_castling_is_allowed_when_nothing_stands_in_its_way(fen: str, colour: str, wing: str, allowed: bool) -> None:
    assert position(fen).castling(colour, wing).allowed is allowed


@pytest.mark.parametrize(
    ("fen", "colour", "wing", "blocked"),
    [
        (CLEAR_BACK_RANK, "w", "short", ""),
        (QUEEN_AND_KNIGHT, "w", "short", "g1"),
        (QUEEN_AND_KNIGHT, "w", "long", "d1"),
        (QUEEN_AND_KNIGHT, "b", "long", ""),
        (NO_RIGHTS, "w", "short", ""),
        (EVERY_REASON, "w", "short", "g1"),
        (NINE_SIXTY_CLEAR, "w", "short", ""),
        (NINE_SIXTY_KING_STAYS, "w", "short", ""),
        (NINE_SIXTY_ARRAY, "w", "short", "c1 d1 e1 f1 g1"),
        (NINE_SIXTY_ARRAY, "w", "long", "c1 d1"),
    ],
    ids=[
        "clear_short",
        "knight_on_g1",
        "queen_on_d1",
        "black_long_is_clear",
        "no_right",
        "every_reason",
        "nine_sixty_short",
        "nine_sixty_king_stays",
        "nine_sixty_array",
        "nine_sixty_array_long",
    ],
)
def test_a_castling_names_the_units_standing_on_its_path(fen: str, colour: str, wing: str, blocked: str) -> None:
    assert set(position(fen).castling(colour, wing).path_blocked) == set(blocked.split())


@pytest.mark.parametrize(
    ("fen", "colour", "wing", "attacked"),
    [
        (CLEAR_BACK_RANK, "w", "short", []),
        (BISHOP_COVERS_F1, "w", "short", [("f1", "b5")]),
        (EVERY_REASON, "w", "short", [("f1", "a6")]),
        (NINE_SIXTY_E_FILE, "w", "short", [("e1", "e8")]),
        (NINE_SIXTY_E_FILE, "w", "long", []),
        (NINE_SIXTY_KING_STAYS, "w", "short", []),
    ],
    ids=[
        "clear_short",
        "covered_f1",
        "every_reason",
        "nine_sixty_crosses_e1",
        "nine_sixty_long_avoids_e1",
        "nine_sixty_king_stays",
    ],
)
def test_a_castling_names_every_covered_square_the_king_would_cross(
    fen: str, colour: str, wing: str, attacked: list[tuple[str, str]]
) -> None:
    assert named(position(fen).castling(colour, wing).path_attacked) == covers(attacked)


@pytest.mark.parametrize(
    ("fen", "colour", "wing", "right", "rook_present", "check_by"),
    [
        (CLEAR_BACK_RANK, "w", "short", True, True, ""),
        (NO_RIGHTS, "w", "short", False, False, ""),
        (BISHOP_COVERS_F1, "b", "short", False, False, ""),
        (EVERY_REASON, "w", "short", True, True, "e8"),
        (NINE_SIXTY_CLEAR, "w", "short", True, True, ""),
    ],
    ids=[
        "clear_short",
        "no_right",
        "black_has_no_right",
        "every_reason",
        "nine_sixty_short",
    ],
)
def test_a_castling_names_the_right_the_rook_and_the_check(
    fen: str, colour: str, wing: str, right: bool, rook_present: bool, check_by: str
) -> None:
    castling = position(fen).castling(colour, wing)
    assert castling.right is right
    assert castling.rook_present is rook_present
    assert set(castling.king_in_check_by) == set(check_by.split())


def test_a_castling_reports_every_reason_that_applies() -> None:
    """Three reasons hold at once, and all three are answered."""
    castling = position(EVERY_REASON).castling("w", "short")
    assert set(castling.king_in_check_by) == {"e8"}
    assert named(castling.path_attacked) == covers([("f1", "a6")])
    assert set(castling.path_blocked) == {"g1"}
    assert castling.right
    assert castling.rook_present
    assert not castling.allowed


@pytest.mark.parametrize(
    ("variant", "fen"),
    [
        ("chess", CLEAR_BACK_RANK),
        ("chess", QUEEN_AND_KNIGHT),
        ("chess", BISHOP_COVERS_F1),
        ("chess", NO_RIGHTS),
        ("chess", EVERY_REASON),
        ("chess960", NINE_SIXTY_CLEAR),
        ("chess960", NINE_SIXTY_E_FILE),
        ("chess960", NINE_SIXTY_KING_STAYS),
        ("chess960", NINE_SIXTY_ARRAY),
    ],
    ids=[
        "clear",
        "queen_and_knight",
        "covered_f1",
        "no_rights",
        "every_reason",
        "nine_sixty_clear",
        "nine_sixty_e_file",
        "nine_sixty_king_stays",
        "nine_sixty_array",
    ],
)
def test_allowed_is_what_the_move_generator_says_for_the_side_to_move(variant: str, fen: str) -> None:
    game = esca.Game.from_fen(fen, variant=esca.Variant.named(variant))
    colour = game.position.side_to_move
    for wing in ("short", "long"):
        generated = any(
            mv.is_castling and (mv.destination[0] > mv.origin[0]) == (wing == "short") for mv in game.legal_moves()
        )
        assert game.position.castling(colour, wing).allowed is generated, f"{fen} {wing}"


# ---------------------------------------------------------------- en passant


@pytest.mark.parametrize(
    ("fen", "target"),
    [
        (EP_PLAIN, "d6"),
        (EP_NONE, None),
        (EP_NO_TAKER, "d6"),
        (EP_RANK_PIN, "d6"),
    ],
    ids=["plain", "none", "no_taker", "rank_pin"],
)
def test_en_passant_status_names_the_square_a_pawn_skipped(fen: str, target: str | None) -> None:
    assert position(fen).en_passant_status().target == target


@pytest.mark.parametrize(
    ("fen", "offers"),
    [
        (EP_PLAIN, [("e5", True)]),
        (EP_NONE, []),
        (EP_NO_TAKER, []),
        (EP_RANK_PIN, [("e5", False)]),
        (EP_PINNED, [("e5", False)]),
        (EP_PIN_ALONG_RAY, [("e5", True)]),
        (EP_IN_CHECK, [("e5", False)]),
        (EP_ANSWERS_CHECK, [("c5", True)]),
        (EP_TWO_TAKERS, [("c5", True), ("e5", False)]),
    ],
    ids=[
        "plain",
        "none",
        "no_taker",
        "rank_pin",
        "pinned_off_the_ray",
        "pinned_along_the_ray",
        "in_check",
        "answers_the_check",
        "two_takers",
    ],
)
def test_en_passant_status_names_every_pawn_that_could_take(fen: str, offers: list[tuple[str, bool]]) -> None:
    assert ep_offers(fen) == offers


@pytest.mark.parametrize(
    ("fen", "origin", "kind"),
    [
        (EP_PLAIN, "e5", None),
        (EP_PIN_ALONG_RAY, "e5", None),
        (EP_RANK_PIN, "e5", "exposes_king"),
        (EP_PINNED, "e5", "pinned"),
        (EP_IN_CHECK, "e5", "in_check"),
        (EP_TWO_TAKERS, "e5", "pinned"),
    ],
    ids=[
        "plain",
        "pinned_along_the_ray",
        "rank_pin",
        "pinned_off_the_ray",
        "in_check",
        "two_takers",
    ],
)
def test_an_illegal_en_passant_names_what_forbids_it(fen: str, origin: str, kind: str | None) -> None:
    obstacle = ep_capture(fen, origin).forbidden_by
    assert (obstacle.kind if obstacle is not None else None) == kind


def test_the_rank_pin_names_the_slider_the_two_pawns_hide() -> None:
    """The rank pin binds neither pawn on its own, so it names the slider."""
    obstacle = ep_capture(EP_RANK_PIN, "e5").forbidden_by
    assert obstacle.kind == "exposes_king"
    assert obstacle.attacker == "a5"


def test_a_pinned_pawn_names_its_pinner_and_the_ray_it_may_not_leave() -> None:
    obstacle = ep_capture(EP_PINNED, "e5").forbidden_by
    assert obstacle.kind == "pinned"
    assert obstacle.pinner == "b2"
    assert set(obstacle.ray) == {"c3", "d4", "e5", "f6"}


def test_an_en_passant_that_leaves_a_check_standing_names_the_checkers() -> None:
    obstacle = ep_capture(EP_IN_CHECK, "e5").forbidden_by
    assert obstacle.kind == "in_check"
    assert set(obstacle.by) == {"a7"}


# --------------------------------------------------- checks, attacks and rays


@pytest.mark.parametrize(
    ("fen", "checkers"),
    [
        (CLEAR_BACK_RANK, ""),
        (DOUBLE_CHECK, "e2 f3"),
        (EP_ANSWERS_CHECK, "d5"),
        (EP_IN_CHECK, "a7"),
    ],
    ids=["none", "double_check", "a_pawn_that_ran_past", "a_rook_along_the_rank"],
)
def test_checkers_are_the_units_giving_check_to_the_side_to_move(fen: str, checkers: str) -> None:
    assert set(position(fen).checkers()) == set(checkers.split())


@pytest.mark.parametrize(
    ("fen", "square", "colour", "attackers"),
    [
        (CROWD, "e5", "w", "d4 e1 f3"),
        (CROWD, "e5", "b", "d6 e8"),
        (CROWD, "d4", "w", "f3"),
        (CROWD, "d4", "b", "e5"),
        (CROWD, "a1", "w", "e1"),
    ],
    ids=["white_on_e5", "black_on_e5", "white_on_d4", "black_on_d4", "an_empty_square"],
)
def test_attackers_are_the_units_of_a_colour_that_bear_on_a_square(
    fen: str, square: str, colour: str, attackers: str
) -> None:
    assert set(position(fen).attackers(square, colour)) == set(attackers.split())


@pytest.mark.parametrize(
    ("origin", "destination", "expected"),
    [
        ("a1", "d4", "b2 c3"),
        ("a1", "a4", "a2 a3"),
        ("a1", "d1", "b1 c1"),
        ("h1", "a8", "b7 c6 d5 e4 f3 g2"),
        ("e4", "e5", ""),
        ("a1", "b3", ""),
        ("e4", "e4", ""),
    ],
    ids=[
        "a_diagonal",
        "a_file",
        "a_rank",
        "the_long_diagonal",
        "adjacent",
        "unaligned",
        "itself",
    ],
)
def test_between_is_the_squares_two_squares_share_a_line_through(origin: str, destination: str, expected: str) -> None:
    assert set(position(CLEAR_BACK_RANK).between(origin, destination)) == set(expected.split())


@pytest.mark.parametrize(
    ("fen", "colour", "expected"),
    [
        (TWO_PINS, "w", ["d2 b4 e1", "e4 e8 e1"]),
        (TWO_PINS, "b", []),
        (CLEAR_BACK_RANK, "w", []),
    ],
    ids=["two_pins", "nothing_pinned", "a_bare_board"],
)
def test_a_pin_names_the_unit_the_pinner_and_the_king_behind_it(fen: str, colour: str, expected: list[str]) -> None:
    pins = [f"{pin.pinned} {pin.pinner} {pin.king}" for pin in position(fen).pins(colour)]
    assert pins == expected


def test_a_pin_carries_the_ray_the_pinned_unit_may_not_leave() -> None:
    pins = position(TWO_PINS).pins("w")
    assert set(pins[0].ray) == {"c3", "d2"}
    assert set(pins[1].ray) == {"e2", "e3", "e4", "e5", "e6", "e7"}


@pytest.mark.parametrize(
    ("fen", "colour", "expected"),
    [
        (SKEWERED_KING, "b", ["a1 a5 a8"]),
        (SKEWERED_QUEEN, "b", ["a1 d4 f6"]),
        (SKEWERED_KING, "w", []),
        (TWO_PINS, "w", []),
    ],
    ids=[
        "a_king_in_front",
        "a_queen_in_front",
        "the_attacking_side",
        "a_pin_is_not_a_skewer",
    ],
)
def test_a_skewer_names_the_attacker_the_front_unit_and_what_stands_behind(
    fen: str, colour: str, expected: list[str]
) -> None:
    skewers = [f"{skewer.attacker} {skewer.front} {skewer.behind}" for skewer in position(fen).skewers(colour)]
    assert skewers == expected


def test_a_skewer_carries_the_ray_its_two_units_stand_on() -> None:
    skewers = position(SKEWERED_QUEEN).skewers("b")
    assert set(skewers[0].ray) == {"b2", "c3", "d4", "e5"}


# ----------------------------------------------------------------- repetition


@pytest.mark.parametrize(
    ("moves", "count", "plies"),
    [
        ("", 1, [0]),
        ("Nf3 Nf6 Ng1 Ng8", 2, [0, 4]),
        ("Nf3 Nf6 Ng1 Ng8 Nf3 Nf6 Ng1 Ng8", 3, [0, 4, 8]),
        ("Nf3 Nf6 Ng1", 1, [3]),
    ],
    ids=["a_fresh_game", "one_shuffle", "two_shuffles", "mid_shuffle"],
)
def test_a_repetition_lists_every_ply_the_position_has_stood_at(moves: str, count: int, plies: list[int]) -> None:
    repetition = game_of(moves).repetition_status()
    assert repetition.count == count
    assert repetition.plies == plies


def test_a_repetition_starts_over_when_a_castling_right_is_spent() -> None:
    """The rook leaves and comes back: the same placement, one right poorer."""
    moves = "Rhg1 Rhg8 Rh1 Rh8 Rhg1 Rhg8 Rh1 Rh8"
    repetition = game_from(CLEAR_BACK_RANK, moves).repetition_status()
    assert repetition.count == 2
    assert repetition.plies == [4, 8]
    assert [(miss.ply, miss.differs) for miss in repetition.near_misses] == [(0, ["castling_rights"])]


@pytest.mark.parametrize(
    ("fen", "moves", "differs"),
    [
        (START, "Nf3 Nf6 Ng1 Ng8", []),
        (CLEAR_BACK_RANK, "Rhg1 Rhg8 Rh1 Rh8 Rhg1 Rhg8 Rh1 Rh8", ["castling_rights"]),
        (EN_PASSANT_RIGHTS, "e4 Kd8 Kd2 Ke8 Ke1", ["en_passant"]),
        (TRIANGULATION, "Kc1 Ke8 Kc2 Kd8 Kd1", ["side_to_move"]),
    ],
    ids=["none", "castling_rights", "en_passant", "side_to_move"],
)
def test_a_near_miss_says_what_keeps_it_from_counting(fen: str, moves: str, differs: list[str]) -> None:
    repetition = game_from(fen, moves).repetition_status()
    found = [miss.differs for miss in repetition.near_misses]
    assert found == ([differs] if differs else [])


@pytest.mark.parametrize(
    ("fen", "moves", "ply"),
    [
        (EN_PASSANT_RIGHTS, "e4 Kd8 Kd2 Ke8 Ke1", 1),
        (TRIANGULATION, "Kc1 Ke8 Kc2 Kd8 Kd1", 0),
    ],
    ids=["en_passant", "side_to_move"],
)
def test_a_near_miss_names_the_ply_it_stood_at(fen: str, moves: str, ply: int) -> None:
    repetition = game_from(fen, moves).repetition_status()
    assert repetition.near_misses[0].ply == ply


# ----------------------------------------------------------------- fifty move


@pytest.mark.parametrize(
    ("moves", "clock", "to_claim", "to_automatic"),
    [
        ("", 0, 100, 150),
        ("e4", 0, 100, 150),
        ("e4 e5 Nf3", 1, 99, 149),
        ("e4 e5 Nf3 Nc6 Bb5", 3, 97, 147),
    ],
    ids=["a_fresh_game", "a_pawn_move", "one_quiet_ply", "three_quiet_plies"],
)
def test_the_clock_counts_down_to_the_claim_and_to_the_automatic_draw(
    moves: str, clock: int, to_claim: int, to_automatic: int
) -> None:
    fifty = game_of(moves).fifty_move_status()
    assert fifty.clock == clock
    assert fifty.plies_to_claim == to_claim
    assert fifty.plies_to_automatic == to_automatic


@pytest.mark.parametrize(
    ("fen", "moves", "clock", "to_claim", "to_automatic"),
    [
        (CLOCK_AT_99, "Kd1", 100, 0, 50),
        (CLOCK_AT_149, "Kd1", 150, 0, 0),
    ],
    ids=["at_the_claim", "at_the_automatic_draw"],
)
def test_a_clock_past_a_threshold_counts_down_no_further(
    fen: str, moves: str, clock: int, to_claim: int, to_automatic: int
) -> None:
    fifty = game_from(fen, moves).fifty_move_status()
    assert fifty.clock == clock
    assert fifty.plies_to_claim == to_claim
    assert fifty.plies_to_automatic == to_automatic


@pytest.mark.parametrize(
    ("moves", "reset"),
    [
        ("", None),
        ("e4", (1, "pawn_move")),
        ("e4 e5 Nf3", (2, "pawn_move")),
        ("e4 d5 exd5", (3, "capture")),
        ("e4 d5 exd5 Nf6 Nf3", (3, "capture")),
    ],
    ids=[
        "nothing_played",
        "a_pawn_move",
        "a_pawn_move_answered",
        "a_capture",
        "a_capture_then_quiet_plies",
    ],
)
def test_the_last_reset_names_the_move_that_cleared_the_clock(moves: str, reset: tuple[int, str] | None) -> None:
    last = game_of(moves).fifty_move_status().last_reset
    assert (None if last is None else (last.ply, last.kind)) == reset


def test_a_clock_a_game_started_with_has_no_reset() -> None:
    """A game that starts mid-clock has no reset of its own to point at."""
    fifty = game_from(CLOCK_AT_99, "").fifty_move_status()
    assert fifty.clock == 99
    assert fifty.last_reset is None


# ---------------------------------------------------------------- draw status


@pytest.mark.parametrize(
    ("fen", "moves", "automatic", "claimable"),
    [
        (START, "", [], []),
        (OPPOSITE_BISHOPS, "", [], []),
        (BARE_KINGS, "", ["insufficient_material"], []),
        (ONE_BISHOP, "", ["insufficient_material"], []),
        (ONE_KNIGHT, "", ["insufficient_material"], []),
        (SAME_COLOUR_BISHOPS, "", ["insufficient_material"], []),
        (SMOTHERED_STALEMATE, "", ["stalemate"], []),
        (BISHOP_STALEMATE, "", ["stalemate", "insufficient_material"], []),
        (CLOCK_AT_99, "Kd1", [], ["fifty_moves"]),
        (CLOCK_AT_149, "Kd1", ["seventy_five_moves"], ["fifty_moves"]),
        (MATE_ON_THE_CLOCK, "", [], []),
    ],
    ids=[
        "a_fresh_game",
        "a_playable_game",
        "bare_kings",
        "one_bishop",
        "one_knight",
        "same_colour_bishops",
        "smothered_stalemate",
        "stalemate_with_nothing_left",
        "at_the_fifty_move_claim",
        "at_the_automatic_draw",
        "checkmate_ends_it_first",
    ],
)
def test_a_draw_status_lists_every_reason_that_applies(
    fen: str, moves: str, automatic: list[str], claimable: list[str]
) -> None:
    status = game_from(fen, moves).draw_status()
    assert automatic_kinds(status) == automatic
    assert claimable_kinds(status.claimable) == claimable


@pytest.mark.parametrize(
    ("shuffles", "automatic", "claimable"),
    [
        (2, [], ["threefold"]),
        (4, ["fivefold"], ["threefold"]),
    ],
    ids=["threefold", "fivefold"],
)
def test_a_repeated_position_is_claimable_before_it_is_automatic(
    shuffles: int, automatic: list[str], claimable: list[str]
) -> None:
    status = game_of(" ".join([SHUFFLE] * shuffles)).draw_status()
    assert automatic_kinds(status) == automatic
    assert claimable_kinds(status.claimable) == claimable


@pytest.mark.parametrize(
    ("fen", "config"),
    [
        (BARE_KINGS, "k_v_k"),
        (ONE_BISHOP, "kb_v_k"),
        (ONE_KNIGHT, "kn_v_k"),
        (SAME_COLOUR_BISHOPS, "kb_v_kb_same_colour"),
        (OPPOSITE_BISHOPS, None),
    ],
    ids=[
        "bare_kings",
        "one_bishop",
        "one_knight",
        "same_colour_bishops",
        "opposite_bishops",
    ],
)
def test_insufficient_material_names_the_configuration(fen: str, config: str | None) -> None:
    assert material_config(game_from(fen, "").draw_status()) == config


@pytest.mark.parametrize(
    ("fen", "king", "escapes"),
    [
        (SMOTHERED_STALEMATE, "h8", [("g7", "f7 g6"), ("h7", "f7 g6"), ("g8", "f7")]),
        (BISHOP_STALEMATE, "a8", [("a7", "a6"), ("b7", "a6"), ("b8", "a7")]),
        (PINNED_AND_BLOCKED, "a8", [("b8", "a6")]),
    ],
    ids=["smothered", "bishop", "pinned_and_blocked"],
)
def test_a_stalemate_names_the_escape_squares_and_who_covers_them(
    fen: str, king: str, escapes: list[tuple[str, str]]
) -> None:
    detail = stalemate_detail(game_from(fen, "").draw_status())
    assert detail.king == king
    assert named(detail.escape_squares) == covers(escapes)


@pytest.mark.parametrize(
    ("fen", "stuck"),
    [
        (SMOTHERED_STALEMATE, []),
        (PINNED_AND_BLOCKED, [("a7", "blocked"), ("b7", "pinned")]),
    ],
    ids=["nothing_else_left", "a_pawn_and_a_knight"],
)
def test_a_stalemate_says_what_holds_every_other_unit(fen: str, stuck: list[tuple[str, str]]) -> None:
    detail = stalemate_detail(game_from(fen, "").draw_status())
    assert stuck_kinds(detail) == stuck


def test_a_stuck_pinned_unit_names_its_pinner_and_the_ray() -> None:
    detail = stalemate_detail(game_from(PINNED_AND_BLOCKED, "").draw_status())
    held, stuck = detail.stuck_units[1]
    assert held == "b7"
    assert stuck.kind == "pinned"
    assert stuck.pinner == "e4"
    assert set(stuck.ray) == {"b7", "c6", "d5"}


def test_a_claimable_draw_carries_the_evidence_for_the_claim() -> None:
    """A claim carries the count or the clock that earns it."""
    threefold = game_of(" ".join([SHUFFLE] * 2)).draw_status().claimable[0]
    assert threefold.kind == "threefold"
    assert threefold.repetition.count == 3
    assert threefold.repetition.plies == [0, 4, 8]

    fifty = game_from(CLOCK_AT_99, "Kd1").draw_status().claimable[0]
    assert fifty.kind == "fifty_moves"
    assert fifty.fifty_move.clock == 100


# ---------------------------------------------------------------- claims after


@pytest.mark.parametrize(
    ("moves", "next_move", "claims"),
    [
        ("Nf3 Nf6 Ng1 Ng8 Nf3 Nf6 Ng1", "Ng8", ["threefold"]),
        ("Nf3 Nf6 Ng1 Ng8 Nf3 Nf6 Ng1", "e5", []),
        ("Nf3 Nf6 Ng1", "Ng8", []),
    ],
    ids=["a_third_occurrence", "anything_else", "a_second_occurrence"],
)
def test_a_claim_after_a_move_is_what_that_move_would_earn(moves: str, next_move: str, claims: list[str]) -> None:
    game = game_of(moves)
    assert claimable_kinds(game.claims_after(move_named(game, next_move))) == claims


def test_a_move_that_reaches_the_clock_earns_the_fifty_move_claim() -> None:
    game = game_from(CLOCK_AT_99, "")
    assert claimable_kinds(game.claims_after(move_named(game, "Kd1"))) == ["fifty_moves"]


def test_a_move_of_another_position_claims_nothing() -> None:
    game = game_of("")
    opening = move_named(game, "e4")
    game.play_san("e4")
    assert game.claims_after(opening) == []
