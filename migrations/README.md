# Beater Migrations

These migrations define the durable schema contracts for the self-hosted and
scale storage topology.

The current `beaterd` all-in-one binary still uses SQLite files under
`--data-dir` for the local OSS runtime. The Postgres and ClickHouse migrations
are checked in now so containerized self-host deployments have explicit schema
contracts and so future Postgres/ClickHouse `TraceStore` backends can be held to
the same shape instead of inventing incompatible tables.

- `sqlite/`: local OSS runtime schema for `beaterd --data-dir`, including trace,
  queue, idempotency, auth, dataset, review, eval, gate, audit, usage, replay,
  and schema-migration metadata.
- `postgres/`: transactional metadata, quota, bus, auth, dataset, review, eval,
  gate, audit, usage, replay, and trace hot-store schema.
- `clickhouse/`: scale-oriented trace/raw-event schema with tenant-leading sort
  keys and cold/hot retention settings.
