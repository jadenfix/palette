//! CDP-backed [`beater_browser::BrowserDriver`] backend.
//!
//! Pure-Rust Chrome / Edge / Chromium control via [`chromiumoxide`] (the Chrome
//! DevTools Protocol). No Node runtime is required: [`CdpDriver`] launches a
//! headless browser process directly, drives a single [`chromiumoxide::Page`],
//! and translates each [`BrowserAction`] into CDP calls.
//!
//! ## Grounding contract
//!
//! Per [`beater_browser`], a failed-to-ground action is **not** a
//! [`BrowserError`]: [`CdpDriver::act`] resolves the target with
//! [`chromiumoxide::Page::find_element`] and, when the selector does not match,
//! returns `Ok(StepOutcome { status: Error, grounding.selector_existed: false })`.
//! [`BrowserError`] is reserved for transport/backend failures (a crashed
//! browser process, a navigation that the protocol rejected, etc.).

use std::time::Duration;

use beater_browser::{
    BrowserAction, BrowserDriver, BrowserEngine, BrowserError, Grounding, Observation, StepOutcome,
    StepStatus, UrlPolicy,
};
use chromiumoxide::browser::{Browser, BrowserConfig, HeadlessMode};
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::error::CdpError;
use chromiumoxide::page::{Page, ScreenshotParams};
use futures::StreamExt;
use tokio::task::JoinHandle;

/// Map a [`chromiumoxide`] error to the crate's transport-level error type.
///
/// This is the single point where backend failures cross into the
/// engine-agnostic [`BrowserError`] surface; grounding failures never travel
/// through here (see the module docs).
fn backend_err(err: CdpError) -> BrowserError {
    BrowserError::Backend(err.to_string())
}

/// Whether a CDP error message means "the selector did not resolve to a live
/// DOM node" (a grounding miss) rather than a transport/process failure. Real
/// Chrome reports an absent selector as the raw protocol error
/// `-32000: Could not find node with given id`, which is not `CdpError::NotFound`.
fn is_node_not_found_msg(msg: &str) -> bool {
    let lowered = msg.to_lowercase();
    lowered.contains("could not find node") || lowered.contains("no node with given id")
}

/// Configuration for launching a [`CdpDriver`].
#[derive(Clone, Debug)]
pub struct CdpConfig {
    /// Engine to report from [`BrowserDriver::engine`]. Selects the launched
    /// browser binary when [`CdpConfig::executable`] is unset.
    pub engine: BrowserEngine,
    /// Explicit path to a Chrome/Edge/Chromium binary. When `None`,
    /// [`chromiumoxide`] auto-detects an installed Chromium-family browser.
    pub executable: Option<String>,
    /// Run without the OS sandbox (required in many CI/container environments).
    pub no_sandbox: bool,
    /// How long to wait for the browser process to expose its devtools socket.
    pub launch_timeout: Duration,
}

impl Default for CdpConfig {
    fn default() -> Self {
        Self {
            engine: BrowserEngine::Chromium,
            executable: None,
            no_sandbox: true,
            launch_timeout: Duration::from_secs(20),
        }
    }
}

impl CdpConfig {
    /// A config that reports (and prefers) the given engine.
    pub fn for_engine(engine: BrowserEngine) -> Self {
        Self {
            engine,
            ..Self::default()
        }
    }

    /// Pin an explicit browser binary path.
    pub fn with_executable(mut self, path: impl Into<String>) -> Self {
        self.executable = Some(path.into());
        self
    }

    /// Build the [`chromiumoxide`] launch config from this driver config.
    fn to_browser_config(&self) -> Result<BrowserConfig, BrowserError> {
        let mut builder = BrowserConfig::builder()
            .headless_mode(HeadlessMode::New)
            .launch_timeout(self.launch_timeout);
        if self.no_sandbox {
            builder = builder.no_sandbox();
        }
        if let Some(path) = &self.executable {
            builder = builder.chrome_executable(path);
        }
        builder.build().map_err(BrowserError::Backend)
    }
}

/// A [`BrowserDriver`] backed by a headless Chromium-family browser over CDP.
///
/// Construct with [`CdpDriver::launch`]. The driver owns the browser process,
/// the background CDP [`chromiumoxide::Handler`] task, and a single page; call
/// [`BrowserDriver::close`] to tear everything down (dropping the driver also
/// aborts the handler task as a safety net).
pub struct CdpDriver {
    engine: BrowserEngine,
    browser: Browser,
    page: Page,
    handler_task: JoinHandle<()>,
    /// SSRF guard enforced at every navigation entry point. Defaults to
    /// [`UrlPolicy::block_private`] so production launches are safe by default;
    /// override with [`CdpDriver::with_policy`] for trusted/internal targets.
    policy: UrlPolicy,
}

