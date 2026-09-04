//! The Polyglot key and the book format, case by case.
//!
//! Every key is the one the format's own description publishes for the line
//! above it; every move encoding is worked out from the format's bit layout,
//! destination file and rank, origin file and rank, then promotion role.

#![cfg(feature = "polyglot")]

use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use esca::polyglot::{Book, Builder, ENTRY_SIZE, Entry, Raw};
use esca::{CHESS960, CLASSIC, Game, Move, Position, Variant, chess960, classic};
use rstest::rstest;

/// A book of five entries at two of the format's published keys: three moves
/// of the starting position, one thing that is not a move of it, and one
/// reply to 1. e4.
const TINY: &str = "tests/data/tiny.bin";

/// The starting array, whose key the format publishes as `463b96181691fc9c`.
const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// A black pawn that has just advanced two squares with a white pawn beside
/// it, and the white king and a black rook sharing the pawns' rank: the
/// capture would uncover the king, so it is not legal, but the pawn stands
/// beside the target all the same.
const PINNED_CAPTURE: &str = "4k3/8/8/r2pP2K/8/8/8/8 w - d6 0 2";

/// The same double advance with the white pawn three files away, so no pawn
/// stands beside it.
const DISTANT_PAWN: &str = "4k3/8/8/3p3P/8/8/8/4K3 w - d6 0 2";

/// White to castle either way, with its rooks on the classic files.
const CLASSIC_CASTLING: &str = "4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1";

/// A Chess960 array with the kings on b1 and b8 between rooks on the a- and
/// h-files, so castling short takes the king across the board and brings the
/// rook back past it, and castling long moves the king one square.
const NINE_SIXTY_CASTLING: &str = "rk5r/8/8/8/8/8/8/RK5R w KQkq - 0 1";

/// A white pawn one square from promoting.
const PROMOTION: &str = "4k3/P7/8/8/8/8/8/4K3 w - - 0 1";

/// Two games that share their first move and part on the reply.
const SHARED_OPENING: [&str; 2] = ["e2e4 e7e5 g1f3", "e2e4 d7d5 e4d5"];

/// Two games as PGN, one of them unreadable.
const PGN: &str = "[Event \"One\"]\n\n1. e4 e5 1-0\n\n\
                   [Event \"Two\"]\n\n1. e4 Nf6 2. Nf6 0-1\n\n\
                   [Event \"Three\"]\n\n1. e4 c5 1/2-1/2\n";

/// The position `fen` describes.
fn position(fen: &str) -> Position {
    Position::from_fen(fen).expect("a test FEN is a legal position")
}

/// The classic game the space-separated UCI `moves` reach.
fn played(moves: &str) -> Game {
    let mut game = Game::new(classic());
    for text in moves.split_whitespace() {
        game.play_uci(text)
            .unwrap_or_else(|_| panic!("{text} is legal"));
    }
    game
}

/// The one legal move of `position` written `uci`, castling king-to-rook.
fn move_of(variant: &dyn Variant, position: &Position, uci: &str) -> Move {
    variant
        .move_from_uci(position, uci)
        .unwrap_or_else(|_| panic!("{uci} is a legal move"))
}

/// The book checked in as `TINY`.
fn tiny() -> Book {
    Book::open(&Path::new(env!("CARGO_MANIFEST_DIR")).join(TINY)).expect("the fixture is a book")
}

/// A path of this test's own, in the temporary directory.
fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("esca-polyglot-{}-{name}.bin", std::process::id()))
}

/// The same FEN with its castling field replaced.
fn with_rights(fen: &str, rights: &str) -> Position {
    let mut fields: Vec<&str> = fen.split(' ').collect();
    fields[2] = rights;
    position(&fields.join(" "))
}

#[rstest]
#[case::start("", "463b96181691fc9c")]
#[case::e4("e2e4", "823c9b50fd114196")]
#[case::d5("e2e4 d7d5", "0756b94461c50fb0")]
#[case::e5("e2e4 d7d5 e4e5", "662fafb965db29d4")]
#[case::en_passant("e2e4 d7d5 e4e5 f7f5", "22a48b5a8e47ff78")]
#[case::king_moved("e2e4 d7d5 e4e5 f7f5 e1e2", "652a607ca3f242c1")]
#[case::both_kings_moved("e2e4 d7d5 e4e5 f7f5 e1e2 e8f7", "00fdd303c946bdd9")]
#[case::pawn_taken_beside("a2a4 b7b5 h2h4 b5b4 c2c4", "3c8123ea7b067637")]
#[case::en_passant_played("a2a4 b7b5 h2h4 b5b4 c2c4 b4c3 a1a3", "5c3f9b829b279560")]
fn a_line_has_the_key_the_format_publishes(#[case] moves: &str, #[case] key: &str) {
    assert_eq!(
        format!("{:016x}", played(moves).position().polyglot_key()),
        key
    );
}

