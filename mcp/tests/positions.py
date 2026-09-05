"""The positions the suite is written against, each named for what it shows."""

#: The opening position of classic chess.
START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

#: Kings and rooks alone, with a black bishop on b5 covering f1: White may
#: castle long and may not castle short.
CASTLING_ATTACKED = "r3k2r/8/8/1b6/8/8/8/R3K2R w KQkq - 0 1"

#: The same back rank with both bishops still home: every castling path is
#: blocked by White's own units.
CASTLING_BLOCKED = "r3k2r/8/8/8/8/8/8/RB2KB1R w KQkq - 0 1"

#: Black has just played d7-d5. White's e5 pawn may take it en passant, and
#: does so legally: nothing stands behind the two pawns.
EN_PASSANT = "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3"

#: White's king on a5 and a black rook on h5 share the fifth rank with both
#: pawns: exd6 en passant would clear the rank and expose the king.
EN_PASSANT_EXPOSES_KING = "8/8/8/K2pP2r/8/8/8/7k w - d6 0 1"

#: Black to move with nothing to move: a king on h8 boxed in by a queen on f7.
STALEMATE = "7k/5Q2/6K1/8/8/8/8/8 b - - 0 1"

#: Kings alone.
INSUFFICIENT_MATERIAL = "7k/8/6K1/8/8/8/8/8 w - - 0 1"

#: The moves of Fool's mate, the shortest mate there is.
FOOLS_MATE = ["f3", "e5", "g4", "Qh4#"]

#: Both knights out and back twice: the start position has now stood three
#: times, so a threefold repetition may be claimed.
THREEFOLD = ["Nf3", "Nf6", "Ng1", "Ng8", "Nf3", "Nf6", "Ng1", "Ng8"]

#: Not a FEN at all.
NOT_A_FEN = "nonsense"

#: A FEN whose placement is no legal chess position: no kings.
NO_KINGS = "8/8/8/8/8/8/8/8 w - - 0 1"

#: Fool's mate as a PGN, comment and result marker included.
FOOLS_MATE_PGN = """[Event "Test"]
[Site "?"]
[White "A"]
[Black "B"]
[Result "0-1"]

1. f3 e5 2. g4 {a blunder} Qh4# 0-1
"""
