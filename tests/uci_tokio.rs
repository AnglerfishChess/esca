//! Talking to an engine process on a tokio runtime, over the scripted doubles
//! in `tests/fixtures/`.
//!
//! The cases mirror `tests/uci_engine.rs` — a normal game, a slow engine, one
//! writing garbage, one dying mid-search, one that never answers, and the two
//! Chess960 handshakes — and add what only an async client can be asked: a
//! wait that is given up on, and a search let go of mid-flight.

#![cfg(feature = "tokio")]

use std::path::Path;
use std::time::{Duration, Instant};

use esca::uci::tokio::Engine;
use esca::uci::{Error, Limits, OptionKind, OptionValue, Progress, State};
use esca::{Game, chess960, classic};
use rstest::rstest;
use tokio::time::{sleep, timeout};

mod double;

use double::{BESIDE_ROOK, TIMEOUT, launch, launch_logging, log_of, log_path};

/// A started double, misbehaving as `flags` ask.
async fn fake(flags: &[&str]) -> Engine {
    launch(flags)
        .spawn_tokio()
        .await
        .expect("the double starts")
}

/// A started double that has identified itself.
async fn identified(flags: &[&str]) -> Engine {
    let mut engine = fake(flags).await;
    engine
        .handshake()
        .await
        .expect("the double identifies itself");
    engine
}

/// A double that logs every command it is sent to `path`, identified.
async fn with_log(path: &Path, flags: &[&str]) -> Engine {
    let mut engine = launch_logging(path, flags)
        .spawn_tokio()
        .await
        .expect("the double starts");
    engine
        .handshake()
        .await
        .expect("the double identifies itself");
    engine
}

// -- A normal game ----------------------------------------------------------

