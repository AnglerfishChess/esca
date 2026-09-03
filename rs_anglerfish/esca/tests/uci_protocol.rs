//! The protocol as values: what a command writes, and what one line of engine
//! output reads as.
//!
//! Every case is one line of the protocol, including the shapes engines are
//! known to write in the wild: a keyword after junk, missing arguments, extra
//! spaces, names and values of several words.

#![cfg(feature = "uci")]

use std::time::Duration;

use esca::uci::protocol::{
    Bound, Command, Info, Limits, Message, OptionKind, OptionSpec, OptionValue, ProtocolError,
    Register, Session, Setup, State, Status, Wdl, parse,
};
use esca::{CHESS960, Game, Score, Variant, chess960, classic};
use rstest::rstest;

/// A Chess960 endgame: the white king on b1 with its own rook beside it on
/// c1, so that castling short leaves the king's origin behind.
const BESIDE_ROOK: &str = "4k3/8/8/8/8/8/8/1KR5 w C - 0 1";

// -- Commands ---------------------------------------------------------------

#[rstest]
#[case::uci(Command::Uci, "uci")]
#[case::debug_on(Command::Debug(true), "debug on")]
#[case::debug_off(Command::Debug(false), "debug off")]
#[case::isready(Command::IsReady, "isready")]
#[case::newgame(Command::NewGame, "ucinewgame")]
#[case::stop(Command::Stop, "stop")]
#[case::ponderhit(Command::PonderHit, "ponderhit")]
#[case::quit(Command::Quit, "quit")]
#[case::register_later(Command::Register(Register::Later), "register later")]
fn a_plain_command_is_its_keyword(#[case] command: Command, #[case] line: &str) {
    assert_eq!(command.to_line(), line);
    assert_eq!(command.to_string(), line);
}

#[test]
fn registering_names_the_parts_it_has() {
    let both = Command::Register(Register::Credentials {
        name: Some("Alexander Myodov".to_owned()),
        code: Some("4711".to_owned()),
    });
    assert_eq!(both.to_line(), "register name Alexander Myodov code 4711");

    let code_only = Command::Register(Register::Credentials {
        name: None,
        code: Some("4711".to_owned()),
    });
    assert_eq!(code_only.to_line(), "register code 4711");
}

#[rstest]
#[case::value(Some("64"), "setoption name Hash value 64")]
#[case::button(None, "setoption name Hash")]
fn setting_an_option_names_it_and_its_value(#[case] value: Option<&str>, #[case] line: &str) {
    let command = Command::SetOption {
        name: "Hash".to_owned(),
        value: value.map(str::to_owned),
    };
    assert_eq!(command.to_line(), line);
}

#[test]
fn an_option_name_and_value_may_be_several_words() {
    let command = Command::SetOption {
        name: "Clear Hash".to_owned(),
        value: Some("two words".to_owned()),
    };
    assert_eq!(
        command.to_line(),
        "setoption name Clear Hash value two words"
    );
}

#[test]
fn empty_text_is_written_as_the_protocol_spells_it() {
    assert_eq!(
        OptionValue::String(String::new()).to_text().as_deref(),
        Some("<empty>")
    );
    assert_eq!(OptionValue::Button.to_text(), None);
    assert_eq!(OptionValue::Check(true).to_text().as_deref(), Some("true"));
}

#[rstest]
#[case::startpos(Setup::start(), "position startpos")]
#[case::fen(
    Setup::fen("4k3/8/8/8/8/8/8/4K2R w K - 0 1"),
    "position fen 4k3/8/8/8/8/8/8/4K2R w K - 0 1"
)]
fn a_position_names_where_it_starts(#[case] setup: Setup, #[case] line: &str) {
    assert_eq!(Command::Position(setup).to_line(), line);
}

#[test]
fn a_position_lists_the_moves_played_onto_it() {
    let setup = Setup {
        fen: None,
        moves: ["e2e4", "e7e5"].map(str::to_owned).into(),
    };
    assert_eq!(
        Command::Position(setup).to_line(),
        "position startpos moves e2e4 e7e5"
    );
}

