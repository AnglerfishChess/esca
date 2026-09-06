"""The bundled ECO catalogue: the code and name of a named position."""

from ._esca import Opening
from ._esca import openings_count as count
from ._esca import openings_lookup as lookup

__all__ = ["Opening", "count", "lookup"]
