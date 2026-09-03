"""The `exchange` group, and the static exchange evaluation under it.

Every expectation is worked out from the definitions in `docs/features.md`
§1 and §2.9 for the named position above it. The cases mirror
`tests/facts_exchange.rs`.
"""

from __future__ import annotations

from collections.abc import Callable

import esca
import pytest

#: A rook takes an undefended pawn.
FREE_PAWN = "4k3/8/8/3p4/8/8/3R4/4K3 w - - 0 1"

#: The same pawn, now defended by one of its own.
DEFENDED_PAWN = "4k3/8/2p5/3p4/8/8/3R4/4K3 w - - 0 1"

#: The same, with Black to move and nothing of its own to take.
DEFENDED_PAWN_BLACK = "4k3/8/2p5/3p4/8/8/3R4/4K3 b - - 0 1"

#: A pawn takes a pawn a pawn defends: the even trade.
PAWN_TRADE = "4k3/8/2p5/3p4/4P3/8/8/4K3 w - - 0 1"

#: The same, with Black to move: taking on e4 wins a pawn nothing guards.
PAWN_TRADE_BLACK = "4k3/8/2p5/3p4/4P3/8/8/4K3 b - - 0 1"

#: A queen has to take the pawn the c6 pawn defends.
QUEEN_TAKES_PAWN = "4k3/8/2p5/3p4/8/8/3Q4/4K3 w - - 0 1"

#: Knight for knight, each defended by a pawn on neither side.
KNIGHT_TRADE = "4k3/8/2p5/3n4/8/2N5/8/4K3 w - - 0 1"

#: The same with Black to move: the c3 knight has nothing behind it.
KNIGHT_TRADE_BLACK = "4k3/8/2p5/3n4/8/2N5/8/4K3 b - - 0 1"

#: A knight wins a defended rook: the recapture is worth less than the prize.
KNIGHT_TAKES_ROOK = "4k3/8/2p5/3r4/8/2N5/8/4K3 w - - 0 1"

#: A rook takes a defended knight and loses the difference.
ROOK_TAKES_KNIGHT = "4k3/8/2p5/3n4/8/8/3R4/4K3 w - - 0 1"

#: Rook takes rook and the enemy king takes back: an even trade.
KING_RECAPTURES = "8/8/8/8/8/4k3/3r4/3R3K w - - 0 1"

#: The doubled rook covers d5, so the king may not take back at all.
KING_REFUSED = "8/8/4k3/3p4/8/8/3R4/3R3K w - - 0 1"

#: The same without the second rook: the king takes back and wins the rook.
KING_TAKES_BACK = "8/8/4k3/3p4/8/8/3R4/7K w - - 0 1"

#: Two rooks on the d-file: the second joins once the first has taken.
XRAY_ROOKS = "4k3/8/2p5/3p4/8/8/3R4/3R1K2 w - - 0 1"

#: The same with one rook, so nothing comes back after the recapture.
ONE_ROOK = "4k3/8/2p5/3p4/8/8/3R4/5K2 w - - 0 1"

#: Both sides double on the file; Black's rook has the last word.
BOTH_BATTERIES = "3rk3/8/2p5/3p4/8/8/3R4/3R1K2 w - - 0 1"

#: The bishop that defends d5 may not legally move: an exchange ignores pins.
PINNED_DEFENDER = "2k5/8/2b5/3p4/8/8/3R4/2R1K3 w - - 0 1"

#: A pawn takes a rook and promotes, with nothing to answer it.
PROMOTION_FREE = "r3k3/1P6/8/8/8/8/6K1/8 w - - 0 1"

#: The same, with the second rook ready to take the new queen.
PROMOTION_DEFENDED = "r3k3/1P6/8/8/8/8/6K1/r7 w - - 0 1"

#: The d-pawn has just run past e5, and nothing covers d6.
EP_FREE = "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1"

#: The same with the c7 pawn covering d6.
EP_DEFENDED = "4k3/2p5/8/3pP3/8/8/8/4K3 w - d6 0 1"

