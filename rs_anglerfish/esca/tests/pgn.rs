//! PGN reading and writing, case by case.
//!
//! Every expectation is worked out from the "Standard: Portable Game Notation
//! Specification and Implementation Guide" for the named text above it.

#![cfg(feature = "pgn")]

use std::fs;

use esca::pgn::{self, EXPORT_WIDTH, ErrorKind, GameResult, Node, PgnError};
use esca::{Game, chess960, classic};
use rstest::rstest;

/// A seven-tag roster, four full moves, and a mate.
const PLAIN: &str = r#"[Event "Test"]
[Site "Amsterdam"]
[Date "2024.01.01"]
[Round "1"]
[White "Alice"]
[Black "Bob"]
[Result "1-0"]

1. e4 e5 2. Bc4 Nc6 3. Qh5 Nf6 4. Qxf7# 1-0
"#;

/// Roster tags out of order, with one tag that is not in the roster.
const UNORDERED: &str = r#"[Black "Bob"]
[Event "Order"]
[Opening "Vienna Game"]
[White "Alice"]
[Result "1-0"]

1. e4 e5 2. Nc3 1-0
"#;

/// A comment in every place one can stand: before the game, after a move,
/// at the head of a variation, and running to the end of a line.
const COMMENTED: &str = r#"[Event "Comments"]

{Before the game.} 1. e4 {after e4} ({a variation opens} 1. d4 d5 {after d5}) 1... e5 ;after e5
2. Nf3 *
"#;

/// Variations three deep: an alternative first move, an alternative reply to
/// it, and an alternative to that reply's answer.
const NESTED: &str = r#"[Event "Nested"]

1. e4 (1. d4 d5 (1... Nf6 2. c4 (2. Nf3 g6)) 2. c4) 1... e5 2. Nf3 *
"#;

/// Both glyph forms on one line: the `!`/`?` suffixes and `$` numbers.
const GLYPHS: &str = r#"[Event "Glyphs"]

1. e4! $10 e5 $2 2. Nf3?! Nc6 $13 *
"#;

/// En passant, an underpromotion by capture, and castling on both wings.
const SPECIALS: &str = r#"[Event "Specials"]

1. e4 Nf6 2. e5 d5 3. exd6 e6 4. dxc7 Bd6 5. cxb8=N Rxb8 6. Nc3 O-O 7. b3 Re8
8. Bb2 h6 9. Qe2 a6 10. O-O-O *
"#;

/// The knights on b3, b5 and f5 all reach d4, so the mover needs its square.
const THREE_KNIGHTS: &str = "4k3/8/8/1N3N2/8/1N6/8/4K3 w - - 0 1";

/// A Chess960 middlegame: the king stands on g1 with its rooks on f1 and h1,
/// so castling short moves the rook and leaves the king where it is.
const NINE_SIXTY: &str = r#"[Event "960"]
[Variant "Chess960"]
[SetUp "1"]
[FEN "bqnb1rkr/pp3ppp/3ppn2/2p5/5P2/P2P4/NPP1P1PP/BQ1BNRKR w KQkq - 2 9"]

1. Nf3 c4 2. Re1 cxd3 3. O-O *
"#;

/// Anderssen–Kieseritzky, London 1851: long enough to wrap several times.
const IMMORTAL: &str = r#"[Event "Immortal"]
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
"#;

/// Every tolerated liberty at once: an escape line, no tags, no result,
/// numbers glued to their moves, a `...` continuation, a comment over two
/// lines, and a `;` comment.
const WILD: &str = "%an escape line the reader drops\n\
1.e4 e5 2.Nf3 {a comment\n\
spanning lines} 2... Nc6\n\
;a line comment\n";

/// Three games, of which the second plays a move White has not got.
const STREAM: &str = r#"[Event "One"]

1. e4 e5 1-0

[Event "Two"]

1. e4 Nf6 2. Nf6 0-1

[Event "Three"]