#[tokio::test]
async fn an_engine_names_itself_and_lists_what_it_offers() {
    let mut engine = fake(&[]).await;
    let identity = engine
        .handshake()
        .await
        .expect("the double identifies itself");
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

#[tokio::test]
async fn a_search_reports_and_then_answers() {
    let mut engine = identified(&[]).await;
    engine.new_game().await.expect("a new game");
    let game = Game::new(classic());
    engine
        .set_position(&game)
        .await
        .expect("the start position");

    let answer = {
        let mut search = engine
            .go(&Limits::depth(2), TIMEOUT)
            .await
            .expect("a search");
        let mut reports = Vec::new();
        while let Some(info) = search.next_info().await.expect("the reports arrive") {
            reports.push(info);
        }
        assert_eq!(reports.len(), 3);
        assert_eq!(reports[0].depth, Some(1));
        assert_eq!(reports[0].pv_moves(&game).len(), 1);
        assert_eq!(reports[1].pv_moves(&game).len(), 2);
        assert_eq!(
            reports[2].string.as_deref(),
            Some("thinking about it: hard")
        );
        search.answer().await.expect("an answer")
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
#[tokio::test]
async fn the_moves_played_go_out_with_the_position() {
    let log = log_path("moves.log");
    let mut engine = with_log(&log, &[]).await;

    let mut game = Game::new(classic());
    game.play_san("e4").expect("1. e4 is legal");
    game.play_san("e5").expect("1... e5 is legal");
    engine
        .set_position(&game)
        .await
        .expect("the position goes out");
    engine.is_ready().await.expect("the double has read it");

    assert_eq!(
        log_of(&log),
        ["uci", "position startpos moves e2e4 e7e5", "isready"]
    );
}

#[tokio::test]
async fn an_engine_with_no_move_answers_with_none() {
    let mut engine = identified(&["--no-move"]).await;
    let answer = engine
        .play(&Game::new(classic()), &Limits::depth(1), TIMEOUT)
        .await
        .expect("an answer");
    assert_eq!(answer.best, None);
    assert_eq!(answer.ponder, None);
}

#[tokio::test]
async fn a_search_that_waits_is_ended_by_stop() {
    let mut engine = identified(&[]).await;
    engine
        .set_position(&Game::new(classic()))
        .await
        .expect("a position");
    let mut search = engine
        .go(&Limits::infinite(), TIMEOUT)
        .await
        .expect("a search");
    search.stop().await.expect("stop goes out");
    assert!(search.answer().await.expect("an answer").best.is_some());
}

#[tokio::test]
async fn a_ponder_becomes_a_search_on_a_ponderhit() {
    let mut engine = identified(&[]).await;
    engine
        .set_position(&Game::new(classic()))
        .await
        .expect("a position");
    let mut search = engine
        .go(&Limits::infinite().pondering(), TIMEOUT)
        .await
        .expect("a search");
    assert_eq!(search.state(), State::Pondering);
    search.ponderhit().await.expect("ponderhit goes out");
    assert!(search.answer().await.expect("an answer").best.is_some());
}

/// A search let go of is asked to stop, and the next call waits it out: no
/// drop can wait, so the engine settles at the first opportunity it is given.
#[tokio::test]
async fn letting_a_search_go_settles_the_engine_on_the_next_call() {
    let mut engine = identified(&[]).await;
    engine
        .set_position(&Game::new(classic()))
        .await
        .expect("a position");
    drop(
        engine
            .go(&Limits::infinite(), TIMEOUT)
            .await
            .expect("a search"),
    );
    engine.is_ready().await.expect("the engine still answers");
    assert_eq!(engine.state(), State::Idle);
}

#[tokio::test]
async fn quitting_reaps_the_process() {
    let mut engine = identified(&[]).await;
    assert_eq!(engine.quit().await.expect("it exits"), Some(0));
    assert!(!engine.is_alive());
}

/// An engine that ignores `quit` is killed rather than waited on forever.
#[tokio::test]
async fn an_engine_that_will_not_quit_is_killed() {
    let mut engine = identified(&["--zombie"]).await;
    engine.set_timeout(Duration::from_millis(300));
    engine.quit().await.expect("it is dealt with");
    assert!(!engine.is_alive());
}

// -- Unhappy paths ----------------------------------------------------------

#[tokio::test]
async fn a_silent_engine_times_out_rather_than_hangs() {
    let mut engine = identified(&["--no-readyok"]).await;
    engine.set_timeout(Duration::from_millis(200));
    let error = engine.is_ready().await.expect_err("no readyok ever comes");
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

#[tokio::test]
async fn an_engine_that_never_ends_its_identification_times_out() {
    let mut engine = launch(&["--no-uciok"])
        .timeout(Duration::from_secs(1))
        .spawn_tokio()
        .await
        .expect("the double starts");
    let error = engine.handshake().await.expect_err("no uciok ever comes");
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
#[tokio::test]
async fn a_search_that_outlasts_its_budget_is_a_timeout() {
    let mut engine = identified(&["--slow"]).await;
    engine
        .set_position(&Game::new(classic()))
        .await
        .expect("a position");
    let error = {
        let mut search = engine
            .go(&Limits::depth(2), Duration::from_millis(50))
            .await
            .expect("a search");
        search
            .answer()
            .await
            .expect_err("the double is slower than that")
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
#[tokio::test]
async fn a_slow_engine_answers_within_a_budget_that_fits() {
    let mut engine = identified(&["--slow"]).await;
    let answer = engine
        .play(&Game::new(classic()), &Limits::depth(2), TIMEOUT)
        .await
        .expect("an answer");
    assert!(answer.best.is_some());
}

/// Malformed lines, empty lines and lines out of turn are all survived.
#[tokio::test]
async fn garbage_never_derails_a_search() {
    let mut engine = identified(&["--garbage"]).await;
    let answer = engine
        .play(&Game::new(classic()), &Limits::depth(2), TIMEOUT)
        .await
        .expect("an answer");
    assert!(answer.best.is_some());
    engine
        .is_ready()
        .await
        .expect("the engine is still answerable");
}

/// A second `bestmove` belongs to no search, and is dropped rather than kept.
#[tokio::test]
async fn an_answer_too_many_is_ignored() {
    let mut engine = identified(&["--twice"]).await;
    engine
        .play(&Game::new(classic()), &Limits::depth(1), TIMEOUT)
        .await
        .expect("an answer");
    engine
        .is_ready()
        .await
        .expect("the engine is still answerable");
    assert_eq!(engine.state(), State::Idle);
}

#[tokio::test]
async fn an_engine_that_dies_mid_search_is_reported_as_dead() {
    let mut engine = identified(&["--die-on-go"]).await;
    engine
        .set_position(&Game::new(classic()))
        .await
        .expect("a position");
    let error = {
        let mut search = engine
            .go(&Limits::depth(2), TIMEOUT)
            .await
            .expect("a search");
        search.answer().await.expect_err("the double exits instead")
    };
    assert!(matches!(error, Error::Died { code: Some(3) }), "{error}");

    // Every call after says the same thing.
    let again = engine.is_ready().await.expect_err("the engine is gone");
    assert!(matches!(again, Error::Died { .. }), "{again}");
    assert!(!engine.is_alive());
}

/// There is nothing to stop, so the command never reaches the engine.
#[tokio::test]
async fn a_command_the_conversation_has_no_room_for_is_refused() {
    let mut engine = identified(&[]).await;
    let error = engine.stop().await.expect_err("no search is running");
    assert!(matches!(error, Error::Protocol(_)), "{error}");
    engine.is_ready().await.expect("the engine is untouched");
}

// -- Giving up on a wait ----------------------------------------------------

/// A future dropped mid-wait takes no line with it: the engine answers the
/// next caller as if nothing had happened.
#[tokio::test]
async fn a_wait_given_up_on_leaves_the_engine_usable() {
    let mut engine = identified(&["--slow"]).await;
    let game = Game::new(classic());
    let given_up = timeout(
        Duration::from_millis(50),
        engine.play(&game, &Limits::depth(2), TIMEOUT),
    )
    .await;
    assert!(given_up.is_err(), "the double is slower than that");

    engine.is_ready().await.expect("the engine still answers");
    assert_eq!(engine.state(), State::Idle);
    let answer = engine
        .play(&game, &Limits::depth(2), TIMEOUT)
        .await
        .expect("an answer");
    assert!(answer.best.is_some());
}

/// The same for a search that only ends when it is told to.
#[tokio::test]
async fn a_search_given_up_on_is_stopped() {
    let mut engine = identified(&[]).await;
    engine
        .set_position(&Game::new(classic()))
        .await
        .expect("a position");
    {
        let mut search = engine
            .go(&Limits::infinite(), TIMEOUT)
            .await
            .expect("a search");
        let given_up = timeout(Duration::from_millis(50), search.answer()).await;
        assert!(given_up.is_err(), "nothing comes until the search is told");
    }
    engine.is_ready().await.expect("the engine still answers");
    assert_eq!(engine.state(), State::Idle);
}

// -- The line buffer --------------------------------------------------------

/// The unread lines are capped, so an engine writing faster than it is read
/// costs a bounded amount of memory and the oldest reports, not the answer.
#[tokio::test]
async fn a_flood_of_reports_drops_the_oldest_and_keeps_the_answer() {
    let mut engine = identified(&["--flood"]).await;
    engine
        .set_position(&Game::new(classic()))
        .await
        .expect("a position");
    engine
        .start_search(&Limits::depth(2))
        .await
        .expect("the search starts");

    // Let the double outrun the client, which reads nothing until it does.
    let until = Instant::now() + TIMEOUT;
    while engine.dropped_lines() == 0 && Instant::now() < until {
        sleep(Duration::from_millis(10)).await;
    }
    assert!(engine.dropped_lines() > 0, "the double floods the client");

    let answer = loop {
        match engine
            .next_progress(TIMEOUT)
            .await
            .expect("the search goes on")
        {
            Progress::Done(answer) => break answer,
            Progress::Info(_) => {}
        }
    };
    assert!(answer.best.is_some(), "the answer is never dropped");
    assert_eq!(engine.state(), State::Idle);
}

/// Nothing is dropped from a conversation the client keeps up with.
#[tokio::test]
async fn a_client_that_keeps_up_drops_nothing() {
    let mut engine = identified(&[]).await;
    engine
        .play(&Game::new(classic()), &Limits::depth(2), TIMEOUT)
        .await
        .expect("an answer");
    assert_eq!(engine.dropped_lines(), 0);
}

// -- Options ----------------------------------------------------------------

#[tokio::test]
async fn an_option_is_set_by_the_name_the_engine_declared() {
    let log = log_path("options.log");
    let mut engine = with_log(&log, &[]).await;
    engine
        .set_option("multipv", OptionValue::Spin(3))
        .await
        .expect("MultiPV takes 3");
    engine
        .set_option("Clear Hash", OptionValue::Button)
        .await
        .expect("a button takes no value");
    engine
        .set_option("Debug Log File", OptionValue::String(String::new()))
        .await
        .expect("a string takes empty text");
    engine.is_ready().await.expect("the double has read them");

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

#[rstest]
// The double offers no Contempt at all.
#[case::unknown_name("Contempt", OptionValue::Spin(10))]
// MultiPV stops at 8.
#[case::outside_the_range("MultiPV", OptionValue::Spin(99))]
// MultiPV is a spin, not a check.
#[case::wrong_type("MultiPV", OptionValue::Check(true))]
// Neither of the vars the double declares.
#[case::no_such_var("Style", OptionValue::Combo("Wild wild".to_owned()))]
#[tokio::test]
async fn a_value_the_engine_did_not_declare_is_refused(
    #[case] name: &str,
    #[case] value: OptionValue,
) {
    let mut engine = identified(&[]).await;
    let error = engine
        .set_option(name, value)
        .await
        .expect_err("the double declared no such thing");
    assert!(
        matches!(error, Error::NoSuchOption(_) | Error::BadValue { .. }),
        "{error}"
    );
}

#[tokio::test]
async fn options_are_unknown_until_the_engine_has_listed_them() {
    let mut engine = fake(&[]).await;
    let error = engine
        .set_option("Hash", OptionValue::Spin(32))
        .await
        .expect_err("nothing is known before uci");
    assert!(matches!(error, Error::NotIdentified), "{error}");
}

// -- Chess960 ---------------------------------------------------------------

/// A Chess960 game turns the option on and is written king-to-rook.
#[tokio::test]
async fn a_chess960_game_puts_the_engine_into_chess960() {
    let log = log_path("chess960.log");
    let mut engine = with_log(&log, &[]).await;

    let mut game = Game::from_fen(chess960(), BESIDE_ROOK).expect("the FEN is a legal position");
    game.play_uci("b1c1").expect("castling short is legal");
    engine
        .set_position(&game)
        .await
        .expect("the position goes out");
    engine.is_ready().await.expect("the double has read it");

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
#[tokio::test]
async fn an_engine_that_cannot_play_chess960_is_refused_the_game() {
    let mut engine = identified(&["--no-chess960"]).await;
    let game = Game::from_fen(chess960(), BESIDE_ROOK).expect("the FEN is a legal position");
    let error = engine
        .set_position(&game)
        .await
        .expect_err("the double offers no UCI_Chess960");
    assert!(
        matches!(&error, Error::NoSuchOption(name) if name == "UCI_Chess960"),
        "{error}"
    );
}

/// A classic game needs no option, and its castling is written as two squares.
#[tokio::test]
async fn a_classic_game_is_sent_without_touching_the_option() {
    let log = log_path("classic.log");
    let mut engine = with_log(&log, &[]).await;

    let mut game = Game::from_fen(classic(), "4k3/8/8/8/8/8/8/4K2R w K - 0 1")
        .expect("the FEN is a legal position");
    game.play_uci("e1h1").expect("short castling is legal");
    engine
        .set_position(&game)
        .await
        .expect("the position goes out");
    engine.is_ready().await.expect("the double has read it");

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
#[tokio::test]
async fn lines_can_be_written_and_read_as_they_are() {
    let mut engine = fake(&[]).await;
    engine.send_line("uci").await.expect("the line goes out");
    let mut seen = Vec::new();
    while let Some(line) = engine
        .next_line(TIMEOUT)
        .await
        .expect("the engine is alive")
    {
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

#[tokio::test]
async fn a_read_that_finds_nothing_answers_with_nothing() {
    let mut engine = identified(&[]).await;
    assert_eq!(
        engine
            .next_line(Duration::from_millis(50))
            .await
            .expect("the engine is alive"),
        None
    );
}
