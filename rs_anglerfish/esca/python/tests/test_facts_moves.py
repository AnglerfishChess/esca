"""The `move` schema, fact by fact.

Every expectation is worked out from the definitions in `docs/features.md` §1
and §3 for the named position above it. A case names its move in UCI, castling
written king-to-rook the way `Move.uci` spells it. The cases mirror
`tests/facts_moves.rs`.
"""

from __future__ import annotations

from collections.abc import Callable

import esca
import numpy as np
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

#: A rook with three ways to threaten something: a free rook, a defended queen,
#: and an undefended knight.
THREAT = "4k3/8/8/3q4/7r/2n5/8/R3K3 w - - 0 1"

#: The same three threats a rank flip and a colour swap later.
THREAT_BLACK = "r3k3/8/2N5/7R/3Q4/8/8/4K3 b - - 0 1"

#: A rook check down the d-file: a knight and a bishop can interpose, another
#: rook can take the checker, and the king can step aside.
CHECKED = "R2rk3/8/8/8/8/8/1N4B1/3K4 w - - 0 1"

#: The same check a rank flip and a colour swap later.
CHECKED_BLACK = "3k4/1n4b1/8/8/8/8/8/r2RK3 b - - 0 1"

#: A passed pawn on c6 that can push or take, beside a b-pawn a b-pawn holds up.
PASSERS = "4k3/3n4/2P5/8/1p3p2/8/1P6/4K3 w - - 0 1"

#: The same pair a rank flip and a colour swap later.
PASSERS_BLACK = "4k3/1p6/8/1P3P2/8/2p5/3N4/4K3 b - - 0 1"

#: A b-pawn the c-pawn holds up: pushing past it or taking it makes a passer.
CREATES = "4k3/8/2p5/1P6/8/8/8/4K3 w - - 0 1"

#: The same pawn a rank flip and a colour swap later.
CREATES_BLACK = "4k3/8/8/8/1p6/2P5/8/4K3 b - - 0 1"

#: Two pawns that stay healthy unless the b-pawn takes onto the c-file.
WEAK = "4k3/8/8/8/8/2p5/1PP5/4K3 w - - 0 1"

#: The same pair a rank flip and a colour swap later.
WEAK_BLACK = "4k3/1pp5/2P5/8/8/8/8/4K3 b - - 0 1"

#: The d-pawn covers c3, so the b-pawn running ahead leaves c2 backward.
WEAK2 = "4k3/8/8/8/3p4/8/1PP5/4K3 w - - 0 1"

#: The same pair a rank flip and a colour swap later.
WEAK2_BLACK = "4k3/1pp5/8/3P4/8/8/8/4K3 b - - 0 1"

#: A g-pawn beside the enemy king: either capture empties the g-file for us.
OPENK = "6k1/8/5p1p/6P1/8/8/8/6K1 w - - 0 1"

#: The same pawn a rank flip and a colour swap later.
OPENK_BLACK = "6k1/8/8/8/6p1/5P1P/8/6K1 b - - 0 1"

#: A rook that can reach their king's ring, facing one that holds ours.
RING = "6k1/5ppp/8/8/8/8/r4PPP/1R4K1 w - - 0 1"

#: The same pair of rooks a rank flip and a colour swap later.
RING_BLACK = "1r4k1/R4ppp/8/8/8/8/5PPP/6K1 b - - 0 1"

#: A knight standing between a rook and the enemy queen.
DISC = "3q3k/8/8/8/8/8/3N4/3RK3 w - - 0 1"

#: The same knight a rank flip and a colour swap later.
DISC_BLACK = "3rk3/3n4/8/8/8/8/8/3Q3K b - - 0 1"

#: The same battery uncovering a pawn instead of a queen.
DISC_PAWN = "7k/3p4/8/8/8/8/3N4/3RK3 w - - 0 1"

#: The helpers `conftest.py` hands over.
MoveFactsOf = Callable[..., esca.MoveFacts]


def scaled(value: float, scale: float) -> float:
    """`value / scale` as the `f32` an encoded row carries it."""
    return float(np.float32(value) / np.float32(scale))


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


