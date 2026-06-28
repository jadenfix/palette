//! Playwright-backed [`beater_browser::BrowserDriver`] backend.
//!
//! Covers Chromium / Chrome / Edge / Firefox / WebKit via a bundled Node
//! Playwright runner (`runner/runner.js`) over a versioned JSON line protocol on
//! stdin/stdout (see [`protocol`]).
//!
//! The Rust side spawns `node runner.js` as a child process, performs a
//! [`Launch`](protocol::CommandPayload::Launch) handshake, then maps every
//! [`BrowserDriver`] method onto a request/response round-trip. Grounding
//! (`selector_existed` / `matched_element`) is taken verbatim from the runner so
//! a missing selector surfaces as a [`StepStatus`](beater_browser::StepStatus)
//! `Error` outcome — never a [`BrowserError`].
//!
//! Must pass [`beater_browser::assert_browser_driver_conformance`]; the live
//! conformance test that actually launches Node + Playwright is `#[ignore]`d
//! because neither is available in CI/sandbox.

pub mod protocol;

use async_trait::async_trait;
use beater_browser::{
    BrowserAction, BrowserDriver, BrowserEngine, BrowserError, Grounding, Observation, StepOutcome,
    StepStatus, UrlPolicy,
};
use protocol::{Command, CommandPayload, Response, ResponsePayload, PROTOCOL_VERSION};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command as TokioCommand};

/// Default Node entrypoint, relative to the runner directory.
const RUNNER_ENTRYPOINT: &str = "runner.js";

/// Configuration for spawning a [`PlaywrightDriver`].
#[derive(Clone, Debug)]
pub struct PlaywrightConfig {
    /// Contract engine to expose via [`BrowserDriver::engine`].
    pub engine: BrowserEngine,
    /// Path to the `node` executable.
    pub node_bin: PathBuf,
    /// Path to `runner.js` (the bundled runner script).
    pub runner_script: PathBuf,
    /// Whether to run the browser headless.
    pub headless: bool,
}

impl PlaywrightConfig {
    /// Build a config for `engine`, locating the bundled runner relative to this
    /// crate's source tree (`CARGO_MANIFEST_DIR/runner/runner.js`). `node` is
    /// resolved from `PATH`. Headless by default.
    pub fn new(engine: BrowserEngine) -> Self {
        let runner_script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("runner")
            .join(RUNNER_ENTRYPOINT);
        Self {
            engine,
            node_bin: PathBuf::from("node"),
            runner_script,
            headless: true,
        }
    }

    /// Override the path to the `node` executable.
    pub fn with_node_bin(mut self, node_bin: impl Into<PathBuf>) -> Self {
        self.node_bin = node_bin.into();
        self
    }

    /// Override the path to `runner.js`.
    pub fn with_runner_script(mut self, runner_script: impl Into<PathBuf>) -> Self {
        self.runner_script = runner_script.into();
        self
    }

    /// Set headless mode.
    pub fn with_headless(mut self, headless: bool) -> Self {
        self.headless = headless;
        self
    }

    /// The Playwright engine name (`chromium` | `firefox` | `webkit`) and the
    /// optional `channel` (`chrome` | `msedge`) for this contract engine.
    fn engine_and_channel(&self) -> (String, Option<String>) {
        match self.engine {
            BrowserEngine::Chromium | BrowserEngine::Other => ("chromium".to_string(), None),
            BrowserEngine::Chrome => ("chromium".to_string(), Some("chrome".to_string())),
            BrowserEngine::Edge => ("chromium".to_string(), Some("msedge".to_string())),
            BrowserEngine::Firefox => ("firefox".to_string(), None),
            // Playwright's WebKit engine also stands in for Safari.
            BrowserEngine::Webkit | BrowserEngine::Safari => ("webkit".to_string(), None),
        }
    }
}

/// A [`BrowserDriver`] that drives a real browser through a bundled Node
/// Playwright runner over the [`protocol`] JSON line contract.
pub struct PlaywrightDriver {
    engine: BrowserEngine,
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
    closed: bool,
    /// SSRF guard enforced at every navigation entry point. Defaults to
    /// [`UrlPolicy::block_private`]; override with [`PlaywrightDriver::with_policy`].
    policy: UrlPolicy,
}

