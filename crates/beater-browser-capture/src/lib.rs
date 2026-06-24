//! Capture layer for browser-agent observability.
//!
//! [`BrowserToolProxy`] wraps any [`beater_browser::BrowserDriver`] and turns
//! each step into Beater's canonical record:
//!
//! - the LLM decision (the prompt) → a `Provider` replay cassette event + an
//!   `llm.call` [`CanonicalSpan`], so prompts/code can be iterated and replayed;
//! - the browser action → a `Tool` replay cassette event + a `tool.call`
//!   [`CanonicalSpan`] carrying `browser.*` attributes;
//! - the DOM + screenshot → out-of-line artifacts via [`ArtifactStore`], with
//!   their ids referenced from the span.
//!
//! It also emits the [`StepTriple`] for each step; [`browser_trace`] projects a
//! run's triples into the `{"browser_steps": [...]}` shape the browser
//! evaluators in `beater-eval` read.

use beater_browser::{
    semconv, BrowserAction, BrowserDriver, LlmDecision, Observation, StepStatus, StepTriple,
};
use beater_core::{EnvironmentId, ProjectId, TenantId, TraceId};
use beater_replay::{ReplayEvent, ReplayEventKind, SqliteReplayStore};
use beater_schema::{
    AgentSpanKind, ArtifactRef, CanonicalSpan, RedactionClass, SpanStatus, CANONICAL_SCHEMA_VERSION,
};
use beater_store::ArtifactStore;
use chrono::Utc;
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// Errors raised while capturing a browser step.
#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("browser driver error: {0}")]
    Driver(#[from] beater_browser::BrowserError),
    #[error("artifact store error: {0}")]
    Store(String),
    #[error("replay store error: {0}")]
    Replay(String),
    #[error("serialize error: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("identifier error: {0}")]
    Id(String),
}

/// Trace-scoped identity for everything the proxy records.
#[derive(Clone, Debug)]
pub struct CaptureContext {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub environment_id: EnvironmentId,
    pub trace_id: TraceId,
    pub normalizer_version: String,
}

/// What a single captured step produced: the structured triple plus the
/// canonical spans (`llm.call` when a decision was supplied, then `tool.call`).
#[derive(Clone, Debug)]
pub struct RecordedStep {
    pub triple: StepTriple,
    pub spans: Vec<CanonicalSpan>,
}

/// Wraps a [`BrowserDriver`] and records each step as spans + cassettes +
/// artifacts. Generic over the artifact and replay stores so it works against
/// the in-memory/test stores and the real SQLite/filesystem stores alike.
pub struct BrowserToolProxy<D, A> {
    driver: D,
    artifacts: A,
    replay: SqliteReplayStore,
    ctx: CaptureContext,
    engine: beater_browser::BrowserEngine,
    /// Monotonic span counter — one per emitted `CanonicalSpan`.
    span_seq: u64,
    /// Monotonic logical step counter — one per `step()` call. Distinct from
    /// `span_seq` because a step can emit two spans (llm.call + tool.call).
    step_index: u64,
    replay_seq: u64,
    last_observation: Option<Observation>,
}