@pytest.mark.parametrize(
    ("fen", "uci", "see"),
    [
        (WHEEL, "e4d6", 3),
        (WHEEL, "e4g5", 6),
        (WHEEL, "e4c5", 2),
        (WHEEL, "e4c3", -2),
        (WHEEL, "e4f2", 0),
        (WHEEL_BLACK, "e5g6", -3),
        (WHEEL_BLACK, "e5d3", 3),
        (WHEEL_BLACK, "e5g4", 6),
        (WHEEL_BLACK, "e5c4", 2),
        (WHEEL_BLACK, "e5c6", -2),
        (PROMOTION, "e7d8q", 13),
        (PROMOTION, "e7e8q", -1),
        (PROMOTION_BLACK, "e2d1q", 13),
        (EXCHANGE, "d1d5", -5),
        (EVERY_ROLE, "e1h1", 0),
    ],
    ids=[
        "takes_a_free_knight",
        "takes_a_queen_a_bishop_holds",
        "takes_a_rook_a_queen_holds",
        "takes_a_pawn_a_bishop_holds",
        "quiet_move_onto_a_free_square",
        "quiet_move_onto_a_covered_square",
        "black_takes_a_free_knight",
        "black_takes_a_queen_a_bishop_holds",
        "black_takes_a_rook_a_queen_holds",
        "black_takes_a_pawn_a_bishop_holds",
        "promotion_capture_nothing_answers",
        "promotion_a_rook_answers",
        "black_promotion_capture",
        "rook_walks_onto_a_knight",
        "castling_wins_nothing",
    ],
)
def test_see_is_what_the_move_wins_once_both_sides_stop_taking(
    fen: str, uci: str, see: int, move_facts: MoveFactsOf
) -> None:
    assert move_facts(fen, uci).see == see


@pytest.mark.parametrize(
    ("fen", "uci", "threat"),
    [
        (THREAT, "a1a4", 5),
        (THREAT, "a1d1", 4),
        (THREAT, "a1c1", 3),
        (THREAT, "a1a8", 0),
        (THREAT, "e1f2", 0),
        (DISC, "d2b3", 9),
        (THREAT_BLACK, "a8a5", 5),
        (THREAT_BLACK, "a8d8", 4),
        (THREAT_BLACK, "a8c8", 3),
        (THREAT_BLACK, "a8a1", 0),
        (DISC_BLACK, "d7b6", 9),
    ],
    ids=[
        "rook_eyes_a_free_rook",
        "rook_eyes_a_defended_queen",
        "rook_eyes_a_free_knight",
        "check_threatens_nothing",
        "king_step_threatens_nothing",
        "knight_uncovers_a_queen",
        "black_rook_eyes_a_free_rook",
        "black_rook_eyes_a_defended_queen",
        "black_rook_eyes_a_free_knight",
        "black_check_threatens_nothing",
        "black_knight_uncovers_a_queen",
    ],
)
def test_a_threat_is_the_most_the_next_capture_would_win(
    fen: str, uci: str, threat: int, move_facts: MoveFactsOf
) -> None:
    assert move_facts(fen, uci).threat_created_max == threat


@pytest.mark.parametrize(
    ("fen", "uci", "moves_attacked_unit"),
    [
        (EXCHANGE, "d1d4", True),
        (WHEEL, "e4f2", True),
        (EXCHANGE, "c3c4", False),
        (EXCHANGE, "g1b6", False),
        (EVERY_ROLE, "e1h1", False),
        (EXCHANGE_BLACK, "d8d5", True),
        (WHEEL_BLACK, "e5g6", True),
        (EXCHANGE_BLACK, "c6c5", False),
        (EXCHANGE_BLACK, "g8b3", False),
    ],
    ids=[
        "rook_a_rook_attacks",
        "knight_a_knight_attacks",
        "pawn_nothing_attacks",
        "bishop_nothing_attacks",
        "castling_out_of_a_quiet_corner",
        "black_rook_a_rook_attacks",
        "black_knight_a_knight_attacks",
        "black_pawn_nothing_attacks",
        "black_bishop_nothing_attacks",
    ],
)
def test_a_move_notes_when_the_square_it_leaves_is_under_attack(
    fen: str, uci: str, moves_attacked_unit: bool, move_facts: MoveFactsOf
) -> None:
    assert move_facts(fen, uci).moves_attacked_unit == moves_attacked_unit


