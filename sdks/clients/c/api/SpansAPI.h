#include <stdlib.h>
#include <stdio.h>
#include "../include/apiClient.h"
#include "../include/list.h"
#include "../external/cJSON.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"
#include "../model/canonical_span.h"
#include "../model/error_response.h"
#include "../model/span_io_response.h"


canonical_span_t*
SpansAPI_spansGetSpan(apiClient_t *apiClient, char *tenant_id, char *trace_id, char *span_id, int *unmask, char *reason, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


span_io_response_t*
SpansAPI_spansGetSpanIo(apiClient_t *apiClient, char *tenant_id, char *trace_id, char *span_id, int *unmask, char *reason, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