impl<D, A> BrowserToolProxy<D, A>
where
    D: BrowserDriver,
    A: ArtifactStore,
{
    pub fn new(driver: D, artifacts: A, replay: SqliteReplayStore, ctx: CaptureContext) -> Self {
        let engine = driver.engine();
        Self {
            driver,
            artifacts,
            replay,
            ctx,
            engine,
            span_seq: 0,
            step_index: 0,
            replay_seq: 0,
            last_observation: None,
        }
    }

    /// Borrow the underlying driver (e.g. for backend-specific setup).
    pub fn driver_mut(&mut self) -> &mut D {
        &mut self.driver
    }

    /// Navigate and seed the `observation_before` for the next step.
    pub async fn goto(&mut self, url: &str) -> Result<(), CaptureError> {
        let observation = self.driver.goto(url).await?;
        self.last_observation = Some(observation);
        Ok(())
    }

    /// Execute one observe → decide → act → outcome step and record it.
    pub async fn step(
        &mut self,
        decision: Option<LlmDecision>,
        action: BrowserAction,
    ) -> Result<RecordedStep, CaptureError> {
        let step_seq = self.step_index;
        self.step_index += 1;
        let observation_before = match self.last_observation.take() {
            Some(observation) => observation,
            None => self.driver.observe().await?,
        };

        let mut spans = Vec::new();

        // 1) Record the LLM decision (the prompt) as a Provider cassette + llm.call span.
        if let Some(decision) = &decision {
            let request = decision.prompt.clone();
            let response = json!({
                "output": decision.output,
                "reasoning": decision.reasoning,
                "model": decision.model,
            });
            self.put_replay_event(ReplayEventKind::Provider, request, response)
                .await?;

            let raw_ref = self
                .put_json(&serde_json::to_value(decision)?, RedactionClass::Internal)
                .await?;
            let mut attributes = BTreeMap::new();
            if let Some(reasoning) = &decision.reasoning {
                attributes.insert(semconv::REASONING.to_string(), json!(reasoning));
            }
            if let Some(model) = &decision.model {
                attributes.insert("llm.model".to_string(), json!(model));
            }
            attributes.insert(semconv::STEP_SEQ.to_string(), json!(step_seq));
            let span = self.build_span(
                format!("browser-decision-{step_seq}"),
                AgentSpanKind::LlmCall,
                "browser.decision".to_string(),
                SpanStatus::Ok,
                attributes,
                None,
                raw_ref,
            )?;
            spans.push(span);
        }

        // 2) Execute the action.
        let outcome = self.driver.act(&action).await?;

        // 3) Store DOM + screenshot out of line.
        let dom_bytes = match &outcome.observation.dom_html {
            Some(html) => html.clone().into_bytes(),
            None => self.driver.dom().await?.into_bytes(),
        };
        let dom_ref = self
            .put_bytes(&dom_bytes, "text/html", RedactionClass::Internal)
            .await?;
        let screenshot = self.driver.screenshot().await?;
        let screenshot_ref = self
            .put_bytes(&screenshot, "image/png", RedactionClass::Internal)
            .await?;

        // 4) Record the action as a Tool cassette + tool.call span.
        let request = serde_json::to_value(&action)?;
        let response = json!({
            "status": status_str(outcome.status),
            "grounding": outcome.grounding,
            "url": outcome.observation.url,
        });
        self.put_replay_event(ReplayEventKind::Tool, request, response)
            .await?;

        let triple = StepTriple {
            seq: step_seq,
            observation_before,
            decision,
            action: action.clone(),
            outcome: outcome.clone(),
        };
        let raw_ref = self
            .put_json(&serde_json::to_value(&triple)?, RedactionClass::Internal)
            .await?;

        let mut attributes = BTreeMap::new();
        attributes.insert(semconv::ENGINE.to_string(), json!(self.engine.as_str()));
        attributes.insert(semconv::ACTION.to_string(), json!(action.verb()));
        if let Some(selector) = action.selector() {
            attributes.insert(semconv::SELECTOR.to_string(), json!(selector));
        }
        attributes.insert(semconv::URL.to_string(), json!(outcome.observation.url));
        if let Some(title) = &outcome.observation.title {
            attributes.insert(semconv::TITLE.to_string(), json!(title));
        }
        attributes.insert(
            semconv::SELECTOR_EXISTED.to_string(),
            json!(outcome.grounding.selector_existed),
        );
        attributes.insert(
            semconv::MATCHED_ELEMENT.to_string(),
            json!(outcome.grounding.matched_element),
        );
        attributes.insert(semconv::STEP_SEQ.to_string(), json!(step_seq));
        attributes.insert(
            semconv::STEP_STATUS.to_string(),
            json!(status_str(outcome.status)),
        );
        attributes.insert(
            semconv::DOM_ARTIFACT.to_string(),
            json!(dom_ref.artifact_id.as_str()),
        );
        attributes.insert(
            semconv::SCREENSHOT_ARTIFACT.to_string(),
            json!(screenshot_ref.artifact_id.as_str()),
        );

        let span_status = match outcome.status {
            StepStatus::Ok => SpanStatus::Ok,
            StepStatus::Error => SpanStatus::Error,
        };
        let span = self.build_span(
            format!("browser-step-{step_seq}"),
            AgentSpanKind::ToolCall,
            format!("browser.{}", action.verb()),
            span_status,
            attributes,
            Some(screenshot_ref),
            raw_ref,
        )?;
        spans.push(span);

        self.last_observation = Some(outcome.observation);
        Ok(RecordedStep { triple, spans })
    }

    async fn put_replay_event(
        &mut self,
        kind: ReplayEventKind,
        request: Value,
        response: Value,
    ) -> Result<(), CaptureError> {
        self.replay_seq += 1;
        let event = ReplayEvent::new(
            self.ctx.tenant_id.clone(),
            self.ctx.project_id.clone(),
            self.ctx.trace_id.clone(),
            self.replay_seq,
            kind,
            request,
            response,
        )
        .map_err(|err| CaptureError::Replay(err.to_string()))?;
        self.replay
            .put_event(event)
            .await
            .map_err(|err| CaptureError::Replay(err.to_string()))?;
        Ok(())
    }

    async fn put_bytes(
        &self,
        bytes: &[u8],
        mime: &str,
        redaction: RedactionClass,
    ) -> Result<ArtifactRef, CaptureError> {
        self.artifacts
            .put_bytes(
                &self.ctx.tenant_id,
                &self.ctx.project_id,
                mime,
                redaction,
                bytes,
            )
            .await
            .map_err(|err| CaptureError::Store(err.to_string()))
    }

    async fn put_json(
        &self,
        value: &Value,
        redaction: RedactionClass,
    ) -> Result<ArtifactRef, CaptureError> {
        let bytes = serde_json::to_vec(value)?;
        self.put_bytes(&bytes, "application/json", redaction).await
    }

    #[allow(clippy::too_many_arguments)]
    fn build_span(
        &mut self,
        span_id: String,
        kind: AgentSpanKind,
        name: String,
        status: SpanStatus,
        attributes: BTreeMap<String, Value>,
        output_ref: Option<ArtifactRef>,
        raw_ref: ArtifactRef,
    ) -> Result<CanonicalSpan, CaptureError> {
        let seq = self.span_seq;
        self.span_seq += 1;
        let span_id =
            beater_core::SpanId::new(span_id).map_err(|err| CaptureError::Id(err.to_string()))?;
        Ok(CanonicalSpan {
            schema_version: CANONICAL_SCHEMA_VERSION,
            normalizer_version: self.ctx.normalizer_version.clone(),
            tenant_id: self.ctx.tenant_id.clone(),
            project_id: self.ctx.project_id.clone(),
            environment_id: self.ctx.environment_id.clone(),
            trace_id: self.ctx.trace_id.clone(),
            span_id,
            parent_span_id: None,
            seq,
            kind,
            name,
            status,
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            model: None,
            cost: None,
            tokens: None,
            input_ref: None,
            output_ref,
            attributes,
            unmapped_attrs: json!({}),
            raw_ref,
        })
    }
}

