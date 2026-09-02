//! The evaluation-dump reader, over a synthetic sample and over the real dump.

#![cfg(feature = "lichess")]

use std::path::{Path, PathBuf};

use esca::lichess::{self, Record};
use esca::{CLASSIC, Score};

fn sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/lichess_sample.jsonl.zst")
}

fn sample() -> Vec<Record> {
    lichess::read(&sample_path())
        .expect("the sample opens")
        .collect::<Result<Vec<_>, _>>()
        .expect("every sample line reads")
}

#[test]
fn the_sample_reads_whole() {
    let records = sample();
    assert_eq!(records.len(), 12);
    assert_eq!(
        records[0].epd,
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -"
    );
    assert_eq!(records[0].evals.len(), 1);
    assert_eq!(records[0].evals[0].depth, 36);
    assert_eq!(records[0].evals[0].knodes, 112_000);
    assert_eq!(records[0].evals[0].pvs[0].score, Score::Cp(22));
}

#[test]
fn every_eval_and_pv_of_a_record_is_kept() {
    let records = sample();
    let record = &records[1];
    assert_eq!(record.evals.len(), 2);
    assert_eq!(record.evals[0].pvs.len(), 2);
    assert_eq!(record.evals[1].depth, 22);
}

#[test]
fn a_four_field_fen_parses_with_the_clocks_unknown() {
    for record in sample() {
        let position = record.position().expect("a sample EPD is a position");
        assert!(!position.clocks_known());
        assert_eq!(position.halfmove_clock(), 0);
        assert_eq!(position.fullmove_number(), 1);
        assert_eq!(position.epd(), record.epd);
    }
}

#[test]
fn scores_are_side_relative() {
    let records = sample();
    // White is a pawn up with Black to move: the dump writes +69, which is
    // against the side to move.
    assert_eq!(records[2].evals[0].pvs[0].score, Score::Cp(-69));
    assert_eq!(records[2].evals[0].pvs[1].score, Score::Cp(-163));
    // White to move and mating: unchanged.
    assert_eq!(records[4].evals[0].pvs[0].score, Score::Mate(15));
    // Black to move and mating, written as -1 by the dump.
    assert_eq!(records[5].evals[0].pvs[0].score, Score::Mate(1));
    // White to move and being mated.
    assert_eq!(records[8].evals[0].pvs[0].score, Score::Mate(-1));
    // A score of zero has no side.
    assert_eq!(records[3].evals[0].pvs[0].score, Score::Cp(0));
}

#[test]
fn the_best_move_of_a_line_is_read_in_its_position() {
    for record in sample() {
        let position = record.position().expect("a sample EPD is a position");
        for eval in &record.evals {
            for pv in &eval.pvs {
                pv.best_move(&CLASSIC, &position)
                    .unwrap_or_else(|error| panic!("{error} in {}: {}", record.epd, pv.line));
            }
        }
    }
}

#[test]
fn plain_lines_read_as_well_and_blank_ones_are_skipped() {
    let text = "\n\
        {\"fen\":\"4k3/8/8/8/8/8/4P3/4K3 w - -\",\"evals\":[{\"pvs\":[{\"cp\":5,\"line\":\"e1d2\"}],\"knodes\":1,\"depth\":2}]}\n\
        \n";
    let records: Vec<Record> = lichess::read_from(text.as_bytes())
        .collect::<Result<_, _>>()
        .expect("the lines read");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].evals[0].pvs[0].score, Score::Cp(5));
}

#[test]
fn a_malformed_line_is_an_error_and_the_stream_goes_on() {
    let text = "{\"fen\":\"nonsense\"}\n\
        {\"fen\":\"4k3/8/8/8/8/8/4P3/4K3 w - -\",\"evals\":[{\"pvs\":[{\"cp\":5,\"line\":\"e1d2\"}],\"knodes\":1,\"depth\":2}]}\n";
    let results: Vec<_> = lichess::read_from(text.as_bytes()).collect();
    assert_eq!(results.len(), 2);
    assert!(results[0].is_err());
    assert!(results[1].is_ok());
}

/// The real dump: the first records of it, read and played.
#[test]
#[ignore = "needs the Lichess dump under data-external/"]
fn the_real_dump_reads() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data-external/lichess_db_eval.jsonl.zst");
    if !path.exists() {
        eprintln!("no dump at {}", path.display());
        return;
    }
    let mut seen = 0;
    let mut not_a_position = 0;
    for record in lichess::read(&path).expect("the dump opens").take(10_000) {
        let record = record.expect("a dump line reads");
        assert!(!record.evals.is_empty());
        seen += 1;
        // A handful of dump rows are analysis-board positions no game can
        // reach; the rest must read and play.
        let Ok(position) = record.position() else {
            not_a_position += 1;
            continue;
        };
        for eval in &record.evals {
            for pv in &eval.pvs {
                pv.best_move(&CLASSIC, &position)
                    .unwrap_or_else(|error| panic!("{error} in {}: {}", record.epd, pv.line));
            }
        }
    }
    assert_eq!(seen, 10_000);
    assert!(not_a_position < 100, "{not_a_position} unreachable rows");
}
