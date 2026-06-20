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
    Store(#[from] StoreError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Clone)]
pub struct IngestService {
    artifacts: Arc<dyn ArtifactStore>,
    traces: Arc<dyn TraceStore>,
    bus: Arc<dyn DurableBus>,
    policy: IngestPolicy,
    quota_limiter: Arc<dyn QuotaLimiter>,
    clock: Arc<dyn Clock>,
}

impl IngestService {
    pub fn new(
        artifacts: Arc<dyn ArtifactStore>,
        traces: Arc<dyn TraceStore>,
        bus: Arc<dyn DurableBus>,
        policy: IngestPolicy,
    ) -> Self {
        Self {
            artifacts,
            traces,
            bus,
            policy,
            quota_limiter: Arc::new(InMemoryQuotaLimiter::new()),
            clock: Arc::new(SystemClock),
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
            Ok(published) => published > 0,
            Err(_) => {
                // The trace is already durable; queue a write retry so the worker can
                // idempotently re-write and publish downstream without returning a 500.
                self.publish_trace_write(&prepared)
                    .await
                    .map(|publish| publish.accepted || publish.duplicate)
                    .unwrap_or(false)
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
        let messages = self
            .bus
            .consume_kind_batch(TRACE_WRITE_BATCH_KIND, limit)
            .await
            .map_err(map_bus_error)?;
        self.drain_trace_write_messages(messages).await
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
        self.drain_trace_write_messages(messages).await
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

    async fn drain_trace_write_messages(
        &self,
        messages: Vec<BusMessage>,
    ) -> Result<TraceWriteDrainReport, IngestError> {
        let mut report = TraceWriteDrainReport {
            consumed: messages.len(),
            ..TraceWriteDrainReport::default()
        };
        for message in messages {
            self.process_trace_write_message(message, &mut report)
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
                    self.retry_or_dlq_trace_ingested_counted(
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
                self.retry_or_dlq_trace_ingested_counted(
                    message,
                    reason,
                    will_dead_letter,
                    &mut report,
                )
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
                    self.retry_or_dlq_trace_ingested_counted(
                        message,
                        reason,
                        will_dead_letter,
                        &mut report,
                    )
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
            received_at: Utc::now(),
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
            start_time: request.start_time.unwrap_or_else(Utc::now),
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
            received_at: Utc::now(),
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
                start_time: draft.start_time.unwrap_or_else(Utc::now),
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
    ) -> Result<usize, IngestError> {
        let mut published = 0;
        for trace_id in trace_ids {
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
                published += 1;
            }
        }
        Ok(published)
    }

    async fn process_trace_write_message(
        &self,
        message: BusMessage,
        report: &mut TraceWriteDrainReport,
    ) -> Result<(), IngestError> {
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
                report.downstream_published += published;
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

    async fn retry_or_dlq_counted(
        &self,
        message: BusMessage,
        reason: String,
        will_dead_letter: bool,
        report: &mut TraceWriteDrainReport,
    ) -> Result<(), IngestError> {
        self.bus
            .retry_or_dlq(message, reason)
            .await
            .map_err(map_bus_error)?;
        if will_dead_letter {
            report.dead_lettered += 1;
        } else {
            report.retried += 1;
        }
        Ok(())
    }

    async fn retry_or_dlq_trace_ingested_counted(
        &self,
        message: BusMessage,
        reason: String,
        will_dead_letter: bool,
        report: &mut TraceIngestedDrainReport,
    ) -> Result<(), IngestError> {
        self.bus
            .retry_or_dlq(message, reason)
            .await
            .map_err(map_bus_error)?;
        if will_dead_letter {
            report.dead_lettered += 1;
        } else {
            report.retried += 1;
        }
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
        for (key, value) in attributes {
            if self.policy.denied_attributes.contains(&key) {
                dropped.insert(key, value);
                continue;
            }
            if let Some(allowed) = &self.policy.allowed_attributes {
                if !allowed.contains(&key) {
                    dropped.insert(key, value);
                    continue;
                }
            }
            kept.insert(key, value);
        }
        Ok((kept, json!({ "dropped_attributes": dropped })))
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
}

impl Default for IngestPolicy {
    fn default() -> Self {
        Self {
            max_raw_payload_bytes: 1024 * 1024,
            inline_payload_bytes: 16 * 1024,
            max_attributes: 128,
            allowed_attributes: None,
            denied_attributes: BTreeSet::new(),
            per_project_event_quota: None,
            quota_window_seconds: 60,
            trace_write_max_attempts: 3,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NativeIngestRequest {
    pub scope: TenantScope,
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct QueuedTraceWork {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub trace_id: TraceId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QueuedTraceWrite {
    pub batch: CanonicalTraceBatch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestOutcome {
    pub ack: WriteAck,
    pub downstream_queued: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceIngestedDrainReport {
    pub consumed: usize,
    pub completed: usize,
    pub retried: usize,
    pub dead_lettered: usize,
    pub invalid_messages: usize,
    pub failed_work: usize,
    pub trace_refs: Vec<QueuedTraceWork>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestQueueStatus {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub total_depth: usize,
    pub trace_write_depth: usize,
    pub trace_ingested_depth: usize,
    pub dead_letters: Vec<DeadLetter>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeadLetterReplayReport {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub message_id: String,
    pub reset_attempts: bool,
    pub ack: PublishAck,
}

#[derive(Clone, Debug)]
struct PreparedTraceBatch {
    tenant_id: TenantId,
    project_id: ProjectId,
    queue_key: IdempotencyKey,
    trace_ids: BTreeSet<TraceId>,
    batch: CanonicalTraceBatch,
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
    use beater_core::{Page, PageRequest};
    use beater_schema::{RunFilter, RunSummary, SpanFilter, SpanSummary, TraceView};
    use beater_store_obj::FsArtifactStore;
    use beater_store_sql::{SqliteQuotaLimiter, SqliteTraceStore};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct FailOnceTraceIngestedPublishBus {
        inner: InMemoryBus,
        remaining_failures: AtomicUsize,
    }

    impl FailOnceTraceIngestedPublishBus {
        fn new(capacity: usize) -> Self {
            Self {
                inner: InMemoryBus::new(capacity),
                remaining_failures: AtomicUsize::new(1),
            }
        }
    }

    #[async_trait]
    impl DurableBus for FailOnceTraceIngestedPublishBus {
        async fn publish(&self, message: BusMessage) -> Result<PublishAck, BusError> {
            if message.kind == TRACE_INGESTED_KIND
                && self.remaining_failures.load(Ordering::SeqCst) > 0
            {
                let _ = self.remaining_failures.fetch_update(
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                    |remaining| remaining.checked_sub(1),
                );
                return Err(BusError::Storage(
                    "simulated downstream publish outage".to_string(),
                ));
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
    async fn direct_ingest_does_not_error_after_durable_write_when_downstream_queue_is_full() {
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

        let outcome = service
            .ingest_native(request.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(outcome.ack.accepted_raw, 1);
        assert_eq!(outcome.ack.accepted_spans, 1);
        assert!(!outcome.downstream_queued);
        assert_eq!(bus.depth().await, Ok(0));

        let trace = traces
            .get_trace(request.scope.tenant_id, request.trace_id)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(trace.spans.len(), 1);
    }

    #[tokio::test]
    async fn direct_ingest_falls_back_to_trace_write_after_downstream_publish_failure() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(FailOnceTraceIngestedPublishBus::new(16));
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
            json!("drop")
        );
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
    async fn trace_write_retries_after_downstream_publish_failure_without_double_writes() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(FailOnceTraceIngestedPublishBus::new(16));
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
                        .get_trace(trace_ref.tenant_id, trace_ref.trace_id)
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
        assert_eq!(bus.depth_for_kind(TRACE_INGESTED_KIND).await, Ok(0));
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
                        .get_trace(trace_ref.tenant_id, trace_ref.trace_id)
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
}
