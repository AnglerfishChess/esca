"""The UCI protocol as values, with no I/O.

`Command` writes what a client says; `parse` reads one line of what an engine
says; `Session` says which of them may come next.
"""

from .._esca import Answer, Command, Info, Limits, Message, Option, Session
from .._esca import uci_parse as parse

__all__ = ["Answer", "Command", "Info", "Limits", "Message", "Option", "Session", "parse"]
