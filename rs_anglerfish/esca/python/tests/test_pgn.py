"""PGN reading and writing, case by case.

Every expectation is worked out from the "Standard: Portable Game Notation
Specification and Implementation Guide" for the named text above it. The cases
mirror `tests/pgn.rs`.
"""

from __future__ import annotations

from itertools import pairwise
from pathlib import Path

import esca
import pytest
from esca import pgn

#: The longest movetext line export format writes.
EXPORT_WIDTH = 80

#: A seven-tag roster, four full moves, and a mate.
PLAIN = """[Event "Test"]
[Site "Amsterdam"]
[Date "2024.01.01"]
[Round "1"]
[White "Alice"]
[Black "Bob"]
[Result "1-0"]

1. e4 e5 2. Bc4 Nc6 3. Qh5 Nf6 4. Qxf7# 1-0
"""

#: Roster tags out of order, with one tag that is not in the roster.
UNORDERED = """[Black "Bob"]
[Event "Order"]
[Opening "Vienna Game"]
[White "Alice"]
[Result "1-0"]

1. e4 e5 2. Nc3 1-0
"""

#: A comment in every place one can stand: before the game, after a move, at
#: the head of a variation, and running to the end of a line.
COMMENTED = """[Event "Comments"]

{Before the game.} 1. e4 {after e4} ({a variation opens} 1. d4 d5 {after d5}) 1... e5 ;after e5
2. Nf3 *
"""

#: Variations three deep: an alternative first move, an alternative reply to
#: it, and an alternative to that reply's answer.
NESTED = """[Event "Nested"]

1. e4 (1. d4 d5 (1... Nf6 2. c4 (2. Nf3 g6)) 2. c4) 1... e5 2. Nf3 *
"""

#: Both glyph forms on one line: the `!`/`?` suffixes and `$` numbers.
GLYPHS = """[Event "Glyphs"]

1. e4! $10 e5 $2 2. Nf3?! Nc6 $13 *
"""

#: En passant, an underpromotion by capture, and castling on both wings.
SPECIALS = """[Event "Specials"]

1. e4 Nf6 2. e5 d5 3. exd6 e6 4. dxc7 Bd6 5. cxb8=N Rxb8 6. Nc3 O-O 7. b3 Re8
8. Bb2 h6 9. Qe2 a6 10. O-O-O *
"""

#: The knights on b3, b5 and f5 all reach d4, so the mover needs its square.
THREE_KNIGHTS = "4k3/8/8/1N3N2/8/1N6/8/4K3 w - - 0 1"

#: A Chess960 middlegame: the king stands on g1 with its rooks on f1 and h1,
#: so castling short moves the rook and leaves the king where it is.
NINE_SIXTY = """[Event "960"]
[Variant "Chess960"]
[SetUp "1"]
[FEN "bqnb1rkr/pp3ppp/3ppn2/2p5/5P2/P2P4/NPP1P1PP/BQ1BNRKR w KQkq - 2 9"]

1. Nf3 c4 2. Re1 cxd3 3. O-O *
"""

#: Anderssen-Kieseritzky, London 1851: long enough to wrap several times.
IMMORTAL = """[Event "Immortal"]
[Site "London"]
[Date "1851.06.21"]
[Round "?"]
[White "Anderssen, Adolf"]
[Black "Kieseritzky, Lionel"]
[Result "1-0"]

1. e4 e5 2. f4 exf4 3. Bc4 Qh4+ 4. Kf1 b5 5. Bxb5 Nf6 6. Nf3 Qh6 7. d3 Nh5
8. Nh4 Qg5 9. Nf5 c6 10. g4 Nf6 11. Rg1 cxb5 12. h4 Qg6 13. h5 Qg5 14. Qf3 Ng8
15. Bxf4 Qf6 16. Nc3 Bc5 17. Nd5 Qxb2 18. Bd6 Bxg1 19. e5 Qxa1+ 20. Ke2 Na6
21. Nxg7+ Kd8 22. Qf6+ Nxf6 23. Be7# 1-0
"""

