//! Smoke benchmark — verifies the criterion harness compiles, links, and runs.
//!
//! This bench measures a trivially cheap operation so that:
//! - `cargo bench -p beater-bench --no-run` succeeds in CI (§22 `backend` gate).
//! - The criterion HTML report plumbing is exercised end-to-end.
//!
//! Replace or supplement this bench with real store-level measurements in a
//! future `benches/store.rs` (§20.2 #0.3).

use beater_bench::{span_fixtures, SpanFixtureConfig};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_span_fixture_builder(c: &mut Criterion) {
    let config = SpanFixtureConfig::new(1_000);

    c.bench_function("span_fixtures/build_1k", |b| {
        b.iter(|| {
            let spans = span_fixtures(black_box(&config));
            black_box(spans.len())
        })
    });
}

criterion_group!(benches, bench_span_fixture_builder);
criterion_main!(benches);