1. d4 d5 1/2-1/2
"#;

/// The one game `text` holds.
fn one(text: &str) -> pgn::Game {
    let mut games = pgn::read_str(text);
    let game = games
        .next()
        .expect("the text holds a game")
        .expect("it reads");
    assert!(games.next().is_none(), "the text holds exactly one game");
    game
}

/// Why `text` did not read.
fn failure(text: &str) -> PgnError {
    pgn::read_str(text)
        .next()
        .expect("the text holds a game")
        .expect_err("the game is malformed")
}

/// The move text of a line, space separated.
fn sans(nodes: &[Node]) -> String {
    nodes
        .iter()
        .map(|node| node.san.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The movetext lines of `text`: everything after the blank line that ends
/// the tag section.
fn movetext(text: &str) -> Vec<&str> {
    text.split("\n\n")
        .nth(1)
        .expect("a tag section and a movetext section")
        .lines()
        .collect()
}

/// The first token of a movetext line. A move number and its move are one
/// token, so a line break never falls between them.
fn first_token(line: &str) -> String {
    let mut words = line.split(' ');
    let first = words.next().expect("a line holds a token");
    let numbered = first.ends_with('.')
        && first
            .trim_start_matches('(')
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.');
    if numbered {
        format!(
            "{first} {}",
            words.next().expect("a number precedes a move")
        )
    } else {
        first.to_string()
    }
}

#[test]
fn a_plain_game_reads_its_headers_moves_and_result() {
    let game = one(PLAIN);
    assert_eq!(game.headers.get("Event"), Some("Test"));
    assert_eq!(game.headers.get("Black"), Some("Bob"));
    assert_eq!(game.headers.get("Annotator"), None);
    assert_eq!(sans(game.mainline()), "e4 e5 Bc4 Nc6 Qh5 Nf6 Qxf7#");
    assert_eq!(game.result, GameResult::White);
    assert_eq!(game.mainline_game().expect("the moves are legal").ply(), 7);
}

#[test]
fn a_game_starts_from_the_variant_start_position_unless_a_fen_says_otherwise() {
    let (variant, start) = one(PLAIN).setup().expect("classic chess needs no tags");
    assert_eq!(variant.name(), "chess");
    assert_eq!(start.fen(), classic().start_position(0).fen());

    let (variant, start) = one(NINE_SIXTY).setup().expect("the tags name Chess960");
    assert_eq!(variant.name(), "chess960");
    assert_eq!(start.fullmove_number(), 9);
}

#[test]
fn the_seven_tag_roster_is_written_first_and_the_rest_keep_their_order() {
    let game = one(UNORDERED);
    let read: Vec<&str> = game.headers.iter().map(|(name, _)| name).collect();
    assert_eq!(read, ["Black", "Event", "Opening", "White", "Result"]);

    let written: Vec<&str> = game
        .headers
        .export_order()
        .iter()
        .map(|(name, _)| *name)
        .collect();
    assert_eq!(written, ["Event", "White", "Black", "Result", "Opening"]);
    assert!(
        game.to_string()
            .starts_with("[Event \"Order\"]\n[White \"Alice\"]\n")
    );
}

#[test]
fn a_comment_is_kept_wherever_it_stands() {
    let game = one(COMMENTED);
    assert_eq!(game.comment, "Before the game.");
    assert_eq!(game.mainline()[0].comment_after, "after e4");
    assert_eq!(game.mainline()[1].comment_after, "after e5");

    let variation = &game.mainline()[0].variations[0];
    assert_eq!(sans(variation), "d4 d5");
    assert_eq!(variation[0].comment_before, "a variation opens");
    assert_eq!(variation[1].comment_after, "after d5");
}

#[test]
fn a_comment_spanning_lines_becomes_one_line_of_words() {
    let game = one(WILD);
    assert_eq!(game.mainline()[2].comment_after, "a comment spanning lines");
    assert_eq!(game.mainline()[3].comment_after, "a line comment");
}

#[test]
fn variations_nest_to_any_depth() {
    let game = one(NESTED);
    assert_eq!(sans(game.mainline()), "e4 e5 Nf3");

    let first = &game.mainline()[0].variations[0];
    assert_eq!(sans(first), "d4 d5 c4");
    let second = &first[1].variations[0];
    assert_eq!(sans(second), "Nf6 c4");
    let third = &second[1].variations[0];
    assert_eq!(sans(third), "Nf3 g6");
    assert!(third[1].variations.is_empty());
}

#[rstest]
#[case::good("1. e4! *", &[1])]
#[case::poor("1. e4? *", &[2])]
#[case::very_good("1. e4!! *", &[3])]
#[case::blunder("1. e4?? *", &[4])]
#[case::speculative("1. e4!? *", &[5])]
#[case::dubious("1. e4?! *", &[6])]
#[case::numeric("1. e4 $14 *", &[14])]
#[case::both("1. e4! $14 *", &[1, 14])]
fn a_glyph_is_read_in_either_form_and_kept_as_a_number(#[case] text: &str, #[case] nags: &[u16]) {
    let game = one(text);
    assert_eq!(game.mainline()[0].san, "e4");
    assert_eq!(game.mainline()[0].nags, nags);
}

#[test]
fn glyphs_stay_with_the_moves_they_annotate() {
    let game = one(GLYPHS);
    let nags: Vec<&[u16]> = game
        .mainline()
        .iter()
        .map(|node| node.nags.as_slice())
        .collect();
    assert_eq!(nags, [&[1, 10][..], &[2][..], &[6][..], &[13][..]]);
    assert_eq!(sans(game.mainline()), "e4 e5 Nf3 Nc6");
}

#[test]
fn promotion_castling_and_en_passant_keep_their_text() {
    let game = one(SPECIALS);
    assert_eq!(
        sans(game.mainline()),
        "e4 Nf6 e5 d5 exd6 e6 dxc7 Bd6 cxb8=N Rxb8 Nc3 O-O b3 Re8 Bb2 h6 Qe2 a6 O-O-O"
    );
    assert!(game.mainline()[4].mv.is_en_passant());
    assert_eq!(game.mainline()[8].mv.promotion(), Some(esca::Role::Knight));
    assert!(game.mainline()[11].mv.is_castling());
    assert!(game.mainline()[18].mv.is_castling());
}

#[rstest]
#[case::file("4k3/8/8/1N3N2/8/8/8/4K3 w - - 0 1", "Nbd4", "b5")]
#[case::rank("4k3/8/8/1N6/8/1N6/8/4K3 w - - 0 1", "N5d4", "b5")]
#[case::square(THREE_KNIGHTS, "Nb5d4", "b5")]
fn disambiguation_names_the_mover_it_has_to(
    #[case] fen: &str,
    #[case] san: &str,
    #[case] origin: &str,
) {
    let text = format!("[SetUp \"1\"]\n[FEN \"{fen}\"]\n\n1. {san} *\n");
    let game = one(&text);
    assert_eq!(game.mainline()[0].san, san);
    assert_eq!(game.mainline()[0].mv.from().to_string(), origin);
    assert_eq!(game.to_string(), text);
}

#[test]
fn a_chess960_game_castles_king_to_rook() {
    let game = one(NINE_SIXTY);
    assert_eq!(sans(game.mainline()), "Nf3 c4 Re1 cxd3 O-O");
    let castling = game.mainline()[4].mv;
    assert!(castling.is_castling());
    assert_eq!(castling.from().to_string(), "g1");
    assert_eq!(castling.to().to_string(), "h1");

    let played = game.mainline_game().expect("the moves are legal");
    assert_eq!(
        played.position().king_of(esca::Colour::White).to_string(),
        "g1"
    );
    assert_eq!(
        played.to_pgn().headers.get("FEN"),
        Some("bqnb1rkr/pp3ppp/3ppn2/2p5/5P2/P2P4/NPP1P1PP/BQ1BNRKR w HFhf - 2 9")
    );
}

#[rstest]
#[case::white("1. e4 e5 1-0", GameResult::White)]
#[case::black("1. e4 e5 0-1", GameResult::Black)]
#[case::draw("1. e4 e5 1/2-1/2", GameResult::Draw)]
#[case::unknown("1. e4 e5 *", GameResult::Unknown)]
#[case::missing("[Result \"0-1\"]\n\n1. e4 e5", GameResult::Black)]
#[case::missing_and_untagged("1. e4 e5", GameResult::Unknown)]
fn the_result_is_the_marker_or_the_tag_that_stands_in_for_it(
    #[case] text: &str,
    #[case] result: GameResult,
) {
    let game = one(text);
    assert_eq!(game.result, result);
    assert!(game.to_string().trim_end().ends_with(result.as_str()));
}

#[test]
fn a_long_game_wraps_at_the_export_width() {
    assert_eq!(EXPORT_WIDTH, 80);
    let text = one(IMMORTAL).to_string();
    let lines = movetext(&text);
    assert!(lines.len() > 3, "the game needs several lines");
    for line in &lines {
        assert!(line.chars().count() <= EXPORT_WIDTH, "too long: {line}");
    }
    for pair in lines.windows(2) {
        let carried = first_token(pair[1]);
        let joined = pair[0].chars().count() + 1 + carried.chars().count();
        assert!(
            joined > EXPORT_WIDTH,
            "{carried} still fits after {}",
            pair[0]
        );
    }
}

#[test]
fn a_move_number_stays_with_its_move() {
    let text = one(NESTED).to_string();
    assert_eq!(
        movetext(&text),
        ["1. e4 (1. d4 d5 (1... Nf6 2. c4 (2. Nf3 g6)) 2. c4) 1... e5 2. Nf3 *"]
    );
}

#[rstest]
#[case::plain(PLAIN)]
#[case::unordered(UNORDERED)]
#[case::commented(COMMENTED)]
#[case::nested(NESTED)]
#[case::glyphs(GLYPHS)]
#[case::specials(SPECIALS)]
#[case::nine_sixty(NINE_SIXTY)]
#[case::immortal(IMMORTAL)]
#[case::wild(WILD)]
fn writing_a_game_and_reading_it_back_changes_nothing(#[case] text: &str) {
    let game = one(text);
    let written = game.to_string();
    let again = one(&written);
    // Writing puts the tag pairs in export order, which reading keeps.
    assert_eq!(again.headers.export_order(), game.headers.export_order());
    assert_eq!(again.comment, game.comment);
    assert_eq!(again.moves, game.moves);
    assert_eq!(again.result, game.result);
    assert_eq!(again.to_string(), written);
}

#[rstest]
#[case::unterminated_comment(
    "[Event \"Bad\"]\n\n1. e4 {oops\n",
    3,
    7,
    ErrorKind::UnterminatedComment
)]
#[case::illegal_move(
    "[Event \"Bad\"]\n\n1. e4 e5\n2. Nf6\n",
    4, 4, ErrorKind::IllegalMove("Nf6".to_string()))]
