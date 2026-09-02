"""The `move` schema, fact by fact.

Every expectation is worked out from the definitions in `docs/features.md` §1
and §3 for the named position above it. A case names its move in UCI, castling
written king-to-rook the way `Move.uci` spells it. The cases mirror
`tests/facts_moves.rs`.
"""

from __future__ import annotations

from collections.abc import Callable

import esca
import pytest

#: A knight on e4 reaching an enemy unit of every role; only d6 is undefended.
WHEEL = "k7/8/3n1b2/2r3q1/4N3/2p5/8/K7 w - - 0 1"

#: The same wheel a rank flip and a colour swap later, for Black to turn.
WHEEL_BLACK = "k7/8/2P5/4n3/2R3Q1/3N1B2/8/K7 b - - 0 1"

#: A symmetric middlegame: either side can move a unit of every role, and castle.
EVERY_ROLE = "r3k2r/ppp1qppp/2npbn2/8/8/2NPBN2/PPP1QPPP/R3K2R w KQkq - 0 1"

#: The same middlegame with Black to move.
EVERY_ROLE_BLACK = "r3k2r/ppp1qppp/2npbn2/8/8/2NPBN2/PPP1QPPP/R3K2R b KQkq - 0 1"

#: A pawn on e7 that can queen straight ahead or by taking either back-rank unit.
PROMOTION = "k2r1n2/4P3/8/8/8/8/8/6K1 w - - 0 1"

#: The same promotion a rank flip and a colour swap later.
PROMOTION_BLACK = "6k1/8/8/8/8/8/4p3/K2R1N2 b - - 0 1"

#: b8 is attacked by a rook and defended by one: the role that lands decides.
NEW_QUEEN = "7r/1P6/8/8/7k/8/8/1R2K3 w - - 0 1"

#: A lone queen with three ways to check a bare king, only one of them free.
CHECKS = "4k3/8/8/3p4/1Q6/8/8/4K3 w - - 0 1"

#: The same three checks a rank flip and a colour swap later.
CHECKS_BLACK = "4k3/8/8/1q6/3P4/8/8/4K3 b - - 0 1"

#: An open d-file: the rook's squares on it are defended, cheaply attacked, or bare.
EXCHANGE = "3rk3/8/1n6/8/8/2P5/8/3RK1B1 w - - 0 1"

#: The same file a rank flip and a colour swap later.
EXCHANGE_BLACK = "3rk1b1/8/2p5/8/8/1N6/8/3RK3 b - - 0 1"

#: Black has just played d7-d5; the pawn it leaves on d5 is the rook's already.
EN_PASSANT = "4k3/8/8/3pP3/8/8/8/3RK3 w - d6 0 1"

#: The same capture a rank flip and a colour swap later.
EN_PASSANT_BLACK = "3rk3/8/8/8/3Pp3/8/8/4K3 b - d3 0 1"

#: A Chess960 array: the king on b1 between its rooks on a1 and e1.
NINE_SIXTY = "rk2r3/pppqbppp/2n2n2/8/8/2N2N2/PPPQBPPP/RK2R3 w AEae - 0 1"

#: The same array with Black to move.
NINE_SIXTY_BLACK = "rk2r3/pppqbppp/2n2n2/8/8/2N2N2/PPPQBPPP/RK2R3 b AEae - 0 1"

#: The helpers `conftest.py` hands over.
MoveFactsOf = Callable[..., esca.MoveFacts]


@pytest.mark.parametrize(
    ("fen", "uci", "victim"),
    [
        (WHEEL, "e4c3", "p"),
        (WHEEL, "e4d6", "n"),
        (WHEEL, "e4f6", "b"),
        (WHEEL, "e4c5", "r"),
        (WHEEL, "e4g5", "q"),
        (WHEEL, "e4f2", None),
        (WHEEL_BLACK, "e5c6", "p"),
        (WHEEL_BLACK, "e5d3", "n"),
        (WHEEL_BLACK, "e5f3", "b"),
        (WHEEL_BLACK, "e5c4", "r"),
        (WHEEL_BLACK, "e5g4", "q"),
        (WHEEL_BLACK, "e5g6", None),
        (EN_PASSANT, "e5d6", "p"),
        (EN_PASSANT_BLACK, "e4d3", "p"),
    ],
    ids=[
        "takes_pawn",
        "takes_knight",
        "takes_bishop",
        "takes_rook",
        "takes_queen",
        "quiet_move",
        "black_takes_pawn",
        "black_takes_knight",
        "black_takes_bishop",
        "black_takes_rook",
        "black_takes_queen",
        "black_quiet_move",
        "en_passant",
        "black_en_passant",
    ],
)
def test_a_capture_names_the_role_it_removes(fen: str, uci: str, victim: str | None, move_facts: MoveFactsOf) -> None:
    assert move_facts(fen, uci).victim == victim


