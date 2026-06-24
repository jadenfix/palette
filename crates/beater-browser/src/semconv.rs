//! Canonical `browser.*` semantic-convention attribute keys.
//!
//! These constants are the single source of truth for the attribute keys a
//! browser-step span carries. Both the Rust capture layer
//! (`beater-browser-capture`) and the external instrumentation SDKs
//! (`sdks/python-browser-use`, `sdks/ts-stagehand`) emit these exact keys so the
//! OTLP ingest mapping (`beater-otlp`) can normalize any source identically.

/// Browser engine that executed the step (e.g. `chromium`, `firefox`, `webkit`).
pub const ENGINE: &str = "browser.engine";
/// Action verb for the step (e.g. `goto`, `click`, `type`).
pub const ACTION: &str = "browser.action";
/// Target selector for the action, when the action targets an element.
pub const SELECTOR: &str = "browser.selector";
/// URL the page was on when the step executed.
pub const URL: &str = "browser.url";
/// Page title observed for the step.
pub const TITLE: &str = "browser.title";
/// Whether the action's selector resolved to an element in the DOM.
pub const SELECTOR_EXISTED: &str = "browser.selector_existed";
/// Whether the resolved element was the intended target (grounding success).
pub const MATCHED_ELEMENT: &str = "browser.matched_element";
/// Monotonic step sequence number within the agent run.
pub const STEP_SEQ: &str = "browser.step_seq";
/// Step status string (`ok` or `error`).
pub const STEP_STATUS: &str = "browser.step_status";
/// Artifact id of the stored DOM snapshot for the step.
pub const DOM_ARTIFACT: &str = "browser.dom_artifact_id";
/// Artifact id of the stored screenshot for the step.
pub const SCREENSHOT_ARTIFACT: &str = "browser.screenshot_artifact_id";
/// The agent's reasoning text for the decision (from `model_thoughts`).
pub const REASONING: &str = "browser.reasoning";
