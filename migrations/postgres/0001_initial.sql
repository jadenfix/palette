-- Beater self-host schema contract.
-- The current all-in-one binary uses SQLite under --data-dir; this migration
-- is the Postgres contract for the hosted/control-plane path and future
-- Postgres-backed stores.

CREATE TABLE IF NOT EXISTS raw_envelopes (
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  trace_id TEXT,
  payload_hash TEXT NOT NULL,
  received_at TIMESTAMPTZ NOT NULL,
  raw_json JSONB NOT NULL,
  PRIMARY KEY (tenant_id, project_id, idempotency_key)
);

CREATE TABLE IF NOT EXISTS spans (
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  environment_id TEXT NOT NULL,
  trace_id TEXT NOT NULL,
  span_id TEXT NOT NULL,
  seq BIGINT NOT NULL,
  kind TEXT NOT NULL,
  status TEXT NOT NULL,
  name TEXT NOT NULL,
  start_time TIMESTAMPTZ NOT NULL,
  end_time TIMESTAMPTZ,
  -- Per-span roll-up columns derived from the canonical span at write time so
  -- query_runs aggregates run summaries with a backend GROUP BY instead of
  -- materializing every span (ARCHITECTURE.md §8.1). The full canonical object
  -- still lives in span_json.
  model_provider TEXT,
  model_name TEXT,
  cost_currency TEXT,
  cost_micros BIGINT,
  release_id TEXT,
  span_json JSONB NOT NULL,
  PRIMARY KEY (tenant_id, project_id, trace_id, span_id, seq),
  CONSTRAINT spans_kind_known CHECK (
    kind IN (
      'agent.run',
      'agent.turn',
      'agent.plan',
      'agent.step',
      'llm.call',
      'tool.call',
      'mcp.request',
      'retrieval.query',
      'memory.read',
      'memory.write',
      'guardrail.check',
      'human.review',
      'evaluator.run',
      'replay.run'
    )
  ),
  CONSTRAINT spans_status_known CHECK (status IN ('ok', 'error', 'unset'))
);

CREATE INDEX IF NOT EXISTS idx_spans_tenant_trace
  ON spans (tenant_id, trace_id, seq);

CREATE INDEX IF NOT EXISTS idx_spans_tenant_kind_status
  ON spans (tenant_id, kind, status, start_time);

-- Backfill the roll-up columns on databases created before they were added.
-- CREATE TABLE IF NOT EXISTS above skips an existing table, so these run as
-- separate idempotent statements.
ALTER TABLE spans ADD COLUMN IF NOT EXISTS model_provider TEXT;
ALTER TABLE spans ADD COLUMN IF NOT EXISTS model_name TEXT;
ALTER TABLE spans ADD COLUMN IF NOT EXISTS cost_currency TEXT;
ALTER TABLE spans ADD COLUMN IF NOT EXISTS cost_micros BIGINT;
ALTER TABLE spans ADD COLUMN IF NOT EXISTS release_id TEXT;