@pytest.mark.parametrize(
    ("fen", "uci", "mover"),
    [
        (EVERY_ROLE, "a2a3", "p"),
        (EVERY_ROLE, "c3b5", "n"),
        (EVERY_ROLE, "e3d4", "b"),
        (EVERY_ROLE, "a1b1", "r"),
        (EVERY_ROLE, "e2d2", "q"),
        (EVERY_ROLE, "e1f1", "k"),
        (EVERY_ROLE, "e1h1", "k"),
        (EVERY_ROLE_BLACK, "a7a6", "p"),
        (EVERY_ROLE_BLACK, "c6b4", "n"),
        (EVERY_ROLE_BLACK, "e6d5", "b"),
        (EVERY_ROLE_BLACK, "a8b8", "r"),
        (EVERY_ROLE_BLACK, "e7d7", "q"),
        (EVERY_ROLE_BLACK, "e8f8", "k"),
        (EVERY_ROLE_BLACK, "e8h8", "k"),
    ],
    ids=[
        "pawn",
        "knight",
        "bishop",
        "rook",
        "queen",
        "king",
        "castling_king",
        "black_pawn",
        "black_knight",
        "black_bishop",
        "black_rook",
        "black_queen",
        "black_king",
        "black_castling_king",
    ],
)
def test_every_move_names_the_role_that_makes_it(fen: str, uci: str, mover: str, move_facts: MoveFactsOf) -> None:
    assert move_facts(fen, uci).mover == mover


@pytest.mark.parametrize(
    ("fen", "uci", "promotion"),
    [
        (PROMOTION, "e7e8q", "q"),
        (PROMOTION, "e7d8r", "r"),
        (PROMOTION, "e7f8b", "b"),
        (PROMOTION, "e7e8n", "n"),
        (PROMOTION, "g1g2", None),
        (PROMOTION_BLACK, "e2e1q", "q"),
        (PROMOTION_BLACK, "e2f1r", "r"),
        (PROMOTION_BLACK, "e2e1b", "b"),
        (PROMOTION_BLACK, "e2d1n", "n"),
        (PROMOTION_BLACK, "g8g7", None),
    ],
    ids=[
        "to_queen",
        "to_rook",
        "to_bishop",
        "to_knight",
        "no_promotion",
        "black_to_queen",
        "black_to_rook",
        "black_to_bishop",
        "black_to_knight",
        "black_no_promotion",
    ],
)
def test_a_promotion_names_the_role_the_pawn_becomes(
    fen: str, uci: str, promotion: str | None, move_facts: MoveFactsOf
) -> None:
    assert move_facts(fen, uci).promotion == promotion


@pytest.mark.parametrize(
    ("fen", "uci", "gives_check"),
    [
        (CHECKS, "b4b8", True),
        (CHECKS, "b4e7", True),
        (CHECKS, "b4e4", True),
        (CHECKS, "b4d6", False),
        (CHECKS, "b4b3", False),
        (CHECKS_BLACK, "b5b1", True),
        (CHECKS_BLACK, "b5e2", True),
        (CHECKS_BLACK, "b5e5", True),
        (CHECKS_BLACK, "b5d3", False),
        (EXCHANGE, "d1d8", True),
        (PROMOTION, "e7d8q", True),
        (PROMOTION, "e7e8q", False),
        (PROMOTION, "e7d8n", False),
    ],
    ids=[
        "queen_to_the_back_rank",
        "queen_beside_the_king",
        "queen_onto_the_king_file",
        "queen_off_every_line",
        "queen_backwards",
        "black_queen_to_the_back_rank",
        "black_queen_beside_the_king",
        "black_queen_onto_the_king_file",
        "black_queen_off_every_line",
        "rook_takes_rook",
        "new_queen_on_the_rank",
        "new_queen_behind_a_rook",
        "new_knight",
    ],
)
def test_a_move_gives_check_when_it_leaves_the_enemy_king_attacked(
    fen: str, uci: str, gives_check: bool, move_facts: MoveFactsOf
) -> None:
    assert move_facts(fen, uci).gives_check == gives_check