#[case::unknown_variant(
    "[Event \"Bad\"]\n[Variant \"Atomic\"]\n\n1. e4 *\n",
    2, 1, ErrorKind::UnknownVariant("Atomic".to_string()))]
#[case::unreadable_fen(
    "[SetUp \"1\"]\n[FEN \"nonsense\"]\n\n1. e4 *\n",
    2,
    1,
    ErrorKind::BadFen(esca::FenError::FieldCount)
)]
#[case::unterminated_variation(
    "[Event \"Bad\"]\n\n1. e4 (1. d4 *\n",
    3,
    15,
    ErrorKind::UnterminatedVariation
)]
#[case::unterminated_tag("[Event \"Bad]\n\n1. e4 *\n", 1, 1, ErrorKind::UnterminatedString)]
fn malformed_text_is_an_error_at_its_line_and_column(
    #[case] text: &str,
    #[case] line: usize,
    #[case] column: usize,
    #[case] kind: ErrorKind,
) {
    let error = failure(text);
    assert_eq!((error.line, error.column), (line, column));
    assert_eq!(error.kind, kind);
    assert!(
        error
            .to_string()
            .starts_with(&format!("line {line}, column {column}: "))
    );
}

#[test]
fn a_bad_game_does_not_stop_the_stream() {
    let read: Vec<Result<pgn::Game, PgnError>> = pgn::read_str(STREAM).collect();
    assert_eq!(read.len(), 3);
    assert_eq!(
        read[0]
            .as_ref()
            .expect("game one reads")
            .headers
            .get("Event"),
        Some("One")
    );
    assert!(read[1].is_err());
    assert_eq!(
        read[2]
            .as_ref()
            .expect("game three reads")
            .headers
            .get("Event"),
        Some("Three")
    );

    let kept: Vec<pgn::Game> = pgn::read_str(STREAM)
        .skipping()
        .map(|game| game.expect("skipping yields only games that read"))
        .collect();
    let events: Vec<&str> = kept
        .iter()
        .map(|game| game.headers.get("Event").expect("every game is tagged"))
        .collect();
    assert_eq!(events, ["One", "Three"]);
    assert_eq!(pgn::count_games(STREAM.as_bytes()), 2);
}

