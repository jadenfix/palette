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
use beater_core::{EnvironmentId, Money, ProjectId, TenantId, Timestamp, TokenCounts, TraceId};
use beater_replay::{ReplayEvent, ReplayEventKind, SqliteReplayStore};
use beater_schema::{
    AgentSpanKind, ArtifactRef, CanonicalSpan, ModelRef, RedactionClass, SpanStatus,
    CANONICAL_SCHEMA_VERSION,
};
use beater_store::ArtifactStore;
use chrono::{Duration, Utc};
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

/// Internal input for assembling one [`CanonicalSpan`].
struct SpanParts {
    span_id: String,
    kind: AgentSpanKind,
    name: String,
    status: SpanStatus,
    start_time: Timestamp,
    end_time: Timestamp,
    model: Option<ModelRef>,
    cost: Option<Money>,
    tokens: Option<TokenCounts>,
    input_ref: Option<ArtifactRef>,
    output_ref: Option<ArtifactRef>,
    attributes: BTreeMap<String, Value>,
    raw_ref: ArtifactRef,
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
        let step_start = Utc::now();
        let observation_before = match self.last_observation.take() {
            Some(observation) => observation,
            None => self.driver.observe().await?,
        };

        // Perception: capture what the agent SAW before acting (the screenshot +
        // DOM it reasoned over), stored out of line. This is the input to the
        // decision — without it observability is "half" (decision but no view).
        let screenshot_before = self.driver.screenshot().await?;
        let screenshot_before_ref = self
            .put_bytes(&screenshot_before, "image/png", RedactionClass::Sensitive)
            .await?;
        let dom_before_bytes = match &observation_before.dom_html {
            Some(html) => html.clone().into_bytes(),
            None => self.driver.dom().await?.into_bytes(),
        };
        let dom_before_ref = self
            .put_bytes(&dom_before_bytes, "text/html", RedactionClass::Sensitive)
            .await?;

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
                .put_json(&serde_json::to_value(decision)?, RedactionClass::Sensitive)
                .await?;
            let mut attributes = BTreeMap::new();
            if let Some(reasoning) = &decision.reasoning {
                attributes.insert(semconv::REASONING.to_string(), json!(reasoning));
            }
            if let Some(model) = &decision.model {
                attributes.insert("llm.model".to_string(), json!(model));
            }
            if let Some(tokens) = decision.input_tokens {
                attributes.insert(semconv::INPUT_TOKENS.to_string(), json!(tokens));
            }
            if let Some(tokens) = decision.output_tokens {
                attributes.insert(semconv::OUTPUT_TOKENS.to_string(), json!(tokens));
            }
            if let Some(cost) = decision.cost_micros {
                attributes.insert(semconv::COST_MICROS.to_string(), json!(cost));
            }
            attributes.insert(
                semconv::SCREENSHOT_BEFORE_ARTIFACT.to_string(),
                json!(screenshot_before_ref.artifact_id.as_str()),
            );
            attributes.insert(semconv::STEP_SEQ.to_string(), json!(step_seq));
            let decision_end = decision
                .latency_ms
                .map(|ms| step_start + Duration::milliseconds(ms as i64))
                .unwrap_or(step_start);
            let span = self.build_span(SpanParts {
                span_id: format!("browser-decision-{step_seq}"),
                kind: AgentSpanKind::LlmCall,
                name: "browser.decision".to_string(),
                status: SpanStatus::Ok,
                start_time: step_start,
                end_time: decision_end,
                model: decision.model.clone().map(|name| ModelRef {
                    provider: "browser-agent".to_string(),
                    name,
                }),
                cost: decision.cost_micros.map(Money::usd_micros),
                tokens: decision_tokens(decision),
                // The model's input was the perception captured above.
                input_ref: Some(screenshot_before_ref.clone()),
                output_ref: None,
                attributes,
                raw_ref,
            })?;
            spans.push(span);
        }

        // 2) Execute the action, timing it.
        let act_start = Utc::now();
        let outcome = self.driver.act(&action).await?;
        let act_end = Utc::now();

        // 3) Store the result DOM + screenshot out of line.
        let dom_bytes = match &outcome.observation.dom_html {
            Some(html) => html.clone().into_bytes(),
            None => self.driver.dom().await?.into_bytes(),
        };
        let dom_ref = self
            .put_bytes(&dom_bytes, "text/html", RedactionClass::Sensitive)
            .await?;
        let screenshot = self.driver.screenshot().await?;
        let screenshot_ref = self
            .put_bytes(&screenshot, "image/png", RedactionClass::Sensitive)
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
            .put_json(&serde_json::to_value(&triple)?, RedactionClass::Sensitive)
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
        let action_latency_ms = (act_end - act_start).num_milliseconds().max(0);
        attributes.insert(
            semconv::ACTION_LATENCY_MS.to_string(),
            json!(action_latency_ms),
        );
        attributes.insert(
            semconv::DOM_BEFORE_ARTIFACT.to_string(),
            json!(dom_before_ref.artifact_id.as_str()),
        );
        attributes.insert(
            semconv::SCREENSHOT_BEFORE_ARTIFACT.to_string(),
            json!(screenshot_before_ref.artifact_id.as_str()),
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
        let span = self.build_span(SpanParts {
            span_id: format!("browser-step-{step_seq}"),
            kind: AgentSpanKind::ToolCall,
            name: format!("browser.{}", action.verb()),
            status: span_status,
            start_time: act_start,
            end_time: act_end,
            model: None,
            cost: None,
            tokens: None,
            // before = what the agent saw; after = the result page.
            input_ref: Some(screenshot_before_ref),
            output_ref: Some(screenshot_ref),
            attributes,
            raw_ref,
        })?;
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

    fn build_span(&mut self, parts: SpanParts) -> Result<CanonicalSpan, CaptureError> {
        let seq = self.span_seq;
        self.span_seq += 1;
        let span_id = beater_core::SpanId::new(parts.span_id)
            .map_err(|err| CaptureError::Id(err.to_string()))?;
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
            kind: parts.kind,
            name: parts.name,
            status: parts.status,
            start_time: parts.start_time,
            end_time: Some(parts.end_time),
            model: parts.model,
            cost: parts.cost,
            tokens: parts.tokens,
            input_ref: parts.input_ref,
            output_ref: parts.output_ref,
            attributes: parts.attributes,
            unmapped_attrs: json!({}),
            raw_ref: parts.raw_ref,
        })
    }
}