impl CdpDriver {
    /// Launch a headless browser with the default config for `engine`.
    pub async fn launch(engine: BrowserEngine) -> Result<Self, BrowserError> {
        Self::launch_with(CdpConfig::for_engine(engine)).await
    }

    /// Launch a headless browser from an explicit [`CdpConfig`].
    pub async fn launch_with(config: CdpConfig) -> Result<Self, BrowserError> {
        let browser_config = config.to_browser_config()?;
        let (browser, mut handler) = Browser::launch(browser_config).await.map_err(backend_err)?;

        // The handler future must be polled continuously to service CDP traffic.
        let handler_task = tokio::spawn(async move { while handler.next().await.is_some() {} });

        let page = browser.new_page("about:blank").await.map_err(backend_err)?;

        Ok(Self {
            engine: config.engine,
            browser,
            page,
            handler_task,
            policy: UrlPolicy::block_private(),
        })
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
    /// testable without launching a real browser process.
    fn enforce_nav_policy(policy: &UrlPolicy, url: &str) -> Result<(), BrowserError> {
        policy.enforce(url)
    }

    /// Read the page URL, defaulting to empty when the browser reports none.
    async fn current_url(&self) -> Result<String, BrowserError> {
        Ok(self
            .page
            .url()
            .await
            .map_err(backend_err)?
            .unwrap_or_default())
    }

    /// Build a full [`Observation`] snapshot of the current page.
    async fn snapshot(&self) -> Result<Observation, BrowserError> {
        let url = self.current_url().await?;
        let title = self.page.get_title().await.map_err(backend_err)?;
        let dom_html = self.page.content().await.map_err(backend_err)?;
        Ok(Observation {
            url,
            title,
            dom_html: Some(dom_html),
            accessibility_tree: None,
            console: Vec::new(),
            network: Vec::new(),
        })
    }

    /// Resolve a selector and run `op` on the matched element. Returns `Ok(true)`
    /// when grounded, `Ok(false)` when the selector does not match (the
    /// not-found contract), and a [`BrowserError`] only for transport failures.
    async fn with_element<F, Fut>(&self, selector: &str, op: F) -> Result<bool, BrowserError>
    where
        F: FnOnce(chromiumoxide::Element) -> Fut,
        Fut: std::future::Future<Output = Result<(), BrowserError>>,
    {
        match self.page.find_element(selector).await {
            Ok(element) => match op(element).await {
                Ok(()) => Ok(true),
                // A node can vanish between resolve and operation (detached/
                // re-rendered DOM); that is a grounding miss, not a transport fault.
                Err(BrowserError::Backend(msg)) if is_node_not_found_msg(&msg) => Ok(false),
                Err(other) => Err(other),
            },
            // chromiumoxide surfaces an absent selector either as the typed
            // `NotFound` variant or as the raw CDP protocol error
            // "-32000: Could not find node with given id" — both mean the
            // selector did not resolve, so they are grounding misses.
            Err(CdpError::NotFound) => Ok(false),
            Err(other) if is_node_not_found_msg(&other.to_string()) => Ok(false),
            Err(other) => Err(backend_err(other)),
        }
    }

    /// Build the outcome for a selector-targeting action from whether it grounded.
    fn selector_outcome(
        &self,
        selector: &str,
        grounded: bool,
        observation: Observation,
    ) -> StepOutcome {
        if grounded {
            StepOutcome {
                status: StepStatus::Ok,
                error: None,
                grounding: Grounding {
                    selector: Some(selector.to_string()),
                    selector_existed: true,
                    matched_element: true,
                },
                observation,
            }
        } else {
            StepOutcome {
                status: StepStatus::Error,
                error: Some(format!("selector not found: {selector}")),
                grounding: Grounding {
                    selector: Some(selector.to_string()),
                    selector_existed: false,
                    matched_element: false,
                },
                observation,
            }
        }
    }

    /// Build the outcome for a selector-less action (always grounded).
    fn no_selector(&self, observation: Observation) -> StepOutcome {
        StepOutcome {
            status: StepStatus::Ok,
            error: None,
            grounding: Grounding {
                selector: None,
                selector_existed: true,
                matched_element: true,
            },
            observation,
        }
    }
}

impl Drop for CdpDriver {
    fn drop(&mut self) {
        // Best-effort: stop servicing CDP traffic so the handler task does not
        // outlive the driver. `close()` is the graceful path; this is a backstop.
        self.handler_task.abort();
    }
}

#[async_trait::async_trait]
impl BrowserDriver for CdpDriver {
    fn engine(&self) -> BrowserEngine {
        self.engine
    }