#[test]
fn a_thousand_games_stream_one_at_a_time() {
    let mut text = String::new();
    for round in 1..=1000 {
        text.push_str(&format!(
            "[Event \"Generated\"]\n[Round \"{round}\"]\n\n1. e4 e5 2. Nf3 Nc6 1/2-1/2\n\n"
        ));
    }
    let path = std::env::temp_dir().join("esca_pgn_thousand.pgn");
    fs::write(&path, &text).expect("the temporary file is writable");

    let mut seen = 0usize;
    let mut five_hundredth = String::new();
    for game in pgn::read(&path).expect("the file opens") {
        let game = game.expect("every generated game reads");
        seen += 1;
        if seen == 500 {
            five_hundredth = game.headers.get("Round").expect("a Round tag").to_string();
        }
        assert_eq!(game.result, GameResult::Draw);
    }
    assert_eq!(seen, 1000);
    assert_eq!(five_hundredth, "500");
    fs::remove_file(&path).expect("the temporary file is removable");
}

#[test]
fn a_played_game_becomes_pgn_and_the_pgn_plays_it_back() {
    let mut played = Game::new(classic());
    for san in ["e4", "e5", "Nf3", "Nc6"] {
        played.play_san(san).expect("an opening move is legal");
    }
    let game = played.to_pgn();
    assert_eq!(game.headers.get("Event"), Some("?"));
    assert_eq!(game.headers.get("Date"), Some("????.??.??"));
    assert_eq!(game.headers.get("Result"), Some("*"));
    assert_eq!(game.headers.get("FEN"), None);
    assert_eq!(sans(game.mainline()), "e4 e5 Nf3 Nc6");
    assert_eq!(
        game.mainline_game()
            .expect("the moves are legal")
            .position()
            .fen(),
        played.position().fen()
    );
}

#[test]
fn a_chess960_start_is_written_as_a_fen_tag() {
    let game = Game::with_seed(chess960(), 1).to_pgn();
    assert_eq!(game.headers.get("Variant"), Some("Chess960"));
    assert_eq!(game.headers.get("SetUp"), Some("1"));
    assert_eq!(
        game.headers.get("FEN"),
        Some(chess960().start_position(1).fen().as_str())
    );
    assert_eq!(
        game.setup().expect("the tags are readable").0.name(),
        "chess960"
    );
}