/// A game from the standard array is `startpos`, whatever its variant.
#[test]
fn a_classic_game_is_written_from_the_start_position() {
    let mut game = Game::new(classic());
    game.play_san("e4").expect("1. e4 is legal");
    game.play_san("e5").expect("1... e5 is legal");
    let setup = Setup::of_game(&game, esca::CastlingOutput::KingTwoSquares);
    assert_eq!(setup.fen, None);
    assert_eq!(setup.moves, ["e2e4", "e7e5"]);
}

/// Classic castling is the king's two-square move; a GUI reads nothing else.
#[test]
fn classic_castling_is_written_as_two_squares() {
    let mut game = Game::from_fen(classic(), "4k3/8/8/8/8/8/8/4K2R w K - 0 1")
        .expect("the FEN is a legal position");
    game.play_uci("e1h1").expect("short castling is legal");
    let setup = Setup::of_game(&game, esca::CastlingOutput::KingTwoSquares);
    assert_eq!(setup.fen.as_deref(), Some("4k3/8/8/8/8/8/8/4K2R w K - 0 1"));
    assert_eq!(setup.moves, ["e1g1"]);
}

/// Chess960 castling is king-to-rook, which is unambiguous on any back rank.
#[test]
fn chess960_castling_is_written_king_to_rook() {
    let mut game = Game::from_fen(chess960(), BESIDE_ROOK).expect("the FEN is a legal position");
    game.play_uci("b1c1").expect("castling short is legal");
    let setup = Setup::of_game(&game, esca::CastlingOutput::KingToRook);
    assert_eq!(setup.fen.as_deref(), Some(BESIDE_ROOK));
    assert_eq!(setup.moves, ["b1c1"]);
}

#[rstest]
#[case::nothing(Limits::default(), "go")]
#[case::infinite(Limits::infinite(), "go infinite")]
#[case::depth(Limits::depth(12), "go depth 12")]
#[case::nodes(Limits::nodes(50_000), "go nodes 50000")]
#[case::mate(Limits::mate(3), "go mate 3")]
#[case::movetime(Limits::movetime(Duration::from_millis(1500)), "go movetime 1500")]
#[case::ponder(Limits::depth(4).pondering(), "go ponder depth 4")]
fn a_go_names_the_limits_it_has(#[case] limits: Limits, #[case] line: &str) {
    assert_eq!(Command::Go(limits).to_line(), line);
}

#[test]
fn a_clock_is_written_in_milliseconds() {
    let limits = Limits {
        moves_to_go: Some(40),
        ..Limits::clock(
            Duration::from_secs(300),
            Duration::from_secs(299),
            Duration::from_secs(2),
            Duration::from_millis(1500),
        )
    };
    assert_eq!(
        Command::Go(limits).to_line(),
        "go wtime 300000 btime 299000 winc 2000 binc 1500 movestogo 40"
    );
}

/// The move list runs to the end of the line, so it is written last.
#[test]
fn searchmoves_comes_after_every_other_limit() {
    let limits = Limits::depth(6).searching(["e2e4", "d2d4"].map(str::to_owned));
    assert_eq!(
        Command::Go(limits).to_line(),
        "go depth 6 searchmoves e2e4 d2d4"
    );
}

// -- Engine lines -----------------------------------------------------------

#[rstest]
#[case::uciok("uciok", Message::UciOk)]
#[case::readyok("readyok", Message::ReadyOk)]
#[case::spaced("   readyok  ", Message::ReadyOk)]
#[case::after_junk("Fake engine 1.0 uciok", Message::UciOk)]
#[case::registration("registration checking", Message::Registration(Status::Checking))]
#[case::copyprotection("copyprotection ok", Message::CopyProtection(Status::Ok))]
#[case::registration_error("registration error", Message::Registration(Status::Error))]
fn a_plain_line_is_its_keyword(#[case] line: &str, #[case] message: Message) {
    assert_eq!(parse(line), message);
}

