//! Nanoseconds per position over the golden corpus: the whole extractor.

use criterion::{Criterion, criterion_group, criterion_main};
use esca::{CLASSIC, Position, Scratch};
use std::hint::black_box;

fn corpus() -> Vec<Position> {
    include_str!("../tests/data/fens_classic.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|fen| Position::from_fen(fen).expect("a corpus FEN is legal"))
        .collect()
}

fn benchmarks(c: &mut Criterion) {
    let positions = corpus();
    let mut group = c.benchmark_group("facts");
    group.throughput(criterion::Throughput::Elements(positions.len() as u64));
    group.bench_function("extract", |b| {
        let mut scratch = Scratch::new();
        b.iter(|| {
            for position in &positions {
                black_box(position.facts_in(&CLASSIC, &mut scratch));
            }
        });
    });
    group.finish();
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
