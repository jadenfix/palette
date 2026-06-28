//! WebDriver-backed [`beater_browser::BrowserDriver`] backend.
//!
//! [`WebDriverDriver`] drives any running W3C-compliant WebDriver endpoint over
//! [`fantoccini`]. The same code path covers:
//!
//! - **Native Safari** via `safaridriver --enable` (port `4444`), reported as
//!   [`BrowserEngine::Safari`].
//! - **Chrome** via `chromedriver`, **Edge** via `msedgedriver`, **Firefox** via
//!   `geckodriver` — any W3C driver listening on an HTTP endpoint.
//!
//! The driver connects to an already-running WebDriver process (it does not spawn
//! one); point it at the endpoint with [`WebDriverDriver::connect`] /
//! [`WebDriverDriver::connect_with`].
//!
//! Grounding contract (see [`beater_browser::BrowserDriver`]): an action whose
//! selector does not resolve is **not** a [`BrowserError`]. It returns
//! `Ok(StepOutcome)` with [`StepStatus::Error`] and
//! `grounding.selector_existed == false`. [`BrowserError`] is reserved for
//! transport/driver failures.

use async_trait::async_trait;
use beater_browser::{
    BrowserAction, BrowserDriver, BrowserEngine, BrowserError, Grounding, Observation, StepOutcome,
    StepStatus, UrlPolicy,
};
use fantoccini::error::CmdError;
use fantoccini::{Client, ClientBuilder, Locator};

/// Default WebDriver endpoint (matches `safaridriver` / `chromedriver --port 4444`).
pub const DEFAULT_WEBDRIVER_URL: &str = "http://localhost:4444";

/// Map a [`BrowserAction`] selector to the [`Locator`] this backend uses to
/// resolve it. Selectors are treated as CSS selectors (the conformance fixture
/// and the rest of the stack speak CSS).
pub fn locator_for(selector: &str) -> Locator<'_> {
    Locator::Css(selector)
}

/// A [`BrowserDriver`] backed by a live WebDriver session over [`fantoccini`].
pub struct WebDriverDriver {
    client: Client,
    engine: BrowserEngine,
    /// SSRF guard enforced at every navigation entry point. Defaults to
    /// [`UrlPolicy::block_private`]; override with [`WebDriverDriver::with_policy`].
    policy: UrlPolicy,
}

impl WebDriverDriver {
    /// Connect to the [`DEFAULT_WEBDRIVER_URL`] and report `engine`.
    ///
    /// Use [`BrowserEngine::Safari`] for `safaridriver`, [`BrowserEngine::Chrome`]
    /// for `chromedriver`, etc.
    pub async fn connect(engine: BrowserEngine) -> Result<Self, BrowserError> {
        Self::connect_with(DEFAULT_WEBDRIVER_URL, engine).await
    }

    /// Connect to a WebDriver endpoint at `webdriver_url` and report `engine`.
    ///
    /// The WebDriver process (e.g. `safaridriver --enable`, `chromedriver`) must
    /// already be listening at `webdriver_url`.
    pub async fn connect_with(
        webdriver_url: &str,
        engine: BrowserEngine,
    ) -> Result<Self, BrowserError> {
        let client = ClientBuilder::native()
            .connect(webdriver_url)
            .await
            .map_err(|err| BrowserError::Backend(format!("connect to {webdriver_url}: {err}")))?;
        Ok(Self {
            client,
            engine,
            policy: UrlPolicy::block_private(),
        })
    }

    /// Wrap an already-connected [`fantoccini::Client`] (e.g. one built with
    /// custom capabilities) and report `engine`.
    pub fn from_client(client: Client, engine: BrowserEngine) -> Self {
        Self {
            client,
            engine,
            policy: UrlPolicy::block_private(),
        }
    }

