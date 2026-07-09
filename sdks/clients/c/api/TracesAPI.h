#include <stdlib.h>
#include <stdio.h>
#include "../include/apiClient.h"
#include "../include/list.h"
#include "../external/cJSON.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"
#include "../model/error_response.h"
#include "../model/page_run_summary.h"
#include "../model/trace_view.h"


trace_view_t*
TracesAPI_tracesGetTrace(apiClient_t *apiClient, char *tenant_id, char *trace_id, int *unmask, char *reason, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


page_run_summary_t*
TracesAPI_tracesListTraces(apiClient_t *apiClient, char *tenant_id, char *project_id, char *environment_id, char *trace_id, char *kind, char *status, char *started_after, char *started_before, char *model, char *release, long min_cost_micros, long max_cost_micros, long min_latency_ms, long max_latency_ms, int *limit, char *cursor, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


