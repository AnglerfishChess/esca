//! Talking to an engine process, over the scripted doubles in
//! `tests/fixtures/` and, when they are installed, over real engines.
//!
//! Every case is one thing that can happen to a client: a normal game, a slow
//! engine, one writing garbage, one dying mid-search, one that never answers,
//! and the two Chess960 handshakes.

#![cfg(feature = "uci")]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use esca::uci::{Engine, Error, Launch, Limits, OptionKind, OptionValue, State};
use esca::{Game, chess960, classic};

/// Long enough for a subprocess to answer, short enough to fail a test fast.
const TIMEOUT: Duration = Duration::from_secs(5);

/// A Chess960 endgame: the white king on b1 with its own rook beside it on c1.
const BESIDE_ROOK: &str = "4k3/8/8/8/8/8/8/1KR5 w C - 0 1";

/// The interpreter that runs the engine doubles.
fn python() -> &'static str {
    static NAMES: [&str; 3] = ["python3", "python", "py"];
    for name in NAMES {
        let answered = Command::new(name)
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success());
        if answered {
            return name;
        }
    }
    panic!("no Python interpreter on PATH: tried {NAMES:?}");
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// A started double, misbehaving as `flags` ask.
fn fake(flags: &[&str]) -> Engine {
    let mut launch = Launch::new(python())
        .arg(fixture("fake_engine.py"))
        .timeout(TIMEOUT);
    for flag in flags {
        launch = launch.arg(flag);
    }
    launch.spawn().expect("the double starts")
}

/// A started double that has identified itself.
fn identified(flags: &[&str]) -> Engine {
    let mut engine = fake(flags);
    engine.handshake().expect("the double identifies itself");
    engine
}

/// The commands a double wrote to its log, in order.
fn log_of(path: &Path) -> Vec<String> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(str::to_owned)
        .collect()
}

// -- A normal game ----------------------------------------------------------

#[test]
fn an_engine_names_itself_and_lists_what_it_offers() {
    let mut engine = fake(&[]);
    let identity = engine.handshake().expect("the double identifies itself");
    assert_eq!(identity.name.as_deref(), Some("Fake Engine 1.0"));
    assert_eq!(identity.author.as_deref(), Some("The esca test suite"));
    assert_eq!(engine.state(), State::Idle);

    assert_eq!(engine.options().len(), 7);
    let hash = engine.option("Hash").expect("the double offers Hash");
    assert_eq!(
        hash.kind,
        OptionKind::Spin {
            default: Some(16),
            min: Some(1),
            max: Some(1024),
        }
    );
    // Engines match option names without regard to case, and so does esca.
    assert!(engine.option("hash").is_some());
}

#[test]
fn a_search_reports_and_then_answers() {
    let mut engine = identified(&[]);
    engine.new_game().expect("a new game");
    let game = Game::new(classic());
    engine.set_position(&game).expect("the start position");

    let answer = {
        let mut search = engine.go(&Limits::depth(2), TIMEOUT).expect("a search");
        let reports: Vec<_> = (&mut search)
            .collect::<Result<Vec<_>, _>>()
            .expect("the reports arrive");
        assert_eq!(reports.len(), 3);
        assert_eq!(reports[0].depth, Some(1));
        assert_eq!(reports[0].pv_moves(&game).len(), 1);
        assert_eq!(reports[1].pv_moves(&game).len(), 2);
        assert_eq!(
            reports[2].string.as_deref(),
            Some("thinking about it: hard")
        );
        search.answer().expect("an answer")
    };
    let best = answer.best.expect("a move");
    assert_eq!(game.move_to_san(best), "e4");
    assert_eq!(
        answer.ponder.map(|mv| mv.to_string()).as_deref(),
        Some("e7e5")
    );
    assert_eq!(engine.state(), State::Idle);
}

/// The start position is `startpos`, and the moves played follow it.
#[test]
fn the_moves_played_go_out_with_the_position() {
    let log = log_path("moves.log");
    let mut engine = with_log(&log, &[]);
    engine.handshake().expect("the double identifies itself");

    let mut game = Game::new(classic());
    game.play_san("e4").expect("1. e4 is legal");
    game.play_san("e5").expect("1... e5 is legal");
    engine.set_position(&game).expect("the position goes out");
    engine.is_ready().expect("the double has read it");

    assert_eq!(
        log_of(&log),
        ["uci", "position startpos moves e2e4 e7e5", "isready"]
    );
}

#[test]
fn an_engine_with_no_move_answers_with_none() {
    let mut engine = identified(&["--no-move"]);
    let answer = engine
        .play(&Game::new(classic()), &Limits::depth(1), TIMEOUT)
        .expect("an answer");
    assert_eq!(answer.best, None);
    assert_eq!(answer.ponder, None);
}

