use beater_bus::{BusError, BusMessage, DeadLetter, DurableBus, PublishAck};
use beater_core::{
    sha256_hex, Clock, EnvironmentId, IdempotencyKey, ProjectId, Sha256Hash, SpanId, SystemClock,
    TenantId, TenantScope, Timestamp, TokenCounts, TraceId,
};
use beater_schema::{
    make_idempotency_key, AgentSpanKind, ArtifactRef, AuthContext, CanonicalAttrs, CanonicalSpan,
    CanonicalTraceBatch, ModelRef, RawEnvelope, RedactionClass, SourceDialect, SpanStatus,
    TraceCompletionState, WriteAck, CANONICAL_SCHEMA_VERSION, RAW_SCHEMA_VERSION,
};
use beater_store::{ArtifactStore, QuotaLimiter, QuotaReservationRequest, StoreError, TraceStore};
use beater_store_memory::InMemoryQuotaLimiter;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::sync::Arc;

pub const TRACE_INGESTED_KIND: &str = "trace.ingested";
pub const TRACE_WRITE_BATCH_KIND: &str = "trace.write_batch";

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("quota exceeded for tenant={tenant_id} project={project_id}; limit={limit}")]
    QuotaExceeded {
        tenant_id: String,
        project_id: String,
        limit: u64,
        used: u64,
        reset_at: Timestamp,
    },
    #[error("too many attributes: {count} > {limit}")]
    TooManyAttributes { count: usize, limit: usize },
    #[error("payload too large: {size_bytes} > {limit_bytes}")]
    PayloadTooLarge {
        size_bytes: usize,
        limit_bytes: usize,
    },
    #[error("ingest bus is at capacity {capacity}")]
    Backpressure { capacity: usize },
    #[error("ingest resource not found: {0}")]
    NotFound(String),
    #[error(transparent)]
    Import(#[from] ImportError),
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Error raised by a [`SourceImporter`] when it cannot normalize a payload.
#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("source '{source_name}' rejected payload: {message}")]
    Invalid {
        source_name: String,
        message: String,
    },
}

/// Pluggable normalization seam: maps a source-specific payload into a
/// [`RawTraceIngestRequest`], which then flows through the *same* downstream ingest
/// pipeline as every other source (OTLP, native). Implementations are pure and must
/// not perform IO. Register them with [`IngestService::with_importer`] and dispatch
/// by `source` via [`IngestService::import_source`].
pub trait SourceImporter: Send + Sync {
    /// The wire identifier selected by the unified import endpoint (e.g. `"temporal_history"`).
    fn source(&self) -> &'static str;

    /// Normalize raw bytes into a ready-to-ingest request.
    fn normalize(
        &self,
        scope: &TenantScope,
        raw_bytes: &[u8],
        auth: Option<AuthContext>,
    ) -> Result<RawTraceIngestRequest, ImportError>;
}

/// Built-in importer for native canonical span drafts: accepts a JSON body of the
/// shape `{ "spans": [CanonicalSpanDraft, ...] }`. Registered on every
/// [`IngestService`] so the unified import endpoint serves the native case with no
/// external crate.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeSpansImporter;

#[derive(Deserialize)]
struct NativeSpansPayload {
    #[serde(default)]
    spans: Vec<CanonicalSpanDraft>,
}

impl SourceImporter for NativeSpansImporter {
    fn source(&self) -> &'static str {
        "native"
    }

    fn normalize(
        &self,
        scope: &TenantScope,
        raw_bytes: &[u8],
        auth: Option<AuthContext>,
    ) -> Result<RawTraceIngestRequest, ImportError> {
        let payload: NativeSpansPayload =
            serde_json::from_slice(raw_bytes).map_err(|err| ImportError::Invalid {
                source_name: "native".to_string(),
                message: err.to_string(),
            })?;
        Ok(RawTraceIngestRequest {
            scope: scope.clone(),
            source: SourceDialect::Native,
            source_schema_url: Some("beater://native/v1".to_string()),
            source_schema_version: Some("1".to_string()),
            normalizer_version: "beater-native-import-v1".to_string(),
            mime_type: "application/json".to_string(),
            redaction_class: RedactionClass::Sensitive,
            raw_bytes: raw_bytes.to_vec(),
            raw_idempotency_key: None,
            auth_context: auth,
            spans: payload.spans,
        })
    }
}

#[derive(Clone)]
pub struct IngestService {
    artifacts: Arc<dyn ArtifactStore>,
    traces: Arc<dyn TraceStore>,
    bus: Arc<dyn DurableBus>,
    policy: IngestPolicy,
    quota_limiter: Arc<dyn QuotaLimiter>,
    clock: Arc<dyn Clock>,
    importers: Arc<BTreeMap<&'static str, Arc<dyn SourceImporter>>>,
}

impl IngestService {
    pub fn new(
        artifacts: Arc<dyn ArtifactStore>,
        traces: Arc<dyn TraceStore>,
        bus: Arc<dyn DurableBus>,
        policy: IngestPolicy,
    ) -> Self {
        let mut importers: BTreeMap<&'static str, Arc<dyn SourceImporter>> = BTreeMap::new();
        let native = NativeSpansImporter;
        importers.insert(native.source(), Arc::new(native));
        Self {
            artifacts,
            traces,
            bus,
            policy,
            quota_limiter: Arc::new(InMemoryQuotaLimiter::new()),
            clock: Arc::new(SystemClock),
            importers: Arc::new(importers),
        }
    }

    pub fn with_quota_limiter(mut self, quota_limiter: Arc<dyn QuotaLimiter>) -> Self {
        self.quota_limiter = quota_limiter;
        self
    }

    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// Register a [`SourceImporter`], keyed by its `source()` identifier. Replaces any
    /// existing importer with the same key.
    pub fn with_importer(mut self, importer: Arc<dyn SourceImporter>) -> Self {
        let mut importers = (*self.importers).clone();
        importers.insert(importer.source(), importer);
        self.importers = Arc::new(importers);
        self
    }