#[test]
fn the_en_passant_file_is_keyed_when_a_pawn_stands_beside_the_target() {
    let with = position(PINNED_CAPTURE);
    let without = with_ep(PINNED_CAPTURE, "-");
    assert_ne!(with.polyglot_key(), without.polyglot_key());
}

#[test]
fn a_capture_that_would_uncover_the_king_is_still_a_pawn_standing_beside() {
    let position = position(PINNED_CAPTURE);
    assert!(CLASSIC.move_from_uci(&position, "e5d6").is_err());
    assert_ne!(
        position.polyglot_key(),
        with_ep(PINNED_CAPTURE, "-").polyglot_key()
    );
}

#[test]
fn an_en_passant_square_no_pawn_stands_beside_is_not_keyed() {
    assert_eq!(
        position(DISTANT_PAWN).polyglot_key(),
        with_ep(DISTANT_PAWN, "-").polyglot_key()
    );
}

/// The same FEN with its en-passant field replaced.
fn with_ep(fen: &str, square: &str) -> Position {
    let mut fields: Vec<&str> = fen.split(' ').collect();
    fields[3] = square;
    position(&fields.join(" "))
}

#[test]
fn white_to_move_is_the_one_published_turn_constant() {
    let white = position(START);
    let mut fields: Vec<&str> = START.split(' ').collect();
    fields[1] = "b";
    let black = position(&fields.join(" "));
    assert_eq!(
        white.polyglot_key() ^ black.polyglot_key(),
        0xf8d6_26aa_af27_8509
    );
}

#[test]
fn the_clocks_are_no_part_of_the_key() {
    let fresh = position(START);
    let worn = position("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 37 42");
    assert_eq!(fresh.polyglot_key(), worn.polyglot_key());
}

#[rstest]
#[case::white_short("Qkq", "Aca")]
#[case::white_long("Kkq", "Cca")]
#[case::black_short("KQq", "CAa")]
#[case::black_long("KQk", "CAc")]
fn a_castling_right_is_one_constant_whatever_file_its_rook_starts_on(
    #[case] classic_rights: &str,
    #[case] shuffled_rights: &str,
) {
    let classic_all = position(START);
    let classic_less = with_rights(START, classic_rights);
    let shuffled_all = position(NINE_SIXTY_ALL);
    let shuffled_less = with_rights(NINE_SIXTY_ALL, shuffled_rights);
    assert_eq!(
        classic_all.polyglot_key() ^ classic_less.polyglot_key(),
        shuffled_all.polyglot_key() ^ shuffled_less.polyglot_key()
    );
}

/// A Chess960 array whose rooks start on the c- and a-files, so no castling
/// right of it is spelled the classic way.
const NINE_SIXTY_ALL: &str = "rkr5/8/8/8/8/8/8/RKR5 w CAca - 0 1";

#[test]
fn the_book_holds_every_entry_the_file_does() {
    let book = tiny();
    assert_eq!(book.len(), 5);
    assert!(!book.is_empty());
    assert_eq!(book.iter().count(), 5);
}

#[rstest]
#[case::e4(0, "463b96181691fc9c", "e2e4", 100, 0)]
#[case::d4(1, "463b96181691fc9c", "d2d4", 50, 42)]
#[case::not_a_move(2, "463b96181691fc9c", "e2e5", 25, 0)]
#[case::nf3(3, "463b96181691fc9c", "g1f3", 0, 0)]
#[case::e5(4, "823c9b50fd114196", "e7e5", 7, 0)]
fn an_entry_reads_back_the_four_things_the_file_holds(
    #[case] index: usize,
    #[case] key: &str,
    #[case] uci: &str,
    #[case] weight: u16,
    #[case] learn: u32,
) {
    let entry = tiny().get(index).expect("the fixture holds five entries");
    assert_eq!(format!("{:016x}", entry.key), key);
    assert_eq!(entry.uci().as_deref(), Some(uci));
    assert_eq!(entry.weight, weight);
    assert_eq!(entry.learn, learn);
}

#[test]
fn an_index_past_the_end_is_no_entry() {
    assert_eq!(tiny().get(5), None);
}