@pytest.mark.parametrize(
    ("fen", "uci", "blocks_check"),
    [
        (CHECKED, "b2d3", True),
        (CHECKED, "g2d5", True),
        (CHECKED, "a8d8", False),
        (CHECKED, "d1c1", False),
        (CHECKED, "d1e2", False),
        (EXCHANGE, "d1d4", False),
        (CHECKED_BLACK, "b7d6", True),
        (CHECKED_BLACK, "g7d4", True),
        (CHECKED_BLACK, "a1d1", False),
        (CHECKED_BLACK, "d8c8", False),
        (CHECKED_BLACK, "d8e7", False),
    ],
    ids=[
        "knight_interposes",
        "bishop_interposes",
        "rook_takes_the_checker",
        "king_steps_aside",
        "king_steps_forward",
        "no_check_to_block",
        "black_knight_interposes",
        "black_bishop_interposes",
        "black_rook_takes_the_checker",
        "black_king_steps_aside",
        "black_king_steps_forward",
    ],
)
def test_a_move_blocks_check_when_it_lands_between_the_checker_and_the_king(
    fen: str, uci: str, blocks_check: bool, move_facts: MoveFactsOf
) -> None:
    assert move_facts(fen, uci).blocks_check == blocks_check


@pytest.mark.parametrize(
    ("fen", "uci", "advances_passer"),
    [
        (PASSERS, "c6c7", True),
        (PASSERS, "c6d7", True),
        (PASSERS, "b2b3", False),
        (PASSERS, "e1e2", False),
        (EXCHANGE, "d1d4", False),
        (PASSERS_BLACK, "c3c2", True),
        (PASSERS_BLACK, "c3d2", True),
        (PASSERS_BLACK, "b7b6", False),
        (PASSERS_BLACK, "e8e7", False),
    ],
    ids=[
        "passer_pushes",
        "passer_takes",
        "held_up_pawn_pushes",
        "king_steps",
        "rook_moves",
        "black_passer_pushes",
        "black_passer_takes",
        "black_held_up_pawn_pushes",
        "black_king_steps",
    ],
)
def test_a_pawn_advances_a_passer_when_the_pawn_it_moves_was_passed(
    fen: str, uci: str, advances_passer: bool, move_facts: MoveFactsOf
) -> None:
    assert move_facts(fen, uci).advances_passer == advances_passer


@pytest.mark.parametrize(
    ("fen", "uci", "creates_passer"),
    [
        (CREATES, "b5c6", True),
        (CREATES, "b5b6", True),
        (WEAK2, "c2c4", True),
        (CREATES, "e1e2", False),
        (PASSERS, "c6c7", False),
        (CREATES_BLACK, "b4c3", True),
        (CREATES_BLACK, "b4b3", True),
        (WEAK2_BLACK, "c7c5", True),
        (CREATES_BLACK, "e8e7", False),
        (PASSERS_BLACK, "c3c2", False),
    ],
    ids=[
        "takes_the_holder",
        "pushes_past_the_holder",
        "pushes_beside_a_holder",
        "king_steps",
        "passer_only_advances",
        "black_takes_the_holder",
        "black_pushes_past_the_holder",
        "black_pushes_beside_a_holder",
        "black_king_steps",
        "black_passer_only_advances",
    ],
)
def test_a_move_creates_a_passer_when_it_leaves_the_side_with_more_of_them(
    fen: str, uci: str, creates_passer: bool, move_facts: MoveFactsOf
) -> None:
    assert move_facts(fen, uci).creates_passer == creates_passer


