//! Live Postgres `TraceStore` conformance.
//!
//! Boots a throwaway Postgres container, runs the checked-in migration, and runs
//! the shared store-conformance suite against `PgTraceStore`. The test is
//! `#[ignore]`d so a Docker-less `cargo test` still passes; run it explicitly on
//! a Docker CI runner with:
//!
//! ```sh
//! cargo test -p beater-store-sql --test pg_trace_store -- --ignored
//! ```

#![cfg(feature = "postgres")]

use beater_store_conformance::assert_trace_store_conformance;
use beater_store_sql::PgTraceStore;
use testcontainers::{
    GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};

#[tokio::test]
#[ignore = "requires Docker; run with --ignored on a Docker CI runner"]
async fn pg_trace_store_conforms() {
    let container = GenericImage::new("postgres", "16-alpine")
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_exposed_port(5432.tcp())
        .with_env_var("POSTGRES_USER", "beater")
        .with_env_var("POSTGRES_PASSWORD", "beater")
        .with_env_var("POSTGRES_DB", "beater")
        .start()
        .await
        .unwrap_or_else(|err| panic!("start postgres container: {err}"));

    let host = container
        .get_host()
        .await
        .unwrap_or_else(|err| panic!("container host: {err}"));
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .unwrap_or_else(|err| panic!("container port: {err}"));
    let connection_string =
        format!("host={host} port={port} user=beater password=beater dbname=beater");

    let store = PgTraceStore::connect(&connection_string)
        .await
        .unwrap_or_else(|err| panic!("connect/migrate postgres: {err}"));

    assert_trace_store_conformance(store).await;
}

#[tokio::test]
#[ignore = "requires Docker; run with --ignored on a Docker CI runner"]
async fn pg_migration_backfills_run_rollup_columns_from_span_json() {
    let container = GenericImage::new("postgres", "16-alpine")
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_exposed_port(5432.tcp())
        .with_env_var("POSTGRES_USER", "beater")
        .with_env_var("POSTGRES_PASSWORD", "beater")
        .with_env_var("POSTGRES_DB", "beater")
        .start()
        .await
        .unwrap_or_else(|err| panic!("start postgres container: {err}"));

    let host = container
        .get_host()
        .await
        .unwrap_or_else(|err| panic!("container host: {err}"));
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .unwrap_or_else(|err| panic!("container port: {err}"));
    let connection_string =
        format!("host={host} port={port} user=beater password=beater dbname=beater");

    let (client, connection) = tokio_postgres::connect(&connection_string, tokio_postgres::NoTls)
        .await
        .unwrap_or_else(|err| panic!("connect postgres: {err}"));
    tokio::spawn(async move {
        if let Err(err) = connection.await {
            panic!("postgres connection task failed: {err}");
        }
    });
    client
        .batch_execute(
            r#"
            CREATE TABLE spans (
              tenant_id TEXT NOT NULL,
              project_id TEXT NOT NULL,
              environment_id TEXT NOT NULL,
              trace_id TEXT NOT NULL,
              span_id TEXT NOT NULL,
              seq BIGINT NOT NULL,
              kind TEXT NOT NULL,
              status TEXT NOT NULL,
              name TEXT NOT NULL,
              start_time TIMESTAMPTZ NOT NULL,
              end_time TIMESTAMPTZ,
              span_json JSONB NOT NULL,
              PRIMARY KEY (tenant_id, project_id, trace_id, span_id, seq)
            );

            INSERT INTO spans (
              tenant_id, project_id, environment_id, trace_id, span_id, seq,
              kind, status, name, start_time, end_time, span_json
            ) VALUES (
              'tenant', 'project', 'dev', 'trace', 'span', 1,
              'llm.call', 'ok', 'call', '2026-01-01T00:00:00Z', '2026-01-01T00:00:01Z',
              '{
                "model": {"provider": "openai", "name": "gpt-4o"},
                "cost": {"amount_micros": 1234, "currency": "USD"},
                "attributes": {"agent.release_id": "rel-a"}
              }'::jsonb
            );
            "#,
        )
        .await
        .unwrap_or_else(|err| panic!("seed old spans table: {err}"));

    let _store = PgTraceStore::connect(&connection_string)
        .await
        .unwrap_or_else(|err| panic!("connect/migrate postgres: {err}"));

    let row = client
        .query_one(
            r#"
            SELECT model_provider, model_name, cost_currency, cost_micros, release_id
            FROM spans
            WHERE tenant_id = 'tenant' AND project_id = 'project' AND trace_id = 'trace'
            "#,
            &[],
        )
        .await
        .unwrap_or_else(|err| panic!("query migrated span: {err}"));

    let model_provider: Option<String> = row.get("model_provider");
    let model_name: Option<String> = row.get("model_name");
    let cost_currency: Option<String> = row.get("cost_currency");
    let cost_micros: Option<i64> = row.get("cost_micros");
    let release_id: Option<String> = row.get("release_id");
    assert_eq!(model_provider.as_deref(), Some("openai"));
    assert_eq!(model_name.as_deref(), Some("gpt-4o"));
    assert_eq!(cost_currency.as_deref(), Some("USD"));
    assert_eq!(cost_micros, Some(1234));
    assert_eq!(release_id.as_deref(), Some("rel-a"));
}