#: Three white units bear on e5, two black ones defend it.
CROWD = "4rk2/8/3p4/4p3/3P4/5N2/8/4RK2 w - - 0 1"

#: The same with Black to move: e5 takes d4 and the knight takes back.
CROWD_BLACK = "4rk2/8/3p4/4p3/3P4/5N2/8/4RK2 b - - 0 1"

#: A queen hangs on d5; the one on d2 stands beside its king.
QUEEN_FREE = "4k3/8/8/3q4/8/8/3Q4/4K3 w - - 0 1"

#: Both queens are defended by their kings.
QUEEN_DEFENDED = "8/8/4k3/3q4/8/8/3Q4/4K3 w - - 0 1"

#: A rook with an open board and one black pawn covering e5 and g5.
ROOK_WALK = "4k3/8/5p2/R7/8/8/8/4K3 w - - 0 1"

#: Nothing to capture: the short castling is the move to ask about.
CASTLING = "4k3/8/8/8/8/8/8/4K2R w K - 0 1"

#: The king itself takes the pawn on d2.
KING_TAKES = "4k3/8/8/8/8/8/3p4/3K4 w - - 0 1"

#: The untouched array: no capture for either side.
START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

#: A rook checks from e2 and the king may take it.
IN_CHECK = "4k3/8/8/8/8/8/4r3/4K3 w - - 0 1"

#: Pawns on c4 and e4 against d5 and f5, each side's pawns guarding the other's
#: targets.
PAWN_CHAINS = "4k3/8/2p3p1/3p1p2/2P1P3/8/8/4K3 w - - 0 1"

#: The same with Black to move.
PAWN_CHAINS_BLACK = "4k3/8/2p3p1/3p1p2/2P1P3/8/8/4K3 b - - 0 1"

#: The helper `conftest.py` hands over.
FactsOf = Callable[..., esca.Facts]


def see_of(fen: str, uci: str) -> int:
    """The SEE of the move `uci` in `fen`, under classic chess."""
    position = esca.Position.from_fen(fen)
    for annotated in position.facts().moves:
        if annotated.move.uci == uci:
            return position.see_capture(annotated.move)
    raise AssertionError(f"{uci} is not a legal move of {fen}")


@pytest.mark.parametrize(
    ("fen", "uci", "see"),
    [
        (FREE_PAWN, "d2d5", 1),
        (DEFENDED_PAWN, "d2d5", -4),
        (PAWN_TRADE, "e4d5", 0),
        (QUEEN_TAKES_PAWN, "d2d5", -8),
        (KNIGHT_TRADE, "c3d5", 0),
        (KNIGHT_TAKES_ROOK, "c3d5", 2),
        (ROOK_TAKES_KNIGHT, "d2d5", -2),
        (KING_RECAPTURES, "d1d2", 0),
        (KING_REFUSED, "d2d5", 1),
        (KING_TAKES_BACK, "d2d5", -4),
        (XRAY_ROOKS, "d2d5", -3),
        (ONE_ROOK, "d2d5", -4),
        (BOTH_BATTERIES, "d2d5", -4),
        (PINNED_DEFENDER, "d2d5", -4),
        (PROMOTION_FREE, "b7a8q", 13),
        (PROMOTION_DEFENDED, "b7a8q", 4),
        (EP_FREE, "e5d6", 1),
        (EP_DEFENDED, "e5d6", 0),
        (CROWD, "d4e5", 1),
        (CROWD, "f3e5", -1),
        (CROWD, "e1e5", -3),
        (CROWD_BLACK, "e5d4", 0),
        (QUEEN_FREE, "d2d5", 9),
        (QUEEN_DEFENDED, "d2d5", 0),
        (KING_TAKES, "d1d2", 1),
        (ROOK_WALK, "a5e5", -5),
        (ROOK_WALK, "a5a6", 0),
        (CASTLING, "e1h1", 0),
    ],
    ids=[
        "free_pawn",
        "defended_pawn",
        "pawn_trade",
        "queen_takes_pawn",
        "knight_trade",
        "knight_takes_rook",
        "rook_takes_knight",
        "king_recaptures",
        "king_refused",
        "king_takes_back",
        "xray_rooks",
        "one_rook",
        "both_batteries",
        "pinned_defender",
        "promotion_free",
        "promotion_defended",
        "en_passant_free",
        "en_passant_defended",
        "crowd_pawn_first",
        "crowd_knight_first",
        "crowd_rook_first",
        "crowd_black_pawn",
        "queen_free",
        "queen_defended",
        "king_takes_a_pawn",
        "quiet_into_a_pawn",
        "quiet_and_safe",
        "castling",
    ],
)
def test_an_exchange_is_played_out_with_the_least_valuable_attacker_each_time(fen: str, uci: str, see: int) -> None:
    assert see_of(fen, uci) == see


