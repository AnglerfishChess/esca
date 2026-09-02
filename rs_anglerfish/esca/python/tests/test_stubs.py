"""The stubs and the module they describe say the same thing."""

from __future__ import annotations

import ast
from pathlib import Path

import esca
import pytest
from esca import _esca

PACKAGE = Path(esca.__file__).resolve().parent


def parsed(name: str) -> ast.Module:
    return ast.parse((PACKAGE / name).read_text())


def stub_all(name: str) -> list[str]:
    for node in ast.walk(parsed(name)):
        if isinstance(node, ast.Assign) and any(
            isinstance(target, ast.Name) and target.id == "__all__" for target in node.targets
        ):
            assert isinstance(node.value, ast.List)
            return [ast.literal_eval(element) for element in node.value.elts]
    raise AssertionError(f"{name} declares no __all__")


def declared_members(body: list[ast.stmt]) -> set[str]:
    """The names a stub body declares, dunders and private ones dropped."""
    names: set[str] = set()
    for node in body:
        if isinstance(node, ast.FunctionDef):
            names.add(node.name)
        elif isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name):
            names.add(node.target.id)
    return {name for name in names if not name.startswith("_")}


def stub_classes() -> dict[str, set[str]]:
    """The classes `_esca.pyi` declares, less the stub-only TypedDicts."""
    return {
        node.name: declared_members(node.body)
        for node in parsed("_esca.pyi").body
        if isinstance(node, ast.ClassDef)
        and not any(isinstance(base, ast.Name) and base.id == "TypedDict" for base in node.bases)
    }


def test_the_package_exports_what_the_stub_declares() -> None:
    assert esca.__all__ == stub_all("__init__.pyi")


def test_the_lichess_module_exports_what_its_stub_declares() -> None:
    assert esca.lichess.__all__ == stub_all("lichess.pyi")


def test_every_exported_name_exists() -> None:
    assert [name for name in esca.__all__ if not hasattr(esca, name)] == []


def test_the_extension_carries_every_name_the_stub_declares() -> None:
    assert declared_members(parsed("_esca.pyi").body) <= set(dir(_esca))
    assert set(stub_classes()) <= set(dir(_esca))


@pytest.mark.parametrize("name", sorted(stub_classes()))
def test_a_class_and_its_stub_declare_the_same_members(name: str) -> None:
    runtime = {member for member in dir(getattr(_esca, name)) if not member.startswith("_")}
    assert runtime == stub_classes()[name]


def test_the_package_is_typed() -> None:
    assert (PACKAGE / "py.typed").exists()
    assert (PACKAGE / "_esca.pyi").exists()