#[test]
fn a_search_that_waits_is_ended_by_stop() {
    let mut engine = identified(&[]);
    engine
        .set_position(&Game::new(classic()))
        .expect("a position");
    let mut search = engine.go(&Limits::infinite(), TIMEOUT).expect("a search");
    search.stop().expect("stop goes out");
    assert!(search.answer().expect("an answer").best.is_some());
}

#[test]
fn a_ponder_becomes_a_search_on_a_ponderhit() {
    let mut engine = identified(&[]);
    engine
        .set_position(&Game::new(classic()))
        .expect("a position");
    let mut search = engine
        .go(&Limits::infinite().pondering(), TIMEOUT)
        .expect("a search");
    assert_eq!(search.state(), State::Pondering);
    search.ponderhit().expect("ponderhit goes out");
    assert!(search.answer().expect("an answer").best.is_some());
}

/// A search let go of is stopped and drained, so the engine is idle after.
#[test]
fn dropping_a_search_leaves_the_engine_idle() {
    let mut engine = identified(&[]);
    engine
        .set_position(&Game::new(classic()))
        .expect("a position");
    drop(engine.go(&Limits::infinite(), TIMEOUT).expect("a search"));
    assert_eq!(engine.state(), State::Idle);
    engine.is_ready().expect("the engine still answers");
}

#[test]
fn quitting_reaps_the_process() {
    let mut engine = identified(&[]);
    assert_eq!(engine.quit().expect("it exits"), Some(0));
    assert!(!engine.is_alive());
}

/// An engine that ignores `quit` is killed rather than waited on forever.
#[test]
fn an_engine_that_will_not_quit_is_killed() {
    let mut engine = identified(&["--zombie"]);
    engine.set_timeout(Duration::from_millis(300));
    engine.quit().expect("it is dealt with");
    assert!(!engine.is_alive());
}

// -- Unhappy paths ----------------------------------------------------------

#[test]
fn a_silent_engine_times_out_rather_than_hangs() {
    let mut engine = identified(&["--no-readyok"]);
    engine.set_timeout(Duration::from_millis(200));
    let error = engine.is_ready().expect_err("no readyok ever comes");
    assert!(
        matches!(
            error,
            Error::Timeout {
                awaited: "readyok",
                ..
            }
        ),
        "{error}"
    );
}

#[test]
fn an_engine_that_never_ends_its_identification_times_out() {
    let mut engine = Launch::new(python())
        .arg(fixture("fake_engine.py"))
        .arg("--no-uciok")
        .timeout(Duration::from_secs(1))
        .spawn()
        .expect("the double starts");
    let error = engine.handshake().expect_err("no uciok ever comes");
    assert!(
        matches!(
            error,
            Error::Timeout {
                awaited: "uciok",
                ..
            }
        ),
        "{error}"
    );
}

/// A slow engine is a timeout on this search, not a hung client.
#[test]
fn a_search_that_outlasts_its_budget_is_a_timeout() {
    let mut engine = identified(&["--slow"]);
    engine
        .set_position(&Game::new(classic()))
        .expect("a position");
    let error = {
        let mut search = engine
            .go(&Limits::depth(2), Duration::from_millis(50))
            .expect("a search");
        search.answer().expect_err("the double is slower than that")
    };
    assert!(
        matches!(
            error,
            Error::Timeout {
                awaited: "bestmove",
                ..
            }
        ),
        "{error}"
    );
}

/// The same engine, given the time it needs, answers.
#[test]
fn a_slow_engine_answers_within_a_budget_that_fits() {
    let mut engine = identified(&["--slow"]);
    let answer = engine
        .play(&Game::new(classic()), &Limits::depth(2), TIMEOUT)
        .expect("an answer");
    assert!(answer.best.is_some());
}

/// Malformed lines, empty lines and lines out of turn are all survived.
#[test]
fn garbage_never_derails_a_search() {
    let mut engine = identified(&["--garbage"]);
    let answer = engine
        .play(&Game::new(classic()), &Limits::depth(2), TIMEOUT)
        .expect("an answer");
    assert!(answer.best.is_some());
    engine.is_ready().expect("the engine is still answerable");
}

/// A second `bestmove` belongs to no search, and is dropped rather than kept.
#[test]
fn an_answer_too_many_is_ignored() {
    let mut engine = identified(&["--twice"]);
    engine
        .play(&Game::new(classic()), &Limits::depth(1), TIMEOUT)
        .expect("an answer");
    engine.is_ready().expect("the engine is still answerable");
    assert_eq!(engine.state(), State::Idle);
}