@pytest.mark.parametrize(
    ("fen", "uci", "gives_safe_check"),
    [
        (CHECKS, "b4b8", True),
        (CHECKS, "b4e7", False),
        (CHECKS, "b4e4", False),
        (CHECKS, "b4d6", False),
        (CHECKS_BLACK, "b5b1", True),
        (CHECKS_BLACK, "b5e2", False),
        (CHECKS_BLACK, "b5e5", False),
        (EXCHANGE, "d1d8", False),
        (PROMOTION, "e7d8q", True),
    ],
    ids=[
        "check_out_of_reach",
        "check_the_king_can_take",
        "check_a_pawn_covers",
        "not_a_check",
        "black_check_out_of_reach",
        "black_check_the_king_can_take",
        "black_check_a_pawn_covers",
        "check_the_king_recaptures",
        "new_queen_out_of_reach",
    ],
)
def test_a_safe_check_is_a_check_whose_destination_is_safe(
    fen: str, uci: str, gives_safe_check: bool, move_facts: MoveFactsOf
) -> None:
    assert move_facts(fen, uci).gives_safe_check == gives_safe_check


@pytest.mark.parametrize(
    ("fen", "uci", "is_safe"),
    [
        (EXCHANGE, "d1d4", True),
        (EXCHANGE, "d1d5", False),
        (EXCHANGE, "d1d6", False),
        (EXCHANGE, "c3c4", False),
        (EXCHANGE, "d1a1", True),
        (EXCHANGE, "g1b6", True),
        (EXCHANGE, "d1d8", False),
        (EXCHANGE_BLACK, "d8d5", True),
        (EXCHANGE_BLACK, "d8d4", False),
        (EXCHANGE_BLACK, "d8d3", False),
        (EXCHANGE_BLACK, "c6c5", False),
        (EXCHANGE_BLACK, "d8a8", True),
        (EXCHANGE_BLACK, "g8b3", True),
        (EXCHANGE_BLACK, "d8d1", False),
        (NEW_QUEEN, "b7b8q", False),
        (NEW_QUEEN, "b7b8r", True),
        (NEW_QUEEN, "b7b8n", True),
    ],
    ids=[
        "attacked_but_defended",
        "a_knight_answers",
        "attacked_and_alone",
        "pawn_left_undefended",
        "unattacked",
        "takes_a_free_knight",
        "takes_a_defended_rook",
        "black_attacked_but_defended",
        "black_a_knight_answers",
        "black_attacked_and_alone",
        "black_pawn_left_undefended",
        "black_unattacked",
        "black_takes_a_free_knight",
        "black_takes_a_defended_rook",
        "new_queen_a_rook_answers",
        "new_rook_trades_evenly",
        "new_knight_costs_the_rook",
    ],
)
def test_a_safe_destination_is_free_of_pawns_of_cheaper_attackers_and_of_free_captures(
    fen: str, uci: str, is_safe: bool, move_facts: MoveFactsOf
) -> None:
    assert move_facts(fen, uci).is_safe == is_safe


@pytest.mark.parametrize(
    ("fen", "uci", "captures_hanging"),
    [
        (WHEEL, "e4d6", True),
        (WHEEL, "e4g5", False),
        (WHEEL, "e4c3", False),
        (WHEEL, "e4c5", False),
        (WHEEL, "e4f2", False),
        (EXCHANGE, "g1b6", True),
        (EXCHANGE, "d1d8", False),
        (EN_PASSANT, "e5d6", True),
        (WHEEL_BLACK, "e5d3", True),
        (WHEEL_BLACK, "e5g4", False),
        (EXCHANGE_BLACK, "g8b3", True),
        (EXCHANGE_BLACK, "d8d1", False),
    ],
    ids=[
        "takes_the_undefended_knight",
        "takes_a_defended_queen",
        "takes_a_defended_pawn",
        "takes_a_defended_rook",
        "quiet_move",
        "bishop_takes_a_free_knight",
        "rook_takes_a_defended_rook",
        "en_passant_takes_a_free_pawn",
        "black_takes_the_undefended_knight",
        "black_takes_a_defended_queen",
        "black_bishop_takes_a_free_knight",
        "black_rook_takes_a_defended_rook",
    ],
)
def test_a_move_captures_hanging_when_its_victim_is_attacked_and_undefended(
    fen: str, uci: str, captures_hanging: bool, move_facts: MoveFactsOf
) -> None:
    assert move_facts(fen, uci).captures_hanging == captures_hanging