fn status_str(status: StepStatus) -> &'static str {
    match status {
        StepStatus::Ok => "ok",
        StepStatus::Error => "error",
    }
}

/// Project a run's [`StepTriple`]s into the `{"browser_steps": [...]}` trace
/// shape consumed by the browser evaluators in `beater-eval`.
pub fn browser_trace(triples: &[StepTriple]) -> Result<Value, CaptureError> {
    let steps = triples
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(json!({ "browser_steps": steps }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_browser::{
        BrowserEngine, MockDriver, FIXTURE_KNOWN_SELECTOR, FIXTURE_MISSING_SELECTOR,
    };
    use beater_replay::SqliteReplayStore;
    use beater_store_obj::FsArtifactStore;

    fn context() -> CaptureContext {
        CaptureContext {
            tenant_id: TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            project_id: ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            environment_id: EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
            trace_id: TraceId::new("trace-1").unwrap_or_else(|err| panic!("{err}")),
            normalizer_version: "browser-capture-test".to_string(),
        }
    }

    fn decision(action: &str) -> LlmDecision {
        LlmDecision {
            model: Some("claude".to_string()),
            prompt: json!({"messages": [{"role": "user", "content": "click ok"}]}),
            output: json!({"action": action}),
            reasoning: Some("the ok button completes the task".to_string()),
        }
    }

    #[tokio::test]
    async fn captures_spans_cassettes_and_artifacts() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = FsArtifactStore::new(dir.path()).unwrap_or_else(|err| panic!("{err}"));
        let replay = SqliteReplayStore::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let ctx = context();
        let driver = MockDriver::with_conformance_fixture();
        let mut proxy = BrowserToolProxy::new(driver, artifacts, replay, ctx.clone());

        proxy
            .goto("https://fixture.local/page")
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        // A grounded click, then a missing-selector click that records an error.
        let ok = proxy
            .step(
                Some(decision("click")),
                BrowserAction::Click {
                    selector: FIXTURE_KNOWN_SELECTOR.to_string(),
                },
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let miss = proxy
            .step(
                Some(decision("click")),
                BrowserAction::Click {
                    selector: FIXTURE_MISSING_SELECTOR.to_string(),
                },
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        // Each step emits an llm.call span and a tool.call span.
        assert_eq!(ok.spans.len(), 2);
        assert_eq!(ok.spans[0].kind, AgentSpanKind::LlmCall);
        assert_eq!(ok.spans[1].kind, AgentSpanKind::ToolCall);
        assert_eq!(ok.spans[1].status, SpanStatus::Ok);
        assert_eq!(miss.spans[1].status, SpanStatus::Error);

        // Logical step index is contiguous (0, 1, ...) even though each step
        // emits two spans — step_seq must not advance by span count.
        assert_eq!(ok.triple.seq, 0);
        assert_eq!(miss.triple.seq, 1);
        assert_eq!(
            ok.spans[1].attributes.get(semconv::STEP_SEQ),
            Some(&json!(0))
        );
        assert_eq!(
            miss.spans[1].attributes.get(semconv::STEP_SEQ),
            Some(&json!(1))
        );

        // tool.call span carries browser.* attributes + a screenshot artifact.
        let tool = &ok.spans[1];
        assert_eq!(
            tool.attributes.get(semconv::ENGINE),
            Some(&json!(BrowserEngine::Chromium.as_str()))
        );
        assert_eq!(tool.attributes.get(semconv::ACTION), Some(&json!("click")));
        assert_eq!(
            tool.attributes.get(semconv::SELECTOR_EXISTED),
            Some(&json!(true))
        );
        assert!(tool.output_ref.is_some(), "screenshot should be referenced");
        assert!(tool.attributes.contains_key(semconv::DOM_ARTIFACT));

        // Cassettes: one Provider + one Tool event per step = 4 total.
        let events = replay_events(&proxy).await;
        let providers = events
            .iter()
            .filter(|e| e.kind == ReplayEventKind::Provider)
            .count();
        let tools = events
            .iter()
            .filter(|e| e.kind == ReplayEventKind::Tool)
            .count();
        assert_eq!(providers, 2, "each decision records a Provider cassette");
        assert_eq!(tools, 2, "each action records a Tool cassette");

        // Artifacts are retrievable (DOM bytes round-trip via sha-checked store).
        let dom = proxy
            .artifacts
            .get_bytes(tool.output_ref.as_ref().unwrap_or_else(|| panic!("ref")))
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(!dom.is_empty());

        // The evaluator-facing trace projection round-trips and is scorable.
        let trace = browser_trace(&[ok.triple, miss.triple]).unwrap_or_else(|err| panic!("{err}"));
        let steps = trace
            .get("browser_steps")
            .and_then(Value::as_array)
            .unwrap_or_else(|| panic!("browser_steps array"));
        assert_eq!(steps.len(), 2);
    }

    async fn replay_events<D, A>(proxy: &BrowserToolProxy<D, A>) -> Vec<ReplayEvent> {
        proxy
            .replay
            .list_events(
                TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
                ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
                TraceId::new("trace-1").unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"))
    }
}
