//! `beater-browser` — the contract for browser-driving agents in Beater.
//!
//! This crate is the shared foundation for the browser-agent observability
//! feature. It defines:
//!
//! - [`BrowserDriver`], the engine-agnostic trait every native backend
//!   (`beater-browser-{playwright,webdriver,cdp}`) implements.
//! - The structured step types ([`Observation`], [`BrowserAction`],
//!   [`StepOutcome`], [`LlmDecision`], [`StepTriple`]) that flow through the
//!   capture layer and become canonical spans + cassettes.
//! - [`semconv`], the `browser.*` attribute keys shared with the external
//!   instrumentation SDKs.
//! - [`assert_browser_driver_conformance`], the cross-backend conformance suite
//!   (mirrors `beater-store-conformance`), plus [`MockDriver`] so downstream
//!   crates can develop without a live browser.
//!
//! - [`url_policy::UrlPolicy`] / [`UrlPolicy`], an opt-in SSRF guard that
//!   blocks private/loopback/link-local/metadata navigation targets before they
//!   reach a real browser. Default-safe: use `UrlPolicy::block_private()` for
//!   production and `UrlPolicy::allow_all()` for backwards-compatible tests.
//!
//! The crate is deliberately dependency-light (no store/replay/schema deps) so
//! every backend can build against it in isolation.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

pub mod semconv;
pub mod url_policy;

pub use url_policy::UrlPolicy;

/// Selector present in the bundled conformance fixture page.
pub const FIXTURE_KNOWN_SELECTOR: &str = "#beater-known";
/// Selector deliberately absent from the conformance fixture page.
pub const FIXTURE_MISSING_SELECTOR: &str = "#beater-missing";

/// Minimal static HTML a backend can serve to satisfy
/// [`assert_browser_driver_conformance`]. Contains [`FIXTURE_KNOWN_SELECTOR`]
/// and omits [`FIXTURE_MISSING_SELECTOR`].
pub const CONFORMANCE_FIXTURE_HTML: &str = concat!(
    "<!doctype html><html><head><title>Beater Conformance Fixture</title></head>",
    "<body><button id=\"beater-known\">ok</button>",
    "<script>console.log(\"beater-fixture-ready\");</script>",
    "</body></html>",
);

/// Console text the bundled fixture logs on load — backends that capture console
/// output should surface this in the observation after navigating to the fixture.
pub const FIXTURE_CONSOLE_MESSAGE: &str = "beater-fixture-ready";

/// Errors a [`BrowserDriver`] can raise. Note: a failed-to-ground action is NOT
/// an error — it is a successful call returning a [`StepOutcome`] with
/// [`StepStatus::Error`] and `selector_existed == false`. `BrowserError` is for
/// transport/backend failures (crashed process, navigation timeout, etc.).
#[derive(Debug, thiserror::Error)]
pub enum BrowserError {
    /// Navigation to a URL failed at the transport level.
    #[error("navigation failed: {0}")]
    Navigation(String),
    /// The backend (driver process / protocol) failed.
    #[error("browser backend error: {0}")]
    Backend(String),
    /// The navigation target was blocked by a [`UrlPolicy`] SSRF guard.
    ///
    /// The inner string contains the human-readable reason. Callers should
    /// surface this to the agent / user so they understand why the navigation
    /// was rejected rather than seeing a silent timeout.
    #[error("SSRF guard blocked navigation: {0}")]
    SsrfBlocked(String),
}

/// Browser engine that executed a step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrowserEngine {
    Chromium,
    Chrome,
    Edge,
    Firefox,
    Webkit,
    Safari,
    Other,
}

impl BrowserEngine {
    /// Stable lowercase string used in the `browser.engine` attribute.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Chromium => "chromium",
            Self::Chrome => "chrome",
            Self::Edge => "edge",
            Self::Firefox => "firefox",
            Self::Webkit => "webkit",
            Self::Safari => "safari",
            Self::Other => "other",
        }
    }
}

/// An action an agent takes against the page.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action", content = "args")]
pub enum BrowserAction {
    Goto { url: String },
    Click { selector: String },
    Type { selector: String, text: String },
    Scroll { x: i64, y: i64 },
    Select { selector: String, value: String },
    Wait { millis: u64 },
    Extract { selector: String },
}