#[rstest]
#[case::empty("")]
#[case::blank("   ")]
#[case::greeting("Stockfish 17 by the Stockfish developers")]
#[case::no_status("registration maybe")]
#[case::id_without_value("id name")]
#[case::bestmove_without_move("bestmove")]
#[case::option_without_type("option name Hash default 16")]
#[case::option_of_unknown_type("option name Hash type wheel default 16")]
fn a_line_that_is_not_understood_is_kept_whole(#[case] line: &str) {
    assert_eq!(parse(line), Message::Raw(line.to_owned()));
}

#[rstest]
#[case::name("id name Stockfish 17", "name", "Stockfish 17")]
#[case::author(
    "id author the developers (see AUTHORS)",
    "author",
    "the developers (see AUTHORS)"
)]
#[case::spaced("id   name   Fake  Engine ", "name", "Fake  Engine")]
fn an_id_keeps_the_rest_of_its_line(#[case] line: &str, #[case] key: &str, #[case] value: &str) {
    assert_eq!(
        parse(line),
        Message::Id {
            key: key.to_owned(),
            value: value.to_owned(),
        }
    );
}

fn option_of(line: &str) -> OptionSpec {
    match parse(line) {
        Message::Option(spec) => spec,
        other => panic!("expected an option, got {other:?}"),
    }
}

#[test]
fn a_check_option_reads_its_default() {
    let option = option_of("option name Ponder type check default false");
    assert_eq!(option.name, "Ponder");
    assert_eq!(
        option.kind,
        OptionKind::Check {
            default: Some(false)
        }
    );
}

#[test]
fn a_spin_option_reads_its_range() {
    let option = option_of("option name Hash type spin default 16 min 1 max 33554432");
    assert_eq!(
        option.kind,
        OptionKind::Spin {
            default: Some(16),
            min: Some(1),
            max: Some(33_554_432),
        }
    );
    assert_eq!(option.accepts(&OptionValue::Spin(64)), Ok(()));
    assert!(option.accepts(&OptionValue::Spin(0)).is_err());
    assert!(option.accepts(&OptionValue::Check(true)).is_err());
}

#[test]
fn a_combo_option_reads_every_value_it_offers() {
    let option =
        option_of("option name Style type combo default Normal var Solid var Normal var Risky");
    assert_eq!(
        option.kind,
        OptionKind::Combo {
            default: Some("Normal".to_owned()),
            vars: ["Solid", "Normal", "Risky"].map(str::to_owned).into(),
        }
    );
    assert_eq!(
        option.accepts(&OptionValue::Combo("Risky".to_owned())),
        Ok(())
    );
    assert!(
        option
            .accepts(&OptionValue::Combo("Wild".to_owned()))
            .is_err()
    );
}

#[test]
fn a_button_option_carries_no_value() {
    let option = option_of("option name Clear Hash type button");
    assert_eq!(option.name, "Clear Hash");
    assert_eq!(option.kind, OptionKind::Button);
    assert_eq!(option.default_value(), Some(OptionValue::Button));
}

#[test]
fn a_name_and_a_value_may_both_be_several_words() {
    let option = option_of("option name Debug Log File type string default my log.txt");
    assert_eq!(option.name, "Debug Log File");
    assert_eq!(
        option.kind,
        OptionKind::String {
            default: Some("my log.txt".to_owned())
        }
    );
}

/// `<empty>` is the protocol's spelling of an empty string.
#[test]
fn an_empty_string_default_reads_as_empty() {
    let option = option_of("option name SyzygyPath type string default <empty>");
    assert_eq!(
        option.kind,
        OptionKind::String {
            default: Some(String::new())
        }
    );
}

/// A spin without min and max is thinner than the spec asks, and usable.
#[test]
fn a_spin_without_a_range_accepts_every_number() {
    let option = option_of("option name Threads type spin default 1");
    assert_eq!(option.accepts(&OptionValue::Spin(-5)), Ok(()));
}

fn info_of(line: &str) -> Info {
    match parse(line) {
        Message::Info(info) => *info,
        other => panic!("expected an info, got {other:?}"),
    }
}

