//! Live ClickHouse `TraceStore` conformance.
//!
//! Boots a throwaway ClickHouse container, runs the checked-in migration, and
//! runs the shared store-conformance suite against `ClickHouseTraceStore`. The
//! test is `#[ignore]`d so a Docker-less `cargo test` still passes; run it
//! explicitly on a Docker CI runner with:
//!
//! ```sh
//! cargo test -p beater-store-sql --test clickhouse_trace_store -- --ignored
//! ```

#![cfg(feature = "clickhouse")]

use beater_store_conformance::assert_trace_store_conformance;
use beater_store_sql::ClickHouseTraceStore;
use testcontainers::{
    GenericImage,
    core::{IntoContainerPort, WaitFor, wait::HttpWaitStrategy},
    runners::AsyncRunner,
};

#[tokio::test]
#[ignore = "requires Docker; run with --ignored on a Docker CI runner"]
async fn clickhouse_trace_store_conforms() {
    // ClickHouse writes its startup banner to a log file, not stdout, so wait on
    // the HTTP `/ping` endpoint returning 200 instead of scanning container logs.
    let container = GenericImage::new("clickhouse/clickhouse-server", "24-alpine")
        .with_wait_for(WaitFor::http(
            HttpWaitStrategy::new("/ping")
                .with_port(8123.tcp())
                .with_expected_status_code(200u16),
        ))
        .with_exposed_port(8123.tcp())
        .start()
        .await
        .unwrap_or_else(|err| panic!("start clickhouse container: {err}"));

    let host = container
        .get_host()
        .await
        .unwrap_or_else(|err| panic!("container host: {err}"));
    let port = container
        .get_host_port_ipv4(8123)
        .await
        .unwrap_or_else(|err| panic!("container port: {err}"));
    let endpoint = format!("http://{host}:{port}");

    let store = ClickHouseTraceStore::connect(&endpoint)
        .await
        .unwrap_or_else(|err| panic!("connect/migrate clickhouse: {err}"));

    assert_trace_store_conformance(store).await;
}
