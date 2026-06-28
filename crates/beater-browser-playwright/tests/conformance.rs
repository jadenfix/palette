//! Live conformance test for [`PlaywrightDriver`].
//!
//! `#[ignore]`d because it launches a real Node + Playwright + browser, which is
//! not available in CI/sandbox. Run locally with:
//!
//! ```text
//! cd crates/beater-browser-playwright/runner && npm install
//! cargo test -p beater-browser-playwright -- --ignored
//! ```
//!
//! It serves [`beater_browser::CONFORMANCE_FIXTURE_HTML`] from a throwaway
//! single-purpose HTTP server on `127.0.0.1` and runs the cross-backend
//! [`beater_browser::assert_browser_driver_conformance`] suite against the
//! Playwright-backed driver.

use beater_browser::{
    assert_browser_driver_conformance, BrowserDriver, BrowserEngine, UrlPolicy,
    CONFORMANCE_FIXTURE_HTML, FIXTURE_CONSOLE_MESSAGE,
};
use beater_browser_playwright::{PlaywrightConfig, PlaywrightDriver};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Bind an ephemeral local port and serve the fixture HTML to every connection.
/// Returns the base URL the driver should navigate to.
async fn spawn_fixture_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap_or_else(|err| panic!("bind fixture server: {err}"));
    let addr: SocketAddr = listener
        .local_addr()
        .unwrap_or_else(|err| panic!("local_addr: {err}"));
    let base_url = format!("http://{addr}/");

    tokio::spawn(async move {
        while let Ok((stream, _peer)) = listener.accept().await {
            tokio::spawn(serve_one(stream));
        }
    });

    base_url
}

/// Read (and discard) one HTTP request, then write the fixture page back.
async fn serve_one(mut stream: TcpStream) {
    let mut scratch = [0u8; 1024];
    // Best-effort drain of the request line/headers; we serve the same page for
    // any path.
    let _ = stream.read(&mut scratch).await;

    let body = CONFORMANCE_FIXTURE_HTML;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.flush().await;
}

#[tokio::test]
#[ignore = "launches real Node + Playwright + browser; run locally after `npm install` in runner/"]
async fn playwright_driver_passes_conformance() {
    let base_url = spawn_fixture_server().await;

    let config = PlaywrightConfig::new(BrowserEngine::Chromium).with_headless(true);
    let mut driver = PlaywrightDriver::launch(config)
        .await
        .unwrap_or_else(|err| panic!("launch playwright driver: {err}"))
        // Fixture is served on 127.0.0.1; opt this trusted run past the
        // default `block_private` SSRF policy.
        .with_policy(UrlPolicy::allow_all());

    assert_browser_driver_conformance(&mut driver, &base_url).await;
}

#[tokio::test]
#[ignore = "launches real Node + Playwright + browser; run locally after `npm install` in runner/"]
async fn playwright_driver_captures_console_and_network() {
    let base_url = spawn_fixture_server().await;

    let config = PlaywrightConfig::new(BrowserEngine::Chromium).with_headless(true);
    let mut driver = PlaywrightDriver::launch(config)
        .await
        .unwrap_or_else(|err| panic!("launch playwright driver: {err}"))
        // Fixture is served on 127.0.0.1; opt this trusted run past the
        // default `block_private` SSRF policy.
        .with_policy(UrlPolicy::allow_all());

    let observation = driver
        .goto(&base_url)
        .await
        .unwrap_or_else(|err| panic!("goto: {err}"));

    // The fixture logs FIXTURE_CONSOLE_MESSAGE on load — full observability means
    // we surface it, not an empty console.
    assert!(
        observation
            .console
            .iter()
            .any(|msg| msg.text.contains(FIXTURE_CONSOLE_MESSAGE)),
        "expected the fixture console log to be captured, got {:?}",
        observation.console
    );
    // The page load is at least one network request (the document itself).
    assert!(
        !observation.network.is_empty(),
        "expected the document request to be captured"
    );
    assert!(
        observation
            .network
            .iter()
            .any(|req| req.status == Some(200)),
        "expected a 200 document response, got {:?}",
        observation.network
    );

    driver
        .close()
        .await
        .unwrap_or_else(|err| panic!("close: {err}"));
}