#[test]
fn an_info_reads_the_counters_of_a_search() {
    let info = info_of(
        "info depth 12 seldepth 18 time 1234 nodes 987654 nps 800000 hashfull 231 tbhits 4 sbhits 2 cpuload 990",
    );
    assert_eq!(info.depth, Some(12));
    assert_eq!(info.seldepth, Some(18));
    assert_eq!(info.time, Some(Duration::from_millis(1234)));
    assert_eq!(info.nodes, Some(987_654));
    assert_eq!(info.nps, Some(800_000));
    assert_eq!(info.hashfull, Some(231));
    assert_eq!(info.tbhits, Some(4));
    assert_eq!(info.sbhits, Some(2));
    assert_eq!(info.cpuload, Some(990));
    assert!(info.unknown.is_empty());
}

#[rstest]
#[case::centipawns("info score cp 25", Some(Score::Cp(25)), None)]
#[case::negative("info score cp -310", Some(Score::Cp(-310)), None)]
#[case::mate("info score mate 3", Some(Score::Mate(3)), None)]
#[case::mated("info score mate -2", Some(Score::Mate(-2)), None)]
#[case::lower("info score cp 25 lowerbound", Some(Score::Cp(25)), Some(Bound::Lower))]
#[case::upper("info score cp 25 upperbound", Some(Score::Cp(25)), Some(Bound::Upper))]
fn a_score_is_read_with_its_bound(
    #[case] line: &str,
    #[case] score: Option<Score>,
    #[case] bound: Option<Bound>,
) {
    let info = info_of(line);
    assert_eq!(info.score, score);
    assert_eq!(info.bound, bound);
}

#[test]
fn a_win_draw_loss_estimate_is_three_numbers() {
    let info = info_of("info depth 20 score cp 12 wdl 231 640 129 nodes 5");
    assert_eq!(info.score, Some(Score::Cp(12)));
    assert_eq!(
        info.wdl,
        Some(Wdl {
            win: 231,
            draw: 640,
            loss: 129
        })
    );
    assert_eq!(info.nodes, Some(5));
}

#[test]
fn a_variation_runs_to_the_next_keyword() {
    let info = info_of("info multipv 2 score cp 14 pv e2e4 e7e5 g1f3 depth 9");
    assert_eq!(info.multipv, Some(2));
    assert_eq!(info.pv, ["e2e4", "e7e5", "g1f3"]);
    assert_eq!(info.depth, Some(9));
}

#[test]
fn the_move_being_searched_is_reported_with_its_number() {
    let info = info_of("info currmove e2e4 currmovenumber 1");
    assert_eq!(info.currmove.as_deref(), Some("e2e4"));
    assert_eq!(info.currmovenumber, Some(1));
}

#[test]
fn a_refutation_is_the_move_refuted_and_the_line_that_refutes_it() {
    let info = info_of("info refutation d1h5 g6h5");
    assert_eq!(info.refutation, ["d1h5", "g6h5"]);
}

#[rstest]
#[case::with_cpu("info currline 1 e2e4 e7e5", Some(1), &["e2e4", "e7e5"])]
#[case::without_cpu("info currline e2e4 e7e5", None, &["e2e4", "e7e5"])]
fn a_current_line_names_its_cpu_when_the_engine_says(
    #[case] line: &str,
    #[case] cpu: Option<u32>,
    #[case] moves: &[&str],
) {
    let currline = info_of(line).currline.expect("a currline");
    assert_eq!(currline.cpu, cpu);
    assert_eq!(currline.moves, moves);
}

/// `string` takes the rest of the line, spacing and punctuation kept.
#[test]
fn an_info_string_is_the_rest_of_the_line() {
    let info = info_of("info string NNUE evaluation using nn-1234.nnue: enabled  (large)");
    assert_eq!(
        info.string.as_deref(),
        Some("NNUE evaluation using nn-1234.nnue: enabled  (large)")
    );
    assert!(info.depth.is_none());
}

