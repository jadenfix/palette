//! Schema-v2 **reclassification** of stored canonical spans.
//!
//! IMPORTANT — scope of this module: this is *not* a re-normalization from the
//! original raw bytes. It does **not** read `RawEnvelope.body_ref`,
//! `RawEnvelope.source`, or any raw payload to re-derive spans. It operates
//! purely on an existing [`CanonicalSpan`]'s `attributes` bag, reclassifying
//! each key as canonical or non-canonical, and is therefore a lossless, in-place
//! migration of the v1 projection — not a fresh projection from source. The
//! preserved raw payload that *would* enable a true re-normalization from bytes
//! is intentionally out of scope here.
//!
//! This module introduces a second canonical schema version,
//! [`CANONICAL_SCHEMA_VERSION_V2`], and a pure migration from v1 spans to v2
//! spans. It is deliberately self-contained and side-effect free so it can be
//! golden-tested against a frozen fixture and run as a batch backfill.
//!
//! The supplied [`RawEnvelope`] is used solely as an ownership check: it lets
//! [`reproject_span`] refuse to migrate a span across an envelope boundary
//! (mismatched tenant/project/environment). Its raw bytes are never read.
//!
//! ## What v2 changes (the reason a migration exists)
//!
//! v1 carried the *entire* attribute bag on `CanonicalSpan.attributes`, including
//! attributes that fail canonical mapping (see ingest `canonical_mapping`). v2
//! reclassifies that bag: `attributes` retains only canonical keys, and any
//! non-canonical attribute is relocated into `unmapped_attrs.unmapped` (merging
//! with whatever the original normalizer already recorded there). After
//! reprojection, `attributes` is a clean canonical view while the raw artifact +
//! `unmapped_attrs` remain the lossless record. No new keys are derived from the
//! raw payload; v2 only re-partitions keys that already exist on the v1 span.

use beater_schema::{CanonicalSpan, RawEnvelope};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// Second canonical schema version. Defined locally (not in `beater-schema`) so
/// the reprojection can migrate to it without changing the published contract's
/// `CANONICAL_SCHEMA_VERSION`. A v1 span reprojected through [`reproject_span`]
/// or [`reproject_envelope`] is stamped with this version.
pub const CANONICAL_SCHEMA_VERSION_V2: u32 = 2;

/// Normalizer revision stamped on reprojected spans, so a v2 span is
/// distinguishable from one produced by the original ingest normalizer.
pub const REPROJECTION_NORMALIZER_VERSION: &str = "beater-reproject-v2";

/// Outcome of reprojecting one canonical span.
#[derive(Clone, Debug, PartialEq)]
pub struct ReprojectedSpan {
    /// The migrated span (schema_version == [`CANONICAL_SCHEMA_VERSION_V2`]).
    pub span: CanonicalSpan,
    /// Whether the reprojection actually changed anything (false when the span
    /// was already at v2, making the operation idempotent).
    pub migrated: bool,
    /// Attribute keys relocated from `attributes` into `unmapped_attrs.unmapped`.
    pub relocated_keys: Vec<String>,
}

/// Recognized namespace prefixes whose attributes the canonical model
/// understands. Must stay identical to ingest's
/// `beater_ingest::canonical_mapping::CANONICAL_PREFIXES` (the duplication is
/// asserted by [`tests::canonical_prefixes_match_ingest`]).
pub const CANONICAL_PREFIXES: &[&str] = &[
    "llm.",
    "gen_ai.",
    "browser.",
    "resource.",
    "otel.",
    "beater.",
    "agent.",
    "openinference.",
    "w3c.",
    "temporal.",
    "langfuse.",
];

/// Exact keys without a recognized prefix that are still canonical. Must stay
/// identical to ingest's `beater_ingest::canonical_mapping::CANONICAL_EXACT_KEYS`
/// (asserted by [`tests::canonical_exact_keys_match_ingest`]).
pub const CANONICAL_EXACT_KEYS: &[&str] = &[
    "input.value",
    "output.value",
    "model",
    "model_name",
    "provider",
    "cost_micros",
    "currency",
    "input_tokens",
    "output_tokens",
    "reasoning_tokens",
    "cache_read_tokens",
];

/// Classify whether an attribute key maps to the canonical model. Mirrors the
/// ingest-side `canonical_mapping` rules so reprojection and first-pass ingest
/// agree on what "canonical" means.
fn maps_to_canonical(key: &str) -> bool {
    if CANONICAL_EXACT_KEYS.contains(&key) {
        return true;
    }
    CANONICAL_PREFIXES
        .iter()
        .any(|prefix| key.starts_with(prefix))
}

