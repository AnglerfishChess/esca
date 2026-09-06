"""Opening books in the Polyglot format: entries, books, a builder, and a download."""

import hashlib
import os
import tempfile
import urllib.request
from pathlib import Path

from ._esca import POLYGLOT_ENTRY_SIZE
from ._esca import PolyglotBook as Book
from ._esca import PolyglotBookIter as BookIter
from ._esca import PolyglotBuilder as Builder
from ._esca import PolyglotEntry as Entry
from ._esca import PolyglotRaw as Raw

#: How many bytes are read from the network at a time.
CHUNK = 1 << 16


def download(url: str, path: str | os.PathLike[str], *, sha256: str | None = None) -> Path:
    """Streams `url` into `path` and returns it.

    The bytes go to a temporary file in the same directory, which replaces
    `path` only once the whole stream has arrived and, where `sha256` is
    given, its hexadecimal digest matches. A stream that fails or that hashes
    to something else leaves `path` as it was.
    """
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    digest = hashlib.sha256()
    handle, temporary = tempfile.mkstemp(dir=path.parent, prefix=path.name, suffix=".part")
    try:
        with os.fdopen(handle, "wb") as out, urllib.request.urlopen(url) as response:
            while chunk := response.read(CHUNK):
                digest.update(chunk)
                out.write(chunk)
        if sha256 is not None and digest.hexdigest() != sha256.lower():
            raise ValueError(f"{url} hashes to {digest.hexdigest()}, not {sha256.lower()}")
        os.replace(temporary, path)
    except BaseException:
        Path(temporary).unlink(missing_ok=True)
        raise
    return path


__all__ = [
    "POLYGLOT_ENTRY_SIZE",
    "Book",
    "BookIter",
    "Builder",
    "Entry",
    "Raw",
    "download",
]