@pytest.mark.parametrize(
    ("fen", "square", "see"),
    [
        (FREE_PAWN, "d5", 1),
        (DEFENDED_PAWN, "d5", 0),
        (KNIGHT_TAKES_ROOK, "d5", 2),
        (QUEEN_FREE, "d5", 9),
        (QUEEN_DEFENDED, "d5", 0),
        (CROWD, "e5", 1),
        (FREE_PAWN, "d2", 0),
        (FREE_PAWN, "e1", 0),
        (FREE_PAWN, "a1", 0),
    ],
    ids=[
        "free_pawn",
        "defended_pawn",
        "knight_takes_rook",
        "queen_free",
        "queen_defended",
        "crowd_pawn",
        "our_own_rook",
        "a_king",
        "an_empty_square",
    ],
)
def test_the_see_of_a_unit_is_what_the_other_side_wins_by_taking_it(fen: str, square: str, see: int) -> None:
    assert esca.Position.from_fen(fen).see(square) == see


GROUP_IDS = [
    "start",
    "crowd",
    "crowd_black",
    "queen_free",
    "in_check",
    "knight_trade",
    "knight_trade_black",
    "defended_pawn",
    "defended_pawn_black",
    "pawn_chains",
    "pawn_chains_black",
    "pawn_trade_black",
]


@pytest.mark.parametrize(
    ("fen", "best"),
    [
        (START, (0, 0)),
        (CROWD, (1, 0)),
        (CROWD_BLACK, (0, 1)),
        (QUEEN_FREE, (9, 0)),
        (IN_CHECK, (5, 0)),
        (KNIGHT_TRADE, (0, 3)),
        (KNIGHT_TRADE_BLACK, (3, 0)),
        (DEFENDED_PAWN, (-4, 0)),
        (DEFENDED_PAWN_BLACK, (0, -4)),
        (PAWN_CHAINS, (1, 1)),
        (PAWN_CHAINS_BLACK, (1, 1)),
        (PAWN_TRADE_BLACK, (1, 0)),
    ],
    ids=GROUP_IDS,
)
def test_the_best_capture_is_the_largest_see_the_side_has(fen: str, best: tuple[int, int], facts_of: FactsOf) -> None:
    exchange = facts_of(fen).exchange
    assert exchange[esca.US].see_best_capture == best[0]
    assert exchange[esca.THEM].see_best_capture == best[1]


@pytest.mark.parametrize(
    ("fen", "count"),
    [
        (START, (0, 0)),
        (CROWD, (1, 0)),
        (CROWD_BLACK, (0, 1)),
        (QUEEN_FREE, (1, 0)),
        (IN_CHECK, (1, 0)),
        (KNIGHT_TRADE, (0, 1)),
        (KNIGHT_TRADE_BLACK, (1, 0)),
        (DEFENDED_PAWN, (0, 0)),
        (DEFENDED_PAWN_BLACK, (0, 0)),
        (PAWN_CHAINS, (2, 3)),
        (PAWN_CHAINS_BLACK, (3, 2)),
        (PAWN_TRADE_BLACK, (1, 0)),
    ],
    ids=GROUP_IDS,
)
def test_a_positive_capture_is_one_that_wins_material_outright(
    fen: str, count: tuple[int, int], facts_of: FactsOf
) -> None:
    exchange = facts_of(fen).exchange
    assert exchange[esca.US].see_positive_capture_count == count[0]
    assert exchange[esca.THEM].see_positive_capture_count == count[1]