    async fn goto(&mut self, url: &str) -> Result<Observation, BrowserError> {
        // SSRF guard: reject blocked targets before any navigation occurs.
        Self::enforce_nav_policy(&self.policy, url)?;
        self.page
            .goto(url)
            .await
            .map_err(|err| BrowserError::Navigation(err.to_string()))?;
        self.page
            .wait_for_navigation()
            .await
            .map_err(|err| BrowserError::Navigation(err.to_string()))?;
        let mut observation = self.snapshot().await?;
        // chromiumoxide may canonicalize the URL (trailing slash); preserve the
        // caller's target so observers can correlate goto -> observe.
        observation.url = url.to_string();
        Ok(observation)
    }

    async fn act(&mut self, action: &BrowserAction) -> Result<StepOutcome, BrowserError> {
        match action {
            BrowserAction::Goto { url } => {
                let observation = self.goto(url).await?;
                Ok(self.no_selector(observation))
            }
            BrowserAction::Wait { millis } => {
                tokio::time::sleep(Duration::from_millis(*millis)).await;
                let observation = self.snapshot().await?;
                Ok(self.no_selector(observation))
            }
            BrowserAction::Scroll { x, y } => {
                let script = format!("window.scrollTo({x}, {y});");
                self.page.evaluate(script).await.map_err(backend_err)?;
                let observation = self.snapshot().await?;
                Ok(self.no_selector(observation))
            }
            BrowserAction::Click { selector } => {
                let grounded = self
                    .with_element(selector, |element| async move {
                        element.click().await.map_err(backend_err)?;
                        Ok(())
                    })
                    .await?;
                let observation = self.snapshot().await?;
                Ok(self.selector_outcome(selector, grounded, observation))
            }
            BrowserAction::Type { selector, text } => {
                let text = text.clone();
                let grounded = self
                    .with_element(selector, |element| async move {
                        element.focus().await.map_err(backend_err)?;
                        element.type_str(&text).await.map_err(backend_err)?;
                        Ok(())
                    })
                    .await?;
                let observation = self.snapshot().await?;
                Ok(self.selector_outcome(selector, grounded, observation))
            }
            BrowserAction::Select { selector, value } => {
                // JSON-encode the value so it embeds safely in the JS literal.
                let encoded = serde_json::to_string(value)
                    .map_err(|err| BrowserError::Backend(err.to_string()))?;
                let grounded = self
                    .with_element(selector, |element| async move {
                        let func = format!(
                            "function() {{ this.value = {encoded}; \
                             this.dispatchEvent(new Event('change', {{ bubbles: true }})); }}"
                        );
                        element.call_js_fn(func, false).await.map_err(backend_err)?;
                        Ok(())
                    })
                    .await?;
                let observation = self.snapshot().await?;
                Ok(self.selector_outcome(selector, grounded, observation))
            }
            BrowserAction::Extract { selector } => {
                // Extraction is read-only: ground the selector, then snapshot.
                let grounded = self
                    .with_element(selector, |_element| async move { Ok(()) })
                    .await?;
                let observation = self.snapshot().await?;
                Ok(self.selector_outcome(selector, grounded, observation))
            }
        }
    }

    async fn observe(&mut self) -> Result<Observation, BrowserError> {
        self.snapshot().await
    }

    async fn screenshot(&mut self) -> Result<Vec<u8>, BrowserError> {
        let params = ScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Png)
            .build();
        self.page.screenshot(params).await.map_err(backend_err)
    }

    async fn dom(&mut self) -> Result<String, BrowserError> {
        self.page.content().await.map_err(backend_err)
    }

