#include <stdlib.h>
#include <stdio.h>
#include "../include/apiClient.h"
#include "../include/list.h"
#include "../external/cJSON.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"
#include "../model/dead_letter_replay_report.h"
#include "../model/error_response.h"
#include "../model/import_source_http_request.h"
#include "../model/ingest_outcome.h"
#include "../model/ingest_queue_status.h"
#include "../model/native_ingest_request.h"
#include "../model/otlp_ingest_outcome.h"
#include "../model/trace_ingested_drain_report.h"
#include "../model/trace_ingested_reconcile_report.h"
#include "../model/trace_write_drain_report.h"


trace_ingested_drain_report_t*
IngestAPI_drainTraceIngested(apiClient_t *apiClient, char *tenant_id, char *project_id, int *limit, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


trace_write_drain_report_t*
IngestAPI_drainTraceWrites(apiClient_t *apiClient, char *tenant_id, char *project_id, int *limit, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


ingest_queue_status_t*
IngestAPI_getIngestQueueStatus(apiClient_t *apiClient, char *tenant_id, char *project_id, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


ingest_outcome_t*
IngestAPI_importSource(apiClient_t *apiClient, char *tenant_id, char *project_id, char *environment_id, import_source_http_request_t *import_source_http_request, char *durability, char *authorization, char *x_beater_api_key);


ingest_outcome_t*
IngestAPI_ingestNative(apiClient_t *apiClient, native_ingest_request_t *native_ingest_request, char *durability, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


otlp_ingest_outcome_t*
IngestAPI_ingestOtlp(apiClient_t *apiClient, char *tenant_id, char *project_id, char *environment_id, char *durability, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


trace_ingested_reconcile_report_t*
IngestAPI_reconcileTrace(apiClient_t *apiClient, char *tenant_id, char *project_id, char *trace_id, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


dead_letter_replay_report_t*
IngestAPI_replayDeadLetter(apiClient_t *apiClient, char *tenant_id, char *project_id, char *message_id, int *reset_attempts, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