impl PlaywrightDriver {
    /// Spawn `node runner.js`, perform the protocol handshake, and launch the
    /// browser. Returns once the runner reports `ready` and acks the launch.
    pub async fn launch(config: PlaywrightConfig) -> Result<Self, BrowserError> {
        let mut command = TokioCommand::new(&config.node_bin);
        command
            .arg(&config.runner_script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true);

        let mut child = command.spawn().map_err(|err| {
            BrowserError::Backend(format!(
                "failed to spawn {}: {err}",
                display_path(&config.node_bin)
            ))
        })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| BrowserError::Backend("runner stdin was not captured".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| BrowserError::Backend("runner stdout was not captured".to_string()))?;

        let (engine_name, channel) = config.engine_and_channel();
        let mut driver = Self {
            engine: config.engine,
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
            closed: false,
            policy: UrlPolicy::block_private(),
        };

        // Read the startup banner first.
        let ready = driver.read_response().await?;
        match ready.payload {
            ResponsePayload::Ready {
                protocol_version, ..
            } => {
                if protocol_version != PROTOCOL_VERSION {
                    return Err(BrowserError::Backend(format!(
                        "runner protocol version {protocol_version} != expected {PROTOCOL_VERSION}"
                    )));
                }
            }
            ResponsePayload::Error { message } => return Err(BrowserError::Backend(message)),
            other => {
                return Err(BrowserError::Backend(format!(
                    "expected ready banner, got {other:?}"
                )))
            }
        }

        // Launch the browser/context/page.
        let launch = driver
            .request(CommandPayload::Launch {
                protocol_version: PROTOCOL_VERSION,
                engine: engine_name,
                channel,
                headless: config.headless,
            })
            .await?;
        match launch.payload {
            ResponsePayload::Ack | ResponsePayload::Ready { .. } => {}
            ResponsePayload::Error { message } => return Err(BrowserError::Backend(message)),
            other => {
                return Err(BrowserError::Backend(format!(
                    "expected launch ack, got {other:?}"
                )))
            }
        }

        Ok(driver)
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
    /// testable without spawning the Node runner.
    fn enforce_nav_policy(policy: &UrlPolicy, url: &str) -> Result<(), BrowserError> {
        policy.enforce(url)
    }

    /// Allocate the next correlation id.
    fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        id
    }

    /// Send one command and read exactly one correlated response.
    async fn request(&mut self, payload: CommandPayload) -> Result<Response, BrowserError> {
        let id = self.alloc_id();
        let command = Command::new(id, payload);
        let mut line = serde_json::to_string(&command)
            .map_err(|err| BrowserError::Backend(format!("failed to encode command: {err}")))?;
        line.push('\n');
        self.stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|err| BrowserError::Backend(format!("failed to write command: {err}")))?;
        self.stdin
            .flush()
            .await
            .map_err(|err| BrowserError::Backend(format!("failed to flush command: {err}")))?;

        let response = self.read_response().await?;
        if response.id != id {
            return Err(BrowserError::Backend(format!(
                "response id {} did not match request id {id}",
                response.id
            )));
        }
        Ok(response)
    }

    /// Read and decode the next response line from the runner.
    async fn read_response(&mut self) -> Result<Response, BrowserError> {
        let mut line = String::new();
        let read = self
            .stdout
            .read_line(&mut line)
            .await
            .map_err(|err| BrowserError::Backend(format!("failed to read response: {err}")))?;
        if read == 0 {
            return Err(BrowserError::Backend(
                "runner closed stdout unexpectedly".to_string(),
            ));
        }
        serde_json::from_str(line.trim_end()).map_err(|err| {
            BrowserError::Backend(format!("failed to decode response {line:?}: {err}"))
        })
    }

    /// Translate a [`BrowserAction`] into its protocol command.
    fn action_to_command(action: &BrowserAction) -> CommandPayload {
        match action {
            BrowserAction::Goto { url } => CommandPayload::Goto { url: url.clone() },
            BrowserAction::Click { selector } => CommandPayload::Click {
                selector: selector.clone(),
            },
            BrowserAction::Type { selector, text } => CommandPayload::Type {
                selector: selector.clone(),
                text: text.clone(),
            },
            BrowserAction::Scroll { x, y } => CommandPayload::Scroll { x: *x, y: *y },
            BrowserAction::Select { selector, value } => CommandPayload::Select {
                selector: selector.clone(),
                value: value.clone(),
            },
            BrowserAction::Wait { millis } => CommandPayload::Wait { millis: *millis },
            BrowserAction::Extract { selector } => CommandPayload::Extract {
                selector: selector.clone(),
            },
        }
    }

    /// Interpret an observation-style response (`goto`/`observe`).
    fn expect_observation(response: Response) -> Result<Observation, BrowserError> {
        match response.payload {
            ResponsePayload::Observation { observation } => Ok(observation.into()),
            ResponsePayload::Error { message } => Err(BrowserError::Backend(message)),
            other => Err(BrowserError::Backend(format!(
                "expected observation, got {other:?}"
            ))),
        }
    }
}