    /// Source identifiers currently registered (sorted), for diagnostics/discovery.
    pub fn registered_import_sources(&self) -> Vec<&'static str> {
        self.importers.keys().copied().collect()
    }

    /// Normalize a payload from the named `source` and ingest it through the shared
    /// pipeline. `buffered` selects durable buffering vs synchronous write, matching
    /// the OTLP and native ingest paths.
    pub async fn import_source(
        &self,
        source: &str,
        scope: TenantScope,
        raw_bytes: Vec<u8>,
        auth: Option<AuthContext>,
        buffered: bool,
    ) -> Result<IngestOutcome, IngestError> {
        let importer = self.importers.get(source).ok_or_else(|| {
            IngestError::NotFound(format!(
                "no importer registered for source '{source}'; registered: [{}]",
                self.importers
                    .keys()
                    .copied()
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })?;
        let request = importer.normalize(&scope, &raw_bytes, auth)?;
        if buffered {
            self.buffer_raw_trace_batch(request).await
        } else {
            self.ingest_raw_trace_batch(request).await
        }
    }

    pub async fn ingest_native(
        &self,
        request: NativeIngestRequest,
    ) -> Result<IngestOutcome, IngestError> {
        self.enforce_quota_events(&request.scope, 1).await?;
        let prepared = self.prepare_native_batch(request).await?;
        self.write_batch_and_queue_downstream(prepared).await
    }

    pub async fn buffer_native(
        &self,
        request: NativeIngestRequest,
    ) -> Result<IngestOutcome, IngestError> {
        self.enforce_quota_events(&request.scope, 1).await?;
        let prepared = self.prepare_native_batch(request).await?;
        let publish = self.publish_trace_write(&prepared).await?;
        Ok(IngestOutcome {
            ack: batch_publish_ack(&prepared.batch, &publish),
            downstream_queued: publish.accepted,
        })
    }

    pub async fn ingest_raw_trace_batch(
        &self,
        request: RawTraceIngestRequest,
    ) -> Result<IngestOutcome, IngestError> {
        let event_count = request.spans.len() as u64;
        self.enforce_quota_events(&request.scope, event_count)
            .await?;
        let prepared = self.prepare_raw_batch(request).await?;
        self.write_batch_and_queue_downstream(prepared).await
    }

    async fn write_batch_and_queue_downstream(
        &self,
        prepared: PreparedTraceBatch,
    ) -> Result<IngestOutcome, IngestError> {
        let ack = self
            .traces
            .write_batch(prepared.batch.clone())
            .await
            .map_err(IngestError::Store)?;
        let downstream_queued = match self
            .publish_trace_ingested(
                &prepared.tenant_id,
                &prepared.project_id,
                &prepared.queue_key,
                &prepared.trace_ids,
            )
            .await
        {
            Ok(report) => report.queued(),
            Err(_) => {
                // The trace is already durable; queue a write retry so the worker can
                // idempotently re-write and publish downstream. If that fallback
                // cannot be durably queued, surface the error instead of returning a
                // misleading success with vanished downstream work.
                let publish = self.publish_trace_write(&prepared).await?;
                publish.accepted || publish.duplicate
            }
        };

        Ok(IngestOutcome {
            ack,
            downstream_queued,
        })
    }

    pub async fn buffer_raw_trace_batch(
        &self,
        request: RawTraceIngestRequest,
    ) -> Result<IngestOutcome, IngestError> {
        let event_count = request.spans.len() as u64;
        self.enforce_quota_events(&request.scope, event_count)
            .await?;
        let prepared = self.prepare_raw_batch(request).await?;
        let publish = self.publish_trace_write(&prepared).await?;
        Ok(IngestOutcome {
            ack: batch_publish_ack(&prepared.batch, &publish),
            downstream_queued: publish.accepted,
        })
    }

    pub async fn drain_trace_writes(
        &self,
        limit: usize,
    ) -> Result<TraceWriteDrainReport, IngestError> {
        self.drain_trace_writes_with_hook(limit, |_| async { Ok(()) })
            .await
    }

    pub async fn drain_trace_writes_with_hook<F, Fut>(
        &self,
        limit: usize,
        before_write: F,
    ) -> Result<TraceWriteDrainReport, IngestError>
    where
        F: FnMut(QueuedTraceWrite) -> Fut,
        Fut: Future<Output = Result<(), String>>,
    {
        let messages = self
            .bus
            .consume_kind_batch(TRACE_WRITE_BATCH_KIND, limit)
            .await
            .map_err(map_bus_error)?;
        self.drain_trace_write_messages(messages, before_write)
            .await
    }

    pub async fn drain_trace_writes_for(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        limit: usize,
    ) -> Result<TraceWriteDrainReport, IngestError> {
        let messages = self
            .bus
            .consume_scoped_kind_batch(tenant_id, project_id, TRACE_WRITE_BATCH_KIND, limit)
            .await
            .map_err(map_bus_error)?;
        self.drain_trace_write_messages(messages, |_| async { Ok(()) })
            .await
    }

    pub async fn drain_trace_ingested<F, Fut>(
        &self,
        limit: usize,
        process: F,
    ) -> Result<TraceIngestedDrainReport, IngestError>
    where
        F: FnMut(QueuedTraceWork) -> Fut,
        Fut: Future<Output = Result<(), String>>,
    {
        let messages = self
            .bus
            .consume_kind_batch(TRACE_INGESTED_KIND, limit)
            .await
            .map_err(map_bus_error)?;
        self.drain_trace_ingested_messages(messages, process).await
    }

    pub async fn drain_trace_ingested_for<F, Fut>(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        limit: usize,
        process: F,
    ) -> Result<TraceIngestedDrainReport, IngestError>
    where
        F: FnMut(QueuedTraceWork) -> Fut,
        Fut: Future<Output = Result<(), String>>,
    {
        let messages = self
            .bus
            .consume_scoped_kind_batch(tenant_id, project_id, TRACE_INGESTED_KIND, limit)
            .await
            .map_err(map_bus_error)?;
        self.drain_trace_ingested_messages(messages, process).await
    }

    async fn drain_trace_write_messages<F, Fut>(
        &self,
        messages: Vec<BusMessage>,
        mut before_write: F,
    ) -> Result<TraceWriteDrainReport, IngestError>
    where
        F: FnMut(QueuedTraceWrite) -> Fut,
        Fut: Future<Output = Result<(), String>>,
    {
        let mut report = TraceWriteDrainReport {
            consumed: messages.len(),
            ..TraceWriteDrainReport::default()
        };
        for message in messages {
            self.process_trace_write_message(message, &mut report, &mut before_write)
                .await?;
        }
        report.trace_ids.sort();
        report.trace_ids.dedup();
        report.trace_refs.sort();
        report.trace_refs.dedup();
        Ok(report)
    }

    async fn drain_trace_ingested_messages<F, Fut>(
        &self,
        messages: Vec<BusMessage>,
        mut process: F,
    ) -> Result<TraceIngestedDrainReport, IngestError>
    where
        F: FnMut(QueuedTraceWork) -> Fut,
        Fut: Future<Output = Result<(), String>>,
    {
        let mut report = TraceIngestedDrainReport {
            consumed: messages.len(),
            ..TraceIngestedDrainReport::default()
        };
        for message in messages {
            let will_dead_letter = message.attempts.saturating_add(1) >= message.max_attempts;
            let queued = match serde_json::from_slice::<QueuedTraceWork>(&message.payload) {
                Ok(queued) => queued,
                Err(err) => {
                    report.invalid_messages += 1;
                    self.retry_or_dlq_counted(
                        message,
                        format!("invalid trace.ingested payload: {err}"),
                        will_dead_letter,
                        &mut report,
                    )
                    .await?;
                    continue;
                }
            };
            if queued.tenant_id != message.tenant_id || queued.project_id != message.project_id {
                report.invalid_messages += 1;
                let reason = format!(
                    "trace.ingested scope mismatch: message={}/{} payload={}/{}",
                    message.tenant_id, message.project_id, queued.tenant_id, queued.project_id
                );
                self.retry_or_dlq_counted(message, reason, will_dead_letter, &mut report)
                    .await?;
                continue;
            }

            match process(queued.clone()).await {
                Ok(()) => {
                    report.completed += 1;
                    report.trace_refs.push(queued);
                    self.bus.ack(message).await.map_err(map_bus_error)?;
                }
                Err(reason) => {
                    report.failed_work += 1;
                    self.retry_or_dlq_counted(message, reason, will_dead_letter, &mut report)
                        .await?;
                }
            }
        }
        report.trace_refs.sort();
        report.trace_refs.dedup();
        Ok(report)
    }

    pub async fn queue_status(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> Result<IngestQueueStatus, IngestError> {
        let total_depth = self.bus.depth().await.map_err(map_bus_error)?;
        let trace_write_depth = self
            .bus
            .depth_for_kind(TRACE_WRITE_BATCH_KIND)
            .await
            .map_err(map_bus_error)?;
        let trace_ingested_depth = self
            .bus
            .depth_for_kind(TRACE_INGESTED_KIND)
            .await
            .map_err(map_bus_error)?;
        let dead_letters = self
            .bus
            .dlq()
            .await
            .map_err(map_bus_error)?
            .into_iter()
            .filter(|dead_letter| {
                dead_letter.message.tenant_id == tenant_id
                    && dead_letter.message.project_id == project_id
            })
            .collect();
        Ok(IngestQueueStatus {
            tenant_id,
            project_id,
            total_depth,
            trace_write_depth,
            trace_ingested_depth,
            dead_letters,
        })
    }

    pub async fn replay_dead_letter(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        message_id: &str,
        reset_attempts: bool,
    ) -> Result<DeadLetterReplayReport, IngestError> {
        let ack = self
            .bus
            .replay_dead_letter(tenant_id, project_id, message_id, reset_attempts)
            .await
            .map_err(map_bus_error)?;
        Ok(DeadLetterReplayReport {
            tenant_id: tenant_id.clone(),
            project_id: project_id.clone(),
            message_id: message_id.to_string(),
            reset_attempts,
            ack,
        })
    }

    pub async fn reconcile_trace_ingested(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        trace_id: TraceId,
    ) -> Result<TraceIngestedReconcileReport, IngestError> {
        let trace = self
            .traces
            .get_project_trace(tenant_id.clone(), project_id.clone(), trace_id.clone())
            .await
            .map_err(IngestError::Store)?;
        if trace.spans.is_empty() {
            return Err(IngestError::NotFound(format!(
                "trace {} not found for tenant={} project={}",
                trace_id.as_str(),
                tenant_id.as_str(),
                project_id.as_str()
            )));
        }
        let queue_key = IdempotencyKey::new(format!(
            "reconcile:{}:{}:{}",
            tenant_id.as_str(),
            project_id.as_str(),
            trace_id.as_str()
        ))
        .map_err(anyhow::Error::from)?;
        let publish = self
            .publish_trace_ingested(
                &tenant_id,
                &project_id,
                &queue_key,
                &BTreeSet::from([trace_id.clone()]),
            )
            .await?;
        Ok(TraceIngestedReconcileReport {
            tenant_id,
            project_id,
            trace_id,
            span_count: trace.spans.len(),
            downstream_accepted: publish.accepted,
            downstream_duplicate: publish.duplicate,
            downstream_queued: publish.queued(),
        })
    }

    async fn prepare_native_batch(
        &self,
        request: NativeIngestRequest,
    ) -> Result<PreparedTraceBatch, IngestError> {
        let raw_bytes = serde_json::to_vec(&request).map_err(anyhow::Error::from)?;
        if raw_bytes.len() > self.policy.max_raw_payload_bytes {
            return Err(IngestError::PayloadTooLarge {
                size_bytes: raw_bytes.len(),
                limit_bytes: self.policy.max_raw_payload_bytes,
            });
        }

        let raw_ref = self
            .artifacts
            .put_bytes(
                &request.scope.tenant_id,
                &request.scope.project_id,
                "application/json",
                request.redaction_class.clone(),
                &raw_bytes,
            )
            .await
            .map_err(IngestError::Store)?;
        let payload_hash = Sha256Hash::new(sha256_hex(&raw_bytes)).map_err(anyhow::Error::from)?;
        let idempotency_key = request
            .idempotency_key
            .clone()
            .map(Ok)
            .unwrap_or_else(|| {
                make_idempotency_key(
                    &request.scope,
                    &request.trace_id,
                    &request.span_id,
                    request.seq,
                    &payload_hash,
                )
            })
            .map_err(anyhow::Error::from)?;

        let raw = RawEnvelope {
            schema_version: RAW_SCHEMA_VERSION,
            tenant_id: request.scope.tenant_id.clone(),
            project_id: request.scope.project_id.clone(),
            environment_id: request.scope.environment_id.clone(),
            source: SourceDialect::Native,
            source_schema_url: Some("beater://native/v1".to_string()),
            source_schema_version: Some("1".to_string()),
            received_at: self.clock.now(),
            idempotency_key: idempotency_key.clone(),
            payload_hash,
            body_ref: raw_ref.clone(),
            auth_context: request
                .auth_context
                .clone()
                .unwrap_or_else(anonymous_auth_context),
        };

        let (attributes, unmapped_attrs) = self.govern_attributes(request.attributes)?;
        let input_ref = self
            .maybe_payload_artifact(
                &request.scope,
                request.input.as_ref(),
                &request.redaction_class,
                "application/json",
            )
            .await?;
        let output_ref = self
            .maybe_payload_artifact(
                &request.scope,
                request.output.as_ref(),
                &request.redaction_class,
                "application/json",
            )
            .await?;

        let mut canonical_attrs = attributes;
        if input_ref.is_none() {
            if let Some(input) = request.input.clone() {
                canonical_attrs.insert("input.value".to_string(), input);
            }
        }
        if output_ref.is_none() {
            if let Some(output) = request.output.clone() {
                canonical_attrs.insert("output.value".to_string(), output);
            }
        }

        let span = CanonicalSpan {
            schema_version: CANONICAL_SCHEMA_VERSION,
            normalizer_version: "beater-native-v1".to_string(),
            tenant_id: request.scope.tenant_id.clone(),
            project_id: request.scope.project_id.clone(),
            environment_id: request.scope.environment_id.clone(),
            trace_id: request.trace_id.clone(),
            span_id: request.span_id,
            parent_span_id: request.parent_span_id,
            seq: request.seq,
            kind: request.kind,
            name: request.name,
            status: request.status,
            start_time: request.start_time.unwrap_or_else(|| self.clock.now()),
            end_time: request.end_time,
            model: request.model,
            cost: request.cost,
            tokens: request.tokens,
            input_ref,
            output_ref,
            attributes: canonical_attrs,
            unmapped_attrs,
            raw_ref,
        };
        let trace_ids = BTreeSet::from([span.trace_id.clone()]);
        Ok(PreparedTraceBatch {
            tenant_id: request.scope.tenant_id,
            project_id: request.scope.project_id,
            queue_key: idempotency_key,
            trace_ids,
            batch: CanonicalTraceBatch::one(raw, span),
        })
    }

    async fn prepare_raw_batch(
        &self,
        request: RawTraceIngestRequest,
    ) -> Result<PreparedTraceBatch, IngestError> {
        let scope = request.scope.clone();
        if request.raw_bytes.len() > self.policy.max_raw_payload_bytes {
            return Err(IngestError::PayloadTooLarge {
                size_bytes: request.raw_bytes.len(),
                limit_bytes: self.policy.max_raw_payload_bytes,
            });
        }

        let raw_ref = self
            .artifacts
            .put_bytes(
                &scope.tenant_id,
                &scope.project_id,
                &request.mime_type,
                request.redaction_class.clone(),
                &request.raw_bytes,
            )
            .await
            .map_err(IngestError::Store)?;
        let payload_hash =
            Sha256Hash::new(sha256_hex(&request.raw_bytes)).map_err(anyhow::Error::from)?;
        let raw_idempotency_key = request
            .raw_idempotency_key
            .clone()
            .map(Ok)
            .unwrap_or_else(|| {
                IdempotencyKey::new(format!(
                    "raw:{}:{}:{}:{}",
                    request.source.as_str(),
                    scope.tenant_id.as_str(),
                    scope.project_id.as_str(),
                    payload_hash.as_str()
                ))
            })
            .map_err(anyhow::Error::from)?;

        let raw = RawEnvelope {
            schema_version: RAW_SCHEMA_VERSION,
            tenant_id: scope.tenant_id.clone(),
            project_id: scope.project_id.clone(),
            environment_id: scope.environment_id.clone(),
            source: request.source,
            source_schema_url: request.source_schema_url,
            source_schema_version: request.source_schema_version,
            received_at: self.clock.now(),
            idempotency_key: raw_idempotency_key.clone(),
            payload_hash,
            body_ref: raw_ref.clone(),
            auth_context: request.auth_context.unwrap_or_else(anonymous_auth_context),
        };

        let mut trace_ids = BTreeSet::new();
        let mut spans = Vec::with_capacity(request.spans.len());
        for draft in request.spans {
            let (attributes, unmapped_attrs) = self.govern_attributes(draft.attributes)?;
            let input_ref = self
                .maybe_payload_artifact(
                    &scope,
                    draft.input.as_ref(),
                    &request.redaction_class,
                    "application/json",
                )
                .await?;
            let output_ref = self
                .maybe_payload_artifact(
                    &scope,
                    draft.output.as_ref(),
                    &request.redaction_class,
                    "application/json",
                )
                .await?;
            let mut canonical_attrs = attributes;
            if input_ref.is_none() {
                if let Some(input) = draft.input.clone() {
                    canonical_attrs.insert("input.value".to_string(), input);
                }
            }
            if output_ref.is_none() {
                if let Some(output) = draft.output.clone() {
                    canonical_attrs.insert("output.value".to_string(), output);
                }
            }

            let span = CanonicalSpan {
                schema_version: CANONICAL_SCHEMA_VERSION,
                normalizer_version: request.normalizer_version.clone(),
                tenant_id: scope.tenant_id.clone(),
                project_id: scope.project_id.clone(),
                environment_id: scope.environment_id.clone(),
                trace_id: draft.trace_id,
                span_id: draft.span_id,
                parent_span_id: draft.parent_span_id,
                seq: draft.seq,
                kind: draft.kind,
                name: draft.name,
                status: draft.status,
                start_time: draft.start_time.unwrap_or_else(|| self.clock.now()),
                end_time: draft.end_time,
                model: draft.model,
                cost: draft.cost,
                tokens: draft.tokens,
                input_ref,
                output_ref,
                attributes: canonical_attrs,
                unmapped_attrs,
                raw_ref: raw_ref.clone(),
            };
            trace_ids.insert(span.trace_id.clone());
            spans.push(span);
        }

        Ok(PreparedTraceBatch {
            tenant_id: scope.tenant_id,
            project_id: scope.project_id,
            queue_key: raw_idempotency_key,
            trace_ids,
            batch: CanonicalTraceBatch {
                raw_envelopes: vec![raw],
                spans,
            },
        })
    }

    /// Assess the completion state of a stored trace by re-assembling every span
    /// currently persisted for it. Re-running this as late/out-of-order spans land
    /// reflects the trace converging toward completion. Returns `Open` for a trace
    /// with no spans yet.
    pub async fn assess_trace_completion(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        trace_id: TraceId,
    ) -> Result<TraceCompletionState, IngestError> {
        let trace = match self
            .traces
            .get_project_trace(tenant_id, project_id, trace_id)
            .await
        {
            Ok(trace) => trace,
            // A trace with no stored spans yet is `Open`, per the contract above —
            // it has simply not started converging. Other store errors propagate.
            Err(StoreError::NotFound(_)) => return Ok(TraceCompletionState::Open),
            Err(err) => return Err(IngestError::Store(err)),
        };
        Ok(assemble_trace_completion(
            &trace.spans,
            self.clock.now(),
            self.policy.trace_completion,
        ))
    }

    async fn publish_trace_write(
        &self,
        prepared: &PreparedTraceBatch,
    ) -> Result<PublishAck, IngestError> {
        let payload = serde_json::to_vec(&QueuedTraceWrite {
            batch: prepared.batch.clone(),
        })
        .map_err(anyhow::Error::from)?;
        let mut message = BusMessage::new(
            prepared.tenant_id.clone(),
            prepared.project_id.clone(),
            prepared.queue_key.clone(),
            TRACE_WRITE_BATCH_KIND,
            payload,
        );
        message.max_attempts = self.policy.trace_write_max_attempts;
        self.bus.publish(message).await.map_err(map_bus_error)
    }

    async fn publish_trace_ingested(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        base_key: &IdempotencyKey,
        trace_ids: &BTreeSet<TraceId>,
    ) -> Result<TraceIngestedPublishReport, IngestError> {
        let mut report = TraceIngestedPublishReport::default();
        for trace_id in trace_ids {
            report.requested += 1;
            let queue_payload = serde_json::to_vec(&QueuedTraceWork {
                tenant_id: tenant_id.clone(),
                project_id: project_id.clone(),
                trace_id: trace_id.clone(),
            })
            .map_err(anyhow::Error::from)?;
            let queue_key = IdempotencyKey::new(format!(
                "{}:{}:{}",
                base_key.as_str(),
                TRACE_INGESTED_KIND,
                trace_id.as_str()
            ))
            .map_err(anyhow::Error::from)?;
            let ack = self
                .bus
                .publish(BusMessage::new(
                    tenant_id.clone(),
                    project_id.clone(),
                    queue_key,
                    TRACE_INGESTED_KIND,
                    queue_payload,
                ))
                .await
                .map_err(map_bus_error)?;
            if ack.accepted {
                report.accepted += 1;
            }
            if ack.duplicate {
                report.duplicate += 1;
            }
        }
        Ok(report)
    }

    async fn process_trace_write_message<F, Fut>(
        &self,
        message: BusMessage,
        report: &mut TraceWriteDrainReport,
        before_write: &mut F,
    ) -> Result<(), IngestError>
    where
        F: FnMut(QueuedTraceWrite) -> Fut,
        Fut: Future<Output = Result<(), String>>,
    {
        let will_dead_letter = message.attempts.saturating_add(1) >= message.max_attempts;
        let queued = match serde_json::from_slice::<QueuedTraceWrite>(&message.payload) {
            Ok(queued) => queued,
            Err(err) => {
                report.invalid_messages += 1;
                self.retry_or_dlq_counted(
                    message,
                    format!("invalid trace.write_batch payload: {err}"),
                    will_dead_letter,
                    report,
                )
                .await?;
                return Ok(());
            }
        };

        if let Err(reason) = validate_trace_write_scope(&message, &queued.batch) {
            report.invalid_messages += 1;
            self.retry_or_dlq_counted(message, reason, will_dead_letter, report)
                .await?;
            return Ok(());
        }

        if let Err(reason) = before_write(queued.clone()).await {
            report.failed_writes += 1;
            self.retry_or_dlq_counted(message, reason, will_dead_letter, report)
                .await?;
            return Ok(());
        }

        let trace_ids = trace_ids_for_batch(&queued.batch);
        let write_ack = match self.traces.write_batch(queued.batch.clone()).await {
            Ok(write_ack) => write_ack,
            Err(err) => {
                report.failed_writes += 1;
                self.retry_or_dlq_counted(
                    message,
                    format!("trace store write failed: {err}"),
                    will_dead_letter,
                    report,
                )
                .await?;
                return Ok(());
            }
        };
        report.written_raw += write_ack.accepted_raw;
        report.written_spans += write_ack.accepted_spans;
        report.duplicate_raw += write_ack.duplicate_raw;
        report.duplicate_spans += write_ack.duplicate_spans;

        match self
            .publish_trace_ingested(
                &message.tenant_id,
                &message.project_id,
                &message.idempotency_key,
                &trace_ids,
            )
            .await
        {
            Ok(published) => {
                report.downstream_published += published.accepted;
                report
                    .trace_refs
                    .extend(trace_ids.iter().map(|trace_id| QueuedTraceWork {
                        tenant_id: message.tenant_id.clone(),
                        project_id: message.project_id.clone(),
                        trace_id: trace_id.clone(),
                    }));
                report.trace_ids.extend(trace_ids);
                self.bus.ack(message).await.map_err(map_bus_error)?;
                Ok(())
            }
            Err(err) => {
                report.failed_downstream_publishes += 1;
                self.retry_or_dlq_counted(
                    message,
                    format!("downstream publish failed after trace write: {err}"),
                    will_dead_letter,
                    report,
                )
                .await
            }
        }
    }

    async fn retry_or_dlq_counted<R: DrainReport>(
        &self,
        message: BusMessage,
        reason: String,
        will_dead_letter: bool,
        report: &mut R,
    ) -> Result<(), IngestError> {
        self.bus
            .retry_or_dlq(message, reason)
            .await
            .map_err(map_bus_error)?;
        report.note_outcome(will_dead_letter);
        Ok(())
    }

    async fn enforce_quota_events(
        &self,
        scope: &TenantScope,
        event_count: u64,
    ) -> Result<(), IngestError> {
        let Some(limit) = self.policy.per_project_event_quota else {
            return Ok(());
        };
        if event_count == 0 {
            return Ok(());
        }
        let (window_start, reset_at) =
            quota_window_bounds(self.clock.now(), self.policy.quota_window_seconds)?;
        let decision = self
            .quota_limiter
            .reserve_quota(QuotaReservationRequest {
                tenant_id: scope.tenant_id.clone(),
                project_id: scope.project_id.clone(),
                amount: event_count,
                limit,
                window_start,
                reset_at,
            })
            .await
            .map_err(IngestError::Store)?;
        if !decision.accepted {
            return Err(IngestError::QuotaExceeded {
                tenant_id: scope.tenant_id.to_string(),
                project_id: scope.project_id.to_string(),
                limit: decision.limit,
                used: decision.used,
                reset_at: decision.reset_at,
            });
        }
        Ok(())
    }

    fn govern_attributes(
        &self,
        attributes: CanonicalAttrs,
    ) -> Result<(CanonicalAttrs, Value), IngestError> {
        if attributes.len() > self.policy.max_attributes {
            return Err(IngestError::TooManyAttributes {
                count: attributes.len(),
                limit: self.policy.max_attributes,
            });
        }
        let mut kept = BTreeMap::new();
        let mut dropped = BTreeMap::new();
        // Attributes the normalizer carried through but that do NOT correspond to
        // any canonical mapping (a recognized `llm.*`/`gen_ai.*`/`browser.*`/
        // structural key, or a top-level field like model/cost/tokens). These are
        // preserved verbatim under `unmapped_attrs.unmapped` so the canonical
        // projection is honest about *what* it could not interpret. See R2.3.
        let mut unmapped = BTreeMap::new();
        for (key, value) in attributes {
            if self.policy.denied_attributes.contains(&key) {
                dropped.insert(key, json!("[redacted]"));
                continue;
            }
            if let Some(allowed) = &self.policy.allowed_attributes {
                if !allowed.contains(&key) {
                    dropped.insert(key, value);
                    continue;
                }
            }
            if !canonical_mapping::maps_to_canonical(&key) {
                unmapped.insert(key.clone(), value.clone());
            }
            kept.insert(key, value);
        }
        Ok((
            kept,
            json!({ "dropped_attributes": dropped, "unmapped": unmapped }),
        ))
    }

    async fn maybe_payload_artifact(
        &self,
        scope: &TenantScope,
        value: Option<&Value>,
        redaction_class: &RedactionClass,
        mime_type: &str,
    ) -> Result<Option<ArtifactRef>, IngestError> {
        let Some(value) = value else {
            return Ok(None);
        };
        let bytes = serde_json::to_vec(value).map_err(anyhow::Error::from)?;
        if bytes.len() <= self.policy.inline_payload_bytes {
            return Ok(None);
        }
        self.artifacts
            .put_bytes(
                &scope.tenant_id,
                &scope.project_id,
                mime_type,
                redaction_class.clone(),
                &bytes,
            )
            .await
            .map(Some)
            .map_err(IngestError::Store)
    }
}

#[derive(Clone, Debug)]
pub struct IngestPolicy {
    pub max_raw_payload_bytes: usize,
    pub inline_payload_bytes: usize,
    pub max_attributes: usize,
    pub allowed_attributes: Option<BTreeSet<String>>,
    pub denied_attributes: BTreeSet<String>,
    pub per_project_event_quota: Option<u64>,
    pub quota_window_seconds: i64,
    pub trace_write_max_attempts: u32,
    /// Tunables for deriving per-trace [`TraceCompletionState`] during assembly.
    pub trace_completion: TraceCompletionConfig,
}

impl Default for IngestPolicy {
    fn default() -> Self {
        Self {
            max_raw_payload_bytes: 1024 * 1024,
            inline_payload_bytes: 16 * 1024,
            max_attributes: 128,
            allowed_attributes: None,
            denied_attributes: default_denied_attributes(),
            per_project_event_quota: None,
            quota_window_seconds: 60,
            trace_write_max_attempts: 3,
            trace_completion: TraceCompletionConfig::default(),
        }
    }
}

impl IngestPolicy {
    /// Validate that the policy fields form a coherent configuration.
    ///
    /// Call at startup after building the policy from CLI/env args; impossible
    /// combinations (inline cutoff larger than the raw cap, zero limits) are
    /// caught here with a clear diagnostic rather than producing silent
    /// mismatches at runtime.
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.max_raw_payload_bytes >= 1,
            "max_raw_payload_bytes must be at least 1; got {}",
            self.max_raw_payload_bytes
        );
        anyhow::ensure!(
            self.max_attributes >= 1,
            "max_attributes must be at least 1; got {}",
            self.max_attributes
        );
        anyhow::ensure!(
            self.inline_payload_bytes <= self.max_raw_payload_bytes,
            "inline_payload_bytes ({}) must not exceed max_raw_payload_bytes ({}); \
             payloads cannot be inlined when the inline cutoff is larger than the payload cap",
            self.inline_payload_bytes,
            self.max_raw_payload_bytes
        );
        anyhow::ensure!(
            self.trace_completion.idle_timeout > Duration::zero(),
            "trace_idle_timeout must be positive; got {} seconds",
            self.trace_completion.idle_timeout.num_seconds()
        );
        anyhow::ensure!(
            self.trace_completion.late_window > Duration::zero(),
            "trace_late_window must be positive; got {} seconds",
            self.trace_completion.late_window.num_seconds()
        );
        Ok(())
    }
}

