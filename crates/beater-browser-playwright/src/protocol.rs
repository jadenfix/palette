//! Versioned JSON line protocol spoken between the Rust [`PlaywrightDriver`]
//! and the bundled Node Playwright runner (`runner/runner.js`).
//!
//! One JSON object per line in each direction:
//! - Rust writes a [`Command`] to the runner's stdin.
//! - The runner writes a [`Response`] to its stdout.
//!
//! The protocol is deliberately self-describing and engine-agnostic so the same
//! runner can drive Chromium / Chrome / Edge / Firefox / WebKit (selected by the
//! [`Launch`](Command::Launch) command's `engine`/`channel`).
//!
//! [`PlaywrightDriver`]: crate::PlaywrightDriver

use beater_browser::{Grounding, Observation, StepOutcome, StepStatus};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Protocol version. Bumped on any breaking change to the line schema; the
/// runner echoes it back in [`Response::ready`] so the driver can refuse a
/// mismatched runner instead of silently misbehaving.
pub const PROTOCOL_VERSION: u32 = 1;

/// A command the driver sends to the runner (one JSON line on stdin).
///
/// `id` correlates the eventual [`Response`]; the runner must echo it. The
/// payload is a `#[serde(tag = "op")]` union so the runner can switch on a
/// single string field.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Command {
    /// Monotonic correlation id, echoed in the matching [`Response`].
    pub id: u64,
    /// The operation to perform.
    #[serde(flatten)]
    pub payload: CommandPayload,
}

impl Command {
    /// Build a command with the given correlation id and payload.
    pub fn new(id: u64, payload: CommandPayload) -> Self {
        Self { id, payload }
    }
}

/// The discriminated operation carried by a [`Command`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum CommandPayload {
    /// Launch the browser/context/page. Sent once, first.
    Launch {
        /// Protocol version the driver speaks.
        protocol_version: u32,
        /// Engine to launch (`chromium` | `firefox` | `webkit`).
        engine: String,
        /// Optional Playwright `channel` (e.g. `chrome`, `msedge`).
        #[serde(skip_serializing_if = "Option::is_none")]
        channel: Option<String>,
        /// Run without a visible window (true in CI/headless).
        headless: bool,
    },
    /// Navigate to a URL.
    Goto {
        /// Absolute URL to load.
        url: String,
    },
    /// Click an element.
    Click {
        /// CSS selector to target.
        selector: String,
    },
    /// Type text into an element.
    Type {
        /// CSS selector to target.
        selector: String,
        /// Text to enter.
        text: String,
    },
    /// Scroll the page by a delta.
    Scroll {
        /// Horizontal delta in pixels.
        x: i64,
        /// Vertical delta in pixels.
        y: i64,
    },
    /// Select an option in a `<select>`.
    Select {
        /// CSS selector to target.
        selector: String,
        /// Option value to choose.
        value: String,
    },
    /// Wait for a fixed duration.
    Wait {
        /// Milliseconds to wait.
        millis: u64,
    },
    /// Extract the text content of an element.
    Extract {
        /// CSS selector to target.
        selector: String,
    },
    /// Observe current page state without acting.
    Observe,
    /// Capture a PNG screenshot.
    Screenshot,
    /// Capture the current DOM HTML.
    Dom,
    /// Tear down the page/context/browser.
    Close,
}

/// A response the runner sends back (one JSON line on stdout).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Response {
    /// Correlation id echoed from the matching [`Command`]. The `ready` banner
    /// the runner emits at startup carries id `0`.
    pub id: u64,
    /// The result payload.
    #[serde(flatten)]
    pub payload: ResponsePayload,
}

/// The discriminated result carried by a [`Response`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ResponsePayload {
    /// Startup banner: the runner launched and is ready for commands.
    Ready {
        /// Protocol version the runner speaks.
        protocol_version: u32,
        /// Engine actually launched.
        engine: String,
    },
    /// A page observation (reply to `goto`/`observe`).
    Observation {
        /// The page snapshot.
        observation: ObservationWire,
    },
    /// The outcome of an action (reply to click/type/scroll/select/wait/extract).
    Outcome {
        /// Whether the selector resolved to an element in the DOM.
        selector_existed: bool,
        /// Whether the action actually engaged a matched element.
        matched_element: bool,
        /// Selector targeted by the action, if any.
        #[serde(skip_serializing_if = "Option::is_none")]
        selector: Option<String>,
        /// Action-level error (e.g. selector not found). Distinct from a
        /// transport [`Error`](ResponsePayload::Error).
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
        /// Optional value produced by the action (e.g. extracted text).
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<Value>,
        /// Page state after the action ran.
        observation: ObservationWire,
    },
    /// Raw bytes encoded as base64 (reply to `screenshot`).
    Bytes {
        /// Base64-encoded payload (PNG for screenshots).
        base64: String,
    },
    /// Raw text (reply to `dom`).
    Text {
        /// The text payload (DOM HTML).
        text: String,
    },
    /// Acknowledgement with no payload (reply to `close`).
    Ack,
    /// A transport/backend failure. Maps to a `BrowserError`, NOT a failed
    /// grounding.
    Error {
        /// Human-readable failure description.
        message: String,
    },
}

/// Wire form of an [`Observation`]. Mirrors the contract type one-to-one so the
/// conversion below stays total and obvious.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObservationWire {
    /// Current page URL.
    pub url: String,
    /// Page `<title>`, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Convenience copy of the DOM HTML, if captured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dom_html: Option<String>,
    /// Accessibility tree snapshot, if captured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accessibility_tree: Option<Value>,
    /// Console messages observed on the page.
    #[serde(default)]
    pub console: Vec<String>,
}