#[test]
fn the_entries_of_a_position_keep_the_order_the_file_gives_them() {
    let start = position(START);
    let moves: Vec<String> = tiny()
        .entries(&CLASSIC, &start)
        .iter()
        .map(|entry| entry.mv.to_string())
        .collect();
    assert_eq!(moves, ["e2e4", "d2d4", "g1f3"]);
}

#[test]
fn an_entry_naming_no_legal_move_of_the_position_is_refused() {
    let start = position(START);
    let raw = tiny().raw_entries(start.polyglot_key());
    assert_eq!(raw.len(), 4);
    assert_eq!(raw[2].uci().as_deref(), Some("e2e5"));
    assert_eq!(raw[2].decode(&CLASSIC, &start), None);
}

#[test]
fn bits_that_name_no_move_are_no_move() {
    // Promotion code 5, which the format does not define.
    let raw = Raw {
        key: 0,
        mv: 0x531c,
        weight: 1,
        learn: 0,
    };
    assert_eq!(raw.uci(), None);
    assert_eq!(raw.decode(&CLASSIC, &position(START)), None);
}

#[test]
fn a_key_the_book_does_not_hold_has_no_entries() {
    let book = tiny();
    let after_d4 = played("d2d4");
    assert!(
        book.raw_entries(after_d4.position().polyglot_key())
            .is_empty()
    );
    assert!(book.entries(&CLASSIC, after_d4.position()).is_empty());
    assert_eq!(book.best(&CLASSIC, after_d4.position()), None);
    assert_eq!(book.pick(&CLASSIC, after_d4.position(), 0), None);
}

#[test]
fn the_heaviest_entry_is_the_best_one() {
    let start = position(START);
    let best = tiny()
        .best(&CLASSIC, &start)
        .expect("the book knows the start");
    assert_eq!(best.mv.to_string(), "e2e4");
    assert_eq!(best.weight, 100);
}

#[rstest]
#[case::first_share(0, "e2e4")]
#[case::last_of_the_first_share(99, "e2e4")]
#[case::second_share(100, "d2d4")]
#[case::last_of_the_second_share(149, "d2d4")]
#[case::wrapped(150, "e2e4")]
#[case::far_wrapped(1_000_000_099, "e2e4")]
fn a_pick_is_the_entry_the_seed_falls_in(#[case] seed: u64, #[case] uci: &str) {
    // The weights are 100, 50 and 0, so the draw is taken modulo 150 and the
    // move weighed nothing is never drawn.
    let start = position(START);
    let picked = tiny()
        .pick(&CLASSIC, &start, seed)
        .expect("the book knows the start");
    assert_eq!(picked.mv.to_string(), uci);
}

#[test]
fn a_book_written_reads_back_what_was_written() {
    let start = position(START);
    let path = scratch("round-trip");
    let entries = [
        Entry::new(
            start.polyglot_key(),
            move_of(&CLASSIC, &start, "e2e4"),
            9,
            7,
        ),
        Entry::new(
            start.polyglot_key(),
            move_of(&CLASSIC, &start, "d2d4"),
            4,
            0,
        ),
    ];
    Book::write(&path, &entries).expect("the temporary file is writable");

    let book = Book::open(&path).expect("what was written is a book");
    assert_eq!(book.len(), 2);
    assert_eq!(book.entries(&CLASSIC, &start), entries);
    fs::remove_file(&path).expect("the temporary file is removable");
}

#[test]
fn entries_that_share_a_key_and_a_move_are_merged() {
    let start = position(START);
    let e4 = move_of(&CLASSIC, &start, "e2e4");
    let path = scratch("merged");
    Book::write(
        &path,
        &[
            Entry::new(start.polyglot_key(), e4, 3, 5),
            Entry::new(start.polyglot_key(), e4, 4, 0),
        ],
    )
    .expect("the temporary file is writable");

    let book = Book::open(&path).expect("what was written is a book");
    assert_eq!(book.len(), 1);
    assert_eq!(book.get(0).expect("the one entry").weight, 7);
    fs::remove_file(&path).expect("the temporary file is removable");
}

#[test]
fn a_file_that_is_not_whole_entries_is_not_a_book() {
    let path = scratch("ragged");
    fs::write(&path, vec![0u8; ENTRY_SIZE + 1]).expect("the temporary file is writable");
    let error = Book::open(&path).expect_err("a ragged file is not a book");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    fs::remove_file(&path).expect("the temporary file is removable");
}

#[test]
fn an_empty_file_is_an_empty_book() {
    let book = Book::from_bytes(Vec::new()).expect("no entries at all is a book");
    assert!(book.is_empty());
    assert_eq!(book.len(), 0);
}