impl BrowserAction {
    /// The action verb (matches the `browser.action` attribute value).
    pub fn verb(&self) -> &'static str {
        match self {
            Self::Goto { .. } => "goto",
            Self::Click { .. } => "click",
            Self::Type { .. } => "type",
            Self::Scroll { .. } => "scroll",
            Self::Select { .. } => "select",
            Self::Wait { .. } => "wait",
            Self::Extract { .. } => "extract",
        }
    }

    /// The element selector this action targets, if any.
    pub fn selector(&self) -> Option<&str> {
        match self {
            Self::Click { selector }
            | Self::Type { selector, .. }
            | Self::Select { selector, .. }
            | Self::Extract { selector } => Some(selector),
            Self::Goto { .. } | Self::Scroll { .. } | Self::Wait { .. } => None,
        }
    }
}

/// A console message observed on the page during a step (level + text), so
/// silent JS errors the agent triggered are visible to evals/review.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsoleMessage {
    /// `log` | `info` | `warn` | `error` | `debug` (browser-reported).
    pub level: String,
    pub text: String,
}

/// A network request observed during a step — agents often fail on a silent
/// non-2xx or a failed request the DOM doesn't reflect.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkRequest {
    pub method: String,
    pub url: String,
    /// HTTP status, when a response was received.
    #[serde(default)]
    pub status: Option<u16>,
    /// Resource type (`document`, `xhr`, `fetch`, `script`, …) when known.
    #[serde(default)]
    pub resource_type: Option<String>,
    /// Whether the request failed at the transport level (no response).
    #[serde(default)]
    pub failed: bool,
}

/// A snapshot of page state. Bytes (DOM/screenshot) are intentionally kept out
/// of line: backends expose raw DOM via [`BrowserDriver::dom`] and screenshots
/// via [`BrowserDriver::screenshot`]; the capture layer stores them as
/// artifacts. `dom_html` here is a convenience copy for grounding/evaluation.
/// `console`/`network` are events observed since the previous observation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    pub url: String,
    pub title: Option<String>,
    pub dom_html: Option<String>,
    pub accessibility_tree: Option<Value>,
    #[serde(default)]
    pub console: Vec<ConsoleMessage>,
    #[serde(default)]
    pub network: Vec<NetworkRequest>,
}

/// Whether an action resolved to its intended element — the core "grounding"
/// signal for browser agents.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grounding {
    pub selector: Option<String>,
    pub selector_existed: bool,
    pub matched_element: bool,
}

/// Outcome status for a single step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Ok,
    Error,
}

/// The result of executing one [`BrowserAction`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StepOutcome {
    pub status: StepStatus,
    pub error: Option<String>,
    pub grounding: Grounding,
    /// Page state after the action executed (`observation_after`).
    pub observation: Observation,
}

/// The LLM decision that produced an action — captured so prompts/code can be
/// iterated and replayed. `prompt` is the raw model input (the perception the
/// model reasoned over); `output` is the raw model output (browser-use
/// `model_outputs`); `reasoning` is `model_thoughts`. The economics fields make
/// the decision's cost/latency observable so cost/latency evals work on browser
/// runs — all optional, defaulting to absent.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LlmDecision {
    pub model: Option<String>,
    pub prompt: Value,
    pub output: Value,
    pub reasoning: Option<String>,
    #[serde(default)]
    pub input_tokens: Option<u64>,
    #[serde(default)]
    pub output_tokens: Option<u64>,
    #[serde(default)]
    pub cost_micros: Option<i64>,
    #[serde(default)]
    pub latency_ms: Option<u64>,
}

/// The atomic unit of browser-agent observability: observe → decide → act →
/// outcome. This is what evaluators score and what the capture layer turns into
/// canonical spans + replay cassettes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StepTriple {
    pub seq: u64,
    pub observation_before: Observation,
    pub decision: Option<LlmDecision>,
    pub action: BrowserAction,
    pub outcome: StepOutcome,
}

/// Engine-agnostic control surface for a browser session. Every native backend
/// implements this and must pass [`assert_browser_driver_conformance`].
///
/// Contract notes:
/// - `act` returns `Ok(StepOutcome)` even when the selector does not resolve;
///   grounding failure is recorded as `StepStatus::Error` +
///   `grounding.selector_existed == false`, not as a `BrowserError`.
/// - `BrowserError` is reserved for transport/backend failures.
#[async_trait::async_trait]
pub trait BrowserDriver: Send + Sync {
    /// The engine backing this driver.
    fn engine(&self) -> BrowserEngine;
    /// Navigate to `url` and return the resulting observation.
    async fn goto(&mut self, url: &str) -> Result<Observation, BrowserError>;
    /// Execute an action and return its outcome (incl. grounding + after-state).
    async fn act(&mut self, action: &BrowserAction) -> Result<StepOutcome, BrowserError>;
    /// Observe current page state without taking an action.
    async fn observe(&mut self) -> Result<Observation, BrowserError>;
    /// Capture a screenshot of the current page (PNG bytes).
    async fn screenshot(&mut self) -> Result<Vec<u8>, BrowserError>;
    /// Capture the current DOM as HTML.
    async fn dom(&mut self) -> Result<String, BrowserError>;
    /// Tear down the session.
    async fn close(&mut self) -> Result<(), BrowserError>;
}

