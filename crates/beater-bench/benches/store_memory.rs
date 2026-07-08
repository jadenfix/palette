//! In-memory trace-store benchmark scaffold.
//!
//! This is the first real store bench under §20.2 #0.3. It intentionally uses
//! the OSS in-memory backend and small deterministic fixtures so the benchmark
//! compiles quickly in CI while proving the future p95/SLO harness has a real
//! `TraceStore` target.

use beater_bench::{SpanFixtureConfig, canonical_trace_batch};
use beater_core::PageRequest;
use beater_schema::SpanFilter;
use beater_store::TraceStore;
use beater_store_memory::InMemoryTraceStore;
use criterion::{BatchSize, Criterion, Throughput, black_box, criterion_group, criterion_main};
use tokio::runtime::Runtime;

const SPAN_COUNT: usize = 1_000;

fn bench_in_memory_trace_store(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap_or_else(|err| panic!("{err}"));
    let config = SpanFixtureConfig::new(SPAN_COUNT);
    let mut group = c.benchmark_group("store_memory");
    group.throughput(Throughput::Elements(SPAN_COUNT as u64));

    group.bench_function("write_batch/1k", |b| {
        b.iter_batched(
            || canonical_trace_batch(&config),
            |batch| {
                let store = InMemoryTraceStore::new();
                let accepted_spans = runtime.block_on(async {
                    store
                        .write_batch(batch.into())
                        .await
                        .unwrap_or_else(|err| panic!("{err}"))
                        .accepted_spans
                });
                black_box(accepted_spans)
            },
            BatchSize::SmallInput,
        )
    });

    let store = InMemoryTraceStore::new();
    runtime.block_on(async {
        store
            .write_batch(canonical_trace_batch(&config).into())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
    });

    let tenant_id = config.tenant_id.clone();
    let filter = SpanFilter {
        project_id: Some(config.project_id.clone()),
        environment_id: Some(config.environment_id.clone()),
        ..SpanFilter::default()
    };
    let page = PageRequest {
        limit: 100,
        cursor: None,
    };

    group.bench_function("query_spans/page_100", |b| {
        b.iter(|| {
            let page = runtime.block_on(async {
                store
                    .query_spans(tenant_id.clone(), filter.clone(), page.clone())
                    .await
                    .unwrap_or_else(|err| panic!("{err}"))
            });
            black_box(page.items.len())
        })
    });

    group.finish();
}

criterion_group!(benches, bench_in_memory_trace_store);
criterion_main!(benches);
