#include <stdlib.h>
#include <stdio.h>
#include "../include/apiClient.h"
#include "../include/list.h"
#include "../external/cJSON.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"
#include "../model/archive_manifest.h"
#include "../model/archive_query_response.h"
#include "../model/error_response.h"


archive_manifest_t*
ArchiveAPI_archiveArchiveTrace(apiClient_t *apiClient, char *tenant_id, char *project_id, char *trace_id, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


archive_query_response_t*
ArchiveAPI_archiveQueryArchiveSpans(apiClient_t *apiClient, char *tenant_id, char *project_id, char *environment_id, char *trace_id, char *span_id, char *kind, char *status, int *limit, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


