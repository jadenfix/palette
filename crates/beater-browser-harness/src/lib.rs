//! Eval harness for browser agents.
//!
//! `BrowserAgentAdapter` implements the existing `beater_experiments::AgentAdapter`
//! and runs a candidate either by wrapping `browser-use` over a subprocess bridge
//! or by driving a native [`beater_browser::BrowserDriver`]; both emit identical
//! [`beater_browser::StepTriple`]s for A/B comparison over frozen cassettes. A
//! thin `ScenarioRunner` adds seeds/timeouts/concurrency.
//!
//! Stub — implemented in worktree WT-F (`feat/browser-loop`).