impl From<ObservationWire> for Observation {
    fn from(wire: ObservationWire) -> Self {
        Observation {
            url: wire.url,
            title: wire.title,
            dom_html: wire.dom_html,
            accessibility_tree: wire.accessibility_tree,
            console: wire.console,
        }
    }
}

impl From<Observation> for ObservationWire {
    fn from(obs: Observation) -> Self {
        ObservationWire {
            url: obs.url,
            title: obs.title,
            dom_html: obs.dom_html,
            accessibility_tree: obs.accessibility_tree,
            console: obs.console,
        }
    }
}

/// Map an `Outcome` response into the contract [`StepOutcome`].
///
/// The grounding signal is derived entirely from the runner's
/// `selector_existed`/`matched_element` flags. Per the `BrowserDriver` contract,
/// a missing selector is a *successful call* that yields
/// [`StepStatus::Error`] — it is never a transport error.
pub fn outcome_from_response(
    selector_existed: bool,
    matched_element: bool,
    selector: Option<String>,
    error: Option<String>,
    observation: ObservationWire,
) -> StepOutcome {
    let grounding = Grounding {
        selector,
        selector_existed,
        matched_element,
    };
    let (status, error) = if selector_existed {
        (StepStatus::Ok, error)
    } else {
        let described = error.unwrap_or_else(|| {
            grounding
                .selector
                .as_deref()
                .map(|s| format!("selector not found: {s}"))
                .unwrap_or_else(|| "selector not found".to_string())
        });
        (StepStatus::Error, Some(described))
    };
    StepOutcome {
        status,
        error,
        grounding,
        observation: observation.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn click_command_serializes_to_expected_line() {
        let cmd = Command::new(
            7,
            CommandPayload::Click {
                selector: "#beater-known".to_string(),
            },
        );
        let json = serde_json::to_value(&cmd).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(json["id"], 7);
        assert_eq!(json["op"], "click");
        assert_eq!(json["selector"], "#beater-known");
    }

    #[test]
    fn launch_command_carries_channel_and_version() {
        let cmd = Command::new(
            1,
            CommandPayload::Launch {
                protocol_version: PROTOCOL_VERSION,
                engine: "chromium".to_string(),
                channel: Some("msedge".to_string()),
                headless: true,
            },
        );
        let json = serde_json::to_value(&cmd).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(json["op"], "launch");
        assert_eq!(json["channel"], "msedge");
        assert_eq!(json["protocol_version"], PROTOCOL_VERSION);
    }

    #[test]
    fn command_roundtrips_through_line() {
        let cmd = Command::new(
            42,
            CommandPayload::Type {
                selector: "#in".to_string(),
                text: "hello".to_string(),
            },
        );
        let line = serde_json::to_string(&cmd).unwrap_or_else(|err| panic!("{err}"));
        let back: Command = serde_json::from_str(&line).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(back, cmd);
    }

    #[test]
    fn ok_outcome_response_parses_into_step_outcome() {
        let line = r##"{"id":3,"kind":"outcome","selector_existed":true,"matched_element":true,"selector":"#beater-known","observation":{"url":"http://localhost/","title":"T","console":[]}}"##;
        let resp: Response = serde_json::from_str(line).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(resp.id, 3);
        let ResponsePayload::Outcome {
            selector_existed,
            matched_element,
            selector,
            error,
            observation,
            ..
        } = resp.payload
        else {
            panic!("expected outcome payload");
        };
        let outcome = outcome_from_response(
            selector_existed,
            matched_element,
            selector,
            error,
            observation,
        );
        assert_eq!(outcome.status, StepStatus::Ok);
        assert!(outcome.grounding.selector_existed);
        assert!(outcome.grounding.matched_element);
        assert_eq!(outcome.grounding.selector.as_deref(), Some("#beater-known"));
        assert_eq!(outcome.observation.url, "http://localhost/");
        assert!(outcome.error.is_none());
    }

    #[test]
    fn missing_selector_response_is_error_status_not_transport_error() {
        let line = r##"{"id":4,"kind":"outcome","selector_existed":false,"matched_element":false,"selector":"#beater-missing","observation":{"url":"http://localhost/","console":[]}}"##;
        let resp: Response = serde_json::from_str(line).unwrap_or_else(|err| panic!("{err}"));
        let ResponsePayload::Outcome {
            selector_existed,
            matched_element,
            selector,
            error,
            observation,
            ..
        } = resp.payload
        else {
            panic!("expected outcome payload");
        };
        let outcome = outcome_from_response(
            selector_existed,
            matched_element,
            selector,
            error,
            observation,
        );
        assert_eq!(outcome.status, StepStatus::Error);
        assert!(!outcome.grounding.selector_existed);
        assert!(outcome.error.is_some());
    }

    #[test]
    fn ready_banner_parses() {
        let line = r#"{"id":0,"kind":"ready","protocol_version":1,"engine":"chromium"}"#;
        let resp: Response = serde_json::from_str(line).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(resp.id, 0);
        match resp.payload {
            ResponsePayload::Ready {
                protocol_version,
                engine,
            } => {
                assert_eq!(protocol_version, PROTOCOL_VERSION);
                assert_eq!(engine, "chromium");
            }
            other => panic!("expected ready, got {other:?}"),
        }
    }

    #[test]
    fn observation_wire_roundtrips() {
        let obs = Observation {
            url: "http://x/".to_string(),
            title: Some("hi".to_string()),
            dom_html: Some("<html></html>".to_string()),
            accessibility_tree: Some(serde_json::json!({"role": "button"})),
            console: vec!["warn".to_string()],
        };
        let wire: ObservationWire = obs.clone().into();
        let back: Observation = wire.into();
        assert_eq!(back, obs);
    }
}