    /// Override the SSRF [`UrlPolicy`] applied to every navigation (builder).
    ///
    /// Use [`UrlPolicy::allow_all`] for trusted callers that must reach
    /// loopback/private fixtures (e.g. the live conformance suite).
    pub fn with_policy(mut self, policy: UrlPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// SSRF gate applied at every navigation entry point (`goto` and
    /// `act(Goto)`). Extracted as an associated fn so the wiring is unit
    /// testable without a live WebDriver session.
    fn enforce_nav_policy(policy: &UrlPolicy, url: &str) -> Result<(), BrowserError> {
        policy.enforce(url)
    }

    /// Build an [`Observation`] of the current page (url, title, full DOM HTML).
    async fn snapshot(&self) -> Result<Observation, BrowserError> {
        let url = self
            .client
            .current_url()
            .await
            .map_err(cmd_backend)?
            .to_string();
        let title = self.client.title().await.map_err(cmd_backend)?;
        let dom_html = self.client.source().await.map_err(cmd_backend)?;
        Ok(Observation {
            url,
            title: if title.is_empty() { None } else { Some(title) },
            dom_html: Some(dom_html),
            accessibility_tree: None,
            console: Vec::new(),
            network: Vec::new(),
        })
    }

    /// Locate `selector`, returning `Ok(None)` when the element is absent (a
    /// grounding miss, not a transport error) and `Err` only on real failures.
    async fn locate(
        &self,
        selector: &str,
    ) -> Result<Option<fantoccini::elements::Element>, BrowserError> {
        match self.client.find(locator_for(selector)).await {
            Ok(element) => Ok(Some(element)),
            Err(err) if err.is_no_such_element() => Ok(None),
            Err(err) => Err(cmd_backend(err)),
        }
    }

    /// Run one selector-bound action, recording grounding. Returns the grounding
    /// plus an optional transport error message produced while operating on a
    /// resolved element.
    async fn run_selector_action(
        &self,
        action: &BrowserAction,
        selector: &str,
    ) -> Result<(Grounding, Option<String>), BrowserError> {
        let Some(element) = self.locate(selector).await? else {
            return Ok((miss_grounding(selector), None));
        };
        let op = match action {
            BrowserAction::Click { .. } => element.click().await,
            BrowserAction::Type { text, .. } => element.send_keys(text).await,
            BrowserAction::Select { value, .. } => element.select_by_value(value).await,
            // `Extract` only needs the element to exist; reading its HTML proves
            // it is reachable and surfaces any transport error.
            BrowserAction::Extract { .. } => element.html(false).await.map(|_| ()),
            // Non-selector actions never reach this method.
            BrowserAction::Goto { .. }
            | BrowserAction::Scroll { .. }
            | BrowserAction::Wait { .. } => Ok(()),
        };
        match op {
            Ok(()) => Ok((hit_grounding(selector), None)),
            Err(err) => Ok((hit_grounding(selector), Some(err.to_string()))),
        }
    }

    /// Window-scroll to `(x, y)` via injected script.
    async fn scroll(&self, x: i64, y: i64) -> Result<(), BrowserError> {
        self.client
            .execute(
                "window.scrollTo(arguments[0], arguments[1]);",
                vec![serde_json::json!(x), serde_json::json!(y)],
            )
            .await
            .map(|_| ())
            .map_err(cmd_backend)
    }
}

#[async_trait]
impl BrowserDriver for WebDriverDriver {
    fn engine(&self) -> BrowserEngine {
        self.engine
    }

    async fn goto(&mut self, url: &str) -> Result<Observation, BrowserError> {
        // SSRF guard: reject blocked targets before navigating.
        Self::enforce_nav_policy(&self.policy, url)?;
        self.client
            .goto(url)
            .await
            .map_err(|err| BrowserError::Navigation(format!("goto {url}: {err}")))?;
        self.snapshot().await
    }

