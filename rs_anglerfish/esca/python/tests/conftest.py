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
