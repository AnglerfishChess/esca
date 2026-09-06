"""The Polyglot key and the book format, case by case.

Every key is the one the format's own description publishes for the line above
it; every move encoding is worked out from the format's bit layout,
destination file and rank, origin file and rank, then promotion role. The
cases mirror `tests/polyglot.rs`; `download` has no Rust counterpart.
"""

from __future__ import annotations

import hashlib
from pathlib import Path

import esca
import pytest
from esca import polyglot

#: A book of five entries at two of the format's published keys: three moves
#: of the starting position, one thing that is not a move of it, and one reply
#: to 1. e4.
TINY = Path(__file__).resolve().parents[2] / "tests" / "data" / "tiny.bin"

#: The starting array, whose key the format publishes as `463b96181691fc9c`.
START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

#: A black pawn that has just advanced two squares with a white pawn beside
#: it, and the white king and a black rook sharing the pawns' rank: the
#: capture would uncover the king, so it is not legal, but the pawn stands
#: beside the target all the same.
PINNED_CAPTURE = "4k3/8/8/r2pP2K/8/8/8/8 w - d6 0 2"

#: The same double advance with the white pawn three files away, so no pawn
#: stands beside it.
DISTANT_PAWN = "4k3/8/8/3p3P/8/8/8/4K3 w - d6 0 2"

#: White to castle either way, with its rooks on the classic files.
CLASSIC_CASTLING = "4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1"

#: A Chess960 array with the kings on b1 and b8 between rooks on the a- and
#: h-files, so castling short takes the king across the board and brings the
#: rook back past it, and castling long moves the king one square.
NINE_SIXTY_CASTLING = "rk5r/8/8/8/8/8/8/RK5R w KQkq - 0 1"

#: A Chess960 array whose rooks start on the c- and a-files, so no castling
#: right of it is spelled the classic way.
NINE_SIXTY_ALL = "rkr5/8/8/8/8/8/8/RKR5 w CAca - 0 1"

#: A white pawn one square from promoting.
PROMOTION = "4k3/P7/8/8/8/8/8/4K3 w - - 0 1"

#: Two games that share their first move and part on the reply.
SHARED_OPENING = ("e2e4 e7e5 g1f3", "e2e4 d7d5 e4d5")

#: Two games as PGN, one of them unreadable.
PGN = '[Event "One"]\n\n1. e4 e5 1-0\n\n[Event "Two"]\n\n1. e4 Nf6 2. Nf6 0-1\n\n[Event "Three"]\n\n1. e4 c5 1/2-1/2\n'


def position(fen: str) -> esca.Position:
    """The position `fen` describes."""
    return esca.Position.from_fen(fen)


def played(moves: str) -> esca.Game:
    """The classic game the space-separated UCI `moves` reach."""
    game = esca.Game()
    for text in moves.split():
        game.play(text)
    return game


def move_of(fen: str, uci: str, variant: esca.Variant = esca.CLASSIC) -> esca.Move:
    """The one legal move of `fen` written `uci`, castling king-to-rook."""
    game = esca.Game.from_fen(fen, variant=variant)
    for mv in game.legal_moves():
        if mv.uci == uci:
            return mv
    raise AssertionError(f"{uci} is not a legal move of {fen}")


def tiny() -> polyglot.Book:
    """The book checked in as `TINY`."""
    return polyglot.Book(TINY)


def with_field(fen: str, index: int, value: str) -> esca.Position:
    """The same FEN with one field replaced."""
    fields = fen.split(" ")
    fields[index] = value
    return position(" ".join(fields))


@pytest.mark.parametrize(
    ("moves", "key"),
    [
        pytest.param("", "463b96181691fc9c", id="start"),
        pytest.param("e2e4", "823c9b50fd114196", id="e4"),
        pytest.param("e2e4 d7d5", "0756b94461c50fb0", id="d5"),
        pytest.param("e2e4 d7d5 e4e5", "662fafb965db29d4", id="e5"),
        pytest.param("e2e4 d7d5 e4e5 f7f5", "22a48b5a8e47ff78", id="en_passant"),
        pytest.param("e2e4 d7d5 e4e5 f7f5 e1e2", "652a607ca3f242c1", id="king_moved"),
        pytest.param("e2e4 d7d5 e4e5 f7f5 e1e2 e8f7", "00fdd303c946bdd9", id="both_kings_moved"),
        pytest.param("a2a4 b7b5 h2h4 b5b4 c2c4", "3c8123ea7b067637", id="pawn_taken_beside"),
        pytest.param("a2a4 b7b5 h2h4 b5b4 c2c4 b4c3 a1a3", "5c3f9b829b279560", id="en_passant_played"),
    ],
)
def test_a_line_has_the_key_the_format_publishes(moves: str, key: str) -> None:
    assert f"{played(moves).position.polyglot_key:016x}" == key