#: Every tolerated liberty at once: an escape line, no tags, no result,
#: numbers glued to their moves, a `...` continuation, a comment over two
#: lines, and a `;` comment.
WILD = "%an escape line the reader drops\n1.e4 e5 2.Nf3 {a comment\nspanning lines} 2... Nc6\n;a line comment\n"

#: Three games, of which the second plays a move White has not got.
STREAM = """[Event "One"]

1. e4 e5 1-0

[Event "Two"]

1. e4 Nf6 2. Nf6 0-1

[Event "Three"]

1. d4 d5 1/2-1/2
"""


def one(text: str) -> pgn.Game:
    """The one game `text` holds."""
    games = list(pgn.read_string(text))
    assert len(games) == 1, "the text holds exactly one game"
    return games[0]


def sans(nodes: list[pgn.Node]) -> str:
    """The move text of a line, space separated."""
    return " ".join(node.san for node in nodes)


def shape(nodes: list[pgn.Node]) -> list[object]:
    """The tree as plain data: move text, glyphs, comments and variations."""
    return [
        (node.san, node.nags, node.comment_before, node.comment_after, [shape(line) for line in node.variations])
        for node in nodes
    ]


def movetext(text: str) -> list[str]:
    """Everything after the blank line that ends the tag section."""
    return text.split("\n\n")[1].splitlines()


def first_token(line: str) -> str:
    """The first token of a movetext line.

    A move number and its move are one token, so a line break never falls
    between them.
    """
    words = line.split(" ")
    numbered = words[0].endswith(".") and set(words[0].lstrip("(")) <= set("0123456789.")
    return f"{words[0]} {words[1]}" if numbered else words[0]


def test_a_plain_game_reads_its_headers_moves_and_result() -> None:
    game = one(PLAIN)
    assert game.headers["Event"] == "Test"
    assert game.headers["Black"] == "Bob"
    assert "Annotator" not in game.headers
    assert sans(game.mainline()) == "e4 e5 Bc4 Nc6 Qh5 Nf6 Qxf7#"
    assert game.result == "1-0"
    assert game.game().ply == 7


def test_a_game_starts_from_the_variant_start_position_unless_a_fen_says_otherwise() -> None:
    game = one(PLAIN)
    assert game.variant == esca.CLASSIC
    assert game.start_position == esca.CLASSIC.start_position()

    game = one(NINE_SIXTY)
    assert game.variant == esca.CHESS960
    assert game.start_position.fullmove_number == 9


def test_the_seven_tag_roster_is_written_first_and_the_rest_keep_their_order() -> None:
    game = one(UNORDERED)
    assert list(game.headers) == ["Black", "Event", "Opening", "White", "Result"]

    written = [line.split(" ")[0][1:] for line in game.to_string().splitlines() if line.startswith("[")]
    assert written == ["Event", "White", "Black", "Result", "Opening"]


def test_a_comment_is_kept_wherever_it_stands() -> None:
    game = one(COMMENTED)
    assert game.comment == "Before the game."
    assert game.mainline()[0].comment_after == "after e4"
    assert game.mainline()[1].comment_after == "after e5"

    variation = game.mainline()[0].variations[0]
    assert sans(variation) == "d4 d5"
    assert variation[0].comment_before == "a variation opens"
    assert variation[1].comment_after == "after d5"


def test_a_comment_spanning_lines_becomes_one_line_of_words() -> None:
    game = one(WILD)
    assert game.mainline()[2].comment_after == "a comment spanning lines"
    assert game.mainline()[3].comment_after == "a line comment"


def test_variations_nest_to_any_depth() -> None:
    game = one(NESTED)
    assert sans(game.mainline()) == "e4 e5 Nf3"

    first = game.mainline()[0].variations[0]
    assert sans(first) == "d4 d5 c4"
    second = first[1].variations[0]
    assert sans(second) == "Nf6 c4"
    third = second[1].variations[0]
    assert sans(third) == "Nf3 g6"
    assert third[1].variations == []


