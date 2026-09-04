import os
from pathlib import Path
from typing import Final

from ._esca import POLYGLOT_ENTRY_SIZE as POLYGLOT_ENTRY_SIZE
from ._esca import PolyglotBook as Book
from ._esca import PolyglotBookIter as BookIter
from ._esca import PolyglotBuilder as Builder
from ._esca import PolyglotEntry as Entry
from ._esca import PolyglotRaw as Raw

CHUNK: Final[int]

def download(url: str, path: str | os.PathLike[str], *, sha256: str | None = None) -> Path: ...

__all__ = [
    "POLYGLOT_ENTRY_SIZE",
    "Book",
    "BookIter",
    "Builder",
    "Entry",
    "Raw",
    "download",
]