@pytest.mark.parametrize(
    ("fen", "uci", "escapes_attack"),
    [
        (WHEEL, "e4d6", True),
        (WHEEL, "e4f2", True),
        (WHEEL, "e4g5", False),
        (WHEEL, "e4d2", False),
        (EXCHANGE, "d1d4", True),
        (EXCHANGE, "g1b6", False),
        (EXCHANGE, "c3c4", False),
        (EXCHANGE_BLACK, "d8d5", True),
        (EXCHANGE_BLACK, "d8d4", False),
        (EXCHANGE_BLACK, "g8b3", False),
    ],
    ids=[
        "takes_the_attacker",
        "to_a_free_square",
        "to_a_square_that_hangs",
        "to_a_square_a_pawn_covers",
        "rook_to_a_defended_square",
        "bishop_was_not_attacked",
        "pawn_was_not_attacked",
        "black_rook_to_a_defended_square",
        "black_rook_to_a_square_a_knight_holds",
        "black_bishop_was_not_attacked",
    ],
)
def test_a_move_escapes_attack_when_it_leaves_a_held_square_for_a_safe_one(
    fen: str, uci: str, escapes_attack: bool, move_facts: MoveFactsOf
) -> None:
    assert move_facts(fen, uci).escapes_attack == escapes_attack


@pytest.mark.parametrize(
    ("fen", "uci", "to_attacked_by_pawn"),
    [
        (CHECKS, "b4e4", True),
        (CHECKS, "b4c4", True),
        (CHECKS, "b4b8", False),
        (CHECKS, "b4e7", False),
        (WHEEL, "e4d2", True),
        (WHEEL, "e4f2", False),
        (CHECKS_BLACK, "b5e5", True),
        (CHECKS_BLACK, "b5c5", True),
        (CHECKS_BLACK, "b5b1", False),
        (EN_PASSANT, "e5d6", False),
    ],
    ids=[
        "onto_the_pawns_right_diagonal",
        "onto_the_pawns_left_diagonal",
        "onto_the_back_rank",
        "beside_the_king",
        "knight_onto_a_covered_square",
        "knight_onto_a_free_square",
        "black_onto_the_pawns_right_diagonal",
        "black_onto_the_pawns_left_diagonal",
        "black_onto_the_back_rank",
        "en_passant_removes_the_last_pawn",
    ],
)
def test_a_move_notes_when_an_enemy_pawn_covers_its_destination(
    fen: str, uci: str, to_attacked_by_pawn: bool, move_facts: MoveFactsOf
) -> None:
    assert move_facts(fen, uci).to_attacked_by_pawn == to_attacked_by_pawn


@pytest.mark.parametrize(
    ("variant", "fen", "uci", "is_castling"),
    [
        (esca.CLASSIC, EVERY_ROLE, "e1h1", True),
        (esca.CLASSIC, EVERY_ROLE, "e1a1", True),
        (esca.CLASSIC, EVERY_ROLE, "e1f1", False),
        (esca.CLASSIC, EVERY_ROLE_BLACK, "e8h8", True),
        (esca.CLASSIC, EVERY_ROLE_BLACK, "e8a8", True),
        (esca.CLASSIC, EVERY_ROLE_BLACK, "e8f8", False),
        (esca.CHESS960, NINE_SIXTY, "b1e1", True),
        (esca.CHESS960, NINE_SIXTY, "b1a1", True),
        (esca.CHESS960, NINE_SIXTY, "b1c1", False),
        (esca.CHESS960, NINE_SIXTY_BLACK, "b8e8", True),
        (esca.CHESS960, NINE_SIXTY_BLACK, "b8a8", True),
        (esca.CHESS960, NINE_SIXTY_BLACK, "b8c8", False),
    ],
    ids=[
        "short",
        "long",
        "king_step",
        "black_short",
        "black_long",
        "black_king_step",
        "chess960_short",
        "chess960_long",
        "chess960_king_step",
        "chess960_black_short",
        "chess960_black_long",
        "chess960_black_king_step",
    ],
)
def test_castling_is_the_king_move_written_to_its_own_rook(
    variant: esca.Variant, fen: str, uci: str, is_castling: bool, move_facts: MoveFactsOf
) -> None:
    assert move_facts(fen, uci, variant).is_castling == is_castling