@pytest.mark.parametrize(
    ("text", "nags"),
    [
        ("1. e4! *", [1]),
        ("1. e4? *", [2]),
        ("1. e4!! *", [3]),
        ("1. e4?? *", [4]),
        ("1. e4!? *", [5]),
        ("1. e4?! *", [6]),
        ("1. e4 $14 *", [14]),
        ("1. e4! $14 *", [1, 14]),
    ],
    ids=["good", "poor", "very_good", "blunder", "speculative", "dubious", "numeric", "both"],
)
def test_a_glyph_is_read_in_either_form_and_kept_as_a_number(text: str, nags: list[int]) -> None:
    game = one(text)
    assert game.mainline()[0].san == "e4"
    assert game.mainline()[0].nags == nags


def test_glyphs_stay_with_the_moves_they_annotate() -> None:
    game = one(GLYPHS)
    assert [node.nags for node in game.mainline()] == [[1, 10], [2], [6], [13]]
    assert sans(game.mainline()) == "e4 e5 Nf3 Nc6"


def test_promotion_castling_and_en_passant_keep_their_text() -> None:
    game = one(SPECIALS)
    assert sans(game.mainline()) == ("e4 Nf6 e5 d5 exd6 e6 dxc7 Bd6 cxb8=N Rxb8 Nc3 O-O b3 Re8 Bb2 h6 Qe2 a6 O-O-O")
    assert game.mainline()[4].move.is_en_passant
    assert game.mainline()[8].move.promotion == "n"
    assert game.mainline()[11].move.is_castling
    assert game.mainline()[18].move.is_castling


@pytest.mark.parametrize(
    ("fen", "san", "origin"),
    [
        ("4k3/8/8/1N3N2/8/8/8/4K3 w - - 0 1", "Nbd4", "b5"),
        ("4k3/8/8/1N6/8/1N6/8/4K3 w - - 0 1", "N5d4", "b5"),
        (THREE_KNIGHTS, "Nb5d4", "b5"),
    ],
    ids=["file", "rank", "square"],
)
def test_disambiguation_names_the_mover_it_has_to(fen: str, san: str, origin: str) -> None:
    text = f'[SetUp "1"]\n[FEN "{fen}"]\n\n1. {san} *\n'
    game = one(text)
    assert game.mainline()[0].san == san
    assert game.mainline()[0].move.origin == origin
    assert game.to_string() == text


def test_a_chess960_game_castles_king_to_rook() -> None:
    game = one(NINE_SIXTY)
    assert sans(game.mainline()) == "Nf3 c4 Re1 cxd3 O-O"
    castling = game.mainline()[4].move
    assert castling.is_castling
    assert (castling.origin, castling.destination) == ("g1", "h1")

    played = game.game()
    assert played.position.king_of("w") == "g1"
    assert played.to_pgn().headers["FEN"] == ("bqnb1rkr/pp3ppp/3ppn2/2p5/5P2/P2P4/NPP1P1PP/BQ1BNRKR w HFhf - 2 9")


@pytest.mark.parametrize(
    ("text", "result"),
    [
        ("1. e4 e5 1-0", "1-0"),
        ("1. e4 e5 0-1", "0-1"),
        ("1. e4 e5 1/2-1/2", "1/2-1/2"),
        ("1. e4 e5 *", "*"),
        ('[Result "0-1"]\n\n1. e4 e5', "0-1"),
        ("1. e4 e5", "*"),
    ],
    ids=["white", "black", "draw", "unknown", "missing", "missing_and_untagged"],
)
def test_the_result_is_the_marker_or_the_tag_that_stands_in_for_it(text: str, result: str) -> None:
    game = one(text)
    assert game.result == result
    assert game.to_string().rstrip().endswith(result)


def test_a_long_game_wraps_at_the_export_width() -> None:
    lines = movetext(one(IMMORTAL).to_string())
    assert len(lines) > 3, "the game needs several lines"
    for line in lines:
        assert len(line) <= EXPORT_WIDTH, f"too long: {line}"
    for previous, following in pairwise(lines):
        carried = first_token(following)
        assert len(previous) + 1 + len(carried) > EXPORT_WIDTH, f"{carried} still fits after {previous}"


def test_a_move_number_stays_with_its_move() -> None:
    assert movetext(one(NESTED).to_string()) == ["1. e4 (1. d4 d5 (1... Nf6 2. c4 (2. Nf3 g6)) 2. c4) 1... e5 2. Nf3 *"]