fn default_denied_attributes() -> BTreeSet<String> {
    [
        "authorization",
        "cookie",
        "set-cookie",
        "http.request.header.authorization",
        "http.request.header.cookie",
        "http.request.header.proxy-authorization",
        "http.request.header.proxy_authorization",
        "http.request.header.x-api-key",
        "http.request.header.x_api_key",
        "http.response.header.set-cookie",
        "http.response.header.set_cookie",
        "http.url",
        "url.full",
        "gen_ai.prompt",
        "gen_ai.completion",
        "llm.prompt",
        "llm.completion",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

#[derive(Clone, Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct NativeIngestRequest {
    pub scope: TenantScope,
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub parent_span_id: Option<SpanId>,
    pub seq: u64,
    pub kind: AgentSpanKind,
    pub name: String,
    pub status: SpanStatus,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub start_time: Option<Timestamp>,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub end_time: Option<Timestamp>,
    pub model: Option<ModelRef>,
    pub cost: Option<beater_core::Money>,
    pub tokens: Option<TokenCounts>,
    #[schema(value_type = Option<serde_json::Value>)]
    pub input: Option<Value>,
    #[schema(value_type = Option<serde_json::Value>)]
    pub output: Option<Value>,
    #[schema(value_type = std::collections::BTreeMap<String, serde_json::Value>)]
    pub attributes: CanonicalAttrs,
    pub redaction_class: RedactionClass,
    pub idempotency_key: Option<IdempotencyKey>,
    pub auth_context: Option<AuthContext>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawTraceIngestRequest {
    pub scope: TenantScope,
    pub source: SourceDialect,
    pub source_schema_url: Option<String>,
    pub source_schema_version: Option<String>,
    pub normalizer_version: String,
    pub mime_type: String,
    pub redaction_class: RedactionClass,
    pub raw_bytes: Vec<u8>,
    pub raw_idempotency_key: Option<IdempotencyKey>,
    pub auth_context: Option<AuthContext>,
    pub spans: Vec<CanonicalSpanDraft>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanonicalSpanDraft {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub parent_span_id: Option<SpanId>,
    pub seq: u64,
    pub kind: AgentSpanKind,
    pub name: String,
    pub status: SpanStatus,
    pub start_time: Option<Timestamp>,
    pub end_time: Option<Timestamp>,
    pub model: Option<ModelRef>,
    pub cost: Option<beater_core::Money>,
    pub tokens: Option<TokenCounts>,
    pub input: Option<Value>,
    pub output: Option<Value>,
    pub attributes: CanonicalAttrs,
}

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, utoipa::ToSchema,
)]
pub struct QueuedTraceWork {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub trace_id: TraceId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QueuedTraceWrite {
    pub batch: CanonicalTraceBatch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct IngestOutcome {
    pub ack: WriteAck,
    pub downstream_queued: bool,
}

/// A drain report that tallies the outcome of a retry-or-dead-letter decision.
trait DrainReport {
    fn note_outcome(&mut self, dead_lettered: bool);
}

impl DrainReport for TraceWriteDrainReport {
    fn note_outcome(&mut self, dead_lettered: bool) {
        if dead_lettered {
            self.dead_lettered += 1;
        } else {
            self.retried += 1;
        }
    }
}

impl DrainReport for TraceIngestedDrainReport {
    fn note_outcome(&mut self, dead_lettered: bool) {
        if dead_lettered {
            self.dead_lettered += 1;
        } else {
            self.retried += 1;
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TraceWriteDrainReport {
    pub consumed: usize,
    pub written_raw: usize,
    pub written_spans: usize,
    pub duplicate_raw: usize,
    pub duplicate_spans: usize,
    pub downstream_published: usize,
    pub retried: usize,
    pub dead_lettered: usize,
    pub invalid_messages: usize,
    pub failed_writes: usize,
    pub failed_downstream_publishes: usize,
    pub trace_refs: Vec<QueuedTraceWork>,
    pub trace_ids: Vec<TraceId>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TraceIngestedDrainReport {
    pub consumed: usize,
    pub completed: usize,
    pub retried: usize,
    pub dead_lettered: usize,
    pub invalid_messages: usize,
    pub failed_work: usize,
    pub trace_refs: Vec<QueuedTraceWork>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct IngestQueueStatus {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub total_depth: usize,
    pub trace_write_depth: usize,
    pub trace_ingested_depth: usize,
    pub dead_letters: Vec<DeadLetter>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DeadLetterReplayReport {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub message_id: String,
    pub reset_attempts: bool,
    pub ack: PublishAck,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TraceIngestedReconcileReport {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub trace_id: TraceId,
    pub span_count: usize,
    pub downstream_accepted: usize,
    pub downstream_duplicate: usize,
    pub downstream_queued: bool,
}

#[derive(Clone, Debug)]
struct PreparedTraceBatch {
    tenant_id: TenantId,
    project_id: ProjectId,
    queue_key: IdempotencyKey,
    trace_ids: BTreeSet<TraceId>,
    batch: CanonicalTraceBatch,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct TraceIngestedPublishReport {
    requested: usize,
    accepted: usize,
    duplicate: usize,
}

impl TraceIngestedPublishReport {
    fn queued(&self) -> bool {
        self.accepted.saturating_add(self.duplicate) >= self.requested && self.requested > 0
    }
}

/// Canonical-attribute classification used to populate `unmapped_attrs`.
///
/// The normalizer keeps the full attribute bag on the canonical span, but a
/// projection consumer (dashboard, evaluator, export) needs to know which of
/// those attributes the canonical model actually *understands* versus which were
/// merely passed through. An attribute "maps to canonical" when its key is one of
/// the recognized semantic-convention keys (the same keys the OTLP normalizer
/// reads to populate model/cost/tokens/kind/seq/input/output) or belongs to a
/// recognized namespace prefix (`llm.`, `gen_ai.`, `browser.`, `resource.`,
/// `otel.`, `beater.`, `agent.`, `openinference.`, `w3c.`). Everything else
/// "fails canonical mapping" and is recorded under `unmapped_attrs.unmapped`.
pub mod canonical_mapping {
    /// Recognized namespace prefixes whose attributes the canonical model
    /// understands (either projected to a typed field or carried as a known
    /// semantic-convention attribute).
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
    ];

    /// Exact keys without a recognized prefix that are still canonical because the
    /// normalizer reads them to populate typed span fields (model/cost/tokens).
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

    /// True when `key` corresponds to a canonical mapping; false when the
    /// attribute fails canonical mapping and must be preserved in `unmapped_attrs`.
    pub fn maps_to_canonical(key: &str) -> bool {
        if CANONICAL_EXACT_KEYS.contains(&key) {
            return true;
        }
        CANONICAL_PREFIXES
            .iter()
            .any(|prefix| key.starts_with(prefix))
    }
}

pub fn anonymous_auth_context() -> AuthContext {
    AuthContext {
        api_key_id: None,
        scopes: BTreeSet::new(),
    }
}

fn batch_publish_ack(batch: &CanonicalTraceBatch, publish: &PublishAck) -> WriteAck {
    if publish.duplicate {
        return WriteAck {
            accepted_raw: 0,
            accepted_spans: 0,
            duplicate_raw: batch.raw_envelopes.len(),
            duplicate_spans: batch.spans.len(),
        };
    }
    WriteAck {
        accepted_raw: batch.raw_envelopes.len(),
        accepted_spans: batch.spans.len(),
        duplicate_raw: 0,
        duplicate_spans: 0,
    }
}

fn validate_trace_write_scope(
    message: &BusMessage,
    batch: &CanonicalTraceBatch,
) -> Result<(), String> {
    if batch.raw_envelopes.is_empty() && batch.spans.is_empty() {
        return Err("trace.write_batch payload is empty".to_string());
    }
    for raw in &batch.raw_envelopes {
        if raw.tenant_id != message.tenant_id || raw.project_id != message.project_id {
            return Err(format!(
                "trace.write_batch raw scope mismatch: message={}/{} raw={}/{}",
                message.tenant_id, message.project_id, raw.tenant_id, raw.project_id
            ));
        }
    }
    for span in &batch.spans {
        if span.tenant_id != message.tenant_id || span.project_id != message.project_id {
            return Err(format!(
                "trace.write_batch span scope mismatch: message={}/{} span={}/{}",
                message.tenant_id, message.project_id, span.tenant_id, span.project_id
            ));
        }
    }
    Ok(())
}

fn trace_ids_for_batch(batch: &CanonicalTraceBatch) -> BTreeSet<TraceId> {
    batch
        .spans
        .iter()
        .map(|span| span.trace_id.clone())
        .collect()
}

fn quota_window_bounds(
    now: Timestamp,
    window_seconds: i64,
) -> Result<(Timestamp, Timestamp), IngestError> {
    let window_seconds = window_seconds.max(1);
    let window_start_seconds = now.timestamp().div_euclid(window_seconds) * window_seconds;
    let reset_seconds = window_start_seconds
        .checked_add(window_seconds)
        .ok_or_else(|| IngestError::Other(anyhow::anyhow!("quota reset timestamp overflow")))?;
    let window_start =
        DateTime::<Utc>::from_timestamp(window_start_seconds, 0).ok_or_else(|| {
            IngestError::Other(anyhow::anyhow!("invalid quota window start timestamp"))
        })?;
    let reset_at = DateTime::<Utc>::from_timestamp(reset_seconds, 0)
        .ok_or_else(|| IngestError::Other(anyhow::anyhow!("invalid quota reset timestamp")))?;
    Ok((window_start, reset_at))
}

fn map_bus_error(error: BusError) -> IngestError {
    match error {
        BusError::Backpressure { capacity } => IngestError::Backpressure { capacity },
        BusError::NotFound(message) => IngestError::NotFound(message),
        other => IngestError::Other(anyhow::Error::new(other)),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceCompletionInput {
    pub root_span_ended: bool,
    pub open_child_spans: usize,
    pub idle_for: Duration,
    pub idle_timeout: Duration,
    pub late_window_closed: bool,
}

pub fn trace_completion_state(input: TraceCompletionInput) -> TraceCompletionState {
    if input.root_span_ended && input.open_child_spans == 0 && input.late_window_closed {
        return TraceCompletionState::Complete;
    }
    if input.root_span_ended {
        return TraceCompletionState::RootEnded;
    }
    if input.idle_for >= input.idle_timeout && input.open_child_spans == 0 {
        return TraceCompletionState::IdleComplete;
    }
    if input.late_window_closed {
        return TraceCompletionState::LateWindowClosed;
    }
    TraceCompletionState::Open
}

/// Tunables for deriving a [`TraceCompletionState`] from the *currently assembled*
/// spans of a single trace. Distinct from a wall-clock timer: assembly evaluates
/// completion from the span set itself plus the ingest clock, so out-of-order and
/// late-arriving spans are handled by re-evaluation as the set grows.
#[derive(Clone, Copy, Debug)]
pub struct TraceCompletionConfig {
    /// Idle gap after the last observed span end before an open trace is treated
    /// as idle-complete.
    pub idle_timeout: Duration,
    /// How long after the root span ends the trace stays open for late children
    /// (out-of-order / clock-skewed arrivals). Once this window elapses the trace
    /// is [`TraceCompletionState::Complete`]; before it elapses an ended root is
    /// reported as [`TraceCompletionState::RootEnded`].
    pub late_window: Duration,
}

impl Default for TraceCompletionConfig {
    fn default() -> Self {
        Self {
            idle_timeout: Duration::seconds(60),
            late_window: Duration::seconds(10),
        }
    }
}

/// Derive the completion state of an assembled trace from its spans.
///
/// Ordering-agnostic and clock-skew tolerant: the root is identified structurally
/// (no `parent_span_id`, falling back to the lexicographically smallest span id so
/// a trace with a not-yet-arrived root never spuriously completes), open children
/// are counted by missing `end_time`, and the idle gap is measured from the latest
/// span end against `now`. `now` is taken from the ingest clock, and negative gaps
/// (a span whose end is in the future relative to `now` due to clock skew) are
/// clamped so skew cannot force premature completion.
pub fn assemble_trace_completion(
    spans: &[CanonicalSpan],
    now: Timestamp,
    config: TraceCompletionConfig,
) -> TraceCompletionState {
    if spans.is_empty() {
        return TraceCompletionState::Open;
    }

    // Identify the root: a parentless span, else the smallest span id (stable
    // under reordering). Treat the root as ended only when it has an end_time.
    let root = spans
        .iter()
        .filter(|span| span.parent_span_id.is_none())
        .min_by(|a, b| a.span_id.as_str().cmp(b.span_id.as_str()));
    let root_span_ended = root.is_some_and(|span| span.end_time.is_some());

    let open_child_spans = spans
        .iter()
        .filter(|span| span.parent_span_id.is_some() && span.end_time.is_none())
        .count();

    // Latest end across all spans; clamp future-dated ends to `now` so a
    // clock-skewed span cannot produce a negative idle gap.
    let latest_end = spans.iter().filter_map(|span| span.end_time).max();
    let idle_for = latest_end
        .map(|end| {
            let gap = now.signed_duration_since(end);
            if gap < Duration::zero() {
                Duration::zero()
            } else {
                gap
            }
        })
        .unwrap_or_else(Duration::zero);

    // The late window closes once enough idle time has elapsed since the last
    // observed end for late/out-of-order children to be considered drained.
    let late_window_closed = latest_end.is_some_and(|_| idle_for >= config.late_window);

    trace_completion_state(TraceCompletionInput {
        root_span_ended,
        open_child_spans,
        idle_for,
        idle_timeout: config.idle_timeout,
        late_window_closed,
    })
}

pub async fn smoke_trace(service: &IngestService) -> Result<IngestOutcome, IngestError> {
    let scope = TenantScope::new(
        TenantId::new("demo").map_err(anyhow::Error::from)?,
        ProjectId::new("demo").map_err(anyhow::Error::from)?,
        EnvironmentId::new("local").map_err(anyhow::Error::from)?,
    );
    service
        .ingest_native(NativeIngestRequest {
            scope,
            trace_id: TraceId::new("smoke-trace").map_err(anyhow::Error::from)?,
            span_id: SpanId::new("smoke-root").map_err(anyhow::Error::from)?,
            parent_span_id: None,
            seq: 1,
            kind: AgentSpanKind::AgentRun,
            name: "smoke agent run".to_string(),
            status: SpanStatus::Ok,
            start_time: Some(Utc::now()),
            end_time: Some(Utc::now()),
            model: None,
            cost: None,
            tokens: None,
            input: Some(json!({ "prompt": "hello" })),
            output: Some(json!({ "answer": "world" })),
            attributes: BTreeMap::new(),
            redaction_class: RedactionClass::Internal,
            idempotency_key: None,
            auth_context: None,
        })
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use beater_bus::{BusError, DurableBus, InMemoryBus, PublishAck, SqliteDurableBus};
    use beater_core::FixedClock;
    use beater_core::{Page, PageRequest};
    use beater_schema::{RunFilter, RunSummary, SpanFilter, SpanSummary, TraceView};
    use beater_store_obj::FsArtifactStore;
    use beater_store_sql::{SqliteQuotaLimiter, SqliteTraceStore};
    use chrono::TimeZone;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct FailInitialPublishBus {
        inner: InMemoryBus,
        trace_ingested_failures: AtomicUsize,
        trace_write_failures: AtomicUsize,
    }

    impl FailInitialPublishBus {
        fn fail_trace_ingested_once(capacity: usize) -> Self {
            Self::new(capacity, 1, 0)
        }

        fn new(
            capacity: usize,
            trace_ingested_failures: usize,
            trace_write_failures: usize,
        ) -> Self {
            Self {
                inner: InMemoryBus::new(capacity),
                trace_ingested_failures: AtomicUsize::new(trace_ingested_failures),
                trace_write_failures: AtomicUsize::new(trace_write_failures),
            }
        }

        fn fail_publish_if_needed(
            remaining_failures: &AtomicUsize,
            reason: &str,
        ) -> Result<(), BusError> {
            if remaining_failures.load(Ordering::SeqCst) == 0 {
                return Ok(());
            }
            let _ =
                remaining_failures.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |remaining| {
                    remaining.checked_sub(1)
                });
            Err(BusError::Storage(reason.to_string()))
        }
    }

    #[async_trait]
    impl DurableBus for FailInitialPublishBus {
        async fn publish(&self, message: BusMessage) -> Result<PublishAck, BusError> {
            if message.kind == TRACE_INGESTED_KIND {
                Self::fail_publish_if_needed(
                    &self.trace_ingested_failures,
                    "simulated trace.ingested publish outage",
                )?;
            }
            if message.kind == TRACE_WRITE_BATCH_KIND {
                Self::fail_publish_if_needed(
                    &self.trace_write_failures,
                    "simulated trace.write_batch publish outage",
                )?;
            }
            self.inner.publish(message).await
        }

        async fn consume_batch(&self, limit: usize) -> Result<Vec<BusMessage>, BusError> {
            self.inner.consume_batch(limit).await
        }

        async fn consume_kind_batch(
            &self,
            kind: &str,
            limit: usize,
        ) -> Result<Vec<BusMessage>, BusError> {
            self.inner.consume_kind_batch(kind, limit).await
        }

        async fn consume_scoped_kind_batch(
            &self,
            tenant_id: &TenantId,
            project_id: &ProjectId,
            kind: &str,
            limit: usize,
        ) -> Result<Vec<BusMessage>, BusError> {
            self.inner
                .consume_scoped_kind_batch(tenant_id, project_id, kind, limit)
                .await
        }

        async fn ack(&self, message: BusMessage) -> Result<(), BusError> {
            self.inner.ack(message).await
        }

        async fn retry_or_dlq(&self, message: BusMessage, reason: String) -> Result<(), BusError> {
            self.inner.retry_or_dlq(message, reason).await
        }

        async fn replay_dead_letter(
            &self,
            tenant_id: &TenantId,
            project_id: &ProjectId,
            message_id: &str,
            reset_attempts: bool,
        ) -> Result<PublishAck, BusError> {
            self.inner
                .replay_dead_letter(tenant_id, project_id, message_id, reset_attempts)
                .await
        }

        async fn dlq(&self) -> Result<Vec<DeadLetter>, BusError> {
            self.inner.dlq().await
        }

        async fn depth(&self) -> Result<usize, BusError> {
            self.inner.depth().await
        }

        async fn depth_for_kind(&self, kind: &str) -> Result<usize, BusError> {
            self.inner.depth_for_kind(kind).await
        }
    }

    #[tokio::test]
    async fn native_ingest_preserves_raw_and_canonical_span() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        let service = IngestService::new(
            artifacts,
            traces.clone(),
            bus.clone(),
            IngestPolicy::default(),
        );
        let request = fixture_request();

        let outcome = service
            .ingest_native(request.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(outcome.ack.accepted_raw, 1);
        assert_eq!(outcome.ack.accepted_spans, 1);
        assert_eq!(bus.depth().await, Ok(1));

        let trace = traces
            .get_trace(request.scope.tenant_id, request.trace_id)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(trace.spans.len(), 1);
        assert_eq!(trace.spans[0].normalizer_version, "beater-native-v1");
        assert_eq!(trace.spans[0].schema_version, CANONICAL_SCHEMA_VERSION);
        assert_eq!(
            trace.spans[0].unmapped_attrs["dropped_attributes"],
            json!({})
        );
    }

    #[tokio::test]
    async fn native_ingest_uses_injected_clock_for_missing_timestamps() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        let now = Utc
            .with_ymd_and_hms(2026, 1, 1, 0, 0, 42)
            .single()
            .unwrap_or_else(|| panic!("valid timestamp"));
        let service = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default())
            .with_clock(Arc::new(FixedClock::new(now)));
        let idempotency_key =
            IdempotencyKey::new("fixed-clock-native").unwrap_or_else(|err| panic!("{err}"));
        let mut request = fixture_request();
        request.start_time = None;
        request.end_time = None;
        request.idempotency_key = Some(idempotency_key.clone());

        service
            .ingest_native(request.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let raw = traces
            .get_raw_envelope(
                request.scope.tenant_id.clone(),
                request.scope.project_id.clone(),
                idempotency_key,
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"))
            .unwrap_or_else(|| panic!("raw envelope should exist"));
        assert_eq!(raw.received_at, now);

        let trace = traces
            .get_trace(request.scope.tenant_id, request.trace_id)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(trace.spans[0].start_time, now);
        assert_eq!(trace.spans[0].end_time, None);
    }

    #[tokio::test]
    async fn raw_trace_batch_preserves_external_source_bytes_and_envelope() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        let service = IngestService::new(
            artifacts.clone(),
            traces.clone(),
            bus.clone(),
            IngestPolicy::default(),
        );
        let scope = TenantScope::new(
            TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
        );
        let raw_idempotency_key =
            IdempotencyKey::new("otlp-raw-1").unwrap_or_else(|err| panic!("{err}"));
        let raw_bytes = b"\x0a\x05otlp".to_vec();

        let outcome = service
            .ingest_raw_trace_batch(RawTraceIngestRequest {
                scope: scope.clone(),
                source: SourceDialect::Otlp,
                source_schema_url: Some("https://opentelemetry.io/schemas/1.37.0".to_string()),
                source_schema_version: Some("1.37.0".to_string()),
                normalizer_version: "beater-otlp-v1".to_string(),
                mime_type: "application/x-protobuf".to_string(),
                redaction_class: RedactionClass::Internal,
                raw_bytes: raw_bytes.clone(),
                raw_idempotency_key: Some(raw_idempotency_key.clone()),
                auth_context: Some(AuthContext {
                    api_key_id: None,
                    scopes: BTreeSet::from(["trace:write".to_string()]),
                }),
                spans: vec![CanonicalSpanDraft {
                    trace_id: TraceId::new("trace").unwrap_or_else(|err| panic!("{err}")),
                    span_id: SpanId::new("span").unwrap_or_else(|err| panic!("{err}")),
                    parent_span_id: None,
                    seq: 1,
                    kind: AgentSpanKind::LlmCall,
                    name: "llm call".to_string(),
                    status: SpanStatus::Ok,
                    start_time: Some(Utc::now()),
                    end_time: Some(Utc::now()),
                    model: None,
                    cost: None,
                    tokens: None,
                    input: Some(json!("hello")),
                    output: Some(json!("world")),
                    attributes: BTreeMap::from([("otel.span.kind".to_string(), json!("CLIENT"))]),
                }],
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(outcome.ack.accepted_raw, 1);
        assert_eq!(outcome.ack.accepted_spans, 1);
        assert_eq!(bus.depth().await, Ok(1));

        let raw = traces
            .get_raw_envelope(
                scope.tenant_id.clone(),
                scope.project_id.clone(),
                raw_idempotency_key,
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"))
            .unwrap_or_else(|| panic!("raw envelope should be present"));
        assert_eq!(raw.source, SourceDialect::Otlp);
        assert_eq!(raw.source_schema_version.as_deref(), Some("1.37.0"));
        assert_eq!(
            raw.auth_context.scopes,
            BTreeSet::from(["trace:write".to_string()])
        );

        let trace = traces
            .get_trace(
                scope.tenant_id,
                TraceId::new("trace").unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(trace.spans[0].normalizer_version, "beater-otlp-v1");
        assert_eq!(trace.spans[0].raw_ref, raw.body_ref);
        let stored_bytes = artifacts
            .get_bytes(&trace.spans[0].raw_ref)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(stored_bytes, raw_bytes);
    }

    #[tokio::test]
    async fn raw_trace_batch_uses_injected_clock_for_missing_timestamps() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        let now = Utc
            .with_ymd_and_hms(2026, 1, 1, 0, 2, 3)
            .single()
            .unwrap_or_else(|| panic!("valid timestamp"));
        let service = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default())
            .with_clock(Arc::new(FixedClock::new(now)));
        let scope = TenantScope::new(
            TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
        );
        let raw_idempotency_key =
            IdempotencyKey::new("fixed-clock-raw").unwrap_or_else(|err| panic!("{err}"));
        let trace_id = TraceId::new("clock-raw-trace").unwrap_or_else(|err| panic!("{err}"));

        service
            .ingest_raw_trace_batch(RawTraceIngestRequest {
                scope: scope.clone(),
                source: SourceDialect::Otlp,
                source_schema_url: None,
                source_schema_version: None,
                normalizer_version: "beater-otlp-v1".to_string(),
                mime_type: "application/x-protobuf".to_string(),
                redaction_class: RedactionClass::Internal,
                raw_bytes: b"fixed-clock-raw".to_vec(),
                raw_idempotency_key: Some(raw_idempotency_key.clone()),
                auth_context: None,
                spans: vec![CanonicalSpanDraft {
                    trace_id: trace_id.clone(),
                    span_id: SpanId::new("span").unwrap_or_else(|err| panic!("{err}")),
                    parent_span_id: None,
                    seq: 1,
                    kind: AgentSpanKind::LlmCall,
                    name: "llm call".to_string(),
                    status: SpanStatus::Ok,
                    start_time: None,
                    end_time: None,
                    model: None,
                    cost: None,
                    tokens: None,
                    input: None,
                    output: None,
                    attributes: BTreeMap::new(),
                }],
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let raw = traces
            .get_raw_envelope(
                scope.tenant_id.clone(),
                scope.project_id.clone(),
                raw_idempotency_key,
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"))
            .unwrap_or_else(|| panic!("raw envelope should exist"));
        assert_eq!(raw.received_at, now);

        let trace = traces
            .get_trace(scope.tenant_id, trace_id)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(trace.spans[0].start_time, now);
        assert_eq!(trace.spans[0].end_time, None);
    }

    #[tokio::test]
    async fn direct_ingest_reports_backpressure_after_durable_write_when_downstream_retry_is_full()
    {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(0));
        let service = IngestService::new(
            artifacts,
            traces.clone(),
            bus.clone(),
            IngestPolicy::default(),
        );
        let request = fixture_request();

        let error = service
            .ingest_native(request.clone())
            .await
            .err()
            .unwrap_or_else(|| panic!("downstream retry backpressure should fail"));

        assert!(matches!(error, IngestError::Backpressure { capacity: 0 }));
        assert_eq!(bus.depth().await, Ok(0));

        let trace = traces
            .get_trace(request.scope.tenant_id, request.trace_id)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(trace.spans.len(), 1);
    }

    #[tokio::test]
    async fn reconcile_trace_ingested_recovers_after_direct_publish_and_fallback_fail() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(FailInitialPublishBus::new(16, 1, 1));
        let service = IngestService::new(
            artifacts,
            traces.clone(),
            bus.clone(),
            IngestPolicy::default(),
        );
        let request = fixture_request();

        let error = service
            .ingest_native(request.clone())
            .await
            .err()
            .unwrap_or_else(|| panic!("direct ingest should report missing downstream durability"));
        assert!(error
            .to_string()
            .contains("simulated trace.write_batch publish outage"));
        assert_eq!(bus.depth_for_kind(TRACE_WRITE_BATCH_KIND).await, Ok(0));
        assert_eq!(bus.depth_for_kind(TRACE_INGESTED_KIND).await, Ok(0));
        let trace = traces
            .get_project_trace(
                request.scope.tenant_id.clone(),
                request.scope.project_id.clone(),
                request.trace_id.clone(),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(trace.spans.len(), 1);

        let reconcile = service
            .reconcile_trace_ingested(
                request.scope.tenant_id.clone(),
                request.scope.project_id.clone(),
                request.trace_id.clone(),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(reconcile.span_count, 1);
        assert_eq!(reconcile.downstream_accepted, 1);
        assert_eq!(reconcile.downstream_duplicate, 0);
        assert!(reconcile.downstream_queued);
        assert_eq!(bus.depth_for_kind(TRACE_INGESTED_KIND).await, Ok(1));

        let duplicate_reconcile = service
            .reconcile_trace_ingested(
                request.scope.tenant_id.clone(),
                request.scope.project_id.clone(),
                request.trace_id.clone(),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(duplicate_reconcile.downstream_accepted, 0);
        assert_eq!(duplicate_reconcile.downstream_duplicate, 1);
        assert!(duplicate_reconcile.downstream_queued);
        assert_eq!(bus.depth_for_kind(TRACE_INGESTED_KIND).await, Ok(1));

        let downstream = service
            .drain_trace_ingested(10, |trace_ref| {
                let traces = traces.clone();
                async move {
                    let trace = traces
                        .get_project_trace(
                            trace_ref.tenant_id,
                            trace_ref.project_id,
                            trace_ref.trace_id,
                        )
                        .await
                        .map_err(|err| err.to_string())?;
                    if trace.spans.len() != 1 {
                        return Err(format!("expected one span, got {}", trace.spans.len()));
                    }
                    Ok(())
                }
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(downstream.consumed, 1);
        assert_eq!(downstream.completed, 1);
        assert_eq!(bus.depth_for_kind(TRACE_INGESTED_KIND).await, Ok(0));
    }

    #[tokio::test]
    async fn direct_ingest_falls_back_to_trace_write_after_downstream_publish_failure() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(FailInitialPublishBus::fail_trace_ingested_once(16));
        let service = IngestService::new(
            artifacts,
            traces.clone(),
            bus.clone(),
            IngestPolicy::default(),
        );
        let request = fixture_request();

        let outcome = service
            .ingest_native(request.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(outcome.ack.accepted_spans, 1);
        assert!(outcome.downstream_queued);
        assert_eq!(bus.depth_for_kind(TRACE_WRITE_BATCH_KIND).await, Ok(1));
        assert_eq!(bus.depth_for_kind(TRACE_INGESTED_KIND).await, Ok(0));

        let report = service
            .drain_trace_writes(10)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(report.consumed, 1);
        assert_eq!(report.written_spans, 0);
        assert_eq!(report.duplicate_spans, 1);
        assert_eq!(report.failed_downstream_publishes, 0);
        assert_eq!(report.downstream_published, 1);
        assert_eq!(bus.depth_for_kind(TRACE_WRITE_BATCH_KIND).await, Ok(0));
        assert_eq!(bus.depth_for_kind(TRACE_INGESTED_KIND).await, Ok(1));

        let trace = traces
            .get_trace(request.scope.tenant_id, request.trace_id)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(trace.spans.len(), 1);
    }

    #[tokio::test]
    async fn ingest_governs_attributes_and_payload_refs() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        let mut denied = BTreeSet::new();
        denied.insert("secret".to_string());
        let service = IngestService::new(
            artifacts,
            traces.clone(),
            bus,
            IngestPolicy {
                inline_payload_bytes: 4,
                denied_attributes: denied,
                ..IngestPolicy::default()
            },
        );
        let mut request = fixture_request();
        request.input = Some(json!({"large": "payload"}));
        request
            .attributes
            .insert("secret".to_string(), json!("drop"));

        service
            .ingest_native(request.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let trace = traces
            .get_trace(request.scope.tenant_id, request.trace_id)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let span = &trace.spans[0];
        assert!(span.input_ref.is_some());
        assert!(!span.attributes.contains_key("secret"));
        assert_eq!(
            span.unmapped_attrs["dropped_attributes"]["secret"],
            json!("[redacted]")
        );
    }

    #[test]
    fn canonical_mapping_classifies_known_namespaces_and_drops_unknown() {
        // Recognized semantic-convention namespaces and typed-field keys map.
        for canonical in [
            "llm.model_name",
            "gen_ai.usage.input_tokens",
            "browser.action",
            "resource.service.name",
            "otel.span.kind",
            "beater.seq",
            "agent.release_id",
            "openinference.span.kind",
            "input.value",
            "output.value",
            "cost_micros",
        ] {
            assert!(
                canonical_mapping::maps_to_canonical(canonical),
                "{canonical} should map to canonical"
            );
        }
        // Vendor / app-specific attributes fail canonical mapping.
        for unmapped in [
            "safe",
            "user.feature_flag",
            "langchain.chain_type",
            "my_app.custom_field",
        ] {
            assert!(
                !canonical_mapping::maps_to_canonical(unmapped),
                "{unmapped} should NOT map to canonical"
            );
        }
    }

    #[tokio::test]
    async fn ingest_records_unmapped_attrs_for_attributes_that_fail_canonical_mapping() {
        // R2.3 golden test: an attribute key that does not correspond to any
        // canonical mapping is preserved verbatim under `unmapped_attrs.unmapped`,
        // while recognized semantic-convention keys are NOT duplicated there.
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        let service = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
        let mut request = fixture_request();
        request.attributes = BTreeMap::from([
            ("llm.model_name".to_string(), json!("gpt-test")),
            ("openinference.span.kind".to_string(), json!("LLM")),
            ("vendor.custom_signal".to_string(), json!("keep-me")),
            ("user.session".to_string(), json!(42)),
        ]);

        service
            .ingest_native(request.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let trace = traces
            .get_trace(request.scope.tenant_id, request.trace_id)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let span = &trace.spans[0];

        // Golden: exactly the two non-canonical keys land in `unmapped`, with
        // their values byte-for-byte preserved; canonical keys do not appear.
        assert_eq!(
            span.unmapped_attrs["unmapped"],
            json!({
                "vendor.custom_signal": "keep-me",
                "user.session": 42,
            })
        );
        // The full attribute bag is still carried on the span (nothing lost).
        assert!(span.attributes.contains_key("llm.model_name"));
        assert!(span.attributes.contains_key("vendor.custom_signal"));
        // Nothing was denied/dropped here.
        assert_eq!(span.unmapped_attrs["dropped_attributes"], json!({}));
    }

    #[tokio::test]
    async fn project_quota_returns_429_semantics_error() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        let service = IngestService::new(
            artifacts,
            traces,
            bus,
            IngestPolicy {
                per_project_event_quota: Some(1),
                ..IngestPolicy::default()
            },
        );

        service
            .ingest_native(fixture_request())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let error = service
            .ingest_native(fixture_request_with_span("span-2"))
            .await
            .err()
            .unwrap_or_else(|| panic!("quota should fail"));

        assert!(matches!(error, IngestError::QuotaExceeded { limit: 1, .. }));
    }

    #[tokio::test]
    async fn sqlite_project_quota_is_shared_across_service_instances() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let policy = IngestPolicy {
            per_project_event_quota: Some(1),
            quota_window_seconds: 86_400,
            ..IngestPolicy::default()
        };
        let quota_path = tempdir.path().join("quotas.sqlite");
        let first = IngestService::new(
            artifacts.clone(),
            Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}"))),
            Arc::new(InMemoryBus::new(16)),
            policy.clone(),
        )
        .with_quota_limiter(Arc::new(
            SqliteQuotaLimiter::open(&quota_path).unwrap_or_else(|err| panic!("{err}")),
        ));
        let second = IngestService::new(
            artifacts,
            Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}"))),
            Arc::new(InMemoryBus::new(16)),
            policy,
        )
        .with_quota_limiter(Arc::new(
            SqliteQuotaLimiter::open(&quota_path).unwrap_or_else(|err| panic!("{err}")),
        ));

        first
            .ingest_native(fixture_request())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let error = second
            .ingest_native(fixture_request_with_span("span-2"))
            .await
            .err()
            .unwrap_or_else(|| panic!("shared quota should fail"));

        assert!(matches!(
            error,
            IngestError::QuotaExceeded {
                limit: 1,
                used: 1,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn buffered_ingest_survives_trace_store_outage_and_drains_after_recovery() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let bus = Arc::new(InMemoryBus::new(16));
        let unavailable = Arc::new(UnavailableTraceStore);
        let outage_service = IngestService::new(
            artifacts.clone(),
            unavailable,
            bus.clone(),
            IngestPolicy::default(),
        );
        let request = fixture_request();

        let outcome = outage_service
            .buffer_native(request.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(outcome.ack.accepted_raw, 1);
        assert_eq!(outcome.ack.accepted_spans, 1);
        assert_eq!(bus.depth_for_kind(TRACE_WRITE_BATCH_KIND).await, Ok(1));

        let retry_report = outage_service
            .drain_trace_writes(10)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(retry_report.consumed, 1);
        assert_eq!(retry_report.failed_writes, 1);
        assert_eq!(retry_report.retried, 1);
        assert_eq!(bus.depth_for_kind(TRACE_WRITE_BATCH_KIND).await, Ok(1));

        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let recovered = IngestService::new(
            artifacts,
            traces.clone(),
            bus.clone(),
            IngestPolicy::default(),
        );
        let report = recovered
            .drain_trace_writes(10)
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(report.consumed, 1);
        assert_eq!(report.written_raw, 1);
        assert_eq!(report.written_spans, 1);
        assert_eq!(report.downstream_published, 1);
        assert_eq!(report.trace_ids, vec![request.trace_id.clone()]);
        assert_eq!(bus.depth_for_kind(TRACE_WRITE_BATCH_KIND).await, Ok(0));
        assert_eq!(bus.depth_for_kind(TRACE_INGESTED_KIND).await, Ok(1));

        let trace = traces
            .get_trace(request.scope.tenant_id, request.trace_id)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(trace.spans.len(), 1);
        assert_eq!(trace.spans[0].name, "agent run");
    }

    #[tokio::test]
    async fn trace_write_before_write_hook_retries_without_touching_trace_store() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        let service = IngestService::new(
            artifacts,
            traces.clone(),
            bus.clone(),
            IngestPolicy::default(),
        );
        let request = fixture_request();

        service
            .buffer_native(request.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let first = service
            .drain_trace_writes_with_hook(10, |_| async {
                Err("test trace.write pre-write failure".to_string())
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(first.consumed, 1);
        assert_eq!(first.failed_writes, 1);
        assert_eq!(first.written_spans, 0);
        assert_eq!(first.downstream_published, 0);
        assert_eq!(first.retried, 1);
        assert_eq!(bus.depth_for_kind(TRACE_WRITE_BATCH_KIND).await, Ok(1));
        assert_eq!(bus.depth_for_kind(TRACE_INGESTED_KIND).await, Ok(0));
        let empty_trace = traces
            .get_trace(request.scope.tenant_id.clone(), request.trace_id.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(empty_trace.spans.len(), 0);

        let second = service
            .drain_trace_writes(10)
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(second.consumed, 1);
        assert_eq!(second.written_spans, 1);
        assert_eq!(second.downstream_published, 1);
        assert_eq!(bus.depth_for_kind(TRACE_WRITE_BATCH_KIND).await, Ok(0));
        assert_eq!(bus.depth_for_kind(TRACE_INGESTED_KIND).await, Ok(1));
        let trace = traces
            .get_trace(request.scope.tenant_id, request.trace_id)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(trace.spans.len(), 1);
    }

    #[tokio::test]
    async fn trace_write_retries_after_downstream_publish_failure_without_double_writes() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(FailInitialPublishBus::fail_trace_ingested_once(16));
        let service = IngestService::new(
            artifacts,
            traces.clone(),
            bus.clone(),
            IngestPolicy::default(),
        );
        let request = fixture_request();

        service
            .buffer_native(request.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let first = service
            .drain_trace_writes(10)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(first.consumed, 1);
        assert_eq!(first.written_spans, 1);
        assert_eq!(first.failed_writes, 0);
        assert_eq!(first.failed_downstream_publishes, 1);
        assert_eq!(first.retried, 1);
        assert_eq!(bus.depth_for_kind(TRACE_WRITE_BATCH_KIND).await, Ok(1));
        assert_eq!(bus.depth_for_kind(TRACE_INGESTED_KIND).await, Ok(0));
        let trace = traces
            .get_trace(request.scope.tenant_id.clone(), request.trace_id.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(trace.spans.len(), 1);

        let second = service
            .drain_trace_writes(10)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(second.consumed, 1);
        assert_eq!(second.written_spans, 0);
        assert_eq!(second.duplicate_spans, 1);
        assert_eq!(second.failed_writes, 0);
        assert_eq!(second.failed_downstream_publishes, 0);
        assert_eq!(second.downstream_published, 1);
        assert_eq!(bus.depth_for_kind(TRACE_WRITE_BATCH_KIND).await, Ok(0));
        assert_eq!(bus.depth_for_kind(TRACE_INGESTED_KIND).await, Ok(1));
        let trace = traces
            .get_trace(request.scope.tenant_id, request.trace_id)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(trace.spans.len(), 1);
    }

    #[tokio::test]
    async fn trace_ingested_worker_processes_downstream_lane_after_write_drain() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        let service = IngestService::new(
            artifacts,
            traces.clone(),
            bus.clone(),
            IngestPolicy::default(),
        );
        let request = fixture_request();

        service
            .buffer_native(request.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let write_report = service
            .drain_trace_writes(10)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(write_report.downstream_published, 1);
        assert_eq!(bus.depth_for_kind(TRACE_INGESTED_KIND).await, Ok(1));

        let downstream = service
            .drain_trace_ingested(10, |trace_ref| {
                let traces = traces.clone();
                async move {
                    let trace = traces
                        .get_project_trace(
                            trace_ref.tenant_id,
                            trace_ref.project_id,
                            trace_ref.trace_id,
                        )
                        .await
                        .map_err(|err| err.to_string())?;
                    if trace.spans.len() != 1 {
                        return Err(format!("expected one span, got {}", trace.spans.len()));
                    }
                    Ok(())
                }
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(downstream.consumed, 1);
        assert_eq!(downstream.completed, 1);
        assert_eq!(downstream.failed_work, 0);
        assert_eq!(bus.depth_for_kind(TRACE_INGESTED_KIND).await, Ok(0));
    }

    #[tokio::test]
    async fn trace_ingested_unacked_work_recovers_after_sqlite_bus_reopen() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let trace_path = tempdir.path().join("traces.sqlite");
        let bus_path = tempdir.path().join("bus.sqlite");
        let traces =
            Arc::new(SqliteTraceStore::open(&trace_path).unwrap_or_else(|err| panic!("{err}")));
        let bus =
            Arc::new(SqliteDurableBus::open(&bus_path, 16).unwrap_or_else(|err| panic!("{err}")));
        let service = IngestService::new(
            artifacts.clone(),
            traces.clone(),
            bus.clone(),
            IngestPolicy::default(),
        );
        let request = fixture_request();

        service
            .buffer_native(request.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let write_report = service
            .drain_trace_writes(10)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(write_report.downstream_published, 1);
        assert_eq!(bus.depth_for_kind(TRACE_INGESTED_KIND).await, Ok(1));

        let leased = bus
            .consume_kind_batch(TRACE_INGESTED_KIND, 1)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(leased.len(), 1);
        assert_eq!(bus.depth_for_kind(TRACE_INGESTED_KIND).await, Ok(1));
        let status = service
            .queue_status(
                request.scope.tenant_id.clone(),
                request.scope.project_id.clone(),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(status.trace_ingested_depth, 1);
        drop(service);
        drop(bus);
        drop(traces);

        let recovered_bus =
            Arc::new(SqliteDurableBus::open(&bus_path, 16).unwrap_or_else(|err| panic!("{err}")));
        assert_eq!(
            recovered_bus.depth_for_kind(TRACE_INGESTED_KIND).await,
            Ok(1)
        );
        let recovered_traces =
            Arc::new(SqliteTraceStore::open(&trace_path).unwrap_or_else(|err| panic!("{err}")));
        let recovered = IngestService::new(
            artifacts,
            recovered_traces.clone(),
            recovered_bus.clone(),
            IngestPolicy::default(),
        );
        let downstream = recovered
            .drain_trace_ingested(10, move |trace_ref| {
                let recovered_traces = recovered_traces.clone();
                async move {
                    let trace = recovered_traces
                        .get_project_trace(
                            trace_ref.tenant_id,
                            trace_ref.project_id,
                            trace_ref.trace_id,
                        )
                        .await
                        .map_err(|err| err.to_string())?;
                    if trace.spans.len() != 1 {
                        return Err(format!("expected one span, got {}", trace.spans.len()));
                    }
                    Ok(())
                }
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(downstream.completed, 1);
        assert_eq!(
            recovered_bus.depth_for_kind(TRACE_INGESTED_KIND).await,
            Ok(0)
        );
    }

    #[tokio::test]
    async fn trace_ingested_consumer_restart_then_dlq_replay_recovers_work() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let trace_path = tempdir.path().join("traces.sqlite");
        let bus_path = tempdir.path().join("bus.sqlite");
        let traces =
            Arc::new(SqliteTraceStore::open(&trace_path).unwrap_or_else(|err| panic!("{err}")));
        let bus =
            Arc::new(SqliteDurableBus::open(&bus_path, 16).unwrap_or_else(|err| panic!("{err}")));
        let service = IngestService::new(
            artifacts.clone(),
            traces.clone(),
            bus.clone(),
            IngestPolicy::default(),
        );
        let request = fixture_request();
        let tenant_id = request.scope.tenant_id.clone();
        let project_id = request.scope.project_id.clone();
        let trace_id = request.trace_id.clone();

        service
            .buffer_native(request)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let write_report = service
            .drain_trace_writes(10)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(write_report.downstream_published, 1);
        assert_eq!(bus.depth_for_kind(TRACE_INGESTED_KIND).await, Ok(1));

        let leased = bus
            .consume_kind_batch(TRACE_INGESTED_KIND, 1)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(leased.len(), 1);
        assert_eq!(bus.depth_for_kind(TRACE_INGESTED_KIND).await, Ok(1));
        drop(service);
        drop(bus);
        drop(traces);

        let restarted_bus =
            Arc::new(SqliteDurableBus::open(&bus_path, 16).unwrap_or_else(|err| panic!("{err}")));
        assert_eq!(
            restarted_bus.depth_for_kind(TRACE_INGESTED_KIND).await,
            Ok(1)
        );
        let restarted_traces =
            Arc::new(SqliteTraceStore::open(&trace_path).unwrap_or_else(|err| panic!("{err}")));
        let restarted = IngestService::new(
            artifacts,
            restarted_traces.clone(),
            restarted_bus.clone(),
            IngestPolicy::default(),
        );

        for attempt in 1..=3 {
            let report = restarted
                .drain_trace_ingested(10, |_| async {
                    Err("simulated downstream outage after consumer restart".to_string())
                })
                .await
                .unwrap_or_else(|err| panic!("{err}"));
            assert_eq!(report.consumed, 1);
            assert_eq!(report.failed_work, 1);
            if attempt < 3 {
                assert_eq!(report.retried, 1);
                assert_eq!(report.dead_lettered, 0);
            } else {
                assert_eq!(report.retried, 0);
                assert_eq!(report.dead_lettered, 1);
            }
        }

        let status = restarted
            .queue_status(tenant_id.clone(), project_id.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(status.trace_ingested_depth, 0);
        assert_eq!(status.dead_letters.len(), 1);
        assert!(status.dead_letters[0]
            .reason
            .contains("simulated downstream outage after consumer restart"));
        let message_id = status.dead_letters[0].message.message_id.clone();

        let replay = restarted
            .replay_dead_letter(&tenant_id, &project_id, &message_id, true)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(replay.ack.accepted);
        assert_eq!(
            restarted_bus.depth_for_kind(TRACE_INGESTED_KIND).await,
            Ok(1)
        );
        let replayed = restarted
            .drain_trace_ingested(10, move |trace_ref| {
                let restarted_traces = restarted_traces.clone();
                let trace_id = trace_id.clone();
                async move {
                    let trace = restarted_traces
                        .get_project_trace(
                            trace_ref.tenant_id,
                            trace_ref.project_id,
                            trace_ref.trace_id,
                        )
                        .await
                        .map_err(|err| err.to_string())?;
                    if trace.trace_id != trace_id {
                        return Err(format!("unexpected replayed trace {}", trace.trace_id));
                    }
                    if trace.spans.len() != 1 {
                        return Err(format!("expected one span, got {}", trace.spans.len()));
                    }
                    Ok(())
                }
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(replayed.completed, 1);
        let final_status = restarted
            .queue_status(tenant_id, project_id)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(final_status.trace_ingested_depth, 0);
        assert!(final_status.dead_letters.is_empty());
    }

    #[tokio::test]
    async fn trace_write_worker_dlqs_invalid_payload_without_consuming_other_lanes() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let mut poison = BusMessage::new(
            tenant.clone(),
            project.clone(),
            IdempotencyKey::new("poison").unwrap_or_else(|err| panic!("{err}")),
            TRACE_WRITE_BATCH_KIND,
            b"not-json".to_vec(),
        );
        poison.max_attempts = 2;
        bus.publish(poison)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        bus.publish(BusMessage::new(
            tenant,
            project,
            IdempotencyKey::new("downstream").unwrap_or_else(|err| panic!("{err}")),
            TRACE_INGESTED_KIND,
            b"downstream".to_vec(),
        ))
        .await
        .unwrap_or_else(|err| panic!("{err}"));

        let service = IngestService::new(artifacts, traces, bus.clone(), IngestPolicy::default());
        let first = service
            .drain_trace_writes(10)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(first.invalid_messages, 1);
        assert_eq!(first.retried, 1);
        assert_eq!(bus.depth_for_kind(TRACE_WRITE_BATCH_KIND).await, Ok(1));
        assert_eq!(bus.depth_for_kind(TRACE_INGESTED_KIND).await, Ok(1));

        let second = service
            .drain_trace_writes(10)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(second.invalid_messages, 1);
        assert_eq!(second.dead_lettered, 1);
        assert_eq!(bus.depth_for_kind(TRACE_WRITE_BATCH_KIND).await, Ok(0));
        assert_eq!(bus.depth_for_kind(TRACE_INGESTED_KIND).await, Ok(1));
        let dlq = bus.dlq().await.unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(dlq.len(), 1);
        assert!(dlq[0].reason.contains("invalid trace.write_batch payload"));
    }

    #[tokio::test]
    async fn trace_ingested_worker_dlqs_invalid_payload_without_consuming_write_lane() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let mut poison = BusMessage::new(
            tenant.clone(),
            project.clone(),
            IdempotencyKey::new("poison-downstream").unwrap_or_else(|err| panic!("{err}")),
            TRACE_INGESTED_KIND,
            b"not-json".to_vec(),
        );
        poison.max_attempts = 1;
        bus.publish(poison)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        bus.publish(BusMessage::new(
            tenant,
            project,
            IdempotencyKey::new("trace-write").unwrap_or_else(|err| panic!("{err}")),
            TRACE_WRITE_BATCH_KIND,
            b"trace-write".to_vec(),
        ))
        .await
        .unwrap_or_else(|err| panic!("{err}"));

        let service = IngestService::new(artifacts, traces, bus.clone(), IngestPolicy::default());
        let report = service
            .drain_trace_ingested(10, |_| async { Ok(()) })
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(report.invalid_messages, 1);
        assert_eq!(report.dead_lettered, 1);
        assert_eq!(bus.depth_for_kind(TRACE_INGESTED_KIND).await, Ok(0));
        assert_eq!(bus.depth_for_kind(TRACE_WRITE_BATCH_KIND).await, Ok(1));
        let dlq = bus.dlq().await.unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(dlq.len(), 1);
        assert!(dlq[0].reason.contains("invalid trace.ingested payload"));
    }

    #[tokio::test]
    async fn buffered_ingest_maps_bus_backpressure_to_typed_error() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(0));
        let service = IngestService::new(artifacts, traces, bus, IngestPolicy::default());

        let error = service
            .buffer_native(fixture_request())
            .await
            .err()
            .unwrap_or_else(|| panic!("backpressure should fail"));

        assert!(matches!(error, IngestError::Backpressure { capacity: 0 }));
    }

    /// R4.5: the cardinality cap rejects a span carrying more attributes than the
    /// policy's `max_attributes` with a typed `TooManyAttributes` error (the 422
    /// semantics) before anything is written.
    #[tokio::test]
    async fn ingest_rejects_attribute_cardinality_over_cap() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        let service = IngestService::new(
            artifacts,
            traces.clone(),
            bus.clone(),
            IngestPolicy {
                max_attributes: 2,
                ..IngestPolicy::default()
            },
        );
        let mut request = fixture_request();
        request.attributes = BTreeMap::from([
            ("a".to_string(), json!(1)),
            ("b".to_string(), json!(2)),
            ("c".to_string(), json!(3)),
        ]);

        let error = service
            .ingest_native(request.clone())
            .await
            .err()
            .unwrap_or_else(|| panic!("over-cap attributes should be rejected"));
        assert!(matches!(
            error,
            IngestError::TooManyAttributes { count: 3, limit: 2 }
        ));
        // Nothing was written and nothing queued: the cap fires before assembly.
        assert_eq!(bus.depth().await, Ok(0));
        let trace = traces
            .get_trace(request.scope.tenant_id, request.trace_id)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(trace.spans.len(), 0);
    }

    /// R4.5: an allow-list keeps only listed attribute keys and records the rest
    /// under `dropped_attributes`, while a denied key is dropped even if it would
    /// otherwise be allowed.
    #[tokio::test]
    async fn ingest_allow_list_drops_unlisted_and_denied_attributes() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        let service = IngestService::new(
            artifacts,
            traces.clone(),
            bus,
            IngestPolicy {
                allowed_attributes: Some(BTreeSet::from([
                    "keep".to_string(),
                    "secret".to_string(),
                ])),
                denied_attributes: BTreeSet::from(["secret".to_string()]),
                ..IngestPolicy::default()
            },
        );
        let mut request = fixture_request();
        request.attributes = BTreeMap::from([
            ("keep".to_string(), json!("yes")),
            ("drop_me".to_string(), json!("unlisted")),
            ("secret".to_string(), json!("denied")),
        ]);

        service
            .ingest_native(request.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let trace = traces
            .get_trace(request.scope.tenant_id, request.trace_id)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let span = &trace.spans[0];
        assert_eq!(span.attributes.get("keep"), Some(&json!("yes")));
        assert!(!span.attributes.contains_key("drop_me"));
        // Deny wins over allow.
        assert!(!span.attributes.contains_key("secret"));
        let dropped = &span.unmapped_attrs["dropped_attributes"];
        assert_eq!(dropped["drop_me"], json!("unlisted"));
        assert_eq!(dropped["secret"], json!("[redacted]"));
        assert!(dropped.get("keep").is_none());
    }

    /// #126: the default ingest policy must not preserve the highest-risk
    /// observability attributes in the canonical attribute bag.
    #[tokio::test]
    async fn default_policy_drops_standard_sensitive_attributes() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        let service = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
        let mut request = fixture_request();
        request.attributes = BTreeMap::from([
            ("safe".to_string(), json!("keep")),
            (
                "http.request.header.authorization".to_string(),
                json!("Bearer sk-live"),
            ),
            (
                "http.request.header.cookie".to_string(),
                json!("sid=secret"),
            ),
            (
                "url.full".to_string(),
                json!("https://example.test/callback?token=secret"),
            ),
            ("gen_ai.prompt".to_string(), json!("user secret prompt")),
        ]);

        service
            .ingest_native(request.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let trace = traces
            .get_trace(request.scope.tenant_id, request.trace_id)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let span = &trace.spans[0];
        assert_eq!(span.attributes.get("safe"), Some(&json!("keep")));
        let dropped = &span.unmapped_attrs["dropped_attributes"];
        for key in [
            "http.request.header.authorization",
            "http.request.header.cookie",
            "url.full",
            "gen_ai.prompt",
        ] {
            assert!(
                !span.attributes.contains_key(key),
                "{key} must not remain readable as a canonical attribute"
            );
            assert!(
                dropped.get(key).is_some(),
                "{key} should be recorded as dropped provenance"
            );
            assert_eq!(
                dropped[key],
                json!("[redacted]"),
                "{key} value must not survive in dropped provenance"
            );
        }
    }

    #[test]
    fn native_importer_preserves_raw_bytes_as_sensitive() {
        let importer = NativeSpansImporter;
        let scope = TenantScope::new(
            TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
        );

        let request = importer
            .normalize(&scope, br#"{"spans":[]}"#, None)
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(request.redaction_class, RedactionClass::Sensitive);
    }

    /// R4.1: the in-process queue-depth gauge (`DurableBus::depth_for_kind`, NOT
    /// the R13 metrics layer) reflects buffered backlog growing per buffered
    /// ingest and shrinking as the write lane drains.
    ///
    /// NOTE (R13 load test): a full sustained-throughput load test that asserts
    /// the queue-depth gauge stays bounded under backpressure belongs to the R13
    /// metrics/observability layer (a separate track) and is intentionally out of
    /// scope here; this in-process assertion documents the gauge contract that the
    /// load test will drive.
    #[tokio::test]
    async fn queue_depth_gauge_tracks_buffered_backlog_in_process() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        let service = IngestService::new(
            artifacts,
            traces.clone(),
            bus.clone(),
            IngestPolicy::default(),
        );

        // Gauge starts empty.
        assert_eq!(bus.depth_for_kind(TRACE_WRITE_BATCH_KIND).await, Ok(0));

        // Each buffered ingest enqueues one trace.write_batch -> gauge climbs.
        service
            .buffer_native(fixture_request_with_span("span-1"))
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(bus.depth_for_kind(TRACE_WRITE_BATCH_KIND).await, Ok(1));
        let status_one = service
            .queue_status(
                TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
                ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(status_one.trace_write_depth, 1);

        service
            .buffer_native(fixture_request_with_span("span-2"))
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(bus.depth_for_kind(TRACE_WRITE_BATCH_KIND).await, Ok(2));

        // Draining the write lane moves work downstream: write gauge falls to 0,
        // the ingested gauge rises (one downstream message per buffered write).
        let report = service
            .drain_trace_writes(10)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(report.consumed, 2);
        assert_eq!(bus.depth_for_kind(TRACE_WRITE_BATCH_KIND).await, Ok(0));
        assert_eq!(bus.depth_for_kind(TRACE_INGESTED_KIND).await, Ok(2));
    }

    #[test]
    fn trace_completion_is_state_machine() {
        assert_eq!(
            trace_completion_state(TraceCompletionInput {
                root_span_ended: true,
                open_child_spans: 0,
                idle_for: Duration::seconds(1),
                idle_timeout: Duration::seconds(5),
                late_window_closed: true,
            }),
            TraceCompletionState::Complete
        );
        assert_eq!(
            trace_completion_state(TraceCompletionInput {
                root_span_ended: false,
                open_child_spans: 0,
                idle_for: Duration::seconds(10),
                idle_timeout: Duration::seconds(5),
                late_window_closed: false,
            }),
            TraceCompletionState::IdleComplete
        );
    }

    /// R4.7: an out-of-order, late-arriving distributed trace. The root span
    /// arrives last (children first), proving assembly is ordering-agnostic, and
    /// the ended root with no open children inside the late window is `RootEnded`.
    #[test]
    fn assemble_completion_out_of_order_root_last_is_root_ended() {
        let now = base_time();
        // Child spans arrive first (out of order), both closed.
        let spans = vec![
            completion_span("trace", "child-b", Some("root"), now, Some(now + ms(20))),
            completion_span("trace", "child-a", Some("root"), now, Some(now + ms(10))),
            // Root arrives last, ended just now -> within the late window.
            completion_span("trace", "root", None, now, Some(now + ms(30))),
        ];
        // now == latest end => idle_for 0, below the 10s late window.
        let state =
            assemble_trace_completion(&spans, now + ms(30), TraceCompletionConfig::default());
        assert_eq!(state, TraceCompletionState::RootEnded);
    }

    /// R4.7: a late-arriving child keeps the trace open even after the root ended,
    /// until the late window closes; once it closes with the root ended and no
    /// open children, the trace is `Complete`.
    #[test]
    fn assemble_completion_late_window_closes_to_complete() {
        let now = base_time();
        let root_end = now + ms(30);
        let spans = vec![
            completion_span("trace", "root", None, now, Some(root_end)),
            completion_span("trace", "child", Some("root"), now, Some(now + ms(20))),
        ];
        // Evaluate well past the late window with all children closed -> Complete.
        let state = assemble_trace_completion(
            &spans,
            root_end + Duration::seconds(30),
            TraceCompletionConfig::default(),
        );
        assert_eq!(state, TraceCompletionState::Complete);
    }

    /// R4.7: a root that has NOT ended yet, but whose only child closed long ago,
    /// crosses the late window with the open-child count still nonzero because the
    /// root itself is open -> `LateWindowClosed` (not Complete, not RootEnded).
    #[test]
    fn assemble_completion_open_root_after_late_window_is_late_window_closed() {
        let now = base_time();
        // Root present but still open (no end_time); one closed child.
        let spans = vec![
            completion_span("trace", "root", None, now, None),
            completion_span("trace", "child", Some("root"), now, Some(now + ms(20))),
        ];
        // Past the late window but before the idle timeout (60s): the open root
        // is not a child, so open_child_spans == 0 and idle >= late_window but
        // < idle_timeout, and root not ended -> LateWindowClosed.
        let state = assemble_trace_completion(
            &spans,
            now + Duration::seconds(15),
            TraceCompletionConfig::default(),
        );
        assert_eq!(state, TraceCompletionState::LateWindowClosed);
    }

    /// R4.7: clock skew where a span's end_time is in the *future* relative to the
    /// ingest clock must not produce a negative idle gap nor spuriously complete.
    #[test]
    fn assemble_completion_clamps_clock_skewed_future_end() {
        let now = base_time();
        // The only child ends 5 minutes in the "future" vs now (clock skew).
        let spans = vec![
            completion_span("trace", "root", None, now, None),
            completion_span(
                "trace",
                "child",
                Some("root"),
                now,
                Some(now + Duration::minutes(5)),
            ),
        ];
        // idle_for is clamped to zero, so the late window has NOT closed and the
        // open child keeps the trace Open despite the skew.
        let state = assemble_trace_completion(&spans, now, TraceCompletionConfig::default());
        assert_eq!(state, TraceCompletionState::Open);
    }

    /// R4.7: trace completion is derived from the stored span set — after a
    /// native ingest, re-assessing the persisted trace reports its state.
    #[tokio::test]
    async fn native_ingest_assembles_trace_completion_state() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        let now = base_time();
        let service = IngestService::new(artifacts, traces, bus, IngestPolicy::default())
            .with_clock(Arc::new(FixedClock::new(now + Duration::seconds(30))));
        // A parentless span with an end_time -> the assembled root has ended.
        let mut request = fixture_request();
        request.parent_span_id = None;
        request.start_time = Some(now);
        request.end_time = Some(now + ms(10));

        let outcome = service
            .ingest_native(request.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(outcome.ack.accepted_spans, 1);

        // 30s after the only (root) span ended: past the 10s late window, root
        // ended, no open children -> Complete.
        let reassessed = service
            .assess_trace_completion(
                request.scope.tenant_id.clone(),
                request.scope.project_id.clone(),
                request.trace_id.clone(),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(reassessed, TraceCompletionState::Complete);
    }

    fn base_time() -> Timestamp {
        Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
            .single()
            .unwrap_or_else(|| panic!("valid timestamp"))
    }

    fn ms(millis: i64) -> Duration {
        Duration::milliseconds(millis)
    }

    fn completion_span(
        trace_id: &str,
        span_id: &str,
        parent_span_id: Option<&str>,
        start_time: Timestamp,
        end_time: Option<Timestamp>,
    ) -> CanonicalSpan {
        CanonicalSpan {
            schema_version: CANONICAL_SCHEMA_VERSION,
            normalizer_version: "test".to_string(),
            tenant_id: TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            project_id: ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            environment_id: EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
            trace_id: TraceId::new(trace_id).unwrap_or_else(|err| panic!("{err}")),
            span_id: SpanId::new(span_id).unwrap_or_else(|err| panic!("{err}")),
            parent_span_id: parent_span_id
                .map(|id| SpanId::new(id).unwrap_or_else(|err| panic!("{err}"))),
            seq: 1,
            kind: AgentSpanKind::AgentRun,
            name: "span".to_string(),
            status: SpanStatus::Ok,
            start_time,
            end_time,
            model: None,
            cost: None,
            tokens: None,
            input_ref: None,
            output_ref: None,
            attributes: BTreeMap::new(),
            unmapped_attrs: json!({}),
            raw_ref: ArtifactRef {
                artifact_id: beater_core::ArtifactId::new("raw")
                    .unwrap_or_else(|err| panic!("{err}")),
                uri: "artifact://tenant/project/raw".to_string(),
                sha256: Sha256Hash::new("ab".repeat(32)).unwrap_or_else(|err| panic!("{err}")),
                size_bytes: 2,
                mime_type: "application/json".to_string(),
                redaction_class: RedactionClass::Internal,
            },
        }
    }

    fn fixture_request() -> NativeIngestRequest {
        fixture_request_with_span("span")
    }

    fn fixture_request_with_span(span_id: &str) -> NativeIngestRequest {
        let scope = TenantScope::new(
            TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
        );
        NativeIngestRequest {
            scope,
            trace_id: TraceId::new("trace").unwrap_or_else(|err| panic!("{err}")),
            span_id: SpanId::new(span_id).unwrap_or_else(|err| panic!("{err}")),
            parent_span_id: None,
            seq: 1,
            kind: AgentSpanKind::AgentRun,
            name: "agent run".to_string(),
            status: SpanStatus::Ok,
            start_time: Some(Utc::now()),
            end_time: Some(Utc::now()),
            model: Some(ModelRef {
                provider: "openai".to_string(),
                name: "gpt-test".to_string(),
            }),
            cost: None,
            tokens: Some(TokenCounts {
                input: 10,
                output: 5,
                reasoning: 0,
                cache_read: 0,
            }),
            input: Some(json!({ "question": "hi" })),
            output: Some(json!({ "answer": "hello" })),
            attributes: BTreeMap::from([("safe".to_string(), json!(true))]),
            redaction_class: RedactionClass::Sensitive,
            idempotency_key: None,
            auth_context: None,
        }
    }

    /// R4.7: a trace the store has never seen reports `NotFound`. The completion
    /// contract promises `Open` (the trace simply has not started converging), so
    /// `assess_trace_completion` must map that error into a state, not surface it.
    #[tokio::test]
    async fn assess_trace_completion_maps_not_found_to_open() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(NotFoundTraceStore);
        let bus = Arc::new(InMemoryBus::new(16));
        let service = IngestService::new(artifacts, traces, bus, IngestPolicy::default());

        let state = service
            .assess_trace_completion(
                TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
                ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
                TraceId::new("missing").unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("expected Open, got error: {err}"));
        assert_eq!(state, TraceCompletionState::Open);
    }

    /// Trace store that reports every trace as `NotFound` — exercises the
    /// completion contract's "no stored spans yet -> Open" mapping.
    struct NotFoundTraceStore;

    #[async_trait::async_trait]
    impl TraceStore for NotFoundTraceStore {
        async fn write_batch(
            &self,
            _batch: CanonicalTraceBatch,
        ) -> beater_store::StoreResult<WriteAck> {
            Err(StoreError::NotFound("trace".to_string()))
        }

        async fn get_trace(
            &self,
            _tenant: TenantId,
            _trace: TraceId,
        ) -> beater_store::StoreResult<TraceView> {
            Err(StoreError::NotFound("trace".to_string()))
        }

        async fn get_project_trace(
            &self,
            _tenant: TenantId,
            _project: ProjectId,
            _trace: TraceId,
        ) -> beater_store::StoreResult<TraceView> {
            Err(StoreError::NotFound("trace".to_string()))
        }

        async fn get_raw_envelope(
            &self,
            _tenant: TenantId,
            _project: ProjectId,
            _idempotency_key: IdempotencyKey,
        ) -> beater_store::StoreResult<Option<RawEnvelope>> {
            Err(StoreError::NotFound("trace".to_string()))
        }

        async fn query_runs(
            &self,
            _tenant: TenantId,
            _filter: RunFilter,
            _page: PageRequest,
        ) -> beater_store::StoreResult<Page<RunSummary>> {
            Err(StoreError::NotFound("trace".to_string()))
        }

        async fn query_spans(
            &self,
            _tenant: TenantId,
            _filter: SpanFilter,
            _page: PageRequest,
        ) -> beater_store::StoreResult<Page<SpanSummary>> {
            Err(StoreError::NotFound("trace".to_string()))
        }
    }

    struct UnavailableTraceStore;

    #[async_trait::async_trait]
    impl TraceStore for UnavailableTraceStore {
        async fn write_batch(
            &self,
            _batch: CanonicalTraceBatch,
        ) -> beater_store::StoreResult<WriteAck> {
            Err(StoreError::Backend("trace store unavailable".to_string()))
        }

        async fn get_trace(
            &self,
            _tenant: TenantId,
            _trace: TraceId,
        ) -> beater_store::StoreResult<TraceView> {
            Err(StoreError::Backend("trace store unavailable".to_string()))
        }

        async fn get_project_trace(
            &self,
            _tenant: TenantId,
            _project: ProjectId,
            _trace: TraceId,
        ) -> beater_store::StoreResult<TraceView> {
            Err(StoreError::Backend("trace store unavailable".to_string()))
        }

        async fn get_raw_envelope(
            &self,
            _tenant: TenantId,
            _project: ProjectId,
            _idempotency_key: IdempotencyKey,
        ) -> beater_store::StoreResult<Option<RawEnvelope>> {
            Err(StoreError::Backend("trace store unavailable".to_string()))
        }

        async fn query_runs(
            &self,
            _tenant: TenantId,
            _filter: RunFilter,
            _page: PageRequest,
        ) -> beater_store::StoreResult<Page<RunSummary>> {
            Err(StoreError::Backend("trace store unavailable".to_string()))
        }

        async fn query_spans(
            &self,
            _tenant: TenantId,
            _filter: SpanFilter,
            _page: PageRequest,
        ) -> beater_store::StoreResult<Page<SpanSummary>> {
            Err(StoreError::Backend("trace store unavailable".to_string()))
        }
    }

    // -----------------------------------------------------------------------
    // IngestPolicy::validate() tests
    // -----------------------------------------------------------------------

    #[test]
    fn policy_validate_accepts_defaults() {
        IngestPolicy::default()
            .validate()
            .unwrap_or_else(|err| panic!("default policy must be valid: {err}"));
    }

    #[test]
    fn policy_validate_rejects_inline_exceeds_raw() {
        let policy = IngestPolicy {
            max_raw_payload_bytes: 100,
            inline_payload_bytes: 200,
            ..IngestPolicy::default()
        };
        let err = policy.validate().unwrap_err();
        assert!(
            err.to_string().contains("inline_payload_bytes"),
            "expected mention of inline_payload_bytes in: {err}"
        );
    }

    #[test]
    fn policy_validate_rejects_zero_max_raw_payload_bytes() {
        let policy = IngestPolicy {
            max_raw_payload_bytes: 0,
            inline_payload_bytes: 0,
            ..IngestPolicy::default()
        };
        let err = policy.validate().unwrap_err();
        assert!(
            err.to_string().contains("max_raw_payload_bytes"),
            "expected mention of max_raw_payload_bytes in: {err}"
        );
    }

    #[test]
    fn policy_validate_rejects_zero_max_attributes() {
        let policy = IngestPolicy {
            max_attributes: 0,
            ..IngestPolicy::default()
        };
        let err = policy.validate().unwrap_err();
        assert!(
            err.to_string().contains("max_attributes"),
            "expected mention of max_attributes in: {err}"
        );
    }

    #[test]
    fn policy_validate_rejects_zero_idle_timeout() {
        let policy = IngestPolicy {
            trace_completion: TraceCompletionConfig {
                idle_timeout: Duration::zero(),
                ..TraceCompletionConfig::default()
            },
            ..IngestPolicy::default()
        };
        let err = policy.validate().unwrap_err();
        assert!(
            err.to_string().contains("trace_idle_timeout"),
            "expected mention of trace_idle_timeout in: {err}"
        );
    }

    #[test]
    fn policy_validate_rejects_zero_late_window() {
        let policy = IngestPolicy {
            trace_completion: TraceCompletionConfig {
                late_window: Duration::zero(),
                ..TraceCompletionConfig::default()
            },
            ..IngestPolicy::default()
        };
        let err = policy.validate().unwrap_err();
        assert!(
            err.to_string().contains("trace_late_window"),
            "expected mention of trace_late_window in: {err}"
        );
    }

    /// R248: a span that exceeds a custom (small) raw payload cap is rejected with
    /// PayloadTooLarge — proving that the max_raw_payload_bytes knob is wired
    /// through to ingest enforcement.
    #[tokio::test]
    async fn ingest_rejects_payload_over_custom_raw_limit() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        // 64 bytes is far below the ~300-500 bytes a fixture request serialises to.
        let service = IngestService::new(
            artifacts,
            traces,
            bus.clone(),
            IngestPolicy {
                max_raw_payload_bytes: 64,
                inline_payload_bytes: 32,
                ..IngestPolicy::default()
            },
        );
        let err = service
            .ingest_native(fixture_request())
            .await
            .err()
            .unwrap_or_else(|| panic!("should be rejected over the 64-byte cap"));
        assert!(
            matches!(err, IngestError::PayloadTooLarge { .. }),
            "expected PayloadTooLarge, got {err}"
        );
        assert_eq!(
            bus.depth().await,
            Ok(0),
            "nothing should be queued on rejection"
        );
    }

    /// R248: the same span that fails under a tiny custom cap succeeds under a
    /// raised cap — proving that raising max_raw_payload_bytes changes behavior.
    #[tokio::test]
    async fn ingest_accepts_payload_within_raised_raw_limit() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        // 4 KiB — large enough for the fixture request, much smaller than the 1 MiB default.
        let service = IngestService::new(
            artifacts,
            traces,
            bus.clone(),
            IngestPolicy {
                max_raw_payload_bytes: 4096,
                ..IngestPolicy::default()
            },
        );
        let outcome = service
            .ingest_native(fixture_request())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(outcome.ack.accepted_raw, 1);
        assert_eq!(bus.depth().await, Ok(1));
    }
}