/// Reproject a single canonical span to schema v2 by **reclassifying** its
/// existing `attributes` bag: canonical keys stay on `attributes`, non-canonical
/// keys are relocated into `unmapped_attrs.unmapped`. Pure and idempotent: a span
/// already at v2 is returned unchanged with `migrated == false`.
///
/// This does not read `raw`'s bytes and does not re-derive any attribute from
/// source. `raw` is the preserved envelope the span was projected from, used only
/// as an ownership check: the function asserts the span belongs to it (same
/// tenant/project/environment) and refuses to reproject across an envelope
/// boundary.
pub fn reproject_span(
    raw: &RawEnvelope,
    span: &CanonicalSpan,
) -> Result<ReprojectedSpan, ReprojectError> {
    if span.tenant_id != raw.tenant_id
        || span.project_id != raw.project_id
        || span.environment_id != raw.environment_id
    {
        return Err(ReprojectError::EnvelopeBoundary);
    }
    if span.schema_version >= CANONICAL_SCHEMA_VERSION_V2 {
        return Ok(ReprojectedSpan {
            span: span.clone(),
            migrated: false,
            relocated_keys: Vec::new(),
        });
    }

    let mut canonical_attrs = BTreeMap::new();
    let mut relocated: Map<String, Value> = Map::new();
    let mut relocated_keys = Vec::new();
    for (key, value) in &span.attributes {
        if maps_to_canonical(key) {
            canonical_attrs.insert(key.clone(), value.clone());
        } else {
            relocated.insert(key.clone(), value.clone());
            relocated_keys.push(key.clone());
        }
    }

    // Merge relocated attributes into the existing `unmapped_attrs.unmapped`
    // bag, preserving any keys the original normalizer already recorded.
    let mut unmapped_attrs = match span.unmapped_attrs.clone() {
        Value::Object(map) => map,
        _ => Map::new(),
    };
    let mut merged_unmapped = match unmapped_attrs.remove("unmapped") {
        Some(Value::Object(map)) => map,
        _ => Map::new(),
    };
    for (key, value) in relocated {
        merged_unmapped.insert(key, value);
    }
    unmapped_attrs.insert("unmapped".to_string(), Value::Object(merged_unmapped));

    let mut migrated_span = span.clone();
    migrated_span.schema_version = CANONICAL_SCHEMA_VERSION_V2;
    migrated_span.normalizer_version = REPROJECTION_NORMALIZER_VERSION.to_string();
    migrated_span.attributes = canonical_attrs;
    migrated_span.unmapped_attrs = Value::Object(unmapped_attrs);

    Ok(ReprojectedSpan {
        span: migrated_span,
        migrated: true,
        relocated_keys,
    })
}

/// Reproject every span projected from one raw envelope. Spans whose
/// tenant/project/environment do not match the envelope are rejected.
pub fn reproject_envelope(
    raw: &RawEnvelope,
    spans: &[CanonicalSpan],
) -> Result<Vec<ReprojectedSpan>, ReprojectError> {
    spans.iter().map(|span| reproject_span(raw, span)).collect()
}

