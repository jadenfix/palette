#include <stdlib.h>
#include <stdio.h>
#include "../include/apiClient.h"
#include "../include/list.h"
#include "../external/cJSON.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"
#include "../model/create_dataset_request.h"
#include "../model/create_dataset_version_request.h"
#include "../model/dataset.h"
#include "../model/dataset_case.h"
#include "../model/dataset_version_snapshot.h"
#include "../model/error_response.h"
#include "../model/promote_trace_case_request.h"


dataset_t*
DatasetsAPI_datasetsCreateDataset(apiClient_t *apiClient, char *tenant_id, char *project_id, create_dataset_request_t *create_dataset_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


dataset_version_snapshot_t*
DatasetsAPI_datasetsCreateDatasetVersion(apiClient_t *apiClient, char *tenant_id, char *project_id, char *dataset_id, create_dataset_version_request_t *create_dataset_version_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


dataset_case_t*
DatasetsAPI_datasetsPromoteDatasetCaseFromTrace(apiClient_t *apiClient, char *tenant_id, char *project_id, char *dataset_id, promote_trace_case_request_t *promote_trace_case_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


