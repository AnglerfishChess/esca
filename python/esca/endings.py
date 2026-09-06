"""Named endings: what material is left, which ending of the books it is, what
theory says the result is, and the method that gets it.

`classify(position)` answers with one `Ending`. Its class, verdict and
technique are objects, each carrying a one-sentence `describe()`; every one
compares equal to, and prints as, its `snake_case` name. The
position-specific facts that overturn the general case are grouped in
`EndingEvidence`, each behind the reason it belongs to.
"""

from ._esca import (
    Bishops,
    Ending,
    EndingClass,
    EndingEvidence,
    EndingTechnique,
    EndingVerdict,
    MaterialSignature,
    PawnRace,
)
from ._esca import endings_classify as classify

__all__ = [
    "Bishops",
    "Ending",
    "EndingClass",
    "EndingEvidence",
    "EndingTechnique",
    "EndingVerdict",
    "MaterialSignature",
    "PawnRace",
    "classify",
]