UPDATE spans
SET
  model_provider = COALESCE(model_provider, span_json #>> '{model,provider}'),
  model_name = COALESCE(model_name, span_json #>> '{model,name}'),
  cost_currency = COALESCE(cost_currency, span_json #>> '{cost,currency}'),
  cost_micros = COALESCE(
    cost_micros,
    CASE
      WHEN (span_json #>> '{cost,amount_micros}') ~ '^-?[0-9]+$'
      THEN (span_json #>> '{cost,amount_micros}')::BIGINT
      ELSE NULL
    END
  ),
  release_id = COALESCE(
    release_id,
    span_json #>> '{attributes,beater.release_id}',
    span_json #>> '{attributes,agent.release_id}',
    span_json #>> '{attributes,deployment.release_id}',
    span_json #>> '{attributes,release.id}',
    span_json #>> '{attributes,release_id}'
  )
WHERE model_provider IS NULL
   OR model_name IS NULL
   OR cost_currency IS NULL
   OR cost_micros IS NULL
   OR release_id IS NULL;

CREATE INDEX IF NOT EXISTS idx_spans_scope_start
  ON spans (tenant_id, project_id, environment_id, start_time DESC);

CREATE TABLE IF NOT EXISTS quota_counters (
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  window_start TIMESTAMPTZ NOT NULL,
  reset_at TIMESTAMPTZ NOT NULL,
  used_events BIGINT NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  PRIMARY KEY (tenant_id, project_id, window_start)
);

CREATE TABLE IF NOT EXISTS organizations (
  tenant_id TEXT NOT NULL,
  organization_id TEXT NOT NULL,
  display_name TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  PRIMARY KEY (tenant_id, organization_id)
);

CREATE TABLE IF NOT EXISTS projects (
  tenant_id TEXT NOT NULL,
  organization_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  display_name TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  PRIMARY KEY (tenant_id, project_id)
);

CREATE TABLE IF NOT EXISTS environments (
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  environment_id TEXT NOT NULL,
  display_name TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  PRIMARY KEY (tenant_id, project_id, environment_id)
);

CREATE TABLE IF NOT EXISTS role_bindings (
  tenant_id TEXT NOT NULL,
  project_id TEXT,
  principal_id TEXT NOT NULL,
  role TEXT NOT NULL,
  permissions_json JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_role_bindings_unique_scope
  ON role_bindings (tenant_id, COALESCE(project_id, ''), principal_id, role);

CREATE TABLE IF NOT EXISTS queue_messages (
  message_id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  kind TEXT NOT NULL,
  enqueued_at TIMESTAMPTZ NOT NULL,
  message_json JSONB NOT NULL,
  UNIQUE (tenant_id, project_id, kind, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_queue_order
  ON queue_messages (enqueued_at, message_id);

CREATE TABLE IF NOT EXISTS inflight_messages (
  message_id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  kind TEXT NOT NULL,
  leased_at TIMESTAMPTZ NOT NULL,
  message_json JSONB NOT NULL,
  UNIQUE (tenant_id, project_id, kind, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_inflight_kind
  ON inflight_messages (kind, leased_at, message_id);

CREATE TABLE IF NOT EXISTS dead_letters (
  message_id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  kind TEXT NOT NULL,
  failed_at TIMESTAMPTZ NOT NULL,
  dead_letter_json JSONB NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_dead_letters_order
  ON dead_letters (failed_at, message_id);

CREATE TABLE IF NOT EXISTS api_keys (
  api_key_id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  environment_id TEXT NOT NULL,
  secret_hash TEXT NOT NULL,
  scopes_json JSONB NOT NULL,
  active BOOLEAN NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  rotated_at TIMESTAMPTZ,
  last_used_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_api_keys_scope
  ON api_keys (tenant_id, project_id, environment_id, active);

CREATE TABLE IF NOT EXISTS encrypted_provider_secrets (
  provider_secret_id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  provider TEXT NOT NULL,
  display_name TEXT NOT NULL,
  key_id TEXT NOT NULL,
  nonce BYTEA NOT NULL,
  ciphertext BYTEA NOT NULL,
  active BOOLEAN NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  rotated_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_encrypted_provider_secrets_scope
  ON encrypted_provider_secrets (tenant_id, project_id, provider, active);

CREATE TABLE IF NOT EXISTS datasets (
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  dataset_id TEXT NOT NULL,
  name TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  dataset_json JSONB NOT NULL,
  PRIMARY KEY (tenant_id, project_id, dataset_id)
);

CREATE TABLE IF NOT EXISTS dataset_cases (
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  dataset_id TEXT NOT NULL,
  case_id TEXT NOT NULL,
  source_trace_id TEXT NOT NULL,
  source_span_id TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  case_json JSONB NOT NULL,
  PRIMARY KEY (tenant_id, project_id, dataset_id, case_id)
);

CREATE TABLE IF NOT EXISTS dataset_versions (
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  dataset_id TEXT NOT NULL,
  version_id TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  PRIMARY KEY (tenant_id, project_id, dataset_id, version_id)
);

CREATE TABLE IF NOT EXISTS dataset_version_cases (
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  dataset_id TEXT NOT NULL,
  version_id TEXT NOT NULL,
  case_id TEXT NOT NULL,
  position BIGINT NOT NULL,
  PRIMARY KEY (tenant_id, project_id, dataset_id, version_id, case_id)
);

CREATE TABLE IF NOT EXISTS dataset_eval_reports (
  report_id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  dataset_id TEXT NOT NULL,
  version_id TEXT NOT NULL,
  evaluator_version_id TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  report_json JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS experiment_runs (
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  experiment_run_id TEXT NOT NULL,
  dataset_id TEXT NOT NULL,
  dataset_version_id TEXT NOT NULL,
  baseline_release_id TEXT NOT NULL,
  candidate_release_id TEXT NOT NULL,
  evaluator_version_id TEXT NOT NULL,
  decision TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  report_json JSONB NOT NULL,
  PRIMARY KEY (tenant_id, project_id, experiment_run_id)
);

CREATE TABLE IF NOT EXISTS gates (
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  gate_id TEXT NOT NULL,
  name TEXT NOT NULL,
  dataset_id TEXT,
  evaluator_version_id TEXT,
  inconclusive_policy TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  definition_json JSONB NOT NULL,
  PRIMARY KEY (tenant_id, project_id, gate_id)
);

CREATE TABLE IF NOT EXISTS gate_runs (
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  gate_run_id TEXT NOT NULL,
  gate_id TEXT NOT NULL,
  experiment_run_id TEXT NOT NULL,
  experiment_decision TEXT NOT NULL,
  passed BOOLEAN NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  report_json JSONB NOT NULL,
  PRIMARY KEY (tenant_id, project_id, gate_run_id)
);

CREATE INDEX IF NOT EXISTS idx_gate_runs_latest
  ON gate_runs (tenant_id, project_id, gate_id, created_at DESC, gate_run_id DESC);

CREATE TABLE IF NOT EXISTS review_queues (
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  queue_id TEXT NOT NULL,
  name TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  queue_json JSONB NOT NULL,
  PRIMARY KEY (tenant_id, project_id, queue_id)
);

CREATE TABLE IF NOT EXISTS review_tasks (
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  queue_id TEXT NOT NULL,
  task_id TEXT NOT NULL,
  trace_id TEXT NOT NULL,
  span_id TEXT,
  dataset_id TEXT,
  dataset_case_id TEXT,
  priority INTEGER NOT NULL,
  state TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  task_json JSONB NOT NULL,
  PRIMARY KEY (tenant_id, project_id, queue_id, task_id)
);

CREATE INDEX IF NOT EXISTS idx_review_tasks_queue_state
  ON review_tasks (tenant_id, project_id, queue_id, state, priority DESC, created_at ASC);

CREATE TABLE IF NOT EXISTS review_annotations (
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  queue_id TEXT NOT NULL,
  task_id TEXT NOT NULL,
  annotation_id TEXT NOT NULL,
  reviewer_id TEXT NOT NULL,
  verdict TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  annotation_json JSONB NOT NULL,
  PRIMARY KEY (tenant_id, project_id, queue_id, task_id, annotation_id)
);

CREATE INDEX IF NOT EXISTS idx_review_annotations_task
  ON review_annotations (tenant_id, project_id, queue_id, task_id, created_at ASC);

CREATE TABLE IF NOT EXISTS judge_audit_records (
  judge_call_id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  evaluator_id TEXT NOT NULL,
  provider TEXT NOT NULL,
  provider_secret_id TEXT NOT NULL,
  model TEXT NOT NULL,
  request_hash TEXT NOT NULL,
  response_hash TEXT NOT NULL,
  score DOUBLE PRECISION NOT NULL,
  provider_cost_json JSONB NOT NULL,
  charged_cost_json JSONB NOT NULL,
  cached BOOLEAN NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  response_json JSONB NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_judge_audit_scope
  ON judge_audit_records (tenant_id, project_id, created_at, judge_call_id);

CREATE INDEX IF NOT EXISTS idx_judge_cache_lookup
  ON judge_audit_records (tenant_id, project_id, request_hash, cached, created_at);

CREATE TABLE IF NOT EXISTS usage_records (
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  usage_record_id TEXT NOT NULL,
  meter TEXT NOT NULL,
  quantity BIGINT NOT NULL,
  unit TEXT NOT NULL,
  source_kind TEXT NOT NULL,
  source_id TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  record_json JSONB NOT NULL,
  PRIMARY KEY (tenant_id, project_id, usage_record_id),
  UNIQUE (tenant_id, project_id, meter, source_kind, source_id)
);

CREATE INDEX IF NOT EXISTS idx_usage_records_list
  ON usage_records (tenant_id, project_id, created_at, usage_record_id);

CREATE TABLE IF NOT EXISTS audit_events (
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  audit_event_id TEXT NOT NULL,
  environment_id TEXT,
  actor_api_key_id TEXT,
  action TEXT NOT NULL,
  resource_type TEXT NOT NULL,
  resource_id TEXT NOT NULL,
  outcome TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  event_json JSONB NOT NULL,
  PRIMARY KEY (tenant_id, project_id, audit_event_id)
);

CREATE INDEX IF NOT EXISTS idx_audit_events_list
  ON audit_events (tenant_id, project_id, created_at, audit_event_id);

CREATE INDEX IF NOT EXISTS idx_audit_events_resource
  ON audit_events (tenant_id, project_id, resource_type, resource_id);

CREATE TABLE IF NOT EXISTS replay_events (
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  trace_id TEXT NOT NULL,
  seq BIGINT NOT NULL,
  kind TEXT NOT NULL,
  request_hash TEXT NOT NULL,
  response_hash TEXT NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  event_json JSONB NOT NULL,
  PRIMARY KEY (tenant_id, project_id, trace_id, seq, kind, request_hash)
);

CREATE INDEX IF NOT EXISTS idx_replay_events_trace_order
  ON replay_events (tenant_id, project_id, trace_id, seq);

CREATE TABLE IF NOT EXISTS calibration_reports (
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  calibration_report_id TEXT NOT NULL,
  dataset_id TEXT NOT NULL,
  dataset_version_id TEXT NOT NULL,
  evaluator_version_id TEXT NOT NULL,
  eval_report_id TEXT NOT NULL,
  cohen_kappa DOUBLE PRECISION NOT NULL,
  observed_agreement DOUBLE PRECISION NOT NULL,
  sample_count BIGINT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  report_json JSONB NOT NULL,
  PRIMARY KEY (tenant_id, project_id, calibration_report_id)
);

CREATE INDEX IF NOT EXISTS idx_calibration_reports_latest
  ON calibration_reports (
    tenant_id, project_id, dataset_id, dataset_version_id,
    evaluator_version_id, created_at DESC, calibration_report_id DESC
  );