#[rstest]
#[case::classic_short(&CLASSIC, CLASSIC_CASTLING, "e1h1")]
#[case::classic_long(&CLASSIC, CLASSIC_CASTLING, "e1a1")]
#[case::chess960_short(&CHESS960, NINE_SIXTY_CASTLING, "b1h1")]
#[case::chess960_long(&CHESS960, NINE_SIXTY_CASTLING, "b1a1")]
fn castling_is_written_king_takes_rook(
    #[case] variant: &dyn Variant,
    #[case] fen: &str,
    #[case] uci: &str,
) {
    let position = position(fen);
    let castling = move_of(variant, &position, uci);
    assert!(castling.is_castling());
    let raw = Raw::from(Entry::new(position.polyglot_key(), castling, 1, 0));
    assert_eq!(raw.uci().as_deref(), Some(uci));
    assert_eq!(
        raw.decode(variant, &position).map(|entry| entry.mv),
        Some(castling)
    );
}

#[rstest]
#[case::queen("a7a8q", 0x4c38)]
#[case::rook("a7a8r", 0x3c38)]
#[case::bishop("a7a8b", 0x2c38)]
#[case::knight("a7a8n", 0x1c38)]
fn a_promotion_carries_the_role_in_its_top_bits(#[case] uci: &str, #[case] bits: u16) {
    let position = position(PROMOTION);
    let promotion = move_of(&CLASSIC, &position, uci);
    let raw = Raw::from(Entry::new(position.polyglot_key(), promotion, 1, 0));
    assert_eq!(raw.mv, bits);
    assert_eq!(raw.uci().as_deref(), Some(uci));
}

#[test]
fn a_builder_weighs_a_move_by_how_many_games_played_it() {
    let mut builder = Builder::new();
    for moves in SHARED_OPENING {
        builder.add_game(&played(moves));
    }
    let path = scratch("counted");
    builder
        .write(&path)
        .expect("the temporary file is writable");

    let book = Book::open(&path).expect("what was written is a book");
    let start = position(START);
    let opening = book.entries(&CLASSIC, &start);
    assert_eq!(opening.len(), 1);
    assert_eq!(opening[0].mv.to_string(), "e2e4");
    assert_eq!(opening[0].weight, 2);

    let replies = book.entries(&CLASSIC, played("e2e4").position());
    let moves: Vec<String> = replies.iter().map(|entry| entry.mv.to_string()).collect();
    assert_eq!(moves, ["d7d5", "e7e5"]);
    assert!(replies.iter().all(|entry| entry.weight == 1));
    fs::remove_file(&path).expect("the temporary file is removable");
}

#[test]
fn a_builder_counts_no_move_past_its_maximum_ply() {
    let mut builder = Builder::new().max_ply(2);
    for moves in SHARED_OPENING {
        builder.add_game(&played(moves));
    }
    // The shared first move and the two replies, and nothing of ply three.
    assert_eq!(builder.len(), 3);
    let deeper: Vec<Raw> = builder
        .entries()
        .into_iter()
        .filter(|entry| entry.key == played("e2e4 e7e5").position().polyglot_key())
        .collect();
    assert!(deeper.is_empty());
}

#[test]
fn a_builder_drops_a_move_too_few_games_played() {
    let mut builder = Builder::new().min_count(2);
    for moves in SHARED_OPENING {
        builder.add_game(&played(moves));
    }
    let entries = builder.entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].key, position(START).polyglot_key());
    assert_eq!(entries[0].weight, 2);
}

#[test]
fn a_builder_is_empty_until_it_is_given_a_game() {
    let builder = Builder::new();
    assert!(builder.is_empty());
    assert_eq!(builder.len(), 0);
    assert!(builder.entries().is_empty());
}

#[test]
#[cfg(feature = "pgn")]
fn a_builder_reads_every_game_a_pgn_source_holds() {
    let mut builder = Builder::new();
    // The middle game plays a move White has not got, and is skipped.
    assert_eq!(builder.add_pgn(Cursor::new(PGN)), 2);

    let entries = builder.entries();
    let start = entries
        .iter()
        .find(|entry| entry.key == position(START).polyglot_key())
        .expect("both games opened with 1. e4");
    assert_eq!(start.uci().as_deref(), Some("e2e4"));
    assert_eq!(start.weight, 2);
}

#[test]
fn a_game_of_another_variant_is_keyed_by_the_same_rules() {
    let mut game = Game::from_fen(chess960(), NINE_SIXTY_CASTLING).expect("a Chess960 position");
    let before = game.position().polyglot_key();
    game.play_uci("b1h1").expect("castling short is legal");
    assert_ne!(game.position().polyglot_key(), before);
}