@pytest.mark.parametrize(
    ("fen", "count"),
    [
        (START, (0, 0)),
        (CROWD, (0, 1)),
        (CROWD_BLACK, (1, 0)),
        (QUEEN_FREE, (0, 1)),
        (IN_CHECK, (0, 0)),
        (KNIGHT_TRADE, (1, 0)),
        (KNIGHT_TRADE_BLACK, (0, 1)),
        (DEFENDED_PAWN, (0, 0)),
        (DEFENDED_PAWN_BLACK, (0, 0)),
        (PAWN_CHAINS, (1, 0)),
        (PAWN_CHAINS_BLACK, (0, 1)),
        (PAWN_TRADE_BLACK, (0, 1)),
    ],
    ids=GROUP_IDS,
)
def test_an_equal_capture_is_one_the_exchange_leaves_level(fen: str, count: tuple[int, int], facts_of: FactsOf) -> None:
    exchange = facts_of(fen).exchange
    assert exchange[esca.US].see_equal_capture_count == count[0]
    assert exchange[esca.THEM].see_equal_capture_count == count[1]


@pytest.mark.parametrize(
    ("fen", "total"),
    [
        (START, (0, 0)),
        (CROWD, (1, 0)),
        (CROWD_BLACK, (0, 1)),
        (QUEEN_FREE, (9, 0)),
        (IN_CHECK, (5, 0)),
        (KNIGHT_TRADE, (0, 3)),
        (KNIGHT_TRADE_BLACK, (3, 0)),
        (DEFENDED_PAWN, (0, 0)),
        (DEFENDED_PAWN_BLACK, (0, 0)),
        (PAWN_CHAINS, (2, 3)),
        (PAWN_CHAINS_BLACK, (3, 2)),
        (PAWN_TRADE_BLACK, (1, 0)),
    ],
    ids=GROUP_IDS,
)
def test_the_positive_total_adds_up_the_captures_that_win_material(
    fen: str, total: tuple[int, int], facts_of: FactsOf
) -> None:
    exchange = facts_of(fen).exchange
    assert exchange[esca.US].see_positive_total == total[0]
    assert exchange[esca.THEM].see_positive_total == total[1]


def test_the_them_block_is_empty_when_we_are_in_check(facts_of: FactsOf) -> None:
    """In check there is no null move, so the `them` block is zero and
    `tactics.them` says why."""
    facts = facts_of(IN_CHECK)
    assert facts.state.in_check
    assert not facts.tactics[esca.THEM].available
    theirs = facts.exchange[esca.THEM]
    assert theirs.see_best_capture == 0
    assert theirs.see_positive_capture_count == 0
    assert theirs.see_equal_capture_count == 0
    assert theirs.see_positive_total == 0


@pytest.mark.parametrize(
    "fen",
    [CROWD, PAWN_CHAINS, QUEEN_FREE, START],
    ids=["crowd", "pawn_chains", "queen_free", "start"],
)
def test_the_three_counts_partition_the_captures(fen: str, facts_of: FactsOf) -> None:
    """The block's counts and the move list agree: every capture falls in
    exactly one of the three classes."""
    facts = facts_of(fen)
    position = esca.Position.from_fen(fen)
    ours = facts.exchange[esca.US]
    captures = [position.see_capture(annotated.move) for annotated in facts.moves if annotated.facts.victim is not None]

    negative = sum(1 for see in captures if see < 0)
    assert ours.see_positive_capture_count + ours.see_equal_capture_count + negative == len(captures)
    assert ours.see_positive_total == sum(see for see in captures if see > 0)
    assert ours.see_best_capture == max(captures, default=0)
