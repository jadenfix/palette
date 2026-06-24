//! Capture layer: records a [`beater_browser::BrowserDriver`] session as
//! canonical spans + replay cassettes + artifacts.
//!
//! `BrowserToolProxy<D>` wraps any driver and, per step, stores DOM/screenshot
//! via `ArtifactStore::put_bytes`, records the LLM decision as an `llm.call`
//! span + `Provider` cassette (the prompt), and the action as a `tool.call` span
//! + `Tool` cassette, emitting the full [`beater_browser::StepTriple`].
//!
//! Stub — implemented in worktree WT-D (`feat/browser-capture`). Develops
//! against [`beater_browser::MockDriver`].