@pytest.mark.parametrize(
    ("fen", "uci", "weaknesses"),
    [
        (WEAK, "b2c3", (True, True, False)),
        (WEAK, "b2b3", (False, False, False)),
        (WEAK, "b2b4", (False, False, False)),
        (WEAK2, "b2b4", (False, False, True)),
        (WEAK2, "b2b3", (False, False, True)),
        (WEAK2, "c2c3", (False, False, False)),
        (WEAK2, "e1d1", (False, False, False)),
        (WEAK_BLACK, "b7c6", (True, True, False)),
        (WEAK_BLACK, "b7b6", (False, False, False)),
        (WEAK_BLACK, "b7b5", (False, False, False)),
        (WEAK2_BLACK, "b7b5", (False, False, True)),
        (WEAK2_BLACK, "b7b6", (False, False, True)),
        (WEAK2_BLACK, "c7c6", (False, False, False)),
    ],
    ids=[
        "takes_onto_its_neighbours_file",
        "pushes",
        "double_pushes",
        "runs_ahead_of_its_neighbour",
        "steps_ahead_of_its_neighbour",
        "neighbour_keeps_up",
        "king_steps",
        "black_takes_onto_its_neighbours_file",
        "black_pushes",
        "black_double_pushes",
        "black_runs_ahead_of_its_neighbour",
        "black_steps_ahead_of_its_neighbour",
        "black_neighbour_keeps_up",
    ],
)
def test_a_move_creates_a_weakness_when_it_leaves_the_side_with_more_weak_pawns(
    fen: str, uci: str, weaknesses: tuple[bool, bool, bool], move_facts: MoveFactsOf
) -> None:
    facts = move_facts(fen, uci)
    assert (facts.creates_isolated, facts.creates_doubled, facts.creates_backward) == weaknesses


@pytest.mark.parametrize(
    ("fen", "uci", "opens"),
    [
        (OPENK, "g5f6", True),
        (OPENK, "g5h6", True),
        (OPENK, "g5g6", False),
        (OPENK, "g1f1", False),
        (EN_PASSANT, "e5d6", True),
        (OPENK_BLACK, "g4f3", True),
        (OPENK_BLACK, "g4h3", True),
        (OPENK_BLACK, "g4g3", False),
        (OPENK_BLACK, "g8f8", False),
    ],
    ids=[
        "takes_to_the_left",
        "takes_to_the_right",
        "stays_on_the_file",
        "king_steps",
        "pawn_leaves_a_file_away_from_their_king",
        "black_takes_to_the_left",
        "black_takes_to_the_right",
        "black_stays_on_the_file",
        "black_king_steps",
    ],
)
def test_a_move_opens_a_file_at_their_king_when_our_last_pawn_leaves_it(
    fen: str, uci: str, opens: bool, move_facts: MoveFactsOf
) -> None:
    assert move_facts(fen, uci).opens_file_at_enemy_king == opens


@pytest.mark.parametrize(
    ("fen", "uci", "deltas"),
    [
        (RING, "b1b7", (1, 0)),
        (RING, "b1b8", (1, 0)),
        (RING, "b1a1", (0, 0)),
        (RING, "g1h1", (0, -1)),
        (RING, "f2f3", (0, 0)),
        (EXCHANGE, "g1b6", (1, 0)),
        (RING_BLACK, "b8b2", (1, 0)),
        (RING_BLACK, "b8b1", (1, 0)),
        (RING_BLACK, "b8a8", (0, 0)),
        (RING_BLACK, "g8h8", (0, -1)),
        (RING_BLACK, "f7f6", (0, 0)),
    ],
    ids=[
        "rook_reaches_their_seventh",
        "rook_reaches_their_back_rank",
        "rook_steps_aside",
        "king_walks_out_of_a_rooks_reach",
        "pawn_push_changes_neither",
        "knight_takes_a_ring_attacker",
        "black_rook_reaches_their_second",
        "black_rook_reaches_their_back_rank",
        "black_rook_steps_aside",
        "black_king_walks_out_of_a_rooks_reach",
        "black_pawn_push_changes_neither",
    ],
)
def test_a_move_states_what_it_does_to_both_king_rings(
    fen: str, uci: str, deltas: tuple[int, int], move_facts: MoveFactsOf
) -> None:
    facts = move_facts(fen, uci)
    assert (facts.our_ring_attackers_delta, facts.their_ring_attackers_delta) == deltas