    async fn act(&mut self, action: &BrowserAction) -> Result<StepOutcome, BrowserError> {
        let (grounding, op_error) = match action {
            BrowserAction::Goto { url } => {
                // SSRF guard: enforce on the action surface too — this branch
                // navigates directly rather than delegating to `goto`.
                Self::enforce_nav_policy(&self.policy, url)?;
                self.client
                    .goto(url)
                    .await
                    .map_err(|err| BrowserError::Navigation(format!("goto {url}: {err}")))?;
                (no_selector_grounding(), None)
            }
            BrowserAction::Scroll { x, y } => {
                self.scroll(*x, *y).await?;
                (no_selector_grounding(), None)
            }
            BrowserAction::Wait { millis } => {
                tokio::time::sleep(std::time::Duration::from_millis(*millis)).await;
                (no_selector_grounding(), None)
            }
            BrowserAction::Click { selector }
            | BrowserAction::Type { selector, .. }
            | BrowserAction::Select { selector, .. }
            | BrowserAction::Extract { selector } => {
                self.run_selector_action(action, selector).await?
            }
        };

        // Status is Error when the selector failed to ground OR an operation on a
        // grounded element hit a transport error; both are reported in-band.
        let (status, error) = if !grounding.selector_existed {
            let label = grounding.selector.clone().unwrap_or_default();
            (
                StepStatus::Error,
                Some(format!("selector not found: {label}")),
            )
        } else if let Some(message) = op_error {
            (StepStatus::Error, Some(message))
        } else {
            (StepStatus::Ok, None)
        };

        let observation = self.snapshot().await?;
        Ok(StepOutcome {
            status,
            error,
            grounding,
            observation,
        })
    }

    async fn observe(&mut self) -> Result<Observation, BrowserError> {
        self.snapshot().await
    }

    async fn screenshot(&mut self) -> Result<Vec<u8>, BrowserError> {
        self.client.screenshot().await.map_err(cmd_backend)
    }

    async fn dom(&mut self) -> Result<String, BrowserError> {
        self.client.source().await.map_err(cmd_backend)
    }

    async fn close(&mut self) -> Result<(), BrowserError> {
        // `Client` is cheaply cloneable (shared session handle); clone so we can
        // consume it in `close()` without moving out of `&mut self`.
        self.client.clone().close().await.map_err(cmd_backend)
    }
}

/// Grounding for an action that targets no selector (goto/scroll/wait).
fn no_selector_grounding() -> Grounding {
    Grounding {
        selector: None,
        selector_existed: true,
        matched_element: true,
    }
}

/// Grounding for a selector that resolved to an element.
fn hit_grounding(selector: &str) -> Grounding {
    Grounding {
        selector: Some(selector.to_string()),
        selector_existed: true,
        matched_element: true,
    }
}

/// Grounding for a selector that did not resolve.
fn miss_grounding(selector: &str) -> Grounding {
    Grounding {
        selector: Some(selector.to_string()),
        selector_existed: false,
        matched_element: false,
    }
}

/// Map a fantoccini command error to a backend [`BrowserError`].
fn cmd_backend(err: CmdError) -> BrowserError {
    BrowserError::Backend(err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locator_maps_selector_to_css() {
        // Every selector-bound action resolves via a CSS locator.
        match locator_for("#beater-known") {
            Locator::Css(value) => assert_eq!(value, "#beater-known"),
            other => panic!("expected Locator::Css, got {other:?}"),
        }
    }

    #[test]
    fn grounding_helpers_record_existence() {
        let hit = hit_grounding("#a");
        assert!(hit.selector_existed);
        assert!(hit.matched_element);
        assert_eq!(hit.selector.as_deref(), Some("#a"));

        let miss = miss_grounding("#b");
        assert!(!miss.selector_existed);
        assert!(!miss.matched_element);
        assert_eq!(miss.selector.as_deref(), Some("#b"));

        let none = no_selector_grounding();
        assert!(none.selector_existed);
        assert!(none.selector.is_none());
    }

    #[test]
    fn action_selectors_route_through_grounding() {
        // The actions that must ground a selector all expose one via the contract.
        let click = BrowserAction::Click {
            selector: "#beater-known".to_string(),
        };
        assert_eq!(click.selector(), Some("#beater-known"));
        assert_eq!(click.verb(), "click");

        let typing = BrowserAction::Type {
            selector: "#field".to_string(),
            text: "hi".to_string(),
        };
        assert_eq!(typing.selector(), Some("#field"));

        let select = BrowserAction::Select {
            selector: "#dropdown".to_string(),
            value: "v".to_string(),
        };
        assert_eq!(select.selector(), Some("#dropdown"));

        let extract = BrowserAction::Extract {
            selector: "#out".to_string(),
        };
        assert_eq!(extract.selector(), Some("#out"));

        // Non-selector actions carry no selector and never hit `locate`.
        assert_eq!(
            BrowserAction::Scroll { x: 0, y: 10 }.selector(),
            None,
            "scroll targets no element"
        );
        assert_eq!(BrowserAction::Wait { millis: 5 }.selector(), None);
        assert_eq!(
            BrowserAction::Goto {
                url: "https://example.com".to_string(),
            }
            .selector(),
            None
        );
    }

    /// Live conformance against a real WebDriver endpoint.
    ///
    /// Ignored by default: requires a running WebDriver server (e.g.
    /// `safaridriver --enable` then `safaridriver -p 4444`, or `chromedriver
    /// --port=4444`). It serves [`beater_browser::CONFORMANCE_FIXTURE_HTML`] on a
    /// local TCP port, then runs the shared conformance suite.
    ///
    /// Run with, e.g.:
    /// `WEBDRIVER_URL=http://localhost:4444 cargo test -p beater-browser-webdriver \
    ///   -- --ignored live_conformance`
    #[tokio::test]
    #[ignore = "requires a running WebDriver server (safaridriver/chromedriver/...)"]
    async fn live_conformance() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        let webdriver_url =
            std::env::var("WEBDRIVER_URL").unwrap_or_else(|_| DEFAULT_WEBDRIVER_URL.to_string());
        // Pick the engine from the env so the same test covers Safari/Chrome/etc.
        let engine = match std::env::var("WEBDRIVER_ENGINE").as_deref() {
            Ok("safari") => BrowserEngine::Safari,
            Ok("chrome") => BrowserEngine::Chrome,
            Ok("edge") => BrowserEngine::Edge,
            Ok("firefox") => BrowserEngine::Firefox,
            _ => BrowserEngine::Other,
        };

        // Minimal single-shot static server: bind, then serve the fixture HTML to
        // every connection on a background task for the duration of the test.
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .unwrap_or_else(|err| panic!("bind fixture server: {err}"));
        let addr = listener
            .local_addr()
            .unwrap_or_else(|err| panic!("local_addr: {err}"));
        let base_url = format!("http://{addr}/");

        let server = tokio::spawn(async move {
            let body = beater_browser::CONFORMANCE_FIXTURE_HTML;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\
                 Content-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    break;
                };
                let response = response.clone();
                tokio::spawn(async move {
                    // Drain the request line(s) so the client gets a clean response.
                    let mut scratch = [0u8; 1024];
                    let _ = socket.read(&mut scratch).await;
                    let _ = socket.write_all(response.as_bytes()).await;
                    let _ = socket.flush().await;
                });
            }
        });