/// An in-memory [`BrowserDriver`] for tests and downstream development. It does
/// not load real pages; selectors it should treat as present are seeded.
///
/// ## SSRF policy
///
/// By default `MockDriver` uses [`UrlPolicy::allow_all`] so existing test
/// suites are unaffected. Pass [`UrlPolicy::block_private()`] to
/// [`MockDriver::with_policy`] to exercise the guard in unit tests without a
/// real browser:
///
/// ```rust,no_run
/// use beater_browser::{BrowserDriver, BrowserEngine, MockDriver, UrlPolicy};
/// # async fn example() {
/// let mut driver = MockDriver::new(BrowserEngine::Chromium)
///     .with_policy(UrlPolicy::block_private());
/// let err = driver.goto("http://169.254.169.254").await.unwrap_err();
/// assert!(err.to_string().contains("SSRF guard"));
/// # }
/// ```
pub struct MockDriver {
    engine: BrowserEngine,
    url: String,
    title: Option<String>,
    selectors: BTreeSet<String>,
    policy: UrlPolicy,
}

impl MockDriver {
    /// An empty mock with no known selectors and `UrlPolicy::allow_all()`
    /// (backwards-compatible default).
    pub fn new(engine: BrowserEngine) -> Self {
        Self {
            engine,
            url: String::new(),
            title: None,
            selectors: BTreeSet::new(),
            policy: UrlPolicy::allow_all(),
        }
    }

    /// A mock seeded to satisfy [`assert_browser_driver_conformance`].
    pub fn with_conformance_fixture() -> Self {
        let mut selectors = BTreeSet::new();
        selectors.insert(FIXTURE_KNOWN_SELECTOR.to_string());
        Self {
            engine: BrowserEngine::Chromium,
            url: String::new(),
            title: Some("Beater Conformance Fixture".to_string()),
            selectors,
            policy: UrlPolicy::allow_all(),
        }
    }

    /// Replace the URL policy on this mock (builder method).
    ///
    /// Use `UrlPolicy::block_private()` to write tests that exercise SSRF
    /// policy enforcement without a real browser.
    pub fn with_policy(mut self, policy: UrlPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Mark `selector` as present in the mocked page.
    pub fn seed_selector(mut self, selector: &str) -> Self {
        self.selectors.insert(selector.to_string());
        self
    }

    fn observation(&self) -> Observation {
        Observation {
            url: self.url.clone(),
            title: self.title.clone(),
            dom_html: Some(CONFORMANCE_FIXTURE_HTML.to_string()),
            accessibility_tree: None,
            console: Vec::new(),
            network: Vec::new(),
        }
    }
}

#[async_trait::async_trait]
impl BrowserDriver for MockDriver {
    fn engine(&self) -> BrowserEngine {
        self.engine
    }

    async fn goto(&mut self, url: &str) -> Result<Observation, BrowserError> {
        self.policy.enforce(url)?;
        self.url = url.to_string();
        Ok(self.observation())
    }

    async fn act(&mut self, action: &BrowserAction) -> Result<StepOutcome, BrowserError> {
        if let BrowserAction::Goto { url } = action {
            self.policy.enforce(url)?;
            self.url = url.clone();
        }
        let grounding = match action.selector() {
            Some(selector) => {
                let exists = self.selectors.contains(selector);
                Grounding {
                    selector: Some(selector.to_string()),
                    selector_existed: exists,
                    matched_element: exists,
                }
            }
            None => Grounding {
                selector: None,
                selector_existed: true,
                matched_element: true,
            },
        };
        let (status, error) = if grounding.selector_existed {
            (StepStatus::Ok, None)
        } else {
            let selector = grounding.selector.clone().unwrap_or_default();
            (
                StepStatus::Error,
                Some(format!("selector not found: {selector}")),
            )
        };
        Ok(StepOutcome {
            status,
            error,
            grounding,
            observation: self.observation(),
        })
    }

    async fn observe(&mut self) -> Result<Observation, BrowserError> {
        Ok(self.observation())
    }