#[async_trait]
impl BrowserDriver for PlaywrightDriver {
    fn engine(&self) -> BrowserEngine {
        self.engine
    }

    async fn goto(&mut self, url: &str) -> Result<Observation, BrowserError> {
        // SSRF guard: reject blocked targets before issuing the navigate command.
        Self::enforce_nav_policy(&self.policy, url)?;
        let response = self
            .request(CommandPayload::Goto {
                url: url.to_string(),
            })
            .await?;
        match response.payload {
            ResponsePayload::Observation { observation } => Ok(observation.into()),
            ResponsePayload::Error { message } => Err(BrowserError::Navigation(message)),
            other => Err(BrowserError::Backend(format!(
                "expected observation, got {other:?}"
            ))),
        }
    }

    async fn act(&mut self, action: &BrowserAction) -> Result<StepOutcome, BrowserError> {
        // `goto` is navigation, not a grounded action: forward to `goto` and
        // synthesize a successful outcome from the resulting observation.
        if let BrowserAction::Goto { url } = action {
            let observation = self.goto(url).await?;
            return Ok(StepOutcome {
                status: StepStatus::Ok,
                error: None,
                grounding: Grounding {
                    selector: None,
                    selector_existed: true,
                    matched_element: true,
                },
                observation,
            });
        }

        let response = self.request(Self::action_to_command(action)).await?;
        match response.payload {
            ResponsePayload::Outcome {
                selector_existed,
                matched_element,
                selector,
                error,
                observation,
                ..
            } => Ok(protocol::outcome_from_response(
                selector_existed,
                matched_element,
                selector,
                error,
                observation,
            )),
            ResponsePayload::Error { message } => Err(BrowserError::Backend(message)),
            other => Err(BrowserError::Backend(format!(
                "expected outcome, got {other:?}"
            ))),
        }
    }

    async fn observe(&mut self) -> Result<Observation, BrowserError> {
        let response = self.request(CommandPayload::Observe).await?;
        Self::expect_observation(response)
    }

    async fn screenshot(&mut self) -> Result<Vec<u8>, BrowserError> {
        let response = self.request(CommandPayload::Screenshot).await?;
        match response.payload {
            ResponsePayload::Bytes { base64 } => decode_base64(&base64)
                .map_err(|err| BrowserError::Backend(format!("invalid screenshot base64: {err}"))),
            ResponsePayload::Error { message } => Err(BrowserError::Backend(message)),
            other => Err(BrowserError::Backend(format!(
                "expected bytes, got {other:?}"
            ))),
        }
    }

    async fn dom(&mut self) -> Result<String, BrowserError> {
        let response = self.request(CommandPayload::Dom).await?;
        match response.payload {
            ResponsePayload::Text { text } => Ok(text),
            ResponsePayload::Error { message } => Err(BrowserError::Backend(message)),
            other => Err(BrowserError::Backend(format!(
                "expected text, got {other:?}"
            ))),
        }
    }