#[test]
fn an_engine_that_dies_mid_search_is_reported_as_dead() {
    let mut engine = identified(&["--die-on-go"]);
    engine
        .set_position(&Game::new(classic()))
        .expect("a position");
    let error = {
        let mut search = engine.go(&Limits::depth(2), TIMEOUT).expect("a search");
        search.answer().expect_err("the double exits instead")
    };
    assert!(matches!(error, Error::Died { code: Some(3) }), "{error}");

    // Every call after says the same thing.
    let again = engine.is_ready().expect_err("the engine is gone");
    assert!(matches!(again, Error::Died { .. }), "{again}");
    assert!(!engine.is_alive());
}

/// There is nothing to stop, so the command never reaches the engine.
#[test]
fn a_command_the_conversation_has_no_room_for_is_refused() {
    let mut engine = identified(&[]);
    let error = engine.stop().expect_err("no search is running");
    assert!(matches!(error, Error::Protocol(_)), "{error}");
    engine.is_ready().expect("the engine is untouched");
}

// -- Options ----------------------------------------------------------------

#[test]
fn an_option_is_set_by_the_name_the_engine_declared() {
    let log = log_path("options.log");
    let mut engine = with_log(&log, &[]);
    engine.handshake().expect("the double identifies itself");
    engine
        .set_option("multipv", OptionValue::Spin(3))
        .expect("MultiPV takes 3");
    engine
        .set_option("Clear Hash", OptionValue::Button)
        .expect("a button takes no value");
    engine
        .set_option("Debug Log File", OptionValue::String(String::new()))
        .expect("a string takes empty text");
    engine.is_ready().expect("the double has read them");

    let sent = log_of(&log);
    assert!(
        sent.contains(&"setoption name MultiPV value 3".to_owned()),
        "{sent:?}"
    );
    assert!(
        sent.contains(&"setoption name Clear Hash".to_owned()),
        "{sent:?}"
    );
    assert!(
        sent.contains(&"setoption name Debug Log File value <empty>".to_owned()),
        "{sent:?}"
    );
}

#[test]
fn an_option_the_engine_does_not_offer_is_refused() {
    let mut engine = identified(&[]);
    let error = engine
        .set_option("Contempt", OptionValue::Spin(10))
        .expect_err("the double offers no Contempt");
    assert!(matches!(error, Error::NoSuchOption(name) if name == "Contempt"));
}

#[test]
fn a_value_outside_what_the_engine_declared_is_refused() {
    let mut engine = identified(&[]);
    let error = engine
        .set_option("MultiPV", OptionValue::Spin(99))
        .expect_err("MultiPV stops at 8");
    assert!(matches!(error, Error::BadValue { .. }), "{error}");

    let wrong_type = engine
        .set_option("MultiPV", OptionValue::Check(true))
        .expect_err("MultiPV is not a check");
    assert!(matches!(wrong_type, Error::BadValue { .. }), "{wrong_type}");
}

#[test]
fn options_are_unknown_until_the_engine_has_listed_them() {
    let mut engine = fake(&[]);
    let error = engine
        .set_option("Hash", OptionValue::Spin(32))
        .expect_err("nothing is known before uci");
    assert!(matches!(error, Error::NotIdentified), "{error}");
}

// -- Chess960 ---------------------------------------------------------------

/// A Chess960 game turns the option on and is written king-to-rook.
#[test]
fn a_chess960_game_puts_the_engine_into_chess960() {
    let log = log_path("chess960.log");
    let mut engine = with_log(&log, &[]);
    engine.handshake().expect("the double identifies itself");

    let mut game = Game::from_fen(chess960(), BESIDE_ROOK).expect("the FEN is a legal position");
    game.play_uci("b1c1").expect("castling short is legal");
    engine.set_position(&game).expect("the position goes out");
    engine.is_ready().expect("the double has read it");

    assert_eq!(
        log_of(&log),
        [
            "uci".to_owned(),
            "setoption name UCI_Chess960 value true".to_owned(),
            "isready".to_owned(),
            format!("position fen {BESIDE_ROOK} moves b1c1"),
            "isready".to_owned(),
        ]
    );
}

/// An engine without the option gets an error rather than the wrong game.
#[test]
fn an_engine_that_cannot_play_chess960_is_refused_the_game() {
    let mut engine = identified(&["--no-chess960"]);
    let game = Game::from_fen(chess960(), BESIDE_ROOK).expect("the FEN is a legal position");
    let error = engine
        .set_position(&game)
        .expect_err("the double offers no UCI_Chess960");
    assert!(
        matches!(&error, Error::NoSuchOption(name) if name == "UCI_Chess960"),
        "{error}"
    );
}