    async fn screenshot(&mut self) -> Result<Vec<u8>, BrowserError> {
        // PNG file signature — non-empty deterministic bytes for tests.
        Ok(vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
    }

    async fn dom(&mut self) -> Result<String, BrowserError> {
        Ok(CONFORMANCE_FIXTURE_HTML.to_string())
    }

    async fn close(&mut self) -> Result<(), BrowserError> {
        Ok(())
    }
}

/// Drive a fixed scenario against any [`BrowserDriver`] and assert the invariants
/// every backend must uphold. `base_url` is where [`CONFORMANCE_FIXTURE_HTML`] is
/// served (real backends spin a local static server; [`MockDriver`] ignores the
/// content but is seeded to match).
///
/// Panics on any violation so it can be called directly from a `#[test]`.
pub async fn assert_browser_driver_conformance<D: BrowserDriver>(driver: &mut D, base_url: &str) {
    let _ = driver.engine();

    let landed = driver
        .goto(base_url)
        .await
        .unwrap_or_else(|err| panic!("goto failed: {err}"));
    assert_eq!(landed.url, base_url, "goto should set the current url");

    let observed = driver
        .observe()
        .await
        .unwrap_or_else(|err| panic!("observe failed: {err}"));
    assert_eq!(
        observed.url, base_url,
        "observe url should match goto target"
    );

    let grounded = driver
        .act(&BrowserAction::Click {
            selector: FIXTURE_KNOWN_SELECTOR.to_string(),
        })
        .await
        .unwrap_or_else(|err| panic!("act(known) failed: {err}"));
    assert_eq!(
        grounded.status,
        StepStatus::Ok,
        "known selector should yield ok status"
    );
    assert!(
        grounded.grounding.selector_existed,
        "known selector should resolve"
    );
    assert_eq!(
        grounded.grounding.selector.as_deref(),
        Some(FIXTURE_KNOWN_SELECTOR)
    );

    let missed = driver
        .act(&BrowserAction::Click {
            selector: FIXTURE_MISSING_SELECTOR.to_string(),
        })
        .await
        .unwrap_or_else(|err| panic!("act(missing) failed: {err}"));
    assert!(
        !missed.grounding.selector_existed,
        "missing selector must not resolve"
    );
    assert_eq!(
        missed.status,
        StepStatus::Error,
        "missing selector should record error status (not a BrowserError)"
    );

    let shot = driver
        .screenshot()
        .await
        .unwrap_or_else(|err| panic!("screenshot failed: {err}"));
    assert!(!shot.is_empty(), "screenshot should return bytes");

    let markup = driver
        .dom()
        .await
        .unwrap_or_else(|err| panic!("dom failed: {err}"));
    assert!(!markup.is_empty(), "dom should return markup");

    driver
        .close()
        .await
        .unwrap_or_else(|err| panic!("close failed: {err}"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_driver_passes_conformance() {
        let mut driver = MockDriver::with_conformance_fixture();
        assert_browser_driver_conformance(&mut driver, "https://fixture.local/page").await;
    }

    #[test]
    fn action_verb_and_selector() {
        let click = BrowserAction::Click {
            selector: "#x".to_string(),
        };
        assert_eq!(click.verb(), "click");
        assert_eq!(click.selector(), Some("#x"));

        let goto = BrowserAction::Goto {
            url: "https://example.com".to_string(),
        };
        assert_eq!(goto.verb(), "goto");
        assert_eq!(goto.selector(), None);
    }

    #[test]
    fn step_triple_roundtrips() {
        let triple = StepTriple {
            seq: 1,
            observation_before: Observation {
                url: "https://example.com".to_string(),
                title: None,
                dom_html: None,
                accessibility_tree: None,
                console: Vec::new(),
                network: Vec::new(),
            },
            decision: Some(LlmDecision {
                model: Some("claude".to_string()),
                prompt: serde_json::json!({"messages": []}),
                output: serde_json::json!({"action": "click"}),
                reasoning: Some("click the button".to_string()),
                input_tokens: None,
                output_tokens: None,
                cost_micros: None,
                latency_ms: None,
            }),
            action: BrowserAction::Click {
                selector: FIXTURE_KNOWN_SELECTOR.to_string(),
            },
            outcome: StepOutcome {
                status: StepStatus::Ok,
                error: None,
                grounding: Grounding {
                    selector: Some(FIXTURE_KNOWN_SELECTOR.to_string()),
                    selector_existed: true,
                    matched_element: true,
                },
                observation: Observation {
                    url: "https://example.com".to_string(),
                    title: None,
                    dom_html: None,
                    accessibility_tree: None,
                    console: Vec::new(),
                    network: Vec::new(),
                },
            },
        };
        let encoded = serde_json::to_string(&triple).unwrap_or_else(|err| panic!("{err}"));
        let decoded: StepTriple =
            serde_json::from_str(&encoded).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(decoded, triple);
    }

    // ── MockDriver + UrlPolicy integration ─────────────────────────────────

    /// Confirm that `allow_all` (the backwards-compatible default) does not
    /// break the existing conformance suite — this test is a regression guard
    /// for the policy wiring added in `goto`.
    #[tokio::test]
    async fn mock_driver_allow_all_passes_conformance() {
        // MockDriver::with_conformance_fixture uses allow_all by default.
        let mut driver = MockDriver::with_conformance_fixture();
        assert_browser_driver_conformance(&mut driver, "https://fixture.local/page").await;
    }

    /// A `MockDriver` armed with `block_private` must reject a goto to the
    /// AWS/GCP IMDS endpoint — the canonical SSRF target.
    #[tokio::test]
    async fn mock_driver_block_private_rejects_metadata_endpoint() {
        let mut driver =
            MockDriver::new(BrowserEngine::Chromium).with_policy(UrlPolicy::block_private());
        let Err(err) = driver
            .goto("http://169.254.169.254/latest/meta-data/iam/security-credentials/")
            .await
        else {
            panic!("expected SsrfBlocked for metadata endpoint");
        };
        assert!(
            matches!(err, BrowserError::SsrfBlocked(_)),
            "expected SsrfBlocked, got: {err:?}"
        );
        assert!(
            err.to_string().contains("SSRF guard"),
            "error text should mention SSRF guard: {err}"
        );
    }

    /// A `MockDriver` armed with `block_private` must reject navigations to
    /// localhost/loopback addresses commonly used for internal service probing.
    #[tokio::test]
    async fn mock_driver_block_private_rejects_loopback() {
        let mut driver =
            MockDriver::new(BrowserEngine::Chromium).with_policy(UrlPolicy::block_private());

        for url in &[
            "http://127.0.0.1",
            "http://localhost",
            "http://[::1]",
            "http://10.0.0.1",
            "http://192.168.1.1",
        ] {
            let Err(err) = driver.goto(url).await else {
                panic!("expected SsrfBlocked for {url:?}");
            };
            assert!(
                matches!(err, BrowserError::SsrfBlocked(_)),
                "expected SsrfBlocked for {url:?}, got: {err:?}"
            );
        }
    }

    /// A `MockDriver` armed with `block_private` must reject `file://` URLs —
    /// a common path to local file read via browser automation.
    #[tokio::test]
    async fn mock_driver_block_private_rejects_file_scheme() {
        let mut driver =
            MockDriver::new(BrowserEngine::Chromium).with_policy(UrlPolicy::block_private());
        let Err(err) = driver.goto("file:///etc/passwd").await else {
            panic!("expected SsrfBlocked for file scheme");
        };
        assert!(
            matches!(err, BrowserError::SsrfBlocked(_)),
            "expected SsrfBlocked, got: {err:?}"
        );
    }

    /// A `MockDriver` armed with `block_private` must allow navigation to
    /// publicly-routable HTTPS URLs — the normal operating case.
    #[tokio::test]
    async fn mock_driver_block_private_allows_public_https() {
        let mut driver =
            MockDriver::with_conformance_fixture().with_policy(UrlPolicy::block_private());
        // Should succeed (no error) and return the mocked observation.
        let obs = driver
            .goto("https://example.com/page")
            .await
            .unwrap_or_else(|err| panic!("unexpected block: {err}"));
        assert_eq!(obs.url, "https://example.com/page");
    }

    /// A `BrowserAction::Goto` dispatched through `act()` must also enforce
    /// the policy — the guard must not be bypassable via the action surface.
    #[tokio::test]
    async fn mock_driver_block_private_enforced_via_act() {
        let mut driver =
            MockDriver::new(BrowserEngine::Chromium).with_policy(UrlPolicy::block_private());
        let action = BrowserAction::Goto {
            url: "http://169.254.169.254".to_string(),
        };
        let Err(err) = driver.act(&action).await else {
            panic!("expected SsrfBlocked via act()");
        };
        assert!(
            matches!(err, BrowserError::SsrfBlocked(_)),
            "expected SsrfBlocked via act(), got: {err:?}"
        );
    }
}