fn status_str(status: StepStatus) -> &'static str {
    match status {
        StepStatus::Ok => "ok",
        StepStatus::Error => "error",
    }
}

/// Build `TokenCounts` from a decision's token fields, or `None` if neither is set.
fn decision_tokens(decision: &LlmDecision) -> Option<TokenCounts> {
    match (decision.input_tokens, decision.output_tokens) {
        (None, None) => None,
        (input, output) => Some(TokenCounts {
            input: input.unwrap_or(0),
            output: output.unwrap_or(0),
            reasoning: 0,
            cache_read: 0,
        }),
    }
}

/// Project a run's [`StepTriple`]s into the `{"browser_steps": [...]}` trace
/// shape consumed by the browser evaluators in `beater-eval`.
pub fn browser_trace(triples: &[StepTriple]) -> Result<Value, CaptureError> {
    let steps = triples
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()?;
    // Aggregate decision economics to the top level so the existing CostBudget /
    // LatencyBudgetMs evaluators (which read trace.cost_micros / trace.latency_ms)
    // also work on browser runs.
    let cost_micros: i64 = triples
        .iter()
        .filter_map(|triple| triple.decision.as_ref().and_then(|d| d.cost_micros))
        .sum();
    let latency_ms: u64 = triples
        .iter()
        .filter_map(|triple| triple.decision.as_ref().and_then(|d| d.latency_ms))
        .sum();
    Ok(json!({
        "browser_steps": steps,
        "cost_micros": cost_micros,
        "latency_ms": latency_ms,
    }))
}