def test_the_en_passant_file_is_keyed_when_a_pawn_stands_beside_the_target() -> None:
    assert position(PINNED_CAPTURE).polyglot_key != with_field(PINNED_CAPTURE, 3, "-").polyglot_key


def test_a_capture_that_would_uncover_the_king_is_still_a_pawn_standing_beside() -> None:
    game = esca.Game.from_fen(PINNED_CAPTURE)
    assert [mv.uci for mv in game.legal_moves() if mv.uci == "e5d6"] == []
    assert game.position.polyglot_key != with_field(PINNED_CAPTURE, 3, "-").polyglot_key


def test_an_en_passant_square_no_pawn_stands_beside_is_not_keyed() -> None:
    assert position(DISTANT_PAWN).polyglot_key == with_field(DISTANT_PAWN, 3, "-").polyglot_key


def test_white_to_move_is_the_one_published_turn_constant() -> None:
    white = position(START)
    black = with_field(START, 1, "b")
    assert white.polyglot_key ^ black.polyglot_key == 0xF8D626AAAF278509


def test_the_clocks_are_no_part_of_the_key() -> None:
    worn = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 37 42"
    assert position(START).polyglot_key == position(worn).polyglot_key


@pytest.mark.parametrize(
    ("classic_rights", "shuffled_rights"),
    [
        pytest.param("Qkq", "Aca", id="white_short"),
        pytest.param("Kkq", "Cca", id="white_long"),
        pytest.param("KQq", "CAa", id="black_short"),
        pytest.param("KQk", "CAc", id="black_long"),
    ],
)
def test_a_castling_right_is_one_constant_whatever_file_its_rook_starts_on(
    classic_rights: str, shuffled_rights: str
) -> None:
    classic_difference = position(START).polyglot_key ^ with_field(START, 2, classic_rights).polyglot_key
    shuffled_difference = (
        position(NINE_SIXTY_ALL).polyglot_key ^ with_field(NINE_SIXTY_ALL, 2, shuffled_rights).polyglot_key
    )
    assert classic_difference == shuffled_difference


def test_the_book_holds_every_entry_the_file_does() -> None:
    book = tiny()
    assert len(book) == 5
    assert len(list(book)) == 5


@pytest.mark.parametrize(
    ("index", "key", "uci", "weight", "learn"),
    [
        pytest.param(0, "463b96181691fc9c", "e2e4", 100, 0, id="e4"),
        pytest.param(1, "463b96181691fc9c", "d2d4", 50, 42, id="d4"),
        pytest.param(2, "463b96181691fc9c", "e2e5", 25, 0, id="not_a_move"),
        pytest.param(3, "463b96181691fc9c", "g1f3", 0, 0, id="nf3"),
        pytest.param(4, "823c9b50fd114196", "e7e5", 7, 0, id="e5"),
    ],
)
def test_an_entry_reads_back_the_four_things_the_file_holds(
    index: int, key: str, uci: str, weight: int, learn: int
) -> None:
    entry = tiny().get(index)
    assert entry is not None
    assert f"{entry.key:016x}" == key
    assert entry.uci == uci
    assert entry.weight == weight
    assert entry.learn == learn


def test_an_index_past_the_end_is_no_entry() -> None:
    assert tiny().get(5) is None


def test_the_entries_of_a_position_keep_the_order_the_file_gives_them() -> None:
    entries = tiny().entries(position(START))
    assert [entry.move.uci for entry in entries] == ["e2e4", "d2d4", "g1f3"]


def test_an_entry_naming_no_legal_move_of_the_position_is_refused() -> None:
    start = position(START)
    raw = tiny().raw_entries(start.polyglot_key)
    assert len(raw) == 4
    assert raw[2].uci == "e2e5"
    assert raw[2].decode(start) is None