@pytest.mark.parametrize(
    "text",
    [PLAIN, UNORDERED, COMMENTED, NESTED, GLYPHS, SPECIALS, NINE_SIXTY, IMMORTAL, WILD],
    ids=["plain", "unordered", "commented", "nested", "glyphs", "specials", "nine_sixty", "immortal", "wild"],
)
def test_writing_a_game_and_reading_it_back_changes_nothing(text: str) -> None:
    game = one(text)
    written = game.to_string()
    again = one(written)
    # Writing puts the tag pairs in export order, which reading keeps.
    assert dict(again.headers) == dict(game.headers)
    assert again.comment == game.comment
    assert shape(again.mainline()) == shape(game.mainline())
    assert again.result == game.result
    assert again.to_string() == written


@pytest.mark.parametrize(
    ("text", "line", "column", "message"),
    [
        ('[Event "Bad"]\n\n1. e4 {oops\n', 3, 7, "unterminated comment"),
        ('[Event "Bad"]\n\n1. e4 e5\n2. Nf6\n', 4, 4, "no such legal move: Nf6"),
        ('[Event "Bad"]\n[Variant "Atomic"]\n\n1. e4 *\n', 2, 1, "unknown variant: Atomic"),
        ('[SetUp "1"]\n[FEN "nonsense"]\n\n1. e4 *\n', 2, 1, "a FEN has four or six fields"),
        ('[Event "Bad"]\n\n1. e4 (1. d4 *\n', 3, 15, "unterminated variation"),
        ('[Event "Bad]\n\n1. e4 *\n', 1, 1, "unterminated tag value"),
    ],
    ids=[
        "unterminated_comment",
        "illegal_move",
        "unknown_variant",
        "unreadable_fen",
        "unterminated_variation",
        "unterminated_tag",
    ],
)
def test_malformed_text_is_an_error_at_its_line_and_column(text: str, line: int, column: int, message: str) -> None:
    with pytest.raises(ValueError, match=f"^line {line}, column {column}: ") as raised:
        list(pgn.read_string(text))
    assert message in str(raised.value)


def test_a_bad_game_does_not_stop_the_stream() -> None:
    reader = pgn.read_string(STREAM)
    assert next(reader).headers["Event"] == "One"
    with pytest.raises(ValueError, match="no such legal move"):
        next(reader)
    assert next(reader).headers["Event"] == "Three"

    kept = list(pgn.read_string(STREAM, skip_errors=True))
    assert [game.headers["Event"] for game in kept] == ["One", "Three"]


def test_a_thousand_games_stream_one_at_a_time(tmp_path: Path) -> None:
    path = tmp_path / "thousand.pgn"
    path.write_text(
        "".join(
            f'[Event "Generated"]\n[Round "{round_}"]\n\n1. e4 e5 2. Nf3 Nc6 1/2-1/2\n\n' for round_ in range(1, 1001)
        )
    )

    seen = 0
    five_hundredth = ""
    for game in pgn.read(path):
        seen += 1
        if seen == 500:
            five_hundredth = game.headers["Round"]
        assert game.result == "1/2-1/2"
    assert seen == 1000
    assert five_hundredth == "500"
    assert pgn.count(path) == 1000


def test_a_played_game_becomes_pgn_and_the_pgn_plays_it_back() -> None:
    played = esca.Game()
    for san in ("e4", "e5", "Nf3", "Nc6"):
        played.play_san(san)
    game = played.to_pgn()
    assert game.headers["Event"] == "?"
    assert game.headers["Date"] == "????.??.??"
    assert game.headers["Result"] == "*"
    assert "FEN" not in game.headers
    assert sans(game.mainline()) == "e4 e5 Nf3 Nc6"
    assert game.game().position == played.position
    assert game == pgn.Game.from_game(played)


def test_a_chess960_start_is_written_as_a_fen_tag() -> None:
    game = esca.Game(variant=esca.CHESS960, seed=1).to_pgn()
    assert game.headers["Variant"] == "Chess960"
    assert game.headers["SetUp"] == "1"
    assert game.headers["FEN"] == esca.CHESS960.start_position(1).fen
    assert game.variant == esca.CHESS960
