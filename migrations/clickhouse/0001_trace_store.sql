-- ClickHouse scale trace-store contract.
-- Tenant id leads every ORDER BY key to make tenant-scoped reads the natural
-- access path and to avoid post-fetch isolation.

CREATE DATABASE IF NOT EXISTS beater;

CREATE TABLE IF NOT EXISTS beater.raw_envelopes
(
  tenant_id String,
  project_id String,
  idempotency_key String,
  trace_id Nullable(String),
  payload_hash String,
  received_at DateTime64(6, 'UTC'),
  source LowCardinality(String),
  normalizer_version LowCardinality(String),
  raw_json String
)
ENGINE = MergeTree
PARTITION BY toYYYYMM(received_at)
ORDER BY (tenant_id, project_id, received_at, idempotency_key)
TTL toDateTime(received_at) + INTERVAL 180 DAY
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS beater.spans
(
  tenant_id String,
  project_id String,
  environment_id String,
  trace_id String,
  span_id String,
  parent_span_id Nullable(String),
  seq UInt64,
  kind LowCardinality(String),
  status LowCardinality(String),
  name String,
  start_time DateTime64(6, 'UTC'),
  end_time Nullable(DateTime64(6, 'UTC')),
  duration_ms Nullable(Int64),
  model_provider Nullable(String),
  model_name Nullable(String),
  cost_currency LowCardinality(Nullable(String)),
  cost_micros Nullable(Int64),
  release_id Nullable(String),
  span_json String
)
ENGINE = MergeTree
PARTITION BY toYYYYMM(start_time)
ORDER BY (tenant_id, project_id, environment_id, trace_id, start_time, span_id, seq)
TTL toDateTime(start_time) + INTERVAL 90 DAY
SETTINGS index_granularity = 8192;

-- Run summaries are computed on demand by ClickHouseTraceStore::query_runs with a
-- backend GROUP BY (project_id, trace_id) over the per-span roll-up columns
-- (model_provider, model_name, cost_currency, cost_micros, release_id), which the
-- store now fills at write time. This satisfies ARCHITECTURE.md §8.1: run
-- summaries are aggregated in the backend rather than by materializing every
-- matching span in process.
--
-- Run summaries are still NOT precomputed into a table or materialized view. A
-- `beater.trace_runs` table + `beater.trace_runs_mv` materialized view previously
-- lived here but were removed. Do not reintroduce a run-summary table without a
-- read path that queries it; the on-demand GROUP BY above is the supported path.