def test_bits_that_name_no_move_are_no_move() -> None:
    # Promotion code 5, which the format does not define.
    raw = polyglot.Raw(0, 0x531C)
    assert raw.uci is None
    assert raw.decode(position(START)) is None


def test_a_key_the_book_does_not_hold_has_no_entries() -> None:
    book = tiny()
    after_d4 = played("d2d4").position
    assert book.raw_entries(after_d4.polyglot_key) == []
    assert book.entries(after_d4) == []
    assert book.best(after_d4) is None
    assert book.pick(after_d4, 0) is None


def test_the_heaviest_entry_is_the_best_one() -> None:
    best = tiny().best(position(START))
    assert best is not None
    assert best.move.uci == "e2e4"
    assert best.weight == 100


@pytest.mark.parametrize(
    ("seed", "uci"),
    [
        pytest.param(0, "e2e4", id="first_share"),
        pytest.param(99, "e2e4", id="last_of_the_first_share"),
        pytest.param(100, "d2d4", id="second_share"),
        pytest.param(149, "d2d4", id="last_of_the_second_share"),
        pytest.param(150, "e2e4", id="wrapped"),
        pytest.param(1_000_000_099, "e2e4", id="far_wrapped"),
    ],
)
def test_a_pick_is_the_entry_the_seed_falls_in(seed: int, uci: str) -> None:
    # The weights are 100, 50 and 0, so the draw is taken modulo 150 and the
    # move weighed nothing is never drawn.
    picked = tiny().pick(position(START), seed)
    assert picked is not None
    assert picked.move.uci == uci


def test_a_book_written_reads_back_what_was_written(tmp_path: Path) -> None:
    start = position(START)
    entries = [
        polyglot.Entry(start.polyglot_key, move_of(START, "e2e4"), 9, 7),
        polyglot.Entry(start.polyglot_key, move_of(START, "d2d4"), 4, 0),
    ]
    path = tmp_path / "round-trip.bin"
    polyglot.Book.write(path, entries)

    book = polyglot.Book(path)
    assert len(book) == 2
    assert book.entries(start) == entries


def test_entries_that_share_a_key_and_a_move_are_merged(tmp_path: Path) -> None:
    start = position(START)
    e4 = move_of(START, "e2e4")
    path = tmp_path / "merged.bin"
    polyglot.Book.write(
        path,
        [polyglot.Entry(start.polyglot_key, e4, 3, 5), polyglot.Entry(start.polyglot_key, e4, 4, 0)],
    )

    book = polyglot.Book(path)
    assert len(book) == 1
    entry = book.get(0)
    assert entry is not None
    assert entry.weight == 7


def test_a_file_that_is_not_whole_entries_is_not_a_book(tmp_path: Path) -> None:
    path = tmp_path / "ragged.bin"
    path.write_bytes(bytes(polyglot.POLYGLOT_ENTRY_SIZE + 1))
    with pytest.raises(OSError, match="entries"):
        polyglot.Book(path)


def test_an_empty_file_is_an_empty_book() -> None:
    assert len(polyglot.Book.from_bytes(b"")) == 0


@pytest.mark.parametrize(
    ("variant", "fen", "uci"),
    [
        pytest.param(esca.CLASSIC, CLASSIC_CASTLING, "e1h1", id="classic_short"),
        pytest.param(esca.CLASSIC, CLASSIC_CASTLING, "e1a1", id="classic_long"),
        pytest.param(esca.CHESS960, NINE_SIXTY_CASTLING, "b1h1", id="chess960_short"),
        pytest.param(esca.CHESS960, NINE_SIXTY_CASTLING, "b1a1", id="chess960_long"),
    ],
)
def test_castling_is_written_king_takes_rook(variant: esca.Variant, fen: str, uci: str) -> None:
    board = position(fen)
    castling = move_of(fen, uci, variant)
    assert castling.is_castling
    entry = polyglot.Entry(board.polyglot_key, castling)
    written = polyglot.Raw(entry.key, entry.bits)
    assert written.uci == uci
    decoded = written.decode(board, variant=variant)
    assert decoded is not None
    assert decoded.move == castling