/// Project canonical browser-step spans into the same `{"browser_steps": [...]}`
/// trace shape the browser evaluators read.
///
/// This is the bridge for the **instrument → OTLP → ingest** pillar: a browser
/// agent run that arrives as OTLP from `browser-use`/Stagehand and is normalized
/// into [`CanonicalSpan`]s (e.g. `TraceView.spans`) can be scored and promoted
/// to a dataset case just like a natively captured run. Each `tool.call` span
/// carrying `browser.action` becomes one step, ordered by `browser.step_seq`.
///
/// Note: DOM text is stored out of line (artifacts), so it is not inlined here;
/// grounding, step-efficiency, recovery, and url-based task-success all work,
/// while `BrowserTaskSuccess { dom_contains }` over this projection is limited to
/// what is present (url). For full DOM matching, score the natively captured
/// [`StepTriple`]s via [`browser_trace`].
pub fn browser_trace_from_spans(spans: &[CanonicalSpan]) -> Value {
    let mut steps: Vec<(u64, Value)> = Vec::new();
    for span in spans {
        if span.kind != AgentSpanKind::ToolCall {
            continue;
        }
        let attrs = &span.attributes;
        let action = match attrs.get(semconv::ACTION).and_then(Value::as_str) {
            Some(action) => action.to_string(),
            None => continue,
        };
        let seq = attrs
            .get(semconv::STEP_SEQ)
            .and_then(Value::as_u64)
            .unwrap_or(span.seq);
        let status = attrs
            .get(semconv::STEP_STATUS)
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| match span.status {
                SpanStatus::Error => "error".to_string(),
                _ => "ok".to_string(),
            });
        // Mirror the native `BrowserAction` serialization (#[serde(tag="action",
        // content="args")]) so ingested and natively-captured browser_steps share
        // one `action` shape: { "action": <verb>, "args": { "selector": ... } }.
        let mut args = serde_json::Map::new();
        if let Some(selector) = attrs.get(semconv::SELECTOR).and_then(Value::as_str) {
            args.insert("selector".to_string(), json!(selector));
        }
        let step = json!({
            "seq": seq,
            "action": { "action": action, "args": Value::Object(args) },
            "outcome": {
                "status": status,
                "grounding": {
                    "selector": attrs.get(semconv::SELECTOR).cloned().unwrap_or(Value::Null),
                    "selector_existed": attrs
                        .get(semconv::SELECTOR_EXISTED)
                        .and_then(Value::as_bool),
                    "matched_element": attrs
                        .get(semconv::MATCHED_ELEMENT)
                        .and_then(Value::as_bool),
                },
                "observation": {
                    "url": attrs.get(semconv::URL).cloned().unwrap_or(Value::Null),
                },
            },
        });
        steps.push((seq, step));
    }
    steps.sort_by_key(|(seq, _)| *seq);
    let ordered: Vec<Value> = steps.into_iter().map(|(_, step)| step).collect();
    // Aggregate decision cost (llm.call spans) and action latency (tool.call
    // spans) so cost/latency evals work on ingested runs too — symmetric with
    // the native `browser_trace`.
    let cost_micros: i64 = spans
        .iter()
        .filter_map(|span| {
            span.attributes
                .get(semconv::COST_MICROS)
                .and_then(Value::as_i64)
        })
        .sum();
    let latency_ms: i64 = spans
        .iter()
        .filter_map(|span| {
            span.attributes
                .get(semconv::ACTION_LATENCY_MS)
                .and_then(Value::as_i64)
        })
        .sum();
    json!({
        "browser_steps": ordered,
        "cost_micros": cost_micros,
        "latency_ms": latency_ms,
    })
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
            input_tokens: Some(1200),
            output_tokens: Some(48),
            cost_micros: Some(3400),
            latency_ms: Some(820),
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

        // Full observability: the tool.call span records action latency and BOTH
        // the before (perception) and after (result) DOM + screenshot artifacts.
        assert!(tool.attributes.contains_key(semconv::ACTION_LATENCY_MS));
        assert!(tool.attributes.contains_key(semconv::DOM_BEFORE_ARTIFACT));
        assert!(tool
            .attributes
            .contains_key(semconv::SCREENSHOT_BEFORE_ARTIFACT));
        assert!(
            tool.input_ref.is_some(),
            "the agent's pre-action perception should be referenced as the span input"
        );
        assert_eq!(
            tool.input_ref
                .as_ref()
                .unwrap_or_else(|| panic!("tool input ref"))
                .redaction_class,
            RedactionClass::Sensitive
        );
        assert_eq!(
            tool.output_ref
                .as_ref()
                .unwrap_or_else(|| panic!("tool output ref"))
                .redaction_class,
            RedactionClass::Sensitive
        );
        assert_eq!(tool.raw_ref.redaction_class, RedactionClass::Sensitive);

        // The llm.call span carries the decision's model, cost, tokens, and uses
        // the perception (pre-action screenshot) as its input — so what the agent
        // saw and what the decision cost are both observable.
        let llm = &ok.spans[0];
        assert!(llm.model.is_some(), "decision model should be recorded");
        assert!(llm.cost.is_some(), "decision cost should be recorded");
        assert!(llm.tokens.is_some(), "decision tokens should be recorded");
        assert!(
            llm.input_ref.is_some(),
            "the perception should be the llm.call input"
        );
        assert_eq!(
            llm.input_ref
                .as_ref()
                .unwrap_or_else(|| panic!("llm input ref"))
                .redaction_class,
            RedactionClass::Sensitive
        );
        assert_eq!(llm.raw_ref.redaction_class, RedactionClass::Sensitive);
        assert!(llm.attributes.contains_key(semconv::INPUT_TOKENS));
        assert!(llm.attributes.contains_key(semconv::COST_MICROS));
        // A non-instantaneous decision latency yields end > start.
        assert!(llm.end_time.unwrap_or(llm.start_time) >= llm.start_time);

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
        // Decision economics aggregate to the trace so cost/latency evals run on
        // browser runs (2 decisions x 3400 micros / 820 ms each).
        assert_eq!(trace.get("cost_micros"), Some(&json!(6800)));
        assert_eq!(trace.get("latency_ms"), Some(&json!(1640)));
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

    fn dummy_artifact() -> ArtifactRef {
        ArtifactRef {
            artifact_id: beater_core::ArtifactId::new("a").unwrap_or_else(|err| panic!("{err}")),
            uri: "artifact://t/p/a".to_string(),
            sha256: beater_core::Sha256Hash::new("h").unwrap_or_else(|err| panic!("{err}")),
            size_bytes: 0,
            mime_type: "application/json".to_string(),
            redaction_class: RedactionClass::Internal,
        }
    }

    /// Build a `tool.call` `CanonicalSpan` carrying `browser.*` attributes,
    /// mirroring what `beater-otlp` produces for an ingested external agent run.
    fn ingested_tool_span(seq: u64, selector: &str, matched: bool, url: &str) -> CanonicalSpan {
        let mut attributes = BTreeMap::new();
        attributes.insert(semconv::ENGINE.to_string(), json!("chromium"));
        attributes.insert(semconv::ACTION.to_string(), json!("click"));
        attributes.insert(semconv::SELECTOR.to_string(), json!(selector));
        attributes.insert(semconv::SELECTOR_EXISTED.to_string(), json!(matched));
        attributes.insert(semconv::MATCHED_ELEMENT.to_string(), json!(matched));
        attributes.insert(semconv::STEP_SEQ.to_string(), json!(seq));
        attributes.insert(
            semconv::STEP_STATUS.to_string(),
            json!(if matched { "ok" } else { "error" }),
        );
        attributes.insert(semconv::URL.to_string(), json!(url));
        CanonicalSpan {
            schema_version: CANONICAL_SCHEMA_VERSION,
            normalizer_version: "beater-otlp-v1".to_string(),
            tenant_id: TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            project_id: ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            environment_id: EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
            trace_id: TraceId::new("trace-x").unwrap_or_else(|err| panic!("{err}")),
            span_id: beater_core::SpanId::new(format!("s{seq}"))
                .unwrap_or_else(|err| panic!("{err}")),
            parent_span_id: None,
            seq,
            kind: AgentSpanKind::ToolCall,
            name: "browser.click".to_string(),
            status: if matched {
                SpanStatus::Ok
            } else {
                SpanStatus::Error
            },
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            model: None,
            cost: None,
            tokens: None,
            input_ref: None,
            output_ref: None,
            attributes,
            unmapped_attrs: json!({}),
            raw_ref: dummy_artifact(),
        }
    }

    #[test]
    fn projects_ingested_spans_into_browser_steps() {
        // Out-of-order seq + a non-browser span that must be ignored.
        let spans = vec![
            ingested_tool_span(1, "#pay", true, "https://shop/confirm"),
            ingested_tool_span(0, "#cart", true, "https://shop/cart"),
        ];
        let trace = browser_trace_from_spans(&spans);
        let steps = trace
            .get("browser_steps")
            .and_then(Value::as_array)
            .unwrap_or_else(|| panic!("browser_steps"));
        assert_eq!(steps.len(), 2);
        // Steps are ordered by browser.step_seq.
        assert_eq!(steps[0]["seq"], json!(0));
        assert_eq!(steps[1]["seq"], json!(1));
        assert_eq!(steps[0]["action"]["action"], json!("click"));
        assert_eq!(
            steps[1]["outcome"]["grounding"]["matched_element"],
            json!(true)
        );
        assert_eq!(
            steps[1]["outcome"]["observation"]["url"],
            json!("https://shop/confirm")
        );
        // A miss projects status=error + matched_element=false.
        let miss = browser_trace_from_spans(&[ingested_tool_span(0, "#gone", false, "https://x")]);
        let miss_step = &miss["browser_steps"][0];
        assert_eq!(miss_step["outcome"]["status"], json!("error"));
        assert_eq!(
            miss_step["outcome"]["grounding"]["matched_element"],
            json!(false)
        );
    }
}
