use anyhow::{anyhow, Context};
use beater_core::{ProjectId, TenantId, Timestamp, TraceId};
use beater_schema::{CanonicalSpan, SpanStatus, TraceView};
use beater_security::sign_webhook;
use chrono::Duration;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OnlineSamplingPolicy {
    pub sample_rate_per_mille: u16,
    pub keep_errors: bool,
    pub slow_ms_threshold: Option<u64>,
    pub high_cost_micros_threshold: Option<i64>,
}

impl Default for OnlineSamplingPolicy {
    fn default() -> Self {
        Self {
            sample_rate_per_mille: 100,
            keep_errors: true,
            slow_ms_threshold: Some(30_000),
            high_cost_micros_threshold: Some(50_000),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SamplingReason {
    ErrorTrace,
    SlowTrace,
    HighCostTrace,
    RoutineSampled,
    RoutineDropped,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SamplingDecision {
    pub selected: bool,
    pub reason: SamplingReason,
    pub stable_score_per_mille: u16,
}

pub fn decide_trace_sampling(trace: &TraceView, policy: &OnlineSamplingPolicy) -> SamplingDecision {
    let stable_score_per_mille = stable_trace_score(&trace.tenant_id, &trace.trace_id);
    if policy.keep_errors
        && trace
            .spans
            .iter()
            .any(|span| span.status == SpanStatus::Error)
    {
        return SamplingDecision {
            selected: true,
            reason: SamplingReason::ErrorTrace,
            stable_score_per_mille,
        };
    }
    if let Some(threshold) = policy.slow_ms_threshold {
        if trace_latency_ms(&trace.spans).is_some_and(|latency| latency >= threshold) {
            return SamplingDecision {
                selected: true,
                reason: SamplingReason::SlowTrace,
                stable_score_per_mille,
            };
        }
    }
    if let Some(threshold) = policy.high_cost_micros_threshold {
        if trace_cost_micros(&trace.spans) >= threshold {
            return SamplingDecision {
                selected: true,
                reason: SamplingReason::HighCostTrace,
                stable_score_per_mille,
            };
        }
    }
    let sample_rate = policy.sample_rate_per_mille.min(1000);
    let selected = stable_score_per_mille < sample_rate;
    SamplingDecision {
        selected,
        reason: if selected {
            SamplingReason::RoutineSampled
        } else {
            SamplingReason::RoutineDropped
        },
        stable_score_per_mille,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AlertPolicy {
    pub policy_id: String,
    pub endpoint_url: String,
    pub signing_secret: String,
    pub severity: AlertSeverity,
    pub fire_when_score_at_or_below: f64,
    pub dedupe_window_seconds: i64,
    pub maintenance_windows: Vec<MaintenanceWindow>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaintenanceWindow {
    pub starts_at: Timestamp,
    pub ends_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AlertLinks {
    pub trace_url: String,
    pub cluster_url: Option<String>,
    pub dataset_url: Option<String>,
    pub gate_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AlertInput {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub trace_id: TraceId,
    pub group_key: String,
    pub title: String,
    pub score: f64,
    pub baseline_score: Option<f64>,
    pub links: AlertLinks,
    pub now: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AlertDecision {
    pub emitted: bool,
    pub suppressed_reason: Option<String>,
    pub delivery: Option<WebhookDelivery>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebhookDelivery {
    pub endpoint_url: String,
    pub headers: BTreeMap<String, String>,
    pub body: serde_json::Value,
}

#[derive(Clone, Debug, Default)]
pub struct AlertEngine {
    state: Arc<Mutex<AlertState>>,
}

#[derive(Clone, Debug, Default)]
struct AlertState {
    last_emitted_by_group: BTreeMap<String, Timestamp>,
}

impl AlertEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn evaluate(
        &self,
        policy: &AlertPolicy,
        input: AlertInput,
    ) -> anyhow::Result<AlertDecision> {
        if input.score > policy.fire_when_score_at_or_below {
            return Ok(AlertDecision {
                emitted: false,
                suppressed_reason: Some("score_above_threshold".to_string()),
                delivery: None,
            });
        }
        if in_maintenance_window(&policy.maintenance_windows, input.now) {
            return Ok(AlertDecision {
                emitted: false,
                suppressed_reason: Some("maintenance_window".to_string()),
                delivery: None,
            });
        }
        let group_key = dedupe_key(policy, &input);
        let mut state = self
            .state
            .lock()
            .map_err(|err| anyhow!("alert engine mutex poisoned: {err}"))?;
        if let Some(last_emitted) = state.last_emitted_by_group.get(&group_key) {
            let age = input.now.signed_duration_since(*last_emitted);
            if age >= Duration::zero()
                && age < Duration::seconds(policy.dedupe_window_seconds.max(0))
            {
                return Ok(AlertDecision {
                    emitted: false,
                    suppressed_reason: Some("dedupe_window".to_string()),
                    delivery: None,
                });
            }
        }

        let body = alert_payload(policy, &input);
        let body_bytes = serde_json::to_vec(&body).context("serialize alert payload")?;
        let signature = sign_webhook(policy.signing_secret.as_bytes(), &body_bytes, input.now)?;
        let mut headers = BTreeMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("beater-signature".to_string(), signature.header_value());
        headers.insert("beater-alert-policy".to_string(), policy.policy_id.clone());
        let delivery = WebhookDelivery {
            endpoint_url: policy.endpoint_url.clone(),
            headers,
            body,
        };
        state.last_emitted_by_group.insert(group_key, input.now);
        Ok(AlertDecision {
            emitted: true,
            suppressed_reason: None,
            delivery: Some(delivery),
        })
    }
}

fn alert_payload(policy: &AlertPolicy, input: &AlertInput) -> serde_json::Value {
    serde_json::json!({
        "policy_id": policy.policy_id,
        "severity": policy.severity,
        "tenant_id": input.tenant_id,
        "project_id": input.project_id,
        "trace_id": input.trace_id,
        "group_key": input.group_key,
        "title": input.title,
        "score": input.score,
        "baseline_score": input.baseline_score,
        "threshold": policy.fire_when_score_at_or_below,
        "links": input.links,
        "emitted_at": input.now,
    })
}

fn dedupe_key(policy: &AlertPolicy, input: &AlertInput) -> String {
    format!(
        "{}:{}:{}:{}",
        input.tenant_id.as_str(),
        input.project_id.as_str(),
        policy.policy_id,
        input.group_key
    )
}

fn in_maintenance_window(windows: &[MaintenanceWindow], now: Timestamp) -> bool {
    windows
        .iter()
        .any(|window| now >= window.starts_at && now <= window.ends_at)
}

fn stable_trace_score(tenant_id: &TenantId, trace_id: &TraceId) -> u16 {
    let mut hasher = Sha256::new();
    hasher.update(tenant_id.as_str().as_bytes());
    hasher.update(b":");
    hasher.update(trace_id.as_str().as_bytes());
    let digest = hasher.finalize();
    u16::from_be_bytes([digest[0], digest[1]]) % 1000
}

fn trace_cost_micros(spans: &[CanonicalSpan]) -> i64 {
    spans
        .iter()
        .filter_map(|span| span.cost.as_ref())
        .map(|cost| cost.amount_micros)
        .sum()
}

fn trace_latency_ms(spans: &[CanonicalSpan]) -> Option<u64> {
    let start = spans.iter().map(|span| span.start_time).min()?;
    let end = spans.iter().filter_map(|span| span.end_time).max()?;
    let millis = end.signed_duration_since(start).num_milliseconds();
    Some(millis.max(0) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_core::{Money, TokenCounts};
    use std::collections::BTreeSet;

    fn required_link_keys(delivery: &WebhookDelivery) -> BTreeSet<&'static str> {
        let links = &delivery.body["links"];
        ["trace_url", "cluster_url", "dataset_url", "gate_url"]
            .into_iter()
            .filter(|key| links.get(key).and_then(|value| value.as_str()).is_some())
            .collect()
    }

    use beater_schema::{AgentSpanKind, ArtifactRef, RedactionClass, CANONICAL_SCHEMA_VERSION};
    use chrono::Utc;

    #[test]
    fn sampling_keeps_errors_slow_and_high_cost_traces() {
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let mut trace = fixture_trace(&tenant, SpanStatus::Ok, 1_000, 0);
        let policy = OnlineSamplingPolicy {
            sample_rate_per_mille: 0,
            keep_errors: true,
            slow_ms_threshold: Some(5_000),
            high_cost_micros_threshold: Some(10_000),
        };
        assert_eq!(
            decide_trace_sampling(&trace, &policy).reason,
            SamplingReason::RoutineDropped
        );

        trace.spans[0].status = SpanStatus::Error;
        assert_eq!(
            decide_trace_sampling(&trace, &policy).reason,
            SamplingReason::ErrorTrace
        );

        let slow_trace = fixture_trace(&tenant, SpanStatus::Ok, 6_000, 0);
        assert_eq!(
            decide_trace_sampling(&slow_trace, &policy).reason,
            SamplingReason::SlowTrace
        );

        let costly_trace = fixture_trace(&tenant, SpanStatus::Ok, 1_000, 20_000);
        assert_eq!(
            decide_trace_sampling(&costly_trace, &policy).reason,
            SamplingReason::HighCostTrace
        );
    }

    #[test]
    fn alert_engine_signs_dedupes_and_respects_maintenance_windows() {
        let engine = AlertEngine::new();
        let now = Utc::now();
        let policy = AlertPolicy {
            policy_id: "low-score".to_string(),
            endpoint_url: "https://example.test/webhook".to_string(),
            signing_secret: "secret".to_string(),
            severity: AlertSeverity::Critical,
            fire_when_score_at_or_below: 0.5,
            dedupe_window_seconds: 60,
            maintenance_windows: Vec::new(),
        };
        let input = fixture_alert_input(now);
        let first = engine
            .evaluate(&policy, input.clone())
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(first.emitted);
        let delivery = first
            .delivery
            .as_ref()
            .unwrap_or_else(|| panic!("expected delivery"));
        assert_eq!(
            required_link_keys(delivery),
            BTreeSet::from(["cluster_url", "dataset_url", "gate_url", "trace_url"])
        );
        let body = serde_json::to_vec(&delivery.body).unwrap_or_else(|err| panic!("{err}"));
        beater_security::verify_webhook(
            b"secret",
            &body,
            delivery
                .headers
                .get("beater-signature")
                .unwrap_or_else(|| panic!("missing signature header")),
            now,
            Duration::seconds(300),
        )
        .unwrap_or_else(|err| panic!("{err}"));

        let second = engine
            .evaluate(&policy, input)
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(!second.emitted);
        assert_eq!(second.suppressed_reason.as_deref(), Some("dedupe_window"));

        let maintenance_policy = AlertPolicy {
            maintenance_windows: vec![MaintenanceWindow {
                starts_at: now - Duration::minutes(1),
                ends_at: now + Duration::minutes(1),
            }],
            ..policy
        };
        let suppressed = AlertEngine::new()
            .evaluate(&maintenance_policy, fixture_alert_input(now))
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(!suppressed.emitted);
        assert_eq!(
            suppressed.suppressed_reason.as_deref(),
            Some("maintenance_window")
        );
    }

    fn fixture_alert_input(now: Timestamp) -> AlertInput {
        AlertInput {
            tenant_id: TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            project_id: ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            trace_id: TraceId::new("trace").unwrap_or_else(|err| panic!("{err}")),
            group_key: "eval:exact:low-score".to_string(),
            title: "Eval score dropped".to_string(),
            score: 0.1,
            baseline_score: Some(0.9),
            links: AlertLinks {
                trace_url: "https://beater.test/traces/trace".to_string(),
                cluster_url: Some("https://beater.test/clusters/cluster".to_string()),
                dataset_url: Some("https://beater.test/datasets/dataset".to_string()),
                gate_url: Some("https://beater.test/gates/gate".to_string()),
            },
            now,
        }
    }

    fn fixture_trace(
        tenant: &TenantId,
        status: SpanStatus,
        latency_ms: i64,
        cost_micros: i64,
    ) -> TraceView {
        let started = Utc::now();
        TraceView {
            tenant_id: tenant.clone(),
            trace_id: TraceId::new("trace").unwrap_or_else(|err| panic!("{err}")),
            spans: vec![CanonicalSpan {
                schema_version: CANONICAL_SCHEMA_VERSION,
                normalizer_version: "test".to_string(),
                tenant_id: tenant.clone(),
                project_id: ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
                environment_id: beater_core::EnvironmentId::new("prod")
                    .unwrap_or_else(|err| panic!("{err}")),
                trace_id: TraceId::new("trace").unwrap_or_else(|err| panic!("{err}")),
                span_id: beater_core::SpanId::new("span").unwrap_or_else(|err| panic!("{err}")),
                parent_span_id: None,
                seq: 1,
                kind: AgentSpanKind::AgentRun,
                name: "run".to_string(),
                status,
                start_time: started,
                end_time: Some(started + Duration::milliseconds(latency_ms)),
                model: None,
                cost: Some(Money::usd_micros(cost_micros)),
                tokens: Some(TokenCounts::default()),
                input_ref: None,
                output_ref: None,
                attributes: BTreeMap::new(),
                unmapped_attrs: serde_json::json!({}),
                raw_ref: ArtifactRef {
                    artifact_id: beater_core::ArtifactId::new("raw")
                        .unwrap_or_else(|err| panic!("{err}")),
                    uri: "artifact://tenant/project/raw".to_string(),
                    sha256: beater_core::Sha256Hash::new("ab".repeat(32))
                        .unwrap_or_else(|err| panic!("{err}")),
                    size_bytes: 2,
                    mime_type: "application/json".to_string(),
                    redaction_class: RedactionClass::Internal,
                },
            }],
        }
    }
}
