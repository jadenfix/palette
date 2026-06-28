use beater_core::{
    ArtifactId, EnvironmentId, IdempotencyKey, OrganizationId, PageRequest, ProjectId, Sha256Hash,
    SpanId, TenantId, TraceId,
};
use beater_schema::{
    AgentSpanKind, ArtifactRef, AuthContext, CanonicalSpan, CanonicalTraceBatch, RawEnvelope,
    RedactionClass, RunFilter, SourceDialect, SpanFilter, SpanStatus, RAW_SCHEMA_VERSION,
};
use beater_store::{
    MetadataStore, OrganizationMetadata, ProjectMetadata, QuotaLimiter, QuotaReservationRequest,
    RoleBinding, TraceStore,
};
use chrono::{DateTime, Duration, TimeZone, Utc};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Base timestamp for the trace-store fixtures, anchored to *today* (midnight
/// UTC) rather than a hardcoded calendar date.
///
/// The ClickHouse trace-store tables carry a `TTL` on `start_time` /
/// `received_at` (90 days for spans, 180 for raw envelopes). A fixed past date
/// — e.g. 2026-01-01 — eventually ages past that TTL, at which point a
/// background ClickHouse merge evicts the just-written rows between the two
/// `write_batch` calls and the idempotency lookups see an empty table
/// (`duplicate_spans == 0`). Anchoring the fixtures to the current day keeps the
/// data inside every store's TTL window regardless of the calendar date. Other
/// stores (SQLite/Pg/memory) have no TTL and are unaffected. Truncated to
/// midnight so the per-span `seq`-second offsets remain deterministic and
/// ordered.
fn fixture_base_time() -> DateTime<Utc> {
    let now = Utc::now();
    now.date_naive()
        .and_hms_opt(0, 0, 0)
        .map(|naive| DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
        .unwrap_or(now)
}

pub async fn assert_trace_store_conformance<S>(store: S)
where
    S: TraceStore,
{
    let (batch, tenant, project, trace, idempotency_key) = fixture_batch();

    let first = store
        .write_batch(batch.clone())
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let second = store
        .write_batch(batch)
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    assert_eq!(first.accepted_raw, 1);
    assert_eq!(first.accepted_spans, 2);
    assert_eq!(second.duplicate_raw, 1);
    assert_eq!(second.duplicate_spans, 2);

    let trace_view = store
        .get_trace(tenant.clone(), trace.clone())
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(trace_view.spans.len(), 2);
    assert_eq!(trace_view.spans[0].span_id.as_str(), "root");
    assert_eq!(trace_view.spans[1].span_id.as_str(), "child");

    let raw = store
        .get_raw_envelope(tenant.clone(), project.clone(), idempotency_key)
        .await
        .unwrap_or_else(|err| panic!("{err}"))
        .unwrap_or_else(|| panic!("raw envelope should exist"));
    assert_eq!(raw.source, SourceDialect::Native);

    let spans = store
        .query_spans(
            tenant.clone(),
            SpanFilter {
                project_id: None,
                environment_id: None,
                trace_id: Some(trace.clone()),
                span_id: None,
                kind: Some(AgentSpanKind::AgentStep),
                status: Some(SpanStatus::Ok),
            },
            PageRequest {
                limit: 10,
                cursor: None,
            },
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(spans.items.len(), 1);
    assert_eq!(spans.items[0].span_id.as_str(), "child");

    let first_span_page = store
        .query_spans(
            tenant.clone(),
            SpanFilter {
                trace_id: Some(trace.clone()),
                ..SpanFilter::default()
            },
            PageRequest {
                limit: 1,
                cursor: None,
            },
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(first_span_page.items.len(), 1);
    assert_eq!(first_span_page.items[0].span_id.as_str(), "child");
    // The cursor is an opaque token: in-memory uses an offset, SQLite a keyset
    // seek key (ARCHITECTURE.md §20.2 #0.2). Conformance only requires that a
    // next page is advertised and that feeding the token back resumes cleanly.
    assert!(
        first_span_page.next_cursor.is_some(),
        "first span page should advertise a next cursor"
    );

    let second_span_page = store
        .query_spans(
            tenant.clone(),
            SpanFilter {
                trace_id: Some(trace.clone()),
                ..SpanFilter::default()
            },
            PageRequest {
                limit: 1,
                cursor: first_span_page.next_cursor,
            },
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(second_span_page.items.len(), 1);
    assert_eq!(second_span_page.items[0].span_id.as_str(), "root");
    assert_eq!(second_span_page.next_cursor, None);

    let other_tenant = TenantId::new("other").unwrap_or_else(|err| panic!("{err}"));
    let empty = store
        .query_spans(other_tenant, SpanFilter::default(), PageRequest::default())
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert!(empty.items.is_empty());

    let runs = store
        .query_runs(tenant, RunFilter::default(), PageRequest::default())
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(runs.items.len(), 1);
    assert_eq!(runs.items[0].project_id.as_str(), project.as_str());
    assert_eq!(runs.items[0].span_count, 2);

    let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
    let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
    let other_project = ProjectId::new("other-project").unwrap_or_else(|err| panic!("{err}"));
    let trace = TraceId::new("trace").unwrap_or_else(|err| panic!("{err}"));
    let other_project_batch =
        fixture_project_batch(&tenant, &other_project, &trace, "other-project-root", 3);
    let write_other_project = store
        .write_batch(other_project_batch)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(write_other_project.accepted_spans, 1);

    let same_trace_runs = store
        .query_runs(
            tenant.clone(),
            RunFilter {
                trace_id: Some(trace.clone()),
                ..RunFilter::default()
            },
            PageRequest {
                limit: 10,
                cursor: None,
            },
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(same_trace_runs.items.len(), 2);
    assert!(same_trace_runs
        .items
        .iter()
        .any(|run| run.project_id.as_str() == project.as_str() && run.span_count == 2));
    assert!(same_trace_runs
        .items
        .iter()
        .any(|run| run.project_id.as_str() == other_project.as_str() && run.span_count == 1));

    let project_runs = store
        .query_runs(
            tenant.clone(),
            RunFilter {
                project_id: Some(project.clone()),
                trace_id: Some(trace.clone()),
                ..RunFilter::default()
            },
            PageRequest {
                limit: 10,
                cursor: None,
            },
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(project_runs.items.len(), 1);
    assert_eq!(project_runs.items[0].project_id.as_str(), project.as_str());
    assert_eq!(project_runs.items[0].span_count, 2);

    let other_project_runs = store
        .query_runs(
            tenant.clone(),
            RunFilter {
                project_id: Some(other_project.clone()),
                trace_id: Some(trace.clone()),
                ..RunFilter::default()
            },
            PageRequest {
                limit: 10,
                cursor: None,
            },
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(other_project_runs.items.len(), 1);
    assert_eq!(
        other_project_runs.items[0].project_id.as_str(),
        other_project.as_str()
    );
    assert_eq!(other_project_runs.items[0].span_count, 1);

    let scoped_spans = store
        .query_spans(
            tenant.clone(),
            SpanFilter {
                project_id: Some(project.clone()),
                trace_id: Some(trace.clone()),
                ..SpanFilter::default()
            },
            PageRequest {
                limit: 10,
                cursor: None,
            },
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(scoped_spans.items.len(), 2);
    assert!(scoped_spans
        .items
        .iter()
        .all(|span| span.project_id.as_str() == project.as_str()));

    let scoped_project = store
        .get_project_trace(tenant.clone(), project, trace.clone())
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let scoped_other_project = store
        .get_project_trace(tenant, other_project, trace)
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    assert_eq!(scoped_project.spans.len(), 2);
    assert!(scoped_project
        .spans
        .iter()
        .all(|span| span.project_id.as_str() == "project"));
    assert_eq!(scoped_other_project.spans.len(), 1);
    assert_eq!(
        scoped_other_project.spans[0].span_id.as_str(),
        "other-project-root"
    );
    assert_eq!(
        scoped_other_project.spans[0].project_id.as_str(),
        "other-project"
    );

    let pagination_tenant =
        TenantId::new("pagination-tenant").unwrap_or_else(|err| panic!("{err}"));
    let other_pagination_tenant =
        TenantId::new("other-pagination-tenant").unwrap_or_else(|err| panic!("{err}"));
    let pagination_project =
        ProjectId::new("pagination-project").unwrap_or_else(|err| panic!("{err}"));
    let pagination_other_project =
        ProjectId::new("pagination-other-project").unwrap_or_else(|err| panic!("{err}"));
    let pagination_traces = [
        TraceId::new("pagination-trace-1").unwrap_or_else(|err| panic!("{err}")),
        TraceId::new("pagination-trace-2").unwrap_or_else(|err| panic!("{err}")),
        TraceId::new("pagination-trace-3").unwrap_or_else(|err| panic!("{err}")),
    ];
    for (index, trace) in pagination_traces.iter().enumerate() {
        let write = store
            .write_batch(fixture_project_batch(
                &pagination_tenant,
                &pagination_project,
                trace,
                &format!("pagination-span-{}", index + 1),
                10 + index as u64,
            ))
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(write.accepted_spans, 1);
    }
    let write_other_project = store
        .write_batch(fixture_project_batch(
            &pagination_tenant,
            &pagination_other_project,
            &TraceId::new("pagination-other-project-trace").unwrap_or_else(|err| panic!("{err}")),
            "pagination-other-project-span",
            20,
        ))
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(write_other_project.accepted_spans, 1);
    let write_other_tenant = store
        .write_batch(fixture_project_batch(
            &other_pagination_tenant,
            &pagination_project,
            &TraceId::new("pagination-other-tenant-trace").unwrap_or_else(|err| panic!("{err}")),
            "pagination-other-tenant-span",
            30,
        ))
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(write_other_tenant.accepted_spans, 1);

    let first_span_page = store
        .query_spans(
            pagination_tenant.clone(),
            SpanFilter {
                project_id: Some(pagination_project.clone()),
                ..SpanFilter::default()
            },
            PageRequest {
                limit: 2,
                cursor: None,
            },
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(first_span_page.items.len(), 2);
    assert_eq!(
        span_ids(&first_span_page.items),
        vec!["pagination-span-3", "pagination-span-2"]
    );
    let span_cursor = first_span_page
        .next_cursor
        .unwrap_or_else(|| panic!("span page should expose a next cursor"));

    let second_span_page = store
        .query_spans(
            pagination_tenant.clone(),
            SpanFilter {
                project_id: Some(pagination_project.clone()),
                ..SpanFilter::default()
            },
            PageRequest {
                limit: 2,
                cursor: Some(span_cursor),
            },
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(second_span_page.items.len(), 1);
    assert_eq!(span_ids(&second_span_page.items), vec!["pagination-span-1"]);
    assert_eq!(second_span_page.next_cursor, None);
    assert!(first_span_page
        .items
        .iter()
        .chain(second_span_page.items.iter())
        .all(|span| span.tenant_id == pagination_tenant && span.project_id == pagination_project));

    let zero_limit_span_page = store
        .query_spans(
            pagination_tenant.clone(),
            SpanFilter {
                project_id: Some(pagination_project.clone()),
                ..SpanFilter::default()
            },
            PageRequest {
                limit: 0,
                cursor: None,
            },
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(
        zero_limit_span_page.items.len(),
        1,
        "limit: 0 must normalize to a single-item page rather than returning nothing"
    );
    assert_eq!(
        span_ids(&zero_limit_span_page.items),
        vec!["pagination-span-3"]
    );
    // The cursor is opaque (in-memory offset vs. SQLite keyset seek key); only
    // assert that a further page is advertised, never its literal encoding.
    assert!(
        zero_limit_span_page.next_cursor.is_some(),
        "a normalized limit: 0 page over 3 spans must advertise a next cursor"
    );

    let first_run_page = store
        .query_runs(
            pagination_tenant.clone(),
            RunFilter {
                project_id: Some(pagination_project.clone()),
                ..RunFilter::default()
            },
            PageRequest {
                limit: 1,
                cursor: None,
            },
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(first_run_page.items.len(), 1);
    assert_eq!(
        first_run_page.items[0].trace_id.as_str(),
        "pagination-trace-3"
    );
    assert_eq!(first_run_page.items[0].project_id, pagination_project);
    assert_eq!(first_run_page.items[0].tenant_id, pagination_tenant);
    let run_cursor = first_run_page
        .next_cursor
        .unwrap_or_else(|| panic!("run page should expose a next cursor"));

    let second_run_page = store
        .query_runs(
            pagination_tenant.clone(),
            RunFilter {
                project_id: Some(pagination_project.clone()),
                ..RunFilter::default()
            },
            PageRequest {
                limit: 2,
                cursor: Some(run_cursor),
            },
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(second_run_page.items.len(), 2);
    assert_eq!(
        run_trace_ids(&second_run_page.items),
        vec!["pagination-trace-2", "pagination-trace-1"]
    );
    assert_eq!(second_run_page.next_cursor, None);
    assert!(second_run_page
        .items
        .iter()
        .all(|run| run.tenant_id == pagination_tenant && run.project_id == pagination_project));
}

/// Conformance for **keyset (seek) span pagination** (ARCHITECTURE.md §20.2
/// #0.2): pages must stay stable when a row is inserted into an
/// already-returned page between page fetches.
///
/// This is the property an OFFSET cursor violates — a new high-sorting row
/// shifts every subsequent offset down by one, so the next page re-returns the
/// previous page's last row (a duplicate) and skips a row that should appear.
/// A keyset cursor seeks past the last row actually returned, so the in-flight
/// insert can neither duplicate nor skip an already-paginated row.
///
/// Only call this against backends whose `query_spans` is keyset-based; the
/// in-memory store paginates by offset and is exempt by design.
pub async fn assert_span_pagination_keyset_stability<S>(store: S)
where
    S: TraceStore,
{
    let tenant = TenantId::new("keyset-tenant").unwrap_or_else(|err| panic!("{err}"));
    let project = ProjectId::new("keyset-project").unwrap_or_else(|err| panic!("{err}"));

    // Seed four spans. `seq` drives `start_time` (base + seq seconds), so the
    // newest-first order is seq 4, 3, 2, 1.
    for seq in 1..=4u64 {
        let trace =
            TraceId::new(format!("keyset-trace-{seq}")).unwrap_or_else(|err| panic!("{err}"));
        let ack = store
            .write_batch(fixture_project_batch(
                &tenant,
                &project,
                &trace,
                &format!("keyset-span-{seq}"),
                seq,
            ))
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(ack.accepted_spans, 1);
    }

    let filter = || SpanFilter {
        project_id: Some(project.clone()),
        ..SpanFilter::default()
    };

    // Page 1 (newest two): keyset-span-4, keyset-span-3.
    let first_page = store
        .query_spans(
            tenant.clone(),
            filter(),
            PageRequest {
                limit: 2,
                cursor: None,
            },
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let first_ids: Vec<String> = first_page
        .items
        .iter()
        .map(|span| span.span_id.as_str().to_string())
        .collect();
    assert_eq!(
        first_ids,
        vec!["keyset-span-4".to_string(), "keyset-span-3".to_string()],
        "page 1 should return the two newest spans newest-first"
    );
    let cursor = first_page
        .next_cursor
        .unwrap_or_else(|| panic!("page 1 should advertise a next cursor"));

    // Concurrent insert: a new span (seq 9) that sorts to the *top* — i.e. into
    // the already-returned page-1 region, ahead of the cursor.
    let hot_trace = TraceId::new("keyset-trace-hot").unwrap_or_else(|err| panic!("{err}"));
    let hot_ack = store
        .write_batch(fixture_project_batch(
            &tenant,
            &project,
            &hot_trace,
            "keyset-span-hot",
            9,
        ))
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(hot_ack.accepted_spans, 1);

    // Page 2 resumes from the cursor. With a keyset cursor this is exactly the
    // remaining tail (keyset-span-2, keyset-span-1) regardless of the insert.
    let second_page = store
        .query_spans(
            tenant.clone(),
            filter(),
            PageRequest {
                limit: 2,
                cursor: Some(cursor),
            },
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let second_ids: Vec<String> = second_page
        .items
        .iter()
        .map(|span| span.span_id.as_str().to_string())
        .collect();

    assert_eq!(
        second_ids,
        vec!["keyset-span-2".to_string(), "keyset-span-1".to_string()],
        "page 2 must be the stable tail, unaffected by the concurrent insert"
    );
    // No row appears on both pages (OFFSET would duplicate keyset-span-3 here).
    assert!(
        second_ids.iter().all(|id| !first_ids.contains(id)),
        "keyset pagination must not duplicate an already-returned row"
    );
    // The concurrently inserted high-sorting row is not surfaced mid-stream on a
    // page it does not belong to.
    assert!(
        !second_ids.iter().any(|id| id == "keyset-span-hot"),
        "the concurrently inserted row must not leak into a later page"
    );
    // Nothing was skipped: every originally-visible span past the cursor is
    // still returned exactly once across the two pages.
    let mut seen: Vec<String> = first_ids;
    seen.extend(second_ids);
    for seq in 1..=4u64 {
        let id = format!("keyset-span-{seq}");
        assert_eq!(
            seen.iter().filter(|seen_id| **seen_id == id).count(),
            1,
            "{id} should appear exactly once across the paginated stream"
        );
    }
}

/// Conformance for the **`seq` tiebreaker** in keyset span pagination.
///
/// The trace-store PRIMARY KEY is `(tenant, project, trace, span_id, seq)`, so a
/// re-emitted span shares its `span_id` (and, in practice, its `start_time`)
/// with the earlier version and differs only in `seq`. A keyset key of just
/// `(start_time, span_id)` is therefore NOT unique: if a page boundary lands
/// between two versions, a strict `span_id < cursor` predicate excludes the
/// equal-`span_id` sibling and the second version is silently SKIPPED. The key
/// must carry `seq` as the final tiebreaker so every version is returned exactly
/// once.
///
/// This seeds two versions of one span — identical `span_id` AND identical
/// `start_time`, distinguished only by `seq` (and `name`, so the summaries are
/// distinguishable) — then paginates with `limit = 1` across the boundary and
/// asserts BOTH versions come back, with no skip and no duplicate.
///
/// Only call this against backends whose `query_spans` is keyset-based; the
/// in-memory store paginates by offset and is exempt by design.
pub async fn assert_span_pagination_seq_tiebreak<S>(store: S)
where
    S: TraceStore,
{
    let tenant = TenantId::new("seq-tenant").unwrap_or_else(|err| panic!("{err}"));
    let project = ProjectId::new("seq-project").unwrap_or_else(|err| panic!("{err}"));
    let trace = TraceId::new("seq-trace").unwrap_or_else(|err| panic!("{err}"));

    // Two versions of the SAME span: identical span_id AND start_time, differing
    // only in `seq`. They go in one batch (a re-emission), so both rows persist
    // under the composite PRIMARY KEY. Distinct names let us tell the summaries
    // apart even though their span_id is identical.
    let batch = fixture_versioned_span_batch(
        &tenant,
        &project,
        &trace,
        "dup-span",
        &[(1, "version one"), (2, "version two")],
    );
    let ack = store
        .write_batch(batch)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(
        ack.accepted_spans, 2,
        "both seq versions of the span must persist (distinct PRIMARY KEY)"
    );

    let filter = || SpanFilter {
        project_id: Some(project.clone()),
        span_id: Some(SpanId::new("dup-span").unwrap_or_else(|err| panic!("{err}"))),
        ..SpanFilter::default()
    };

    // Walk the two versions one page at a time. With a unique keyset key this
    // visits each version exactly once; with the buggy `(start_time, span_id)`
    // key the second version is skipped at the page boundary.
    let mut names: Vec<String> = Vec::new();
    let mut cursor = None;
    for _ in 0..3 {
        let page = store
            .query_spans(tenant.clone(), filter(), PageRequest { limit: 1, cursor })
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        for span in &page.items {
            assert_eq!(
                span.span_id.as_str(),
                "dup-span",
                "every page item is a version of the re-emitted span"
            );
            names.push(span.name.clone());
        }
        match page.next_cursor {
            Some(next) => cursor = Some(next),
            None => break,
        }
    }

    names.sort();
    assert_eq!(
        names,
        vec!["version one".to_string(), "version two".to_string()],
        "both seq versions must be returned exactly once across the keyset pages \
         (no skip from a non-unique (start_time, span_id) key, no duplicate)"
    );
}

/// Conformance for tenant-wide keyset pagination when two rows tie on
/// `(start_time, span_id, seq)` and differ only by project/trace.
///
/// `query_spans` allows `project_id` and `trace_id` to be omitted, so the keyset
/// cursor must remain total across all traces in the tenant. Without
/// `project_id` and `trace_id` in the cursor/order, a page boundary between
/// these rows treats the second row as equal to the cursor and skips it.
///
/// Only call this against backends whose `query_spans` is keyset-based; the
/// in-memory store paginates by offset and is exempt by design.
pub async fn assert_span_pagination_tenant_wide_tiebreak<S>(store: S)
where
    S: TraceStore,
{
    let tenant = TenantId::new("tenant-wide-keyset").unwrap_or_else(|err| panic!("{err}"));
    let project_a = ProjectId::new("project-a").unwrap_or_else(|err| panic!("{err}"));
    let project_b = ProjectId::new("project-b").unwrap_or_else(|err| panic!("{err}"));
    let trace_a = TraceId::new("trace-a").unwrap_or_else(|err| panic!("{err}"));
    let trace_b = TraceId::new("trace-b").unwrap_or_else(|err| panic!("{err}"));

    for (project, trace, name) in [
        (&project_a, &trace_a, "project a span"),
        (&project_b, &trace_b, "project b span"),
    ] {
        let ack = store
            .write_batch(fixture_versioned_span_batch(
                &tenant,
                project,
                trace,
                "shared-span",
                &[(1, name)],
            ))
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(ack.accepted_spans, 1);
    }

    let filter = || SpanFilter {
        span_id: Some(SpanId::new("shared-span").unwrap_or_else(|err| panic!("{err}"))),
        ..SpanFilter::default()
    };

    let mut names: Vec<String> = Vec::new();
    let mut cursor = None;
    for _ in 0..3 {
        let page = store
            .query_spans(tenant.clone(), filter(), PageRequest { limit: 1, cursor })
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        names.extend(page.items.iter().map(|span| span.name.clone()));
        match page.next_cursor {
            Some(next) => cursor = Some(next),
            None => break,
        }
    }

    names.sort();
    assert_eq!(
        names,
        vec!["project a span".to_string(), "project b span".to_string()],
        "tenant-wide keyset pagination must return every tied row exactly once"
    );
}

pub async fn assert_metadata_store_conformance<S>(store: S)
where
    S: MetadataStore,
{
    let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
    let organization = OrganizationId::new("org").unwrap_or_else(|err| panic!("{err}"));
    let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
    let environment = EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}"));
    let created_at = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
        .single()
        .unwrap_or_else(|| panic!("valid timestamp"));

    store
        .put_organization(OrganizationMetadata {
            tenant_id: tenant.clone(),
            organization_id: organization.clone(),
            display_name: "Org".to_string(),
            created_at,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    store
        .put_project(ProjectMetadata {
            tenant_id: tenant.clone(),
            organization_id: organization.clone(),
            project_id: project.clone(),
            display_name: "Project".to_string(),
            created_at,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    store
        .put_environment(beater_store::EnvironmentMetadata {
            tenant_id: tenant.clone(),
            project_id: project.clone(),
            environment_id: environment.clone(),
            display_name: "Production".to_string(),
            created_at,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    store
        .put_role_binding(RoleBinding {
            tenant_id: tenant.clone(),
            project_id: Some(project.clone()),
            principal_id: "api-key:admin".to_string(),
            role: "project_admin".to_string(),
            permissions: BTreeSet::from(["admin".to_string()]),
            created_at,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    let loaded_environment = store
        .get_environment(tenant.clone(), project.clone(), environment)
        .await
        .unwrap_or_else(|err| panic!("{err}"))
        .unwrap_or_else(|| panic!("environment should exist"));
    assert_eq!(loaded_environment.display_name, "Production");

    let bindings = store
        .list_role_bindings(tenant, Some(project), "api-key:admin".to_string())
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(bindings.len(), 1);
    assert!(bindings[0].permissions.contains("admin"));
}

pub async fn assert_quota_limiter_conformance<L>(limiter: L)
where
    L: QuotaLimiter,
{
    let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
    let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
    let window_start = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
        .single()
        .unwrap_or_else(|| panic!("valid timestamp"));
    let reset_at = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 1, 0)
        .single()
        .unwrap_or_else(|| panic!("valid timestamp"));

    let first = limiter
        .reserve_quota(QuotaReservationRequest {
            tenant_id: tenant.clone(),
            project_id: project.clone(),
            amount: 2,
            limit: 3,
            window_start,
            reset_at,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert!(first.accepted);
    assert_eq!(first.used, 2);

    let rejected = limiter
        .reserve_quota(QuotaReservationRequest {
            tenant_id: tenant.clone(),
            project_id: project.clone(),
            amount: 2,
            limit: 3,
            window_start,
            reset_at,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert!(!rejected.accepted);
    assert_eq!(rejected.used, 2);
    assert_eq!(rejected.reset_at, reset_at);

    let next_window = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 1, 0)
        .single()
        .unwrap_or_else(|| panic!("valid timestamp"));
    let next_reset = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 2, 0)
        .single()
        .unwrap_or_else(|| panic!("valid timestamp"));
    let after_reset = limiter
        .reserve_quota(QuotaReservationRequest {
            tenant_id: tenant,
            project_id: project,
            amount: 3,
            limit: 3,
            window_start: next_window,
            reset_at: next_reset,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert!(after_reset.accepted);
    assert_eq!(after_reset.used, 3);
}

/// Concurrency invariant for any [`QuotaLimiter`]: under a storm of simultaneous
/// reservations against a *shared* limiter the counter must never overcommit the
/// window. Concretely, for `limit` and per-reservation `amount`:
///
/// * the number of *granted* reservations is exactly `limit / amount`
///   (integer division) — never more, even if many reservers read the counter
///   at the same instant;
/// * `used` is `<= limit` in *every* decision the limiter ever returns;
/// * a denied reservation never advances the counter (the settled `used` equals
///   `granted * amount`).
///
/// This is the billing-critical guard against the classic check-then-act race:
/// two reservers both read a stale `used`, each decide they fit, and both
/// commit — overcommitting the window. A correct limiter makes the
/// read-modify-write atomic so the reservations serialize.
///
/// Runs cleanly against both the in-memory and SQLite backends, so it lives in
/// the shared conformance suite.
pub async fn assert_quota_limiter_concurrency_conformance<L>(limiter: L)
where
    L: QuotaLimiter + 'static,
{
    let limiter = Arc::new(limiter);
    let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
    let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));

    // 50 reservers each asking for 1 against a limit of 10 -> exactly 10 granted.
    let window_one = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
        .single()
        .unwrap_or_else(|| panic!("valid timestamp"));
    let reset_one = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 1, 0)
        .single()
        .unwrap_or_else(|| panic!("valid timestamp"));
    assert_reservation_storm(
        &limiter, &tenant, &project, window_one, reset_one, 50, 1, 10,
    )
    .await;

    // amount > 1: 50 reservers each asking for 3 against a limit of 10. At most
    // 3 may be granted (9 used); a 4th would push used to 12 > 10 and must be
    // denied. A fresh window keeps this independent from the first storm.
    let window_two = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 1, 0)
        .single()
        .unwrap_or_else(|| panic!("valid timestamp"));
    let reset_two = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 2, 0)
        .single()
        .unwrap_or_else(|| panic!("valid timestamp"));
    assert_reservation_storm(
        &limiter, &tenant, &project, window_two, reset_two, 50, 3, 10,
    )
    .await;
}

#[allow(clippy::too_many_arguments)]
async fn assert_reservation_storm<L>(
    limiter: &Arc<L>,
    tenant: &TenantId,
    project: &ProjectId,
    window_start: DateTime<Utc>,
    reset_at: DateTime<Utc>,
    reservers: usize,
    amount: u64,
    limit: u64,
) where
    L: QuotaLimiter + 'static,
{
    let mut handles = Vec::with_capacity(reservers);
    for _ in 0..reservers {
        let limiter = Arc::clone(limiter);
        let request = QuotaReservationRequest {
            tenant_id: tenant.clone(),
            project_id: project.clone(),
            amount,
            limit,
            window_start,
            reset_at,
        };
        handles.push(tokio::spawn(
            async move { limiter.reserve_quota(request).await },
        ));
    }

    let mut decisions = Vec::with_capacity(reservers);
    for handle in handles {
        let decision = handle
            .await
            .unwrap_or_else(|err| panic!("reservation task panicked: {err}"))
            .unwrap_or_else(|err| panic!("reserve_quota failed: {err}"));
        decisions.push(decision);
    }

    // Invariant: the limiter never reports a counter past the limit, in any
    // interleaving, in any decision.
    for decision in &decisions {
        assert!(
            decision.used <= limit,
            "observed used {} exceeding limit {}",
            decision.used,
            limit
        );
    }

    // Invariant: exactly floor(limit / amount) reservations are granted.
    let granted = decisions
        .iter()
        .filter(|decision| decision.accepted)
        .count() as u64;
    let expected = limit / amount;
    assert_eq!(
        granted, expected,
        "granted {granted} reservations against limit {limit} amount {amount}, expected {expected}"
    );

    // Invariant: the settled counter equals granted * amount and never exceeds
    // the limit. A reservation that is guaranteed to overflow is denied and
    // reports the true settled `used`.
    let settle = limiter
        .reserve_quota(QuotaReservationRequest {
            tenant_id: tenant.clone(),
            project_id: project.clone(),
            amount: limit + 1,
            limit,
            window_start,
            reset_at,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert!(!settle.accepted);
    assert_eq!(
        settle.used,
        granted * amount,
        "settled used {} did not match granted {granted} * amount {amount}",
        settle.used
    );
    assert!(settle.used <= limit);
}

fn fixture_batch() -> (
    CanonicalTraceBatch,
    TenantId,
    ProjectId,
    TraceId,
    IdempotencyKey,
) {
    let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
    let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
    let environment = EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}"));
    let trace = TraceId::new("trace").unwrap_or_else(|err| panic!("{err}"));
    let idempotency_key =
        IdempotencyKey::new("tenant:project:trace:raw").unwrap_or_else(|err| panic!("{err}"));
    let body_ref = artifact_ref("raw");
    let raw = RawEnvelope {
        schema_version: RAW_SCHEMA_VERSION,
        tenant_id: tenant.clone(),
        project_id: project.clone(),
        environment_id: environment.clone(),
        source: SourceDialect::Native,
        source_schema_url: Some("beater://native/v1".to_string()),
        source_schema_version: Some("1".to_string()),
        received_at: fixture_base_time(),
        idempotency_key: idempotency_key.clone(),
        payload_hash: body_ref.sha256.clone(),
        body_ref: body_ref.clone(),
        auth_context: AuthContext {
            api_key_id: None,
            scopes: BTreeSet::new(),
        },
    };
    let root = canonical_span(CanonicalSpanFixture {
        tenant: &tenant,
        project: &project,
        environment: &environment,
        trace: &trace,
        span: "root",
        seq: 1,
        kind: AgentSpanKind::AgentRun,
        name: "run",
        raw_ref: body_ref.clone(),
    });
    let child = canonical_span(CanonicalSpanFixture {
        tenant: &tenant,
        project: &project,
        environment: &environment,
        trace: &trace,
        span: "child",
        seq: 2,
        kind: AgentSpanKind::AgentStep,
        name: "step",
        raw_ref: body_ref,
    });
    (
        CanonicalTraceBatch {
            raw_envelopes: vec![raw],
            spans: vec![child, root],
        },
        tenant,
        project,
        trace,
        idempotency_key,
    )
}

fn fixture_project_batch(
    tenant: &TenantId,
    project: &ProjectId,
    trace: &TraceId,
    span_id: &str,
    seq: u64,
) -> CanonicalTraceBatch {
    let environment = EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}"));
    let idempotency_key = IdempotencyKey::new(format!(
        "{}:{}:{}:raw",
        tenant.as_str(),
        project.as_str(),
        trace.as_str()
    ))
    .unwrap_or_else(|err| panic!("{err}"));
    let body_ref = artifact_ref("other-project-raw");
    let raw = RawEnvelope {
        schema_version: RAW_SCHEMA_VERSION,
        tenant_id: tenant.clone(),
        project_id: project.clone(),
        environment_id: environment.clone(),
        source: SourceDialect::Native,
        source_schema_url: Some("beater://native/v1".to_string()),
        source_schema_version: Some("1".to_string()),
        received_at: fixture_base_time(),
        idempotency_key,
        payload_hash: body_ref.sha256.clone(),
        body_ref: body_ref.clone(),
        auth_context: AuthContext {
            api_key_id: None,
            scopes: BTreeSet::new(),
        },
    };
    let span = canonical_span(CanonicalSpanFixture {
        tenant,
        project,
        environment: &environment,
        trace,
        span: span_id,
        seq,
        kind: AgentSpanKind::AgentRun,
        name: "other project run",
        raw_ref: body_ref,
    });
    CanonicalTraceBatch {
        raw_envelopes: vec![raw],
        spans: vec![span],
    }
}

/// Builds one batch holding several *versions* of a single span: every entry
/// shares the same `trace`, `span_id`, and `start_time` and differs only in
/// `(seq, name)`. The composite PRIMARY KEY keeps each `(span_id, seq)` row
/// distinct, so this models a span re-emitted under a new `seq`. The `start_time`
/// is pinned to `fixture_base_time()` for *all* versions (not offset by `seq`),
/// which is exactly the collision the keyset `seq` tiebreaker must survive.
fn fixture_versioned_span_batch(
    tenant: &TenantId,
    project: &ProjectId,
    trace: &TraceId,
    span_id: &str,
    versions: &[(u64, &str)],
) -> CanonicalTraceBatch {
    let environment = EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}"));
    let idempotency_key = IdempotencyKey::new(format!(
        "{}:{}:{}:versioned",
        tenant.as_str(),
        project.as_str(),
        trace.as_str()
    ))
    .unwrap_or_else(|err| panic!("{err}"));
    let body_ref = artifact_ref("versioned-span-raw");
    let raw = RawEnvelope {
        schema_version: RAW_SCHEMA_VERSION,
        tenant_id: tenant.clone(),
        project_id: project.clone(),
        environment_id: environment.clone(),
        source: SourceDialect::Native,
        source_schema_url: Some("beater://native/v1".to_string()),
        source_schema_version: Some("1".to_string()),
        received_at: fixture_base_time(),
        idempotency_key,
        payload_hash: body_ref.sha256.clone(),
        body_ref: body_ref.clone(),
        auth_context: AuthContext {
            api_key_id: None,
            scopes: BTreeSet::new(),
        },
    };
    let spans = versions
        .iter()
        .map(|&(seq, name)| {
            let mut span = canonical_span(CanonicalSpanFixture {
                tenant,
                project,
                environment: &environment,
                trace,
                span: span_id,
                seq,
                kind: AgentSpanKind::AgentRun,
                name,
                raw_ref: body_ref.clone(),
            });
            // Pin every version to the same instant so only `seq` breaks the tie.
            span.start_time = fixture_base_time();
            span
        })
        .collect();
    CanonicalTraceBatch {
        raw_envelopes: vec![raw],
        spans,
    }
}

struct CanonicalSpanFixture<'a> {
    tenant: &'a TenantId,
    project: &'a ProjectId,
    environment: &'a EnvironmentId,
    trace: &'a TraceId,
    span: &'a str,
    seq: u64,
    kind: AgentSpanKind,
    name: &'a str,
    raw_ref: ArtifactRef,
}

fn canonical_span(fixture: CanonicalSpanFixture<'_>) -> CanonicalSpan {
    CanonicalSpan {
        schema_version: beater_schema::CANONICAL_SCHEMA_VERSION,
        normalizer_version: "beater-native-v1".to_string(),
        tenant_id: fixture.tenant.clone(),
        project_id: fixture.project.clone(),
        environment_id: fixture.environment.clone(),
        trace_id: fixture.trace.clone(),
        span_id: SpanId::new(fixture.span).unwrap_or_else(|err| panic!("{err}")),
        parent_span_id: None,
        seq: fixture.seq,
        kind: fixture.kind,
        name: fixture.name.to_string(),
        status: SpanStatus::Ok,
        start_time: fixture_base_time() + Duration::seconds(fixture.seq as i64),
        end_time: None,
        model: None,
        cost: None,
        tokens: None,
        input_ref: None,
        output_ref: None,
        attributes: BTreeMap::new(),
        unmapped_attrs: json!({}),
        raw_ref: fixture.raw_ref,
    }
}

fn artifact_ref(name: &str) -> ArtifactRef {
    ArtifactRef {
        artifact_id: ArtifactId::new(name).unwrap_or_else(|err| panic!("{err}")),
        uri: format!("artifact://tenant/project/{name}"),
        sha256: Sha256Hash::new("ab".repeat(32)).unwrap_or_else(|err| panic!("{err}")),
        size_bytes: 2,
        mime_type: "application/json".to_string(),
        redaction_class: RedactionClass::Internal,
    }
}

fn span_ids(spans: &[beater_schema::SpanSummary]) -> Vec<&str> {
    spans.iter().map(|span| span.span_id.as_str()).collect()
}

fn run_trace_ids(runs: &[beater_schema::RunSummary]) -> Vec<&str> {
    runs.iter().map(|run| run.trace_id.as_str()).collect()
}