@pytest.mark.parametrize(
    ("fen", "uci", "deltas"),
    [
        (EXCHANGE, "d1d4", (0, -1)),
        (EXCHANGE, "d1d6", (1, 0)),
        (EXCHANGE, "c3c4", (1, 0)),
        (EXCHANGE, "g1b6", (0, -1)),
        (WHEEL, "e4d6", (-1, -1)),
        (WHEEL, "e4c3", (0, -1)),
        (EXCHANGE_BLACK, "d8d5", (0, -1)),
        (EXCHANGE_BLACK, "d8d3", (1, 0)),
        (EXCHANGE_BLACK, "c6c5", (1, 0)),
        (EXCHANGE_BLACK, "g8b3", (0, -1)),
        (WHEEL_BLACK, "e5d3", (-1, -1)),
    ],
    ids=[
        "rook_to_a_defended_square",
        "rook_to_a_bare_square",
        "pawn_to_a_bare_square",
        "bishop_takes_the_hanging_knight",
        "knight_takes_the_knight_that_held_it",
        "knight_walks_into_a_bishop",
        "black_rook_to_a_defended_square",
        "black_rook_to_a_bare_square",
        "black_pawn_to_a_bare_square",
        "black_bishop_takes_the_hanging_knight",
        "black_knight_takes_the_knight_that_held_it",
    ],
)
def test_a_move_states_how_many_units_of_each_side_it_leaves_hanging(
    fen: str, uci: str, deltas: tuple[int, int], move_facts: MoveFactsOf
) -> None:
    facts = move_facts(fen, uci)
    assert (facts.own_hanging_delta, facts.their_hanging_delta) == deltas


@pytest.mark.parametrize(
    ("fen", "uci", "leaves"),
    [
        (EXCHANGE, "d1d6", True),
        (EXCHANGE, "d1d5", True),
        (WHEEL, "e4c3", True),
        (EXCHANGE, "c3c4", False),
        (EXCHANGE, "d1d4", False),
        (WHEEL, "e4d6", False),
        (EXCHANGE_BLACK, "d8d3", True),
        (EXCHANGE_BLACK, "d8d4", True),
        (WHEEL_BLACK, "e5c6", True),
        (EXCHANGE_BLACK, "c6c5", False),
        (EXCHANGE_BLACK, "d8d5", False),
    ],
    ids=[
        "rook_walks_onto_a_bare_square",
        "rook_walks_onto_a_knights_square",
        "knight_walks_into_a_bishop",
        "only_a_pawn_is_left_hanging",
        "rook_stays_defended",
        "knight_takes_its_attacker",
        "black_rook_walks_onto_a_bare_square",
        "black_rook_walks_onto_a_knights_square",
        "black_knight_walks_into_a_bishop",
        "black_only_a_pawn_is_left_hanging",
        "black_rook_stays_defended",
    ],
)
def test_a_move_leaves_a_unit_hanging_when_a_square_carries_a_new_hanging_piece(
    fen: str, uci: str, leaves: bool, move_facts: MoveFactsOf
) -> None:
    assert move_facts(fen, uci).leaves_unit_hanging == leaves


@pytest.mark.parametrize(
    ("fen", "uci", "discovers"),
    [
        (DISC, "d2b3", True),
        (DISC, "d2f3", True),
        (DISC, "d1c1", False),
        (DISC, "e1e2", False),
        (DISC_PAWN, "d2b3", False),
        (DISC_BLACK, "d7b6", True),
        (DISC_BLACK, "d7f6", True),
        (DISC_BLACK, "d8c8", False),
        (DISC_BLACK, "e8e7", False),
    ],
    ids=[
        "knight_steps_off_the_file",
        "knight_steps_off_the_other_way",
        "the_rook_itself_moves",
        "king_steps_aside",
        "what_it_uncovers_is_only_a_pawn",
        "black_knight_steps_off_the_file",
        "black_knight_steps_off_the_other_way",
        "black_rook_itself_moves",
        "black_king_steps_aside",
    ],
)
def test_a_move_gives_a_discovered_attack_when_a_slider_it_leaves_standing_gains_one(
    fen: str, uci: str, discovers: bool, move_facts: MoveFactsOf
) -> None:
    assert move_facts(fen, uci).gives_discovered_attack == discovers