/// A classic game needs no option, and its castling is written as two squares.
#[test]
fn a_classic_game_is_sent_without_touching_the_option() {
    let log = log_path("classic.log");
    let mut engine = with_log(&log, &[]);
    engine.handshake().expect("the double identifies itself");

    let mut game = Game::from_fen(classic(), "4k3/8/8/8/8/8/8/4K2R w K - 0 1")
        .expect("the FEN is a legal position");
    game.play_uci("e1h1").expect("short castling is legal");
    engine.set_position(&game).expect("the position goes out");
    engine.is_ready().expect("the double has read it");

    assert_eq!(
        log_of(&log),
        [
            "uci",
            "position fen 4k3/8/8/8/8/8/8/4K2R w K - 0 1 moves e1g1",
            "isready",
        ]
    );
}

// -- The raw interface ------------------------------------------------------

/// Tools and diagnostics address the engine one line at a time.
#[test]
fn lines_can_be_written_and_read_as_they_are() {
    let mut engine = fake(&[]);
    engine.send_line("uci").expect("the line goes out");
    let mut seen = Vec::new();
    while let Some(line) = engine.next_line(TIMEOUT).expect("the engine is alive") {
        seen.push(line.clone());
        if line == "uciok" {
            break;
        }
    }
    assert_eq!(
        seen.first().map(String::as_str),
        Some("id name Fake Engine 1.0")
    );
    assert_eq!(seen.last().map(String::as_str), Some("uciok"));
}

#[test]
fn a_read_that_finds_nothing_answers_with_nothing() {
    let mut engine = identified(&[]);
    assert_eq!(
        engine
            .next_line(Duration::from_millis(50))
            .expect("the engine is alive"),
        None
    );
}

// -- Real engines -----------------------------------------------------------

/// Every real engine to be found plays one move from the start position.
///
/// Ignored by default: it needs an engine installed.
#[test]
#[ignore = "needs a real engine installed"]
fn a_real_engine_identifies_itself_and_plays() {
    let mut found = 0;
    for name in real_engines() {
        let Ok(mut engine) = Launch::new(&name).timeout(Duration::from_secs(20)).spawn() else {
            continue;
        };
        found += 1;
        let identity = engine.handshake().expect("a real engine identifies itself");
        assert!(identity.name.is_some(), "{} names itself", name.display());
        engine.new_game().expect("a new game");

        let game = Game::new(classic());
        let answer = engine
            .play(
                &game,
                &Limits::movetime(Duration::from_millis(200)),
                Duration::from_secs(20),
            )
            .expect("a real engine answers");
        let best = answer.best.expect("a legal move from the start position");
        assert!(
            game.legal_moves().contains(&best),
            "{} plays a legal move",
            name.display()
        );
        engine.quit().expect("it exits");
    }
    assert!(found > 0, "no real engine to be found");
}

/// Every real engine that offers `UCI_Chess960` plays a shuffled position.
///
/// Ignored by default: it needs an engine installed.
#[test]
#[ignore = "needs a real engine installed"]
fn a_real_engine_plays_chess960() {
    for name in real_engines() {
        let Ok(mut engine) = Launch::new(&name).timeout(Duration::from_secs(20)).spawn() else {
            continue;
        };
        engine.handshake().expect("a real engine identifies itself");
        if engine.option("UCI_Chess960").is_none() {
            continue;
        }
        let game = Game::from_fen(chess960(), BESIDE_ROOK).expect("the FEN is a legal position");
        let answer = engine
            .play(
                &game,
                &Limits::movetime(Duration::from_millis(200)),
                Duration::from_secs(20),
            )
            .expect("a real engine answers");
        let best = answer.best.expect("a legal move");
        assert!(
            game.legal_moves().contains(&best),
            "{} plays a legal move",
            name.display()
        );
        engine.quit().expect("it exits");
    }
}

// -- Helpers ----------------------------------------------------------------

/// The engines to try: the well-known ones on PATH, and this workspace's own
/// build of `anglerfish`.
fn real_engines() -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = ["stockfish", "lc0", "anglerfry", "anglerfish"]
        .iter()
        .map(PathBuf::from)
        .collect();
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../target");
    for profile in ["release", "debug"] {
        let built = workspace.join(profile).join("anglerfish");
        if built.exists() {
            found.push(built);
        }
    }
    found
}

/// A double that logs every command it is sent to `path`.
fn with_log(path: &Path, flags: &[&str]) -> Engine {
    let mut launch = Launch::new(python())
        .arg(fixture("fake_engine.py"))
        .arg(format!("--log={}", path.display()))
        .timeout(TIMEOUT);
    for flag in flags {
        launch = launch.arg(flag);
    }
    launch.spawn().expect("the double starts")
}

/// A fresh path for one test's command log.
fn log_path(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!("esca-uci-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("a writable temporary directory");
    let path = directory.join(name);
    let _ = std::fs::remove_file(&path);
    path
}