@pytest.mark.parametrize(
    ("uci", "bits"),
    [
        pytest.param("a7a8q", 0x4C38, id="queen"),
        pytest.param("a7a8r", 0x3C38, id="rook"),
        pytest.param("a7a8b", 0x2C38, id="bishop"),
        pytest.param("a7a8n", 0x1C38, id="knight"),
    ],
)
def test_a_promotion_carries_the_role_in_its_top_bits(uci: str, bits: int) -> None:
    board = position(PROMOTION)
    entry = polyglot.Entry(board.polyglot_key, move_of(PROMOTION, uci))
    assert entry.bits == bits
    assert polyglot.Raw(board.polyglot_key, bits).uci == uci


def test_a_builder_weighs_a_move_by_how_many_games_played_it(tmp_path: Path) -> None:
    builder = polyglot.Builder()
    for moves in SHARED_OPENING:
        builder.add_game(played(moves))
    path = tmp_path / "counted.bin"
    builder.write(path)

    book = polyglot.Book(path)
    opening = book.entries(position(START))
    assert [entry.move.uci for entry in opening] == ["e2e4"]
    assert opening[0].weight == 2

    replies = book.entries(played("e2e4").position)
    assert [entry.move.uci for entry in replies] == ["d7d5", "e7e5"]
    assert [entry.weight for entry in replies] == [1, 1]


def test_a_builder_counts_no_move_past_its_maximum_ply() -> None:
    builder = polyglot.Builder(max_ply=2)
    for moves in SHARED_OPENING:
        builder.add_game(played(moves))
    # The shared first move and the two replies, and nothing of ply three.
    assert len(builder) == 3
    third_ply = played("e2e4 e7e5").position.polyglot_key
    assert [entry for entry in builder.entries() if entry.key == third_ply] == []


def test_a_builder_drops_a_move_too_few_games_played() -> None:
    builder = polyglot.Builder(min_count=2)
    for moves in SHARED_OPENING:
        builder.add_game(played(moves))
    entries = builder.entries()
    assert len(entries) == 1
    assert entries[0].key == position(START).polyglot_key
    assert entries[0].weight == 2


def test_a_builder_is_empty_until_it_is_given_a_game() -> None:
    builder = polyglot.Builder()
    assert len(builder) == 0
    assert builder.entries() == []


def test_a_builder_reads_every_game_a_pgn_source_holds() -> None:
    builder = polyglot.Builder()
    # The middle game plays a move White has not got, and is skipped.
    assert builder.add_pgn_string(PGN) == 2

    start = position(START).polyglot_key
    opening = [entry for entry in builder.entries() if entry.key == start]
    assert [entry.uci for entry in opening] == ["e2e4"]
    assert opening[0].weight == 2


def test_a_game_of_another_variant_is_keyed_by_the_same_rules() -> None:
    game = esca.Game.from_fen(NINE_SIXTY_CASTLING, variant=esca.CHESS960)
    before = game.position.polyglot_key
    game.play("b1h1")
    assert game.position.polyglot_key != before


def test_a_download_streams_the_bytes_to_the_path(tmp_path: Path) -> None:
    source = tmp_path / "source.bin"
    source.write_bytes(TINY.read_bytes())
    target = tmp_path / "books" / "tiny.bin"

    assert polyglot.download(source.as_uri(), target) == target
    assert target.read_bytes() == TINY.read_bytes()
    assert len(polyglot.Book(target)) == 5


def test_a_download_checks_the_digest_it_is_given(tmp_path: Path) -> None:
    source = tmp_path / "source.bin"
    source.write_bytes(b"a book of no entries at all"[:16])
    target = tmp_path / "checked.bin"
    digest = hashlib.sha256(source.read_bytes()).hexdigest()

    polyglot.download(source.as_uri(), target, sha256=digest)
    assert target.read_bytes() == source.read_bytes()


def test_a_download_that_hashes_to_something_else_leaves_the_path_alone(tmp_path: Path) -> None:
    source = tmp_path / "source.bin"
    source.write_bytes(bytes(16))
    target = tmp_path / "kept.bin"
    target.write_bytes(b"what was there before")

    with pytest.raises(ValueError, match="hashes to"):
        polyglot.download(source.as_uri(), target, sha256="00" * 32)
    assert target.read_bytes() == b"what was there before"
    assert list(target.parent.glob("*.part")) == []
