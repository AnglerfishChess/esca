"""Constructors that let a case state its expectation the way the definition reads."""

from __future__ import annotations

from collections.abc import Callable

import esca
import pytest


def _facts_of(fen: str, variant: esca.Variant | None = None) -> esca.Facts:
    return esca.Position.from_fen(fen).facts(variant if variant is not None else esca.CLASSIC)


def _squares(names: str) -> set[str]:
    return set(names.split())


@pytest.fixture(scope="session")
def facts_of() -> Callable[..., esca.Facts]:
    """`facts_of(fen)`, or `facts_of(fen, variant)`: the facts of a FEN, classic by default."""
    return _facts_of


@pytest.fixture(scope="session")
def squares() -> Callable[[str], set[str]]:
    """`squares("e4 d5")`: the squares a space-separated list names."""
    return _squares


def _move_facts(fen: str, uci: str, variant: esca.Variant | None = None) -> esca.MoveFacts:
    for annotated in _facts_of(fen, variant).moves:
        if annotated.move.uci == uci:
            return annotated.facts
    raise AssertionError(f"{uci} is not a legal move of {fen}")


@pytest.fixture(scope="session")
def move_facts() -> Callable[..., esca.MoveFacts]:
    """`move_facts(fen, "e2e4")`, or with a variant: one legal move's facts.

    Castling is written king-to-rook, so `"e1h1"` is the classic short castling.
    """
    return _move_facts