/// Errors raised by reprojection.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ReprojectError {
    #[error(
        "span does not belong to the supplied raw envelope (tenant/project/environment mismatch)"
    )]
    EnvelopeBoundary,
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_core::{
        ArtifactId, EnvironmentId, IdempotencyKey, ProjectId, Sha256Hash, SpanId, TenantId,
    };
    use beater_schema::{
        ArtifactRef, AuthContext, RedactionClass, SourceDialect, CANONICAL_SCHEMA_VERSION,
        RAW_SCHEMA_VERSION,
    };
    use chrono::Utc;
    use serde_json::json;
    use std::collections::BTreeSet;

    /// Frozen fixture: a v1 canonical span exactly as the original normalizer
    /// would have stored it, parsed from a checked-in JSON literal so the golden
    /// assertions below pin the migration's output byte-for-byte.
    const FROZEN_V1_SPAN_JSON: &str = include_str!("../fixtures/reproject_v1_span.json");

    fn fixture_raw() -> RawEnvelope {
        RawEnvelope {
            schema_version: RAW_SCHEMA_VERSION,
            tenant_id: TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            project_id: ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            environment_id: EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
            source: SourceDialect::Otlp,
            source_schema_url: Some("https://opentelemetry.io/schemas/1.37.0".to_string()),
            source_schema_version: Some("1.37.0".to_string()),
            received_at: Utc::now(),
            idempotency_key: IdempotencyKey::new("raw-key").unwrap_or_else(|err| panic!("{err}")),
            payload_hash: Sha256Hash::new("hash").unwrap_or_else(|err| panic!("{err}")),
            body_ref: fixture_artifact(),
            auth_context: AuthContext {
                api_key_id: None,
                scopes: BTreeSet::new(),
            },
        }
    }

    fn fixture_artifact() -> ArtifactRef {
        ArtifactRef {
            artifact_id: ArtifactId::new("artifact").unwrap_or_else(|err| panic!("{err}")),
            uri: "artifact://tenant/project/artifact".to_string(),
            sha256: Sha256Hash::new("hash").unwrap_or_else(|err| panic!("{err}")),
            size_bytes: 0,
            mime_type: "application/x-protobuf".to_string(),
            redaction_class: RedactionClass::Internal,
        }
    }

    fn frozen_v1_span() -> CanonicalSpan {
        serde_json::from_str(FROZEN_V1_SPAN_JSON)
            .unwrap_or_else(|err| panic!("frozen fixture must parse: {err}"))
    }

    #[test]
    fn reprojects_frozen_v1_span_to_v2_relocating_non_canonical_attrs() {
        let raw = fixture_raw();
        let v1 = frozen_v1_span();
        // Sanity: the fixture is genuinely a v1 span carrying mixed attributes.
        assert_eq!(v1.schema_version, CANONICAL_SCHEMA_VERSION);
        assert!(v1.attributes.contains_key("vendor.custom_signal"));
        assert!(v1.attributes.contains_key("llm.model_name"));

        let result = reproject_span(&raw, &v1).unwrap_or_else(|err| panic!("{err}"));
        assert!(result.migrated);
        assert_eq!(result.span.schema_version, CANONICAL_SCHEMA_VERSION_V2);
        assert_eq!(
            result.span.normalizer_version,
            REPROJECTION_NORMALIZER_VERSION
        );

        // Canonical keys survive on `attributes`; non-canonical keys are gone.
        assert!(result.span.attributes.contains_key("llm.model_name"));
        assert!(result
            .span
            .attributes
            .contains_key("openinference.span.kind"));
        assert!(!result.span.attributes.contains_key("vendor.custom_signal"));
        assert!(!result.span.attributes.contains_key("user.session"));

        // Golden: the relocated keys, and the merged `unmapped` bag, are exact.
        let mut relocated = result.relocated_keys.clone();
        relocated.sort();
        assert_eq!(relocated, vec!["user.session", "vendor.custom_signal"]);
        assert_eq!(
            result.span.unmapped_attrs,
            json!({
                "dropped_attributes": {},
                "unmapped": {
                    "vendor.custom_signal": "keep-me",
                    "user.session": 42,
                    "preexisting.unmapped": "was-here-in-v1",
                },
            })
        );
    }

    #[test]
    fn reprojection_is_idempotent() {
        let raw = fixture_raw();
        let v1 = frozen_v1_span();
        let once = reproject_span(&raw, &v1).unwrap_or_else(|err| panic!("{err}"));
        let twice = reproject_span(&raw, &once.span).unwrap_or_else(|err| panic!("{err}"));
        assert!(!twice.migrated, "second reprojection must be a no-op");
        assert!(twice.relocated_keys.is_empty());
        assert_eq!(once.span, twice.span);
    }

    #[test]
    fn reprojection_rejects_envelope_boundary_crossing() {
        let mut raw = fixture_raw();
        raw.tenant_id = TenantId::new("other-tenant").unwrap_or_else(|err| panic!("{err}"));
        let v1 = frozen_v1_span();
        assert_eq!(
            reproject_span(&raw, &v1),
            Err(ReprojectError::EnvelopeBoundary)
        );
    }

    /// The canonical-classification tables here are duplicated verbatim from
    /// ingest (`beater_ingest::canonical_mapping`) with no shared dependency.
    /// These two tests pin that duplication: if either crate's set drifts, the
    /// reprojection and first-pass ingest would disagree on what is "canonical",
    /// so this fails loudly until they are reconciled (or hoisted into a shared
    /// crate). Kept as an equality assertion per the review's lower-risk option.
    #[test]
    fn canonical_prefixes_match_ingest() {
        use beater_ingest::canonical_mapping;
        assert_eq!(
            CANONICAL_PREFIXES,
            canonical_mapping::CANONICAL_PREFIXES,
            "replay and ingest CANONICAL_PREFIXES must stay identical"
        );
    }

    #[test]
    fn canonical_exact_keys_match_ingest() {
        use beater_ingest::canonical_mapping;
        assert_eq!(
            CANONICAL_EXACT_KEYS,
            canonical_mapping::CANONICAL_EXACT_KEYS,
            "replay and ingest CANONICAL_EXACT_KEYS must stay identical"
        );
    }

    /// Belt-and-suspenders: classification agrees key-by-key across a mix of
    /// canonical and non-canonical keys, so the two `maps_to_canonical`
    /// implementations can't disagree on behavior even if the tables matched.
    #[test]
    fn maps_to_canonical_agrees_with_ingest() {
        use beater_ingest::canonical_mapping;
        for key in [
            "llm.model_name",
            "openinference.span.kind",
            "langfuse.observation.type",
            "model",
            "cost_micros",
            "vendor.custom_signal",
            "user.session",
            "random.key",
        ] {
            assert_eq!(
                maps_to_canonical(key),
                canonical_mapping::maps_to_canonical(key),
                "classification of {key:?} must match ingest"
            );
        }
    }

    #[test]
    fn reproject_envelope_migrates_each_span() {
        let raw = fixture_raw();
        let v1 = frozen_v1_span();
        let mut second = v1.clone();
        second.span_id = SpanId::new("span-2").unwrap_or_else(|err| panic!("{err}"));
        second.seq = 2;
        let results = reproject_envelope(&raw, &[v1, second]).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.migrated));
        assert!(results
            .iter()
            .all(|r| r.span.schema_version == CANONICAL_SCHEMA_VERSION_V2));
    }
}