@pytest.mark.parametrize(
    ("fen", "uci", "is_en_passant"),
    [
        (EN_PASSANT, "e5d6", True),
        (EN_PASSANT, "e5e6", False),
        (EN_PASSANT, "d1d5", False),
        (EVERY_ROLE, "a2a3", False),
        (EN_PASSANT_BLACK, "e4d3", True),
        (EN_PASSANT_BLACK, "e4e3", False),
        (EN_PASSANT_BLACK, "d8d4", False),
    ],
    ids=[
        "takes_en_passant",
        "pushes_past",
        "rook_takes_the_pawn",
        "pawn_push",
        "black_takes_en_passant",
        "black_pushes_past",
        "black_rook_takes_the_pawn",
    ],
)
def test_a_pawn_capture_onto_the_en_passant_square_is_marked_en_passant(
    fen: str, uci: str, is_en_passant: bool, move_facts: MoveFactsOf
) -> None:
    assert move_facts(fen, uci).is_en_passant == is_en_passant


# capture, victim P N B R Q, mover P N B R Q K, promotion Q R B N, then the
# eight bits `features.md` §3 lists after them.
# fmt: off
PROMOTION_CAPTURE_ROW = [
    1.0,
    0.0, 0.0, 0.0, 1.0, 0.0,
    1.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    1.0, 0.0, 0.0, 0.0,
    1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0,
]
CASTLING_ROW = [
    0.0,
    0.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0,
]
EN_PASSANT_ROW = [
    1.0,
    1.0, 0.0, 0.0, 0.0, 0.0,
    1.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0,
]
HANGING_CAPTURE_ROW = [
    1.0,
    0.0, 0.0, 0.0, 0.0, 1.0,
    0.0, 1.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
]
BLACK_CAPTURE_ROW = [
    1.0,
    0.0, 1.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0,
]
# fmt: on


@pytest.mark.parametrize(
    ("fen", "uci", "row"),
    [
        (PROMOTION, "e7d8q", PROMOTION_CAPTURE_ROW),
        (EVERY_ROLE, "e1h1", CASTLING_ROW),
        (EN_PASSANT, "e5d6", EN_PASSANT_ROW),
        (WHEEL, "e4g5", HANGING_CAPTURE_ROW),
        (EXCHANGE_BLACK, "g8b3", BLACK_CAPTURE_ROW),
    ],
    ids=["promotion_capture_with_check", "castling", "en_passant", "capture_that_hangs", "black_capture"],
)
def test_a_move_is_twenty_four_values_in_the_order_the_catalogue_lists(fen: str, uci: str, row: list[float]) -> None:
    moves, encoded = esca.encode_moves(fen)
    assert encoded.shape[1] == esca.MOVE_WIDTH == 24
    assert encoded[[move.uci for move in moves].index(uci)].tolist() == row


def test_the_move_facts_of_a_chess960_position_follow_its_own_castling_geometry(
    move_facts: MoveFactsOf,
) -> None:
    """No `move` fact is one of the four `features.md` §4 defines for classic
    chess only, so a Chess960 position is read the same way — except that
    castling starts and ends where its own array puts the king and the rook."""
    short = move_facts(NINE_SIXTY, "b1e1", esca.CHESS960)
    assert short.is_castling
    assert short.mover == "k"
    assert short.victim is None
    assert short.is_safe
    assert not short.gives_check

    long = move_facts(NINE_SIXTY, "b1a1", esca.CHESS960)
    assert long.is_castling
    assert long.mover == "k"
    assert long.is_safe

    # The queen takes a queen a knight recaptures, so the destination is not safe.
    takes_queen = move_facts(NINE_SIXTY, "d2d7", esca.CHESS960)
    assert takes_queen.victim == "q"
    assert not takes_queen.is_safe
    assert not takes_queen.captures_hanging
    assert not takes_queen.is_castling

    # a3 is a bishop's square and the b-pawn's, which is defence enough.
    pawn = move_facts(NINE_SIXTY, "a2a3", esca.CHESS960)
    assert pawn.mover == "p"
    assert pawn.is_safe
    assert not pawn.escapes_attack