    async fn close(&mut self) -> Result<(), BrowserError> {
        if self.closed {
            return Ok(());
        }
        self.closed = true;
        // Best-effort graceful close, then ensure the process is reaped.
        let result = self.request(CommandPayload::Close).await;
        let _ = self.stdin.shutdown().await;
        let _ = self.child.wait().await;
        match result {
            Ok(response) => match response.payload {
                ResponsePayload::Ack => Ok(()),
                ResponsePayload::Error { message } => Err(BrowserError::Backend(message)),
                other => Err(BrowserError::Backend(format!(
                    "expected close ack, got {other:?}"
                ))),
            },
            Err(err) => Err(err),
        }
    }
}

/// Render a path for error messages without panicking on non-UTF-8.
fn display_path(path: &Path) -> String {
    path.as_os_str()
        .to_str()
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("{:?}", path.as_os_str()))
}

/// Decode standard (RFC 4648) base64 without an external dependency. ASCII
/// whitespace is ignored and `=` padding is honored. Returns the decoded bytes.
fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(input.len() / 4 * 3);
    let mut buf = 0u32;
    let mut bits = 0u32;
    for &byte in input.as_bytes() {
        let sextet = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            // Padding and whitespace contribute no sextet.
            b'=' | b' ' | b'\n' | b'\r' | b'\t' => continue,
            other => return Err(format!("invalid base64 byte: {other}")),
        };
        buf = (buf << 6) | u32::from(sextet);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goto_navigation_guard_blocks_ssrf_targets() {
        // Proves the exact guard `goto`/`act(Goto)` invoke rejects blocked
        // targets — including alternate IP encodings — before any runner I/O.
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
            let Err(err) = PlaywrightDriver::enforce_nav_policy(&policy, url) else {
                panic!("expected SsrfBlocked for {url:?}");
            };
            assert!(
                matches!(err, BrowserError::SsrfBlocked(_)),
                "expected SsrfBlocked for {url:?}, got {err:?}"
            );
        }
        assert!(PlaywrightDriver::enforce_nav_policy(&policy, "https://example.com").is_ok());
    }

    #[test]
    fn engine_mapping_picks_channel_and_engine() {
        let chrome = PlaywrightConfig::new(BrowserEngine::Chrome).engine_and_channel();
        assert_eq!(chrome, ("chromium".to_string(), Some("chrome".to_string())));

        let edge = PlaywrightConfig::new(BrowserEngine::Edge).engine_and_channel();
        assert_eq!(edge, ("chromium".to_string(), Some("msedge".to_string())));

        let firefox = PlaywrightConfig::new(BrowserEngine::Firefox).engine_and_channel();
        assert_eq!(firefox, ("firefox".to_string(), None));

        let webkit = PlaywrightConfig::new(BrowserEngine::Webkit).engine_and_channel();
        assert_eq!(webkit, ("webkit".to_string(), None));

        let chromium = PlaywrightConfig::new(BrowserEngine::Chromium).engine_and_channel();
        assert_eq!(chromium, ("chromium".to_string(), None));
    }

    #[test]
    fn action_maps_to_command() {
        let cmd = PlaywrightDriver::action_to_command(&BrowserAction::Type {
            selector: "#a".to_string(),
            text: "hi".to_string(),
        });
        match cmd {
            CommandPayload::Type { selector, text } => {
                assert_eq!(selector, "#a");
                assert_eq!(text, "hi");
            }
            other => panic!("expected type command, got {other:?}"),
        }
    }

    #[test]
    fn base64_decodes_known_vectors() {
        assert_eq!(decode_base64("").unwrap_or_else(|e| panic!("{e}")), b"");
        assert_eq!(
            decode_base64("aGVsbG8=").unwrap_or_else(|e| panic!("{e}")),
            b"hello"
        );
        // PNG signature.
        let png = decode_base64("iVBORw0KGgo=").unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(png, vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
        // Embedded whitespace tolerated.
        assert_eq!(
            decode_base64("aGVs\nbG8=").unwrap_or_else(|e| panic!("{e}")),
            b"hello"
        );
    }

    #[test]
    fn base64_rejects_invalid_byte() {
        assert!(decode_base64("@@@@").is_err());
    }

    #[test]
    fn default_runner_script_points_into_crate() {
        let config = PlaywrightConfig::new(BrowserEngine::Chromium);
        assert!(config.runner_script.ends_with("runner/runner.js"));
        assert_eq!(config.node_bin, PathBuf::from("node"));
        assert!(config.headless);
    }
}