/// Words after `string` are text, not fields, even when they read like one.
#[test]
fn a_keyword_after_info_string_is_text() {
    let info = info_of("info string depth 12 is not a depth");
    assert_eq!(info.string.as_deref(), Some("depth 12 is not a depth"));
    assert_eq!(info.depth, None);
}

#[rstest]
#[case::missing_value("info depth", &["depth"])]
#[case::unreadable_value("info depth deep", &["depth", "deep"])]
#[case::empty_variation("info pv", &["pv"])]
#[case::score_without_a_number("info score cp", &["score", "cp"])]
#[case::unknown_field("info ebf 1.7 depth 3", &["ebf", "1.7"])]
fn a_field_that_is_not_understood_is_kept_as_a_token(#[case] line: &str, #[case] unknown: &[&str]) {
    assert_eq!(info_of(line).unknown, unknown);
}

/// Extra spaces are separators like any other, and a keyword may follow junk.
#[test]
fn spacing_and_leading_junk_do_not_change_what_a_line_says() {
    let info = info_of("garbage   info    depth   3   nodes  17 ");
    assert_eq!(info.depth, Some(3));
    assert_eq!(info.nodes, Some(17));
}

#[rstest]
#[case::plain("bestmove e2e4", Some("e2e4"), None)]
#[case::with_ponder("bestmove e2e4 ponder e7e5", Some("e2e4"), Some("e7e5"))]
#[case::none("bestmove (none)", None, None)]
#[case::null("bestmove 0000", None, None)]
#[case::after_junk("hmm bestmove e2e4", Some("e2e4"), None)]
fn a_bestmove_reads_the_move_and_the_reply_expected(
    #[case] line: &str,
    #[case] best: Option<&str>,
    #[case] ponder: Option<&str>,
) {
    let expected = esca::uci::BestMove {
        best: best.map(str::to_owned),
        ponder: ponder.map(str::to_owned),
    };
    assert_eq!(parse(line), Message::BestMove(expected));
}

#[rstest]
#[case::ponder_without_a_move("bestmove e2e4 ponder")]
#[case::junk_after_the_move("bestmove e2e4 e7e5")]
fn a_broken_bestmove_is_kept_whole(#[case] line: &str) {
    assert_eq!(parse(line), Message::Raw(line.to_owned()));
}

// -- Reading move text against a game ---------------------------------------

#[test]
fn a_variation_reads_as_the_moves_of_the_game_it_was_searched_from() {
    let info = info_of("info pv e2e4 e7e5 g1f3");
    let game = Game::new(classic());
    let pv = info.pv_moves(&game);
    assert_eq!(pv.len(), 3);
    assert_eq!(game.move_to_san(pv[0]), "e4");
}

/// A line the position cannot play is truncated where it stops making sense.
#[test]
fn a_variation_stops_at_the_first_move_that_is_not_legal() {
    let info = info_of("info pv e2e4 e7e5 e1e8");
    assert_eq!(info.pv_moves(&Game::new(classic())).len(), 2);
}

#[test]
fn a_bestmove_reads_as_a_move_of_the_position_searched() {
    let Message::BestMove(best) = parse("bestmove e2e4 ponder e7e5") else {
        panic!("expected a bestmove");
    };
    let game = Game::new(classic());
    let chosen = best.best_move(&game).expect("e2e4 is legal");
    assert_eq!(game.move_to_san(chosen), "e4");
    let reply = best.ponder_move(&game).expect("e7e5 is legal after e2e4");
    assert_eq!(reply.to_string(), "e7e5");
}

#[test]
fn a_bestmove_of_none_names_no_move() {
    let Message::BestMove(best) = parse("bestmove (none)") else {
        panic!("expected a bestmove");
    };
    assert_eq!(best.best_move(&Game::new(classic())), None);
}

/// An engine writes Chess960 castling king-to-rook, and esca reads it.
#[test]
fn chess960_castling_is_read_king_to_rook() {
    let game = Game::from_fen(chess960(), BESIDE_ROOK).expect("the FEN is a legal position");
    let Message::BestMove(best) = parse("bestmove b1c1") else {
        panic!("expected a bestmove");
    };
    let castling = best.best_move(&game).expect("b1c1 is legal");
    assert!(castling.is_castling());
    assert_eq!(CHESS960.move_to_san(game.position(), castling), "O-O");
}