        // Fixture is served on 127.0.0.1; opt this trusted run past the default
        // `block_private` SSRF policy.
        let mut driver = WebDriverDriver::connect_with(&webdriver_url, engine)
            .await
            .unwrap_or_else(|err| panic!("connect to {webdriver_url}: {err}"))
            .with_policy(UrlPolicy::allow_all());

        beater_browser::assert_browser_driver_conformance(&mut driver, &base_url).await;

        server.abort();
    }

    #[test]
    fn goto_navigation_guard_blocks_ssrf_targets() {
        // Proves the exact guard `goto`/`act(Goto)` invoke rejects blocked
        // targets — including alternate IP encodings — before any WebDriver I/O.
        let policy = UrlPolicy::block_private();
        for url in [
            "http://169.254.169.254/latest/meta-data/",
            "http://127.0.0.1",
            "http://localhost",
            "file:///etc/passwd",
            "http://2130706433", // decimal loopback
            "http://0177.0.0.1", // octal loopback
            "http://0x7f.0.0.1", // hex loopback
            "http://127.1",      // short-form loopback
            "http://0xA9FEA9FE", // hex IMDS
        ] {
            let Err(err) = WebDriverDriver::enforce_nav_policy(&policy, url) else {
                panic!("expected SsrfBlocked for {url:?}");
            };
            assert!(
                matches!(err, BrowserError::SsrfBlocked(_)),
                "expected SsrfBlocked for {url:?}, got {err:?}"
            );
        }
        assert!(WebDriverDriver::enforce_nav_policy(&policy, "https://example.com").is_ok());
    }
}
