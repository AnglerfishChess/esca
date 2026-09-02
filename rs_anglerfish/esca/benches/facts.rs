//! Nanoseconds per position, over the golden corpus: the whole extractor, and
//! each group's encoding on its own.

use criterion::{Criterion, criterion_group, criterion_main};
use esca::{CLASSIC, GroupSet, Position, Schema, Scratch};
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
    let schema = Schema::v0();

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

    let mut scratch = Scratch::new();
    let facts: Vec<_> = positions
        .iter()
        .map(|position| position.facts_in(&CLASSIC, &mut scratch))
        .collect();

    let mut group = c.benchmark_group("encode");
    group.throughput(criterion::Throughput::Elements(facts.len() as u64));
    for (index, spec) in schema.groups().iter().enumerate() {
        let selected = GroupSet::only(index);
        let mut out = vec![0.0f32; schema.width_of(selected)];
        group.bench_function(spec.name, |b| {
            b.iter(|| {
                for one in &facts {
                    black_box(one.encode_into(schema, selected, &mut out));
                }
            });
        });
    }
    group.finish();
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