// -- The state machine ------------------------------------------------------

/// The order of one whole conversation, from `uci` to `bestmove`.
#[test]
fn a_session_follows_the_engine_from_identification_to_an_answer() {
    let mut session = Session::new();
    assert_eq!(session.state(), State::Started);

    session.sent(&Command::Uci).expect("uci opens a session");
    assert_eq!(session.state(), State::Identifying);
    session
        .received(&Message::Id {
            key: "name".to_owned(),
            value: "Fake".to_owned(),
        })
        .expect("an identifying engine names itself");
    session.received(&Message::UciOk).expect("uciok ends it");
    assert_eq!(session.state(), State::Idle);

    session.sent(&Command::IsReady).expect("isready is allowed");
    assert_eq!(session.pending_ready(), 1);
    session
        .received(&Message::ReadyOk)
        .expect("readyok answers");
    assert_eq!(session.pending_ready(), 0);

    session
        .sent(&Command::Go(Limits::depth(3)))
        .expect("an idle engine may search");
    assert_eq!(session.state(), State::Searching);
    session
        .received(&Message::BestMove(esca::uci::BestMove::default()))
        .expect("a searching engine may answer");
    assert_eq!(session.state(), State::Idle);
}

#[test]
fn pondering_becomes_searching_on_a_ponderhit() {
    let mut session = identified();
    session
        .sent(&Command::Go(Limits::infinite().pondering()))
        .expect("an idle engine may ponder");
    assert_eq!(session.state(), State::Pondering);
    session
        .sent(&Command::PonderHit)
        .expect("the guess was right");
    assert_eq!(session.state(), State::Searching);
}

#[rstest]
#[case::position(Command::Position(Setup::start()))]
#[case::newgame(Command::NewGame)]
#[case::go(Command::Go(Limits::depth(1)))]
fn a_searching_engine_takes_no_new_work(#[case] command: Command) {
    let mut session = identified();
    session
        .sent(&Command::Go(Limits::infinite()))
        .expect("an idle engine may search");
    assert_eq!(
        session.sent(&command),
        Err(ProtocolError::Command {
            keyword: command.keyword(),
            state: State::Searching,
        })
    );
}

#[rstest]
#[case::stop(Command::Stop)]
#[case::ponderhit(Command::PonderHit)]
fn an_idle_engine_has_no_search_to_end(#[case] command: Command) {
    let mut session = identified();
    assert!(session.sent(&command).is_err());
}

#[test]
fn an_answer_that_no_search_asked_for_is_refused() {
    let mut session = identified();
    assert_eq!(
        session.received(&Message::BestMove(esca::uci::BestMove::default())),
        Err(ProtocolError::Message {
            keyword: "bestmove",
            state: State::Idle,
        })
    );
}

#[test]
fn a_readyok_that_no_isready_asked_for_is_refused() {
    let mut session = identified();
    assert!(session.received(&Message::ReadyOk).is_err());
}

#[test]
fn options_and_names_belong_to_the_identification() {
    let mut session = identified();
    assert!(
        session
            .received(&Message::Option(OptionSpec {
                name: "Hash".to_owned(),
                kind: OptionKind::Button,
            }))
            .is_err()
    );
}

/// Diagnostics and garbage arrive whenever the engine feels like it.
#[test]
fn info_and_raw_lines_are_welcome_at_any_time() {
    let mut session = Session::new();
    session
        .received(&Message::Info(Box::default()))
        .expect("info is always allowed");
    session
        .received(&Message::Raw("hello".to_owned()))
        .expect("garbage is always allowed");
}

/// The session an engine is in once it has answered `uci`.
fn identified() -> Session {
    let mut session = Session::new();
    session.sent(&Command::Uci).expect("uci opens a session");
    session.received(&Message::UciOk).expect("uciok ends it");
    session
}