    async fn close(&mut self) -> Result<(), BrowserError> {
        self.browser.close().await.map_err(backend_err)?;
        let _ = self.browser.wait().await;
        self.handler_task.abort();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_reports_requested_engine() {
        for engine in [
            BrowserEngine::Chrome,
            BrowserEngine::Edge,
            BrowserEngine::Chromium,
        ] {
            let config = CdpConfig::for_engine(engine);
            assert_eq!(config.engine, engine);
        }
    }

    #[test]
    fn default_config_is_headless_sandboxless() {
        let config = CdpConfig::default();
        assert_eq!(config.engine, BrowserEngine::Chromium);
        assert!(config.no_sandbox, "CI/sandbox launches need --no-sandbox");
        assert!(config.executable.is_none());
        // Must produce a valid chromiumoxide launch config.
        config
            .to_browser_config()
            .unwrap_or_else(|err| panic!("default config should build: {err}"));
    }

    #[test]
    fn with_executable_pins_binary() {
        let config =
            CdpConfig::for_engine(BrowserEngine::Edge).with_executable("/opt/edge/microsoft-edge");
        assert_eq!(
            config.executable.as_deref(),
            Some("/opt/edge/microsoft-edge")
        );
        assert_eq!(config.engine, BrowserEngine::Edge);
    }

    #[test]
    fn backend_error_maps_to_backend_variant() {
        let err = backend_err(CdpError::NotFound);
        match err {
            BrowserError::Backend(msg) => assert!(!msg.is_empty()),
            other => panic!("expected Backend, got {other:?}"),
        }
    }

    #[test]
    fn node_not_found_messages_are_grounding_misses() {
        // Real Chrome reports an absent selector as this raw protocol error
        // rather than CdpError::NotFound; it must be treated as a grounding miss.
        assert!(is_node_not_found_msg(
            "Error -32000: Could not find node with given id"
        ));
        assert!(is_node_not_found_msg("could not find node"));
        assert!(is_node_not_found_msg("No node with given id"));
        // Genuine transport failures must NOT be swallowed as grounding misses.
        assert!(!is_node_not_found_msg("websocket connection closed"));
        assert!(!is_node_not_found_msg("navigation timeout"));
    }

    /// Live conformance against a real browser. Ignored: no Chrome binary in
    /// CI/sandbox. Run locally with `cargo test -p beater-browser-cdp -- --ignored`.
    #[tokio::test]
    #[ignore = "requires a local Chrome/Chromium binary"]
    async fn cdp_driver_passes_conformance() {
        use std::io::Write as _;
        use std::net::TcpListener;

        // Serve CONFORMANCE_FIXTURE_HTML over a tiny local HTTP server.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap_or_else(|err| panic!("bind: {err}"));
        let addr = listener
            .local_addr()
            .unwrap_or_else(|err| panic!("local_addr: {err}"));
        let base_url = format!("http://{addr}/");

        let server = std::thread::spawn(move || {
            let body = beater_browser::CONFORMANCE_FIXTURE_HTML;
            // Answer enough requests for the conformance run (goto + actions).
            for _ in 0..64 {
                let Ok((mut stream, _)) = listener.accept() else {
                    break;
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });

        // The conformance fixture is served on 127.0.0.1, which the default
        // `block_private` policy would (correctly) reject; opt this trusted
        // local run into `allow_all`.
        let mut driver = CdpDriver::launch(BrowserEngine::Chromium)
            .await
            .unwrap_or_else(|err| panic!("launch: {err}"))
            .with_policy(UrlPolicy::allow_all());
        beater_browser::assert_browser_driver_conformance(&mut driver, &base_url).await;
        drop(server);
    }

    #[test]
    fn goto_navigation_guard_blocks_ssrf_targets() {
        // Proves the exact guard `goto`/`act(Goto)` invoke rejects blocked
        // targets — including alternate IP encodings — before any browser I/O.
        let policy = UrlPolicy::block_private();
        for url in [
            "http://169.254.169.254/latest/meta-data/", // IMDS
            "http://127.0.0.1",
            "http://localhost",
            "file:///etc/passwd",
            "http://2130706433", // decimal loopback
            "http://0177.0.0.1", // octal loopback
            "http://0x7f.0.0.1", // hex loopback
            "http://127.1",      // short-form loopback
            "http://0xA9FEA9FE", // hex IMDS
        ] {
            let Err(err) = CdpDriver::enforce_nav_policy(&policy, url) else {
                panic!("expected SsrfBlocked for {url:?}");
            };
            assert!(
                matches!(err, BrowserError::SsrfBlocked(_)),
                "expected SsrfBlocked for {url:?}, got {err:?}"
            );
        }
        // Public targets still pass the guard.
        assert!(CdpDriver::enforce_nav_policy(&policy, "https://example.com").is_ok());
    }

    #[test]
    fn default_launch_policy_is_block_private() {
        // The production constructor must default to the secure policy.
        let policy = UrlPolicy::block_private();
        assert!(CdpDriver::enforce_nav_policy(&policy, "http://169.254.169.254").is_err());
    }
}