# capture, victim P N B R Q, mover P N B R Q K, promotion Q R B N, the eight
# bits `features.md` §3 lists after them, then its sixteen after-the-move
# values: see, threat, four bits, three weakness bits, the open-file bit, two
# ring deltas, two hanging deltas, and two bits.
# fmt: off
PROMOTION_CAPTURE_ROW = [
    1.0,
    0.0, 0.0, 0.0, 1.0, 0.0,
    1.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    1.0, 0.0, 0.0, 0.0,
    1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0,
    1.0, scaled(3, 9), 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, scaled(1, 4), 0.0, 0.0, 0.0, 0.0, 0.0,
]
CASTLING_ROW = [
    0.0,
    0.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
]
EN_PASSANT_ROW = [
    1.0,
    1.0, 0.0, 0.0, 0.0, 0.0,
    1.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0,
    scaled(1, 9), 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, scaled(-1, 4), 0.0, 0.0,
]
HANGING_CAPTURE_ROW = [
    1.0,
    0.0, 0.0, 0.0, 0.0, 1.0,
    0.0, 1.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    scaled(6, 9), 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, scaled(-1, 4), 1.0, 0.0,
]
QUIET_BLUNDER_ROW = [
    0.0,
    0.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    scaled(-5, 9), scaled(5, 9), 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, scaled(1, 4), scaled(1, 4), 1.0, 0.0,
]
BLACK_CAPTURE_ROW = [
    1.0,
    0.0, 1.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0,
    scaled(3, 9), scaled(5, 9), 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, scaled(1, 4), 0.0, 0.0, scaled(-1, 4), 0.0, 0.0,
]
# fmt: on


@pytest.mark.parametrize(
    ("fen", "uci", "row"),
    [
        (PROMOTION, "e7d8q", PROMOTION_CAPTURE_ROW),
        (EVERY_ROLE, "e1h1", CASTLING_ROW),
        (EN_PASSANT, "e5d6", EN_PASSANT_ROW),
        (WHEEL, "e4g5", HANGING_CAPTURE_ROW),
        (THREAT, "a1a4", QUIET_BLUNDER_ROW),
        (EXCHANGE_BLACK, "g8b3", BLACK_CAPTURE_ROW),
    ],
    ids=[
        "promotion_capture_with_check",
        "castling",
        "en_passant",
        "capture_that_hangs",
        "quiet_move_that_hangs_a_rook",
        "black_capture",
    ],
)
def test_a_move_is_forty_values_in_the_order_the_catalogue_lists(fen: str, uci: str, row: list[float]) -> None:
    moves, encoded = esca.encode_moves(fen)
    assert encoded.shape[1] == esca.MOVE_WIDTH == 40
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


@pytest.mark.parametrize(
    ("fen", "uci", "threat", "their_ring_delta"),
    [
        (NINE_SIXTY, "b1e1", 0, 0),
        (NINE_SIXTY, "b1a1", 3, 1),
        (NINE_SIXTY, "b1c1", 0, 1),
        (NINE_SIXTY, "a2a3", 0, 0),
        (NINE_SIXTY_BLACK, "b8e8", 0, 0),
        (NINE_SIXTY_BLACK, "b8a8", 3, 1),
        (NINE_SIXTY_BLACK, "b8c8", 0, 1),
        (NINE_SIXTY_BLACK, "a7a6", 0, 0),
    ],
    ids=[
        "short",
        "long",
        "king_step",
        "pawn_push",
        "black_short",
        "black_long",
        "black_king_step",
        "black_pawn_push",
    ],
)
def test_a_chess960_move_reads_its_own_castling_geometry(
    fen: str, uci: str, threat: int, their_ring_delta: int, move_facts: MoveFactsOf
) -> None:
    """Castling long lands the rook on d1, behind the queen, and the pair then
    wins a piece on d7; stepping the king to c1 instead only moves its ring
    under the enemy queen."""
    facts = move_facts(fen, uci, esca.CHESS960)
    assert facts.threat_created_max == threat
    assert facts.their_ring_attackers_delta == their_ring_delta
    assert facts.our_ring_attackers_delta == 0
    assert not facts.blocks_check
    assert not facts.moves_attacked_unit
    assert not facts.leaves_unit_hanging
    assert not facts.gives_discovered_attack
    assert facts.see == 0


@pytest.mark.parametrize(
    ("fen", "uci"),
    [(NINE_SIXTY, "d2d7"), (NINE_SIXTY_BLACK, "d7d2")],
    ids=["white", "black"],
)
def test_a_chess960_capture_states_what_it_leaves_behind(fen: str, uci: str, move_facts: MoveFactsOf) -> None:
    """The queen that walks into three defenders hangs on the square it lands on."""
    facts = move_facts(fen, uci, esca.CHESS960)
    assert facts.moves_attacked_unit
    assert facts.own_hanging_delta == 1
    assert facts.leaves_unit_hanging
    assert facts.our_ring_attackers_delta == 1
    assert facts.threat_created_max == 0
