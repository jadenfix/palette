#include <stdlib.h>
#include <stdio.h>
#include "../include/apiClient.h"
#include "../include/list.h"
#include "../external/cJSON.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"
#include "../model/error_response.h"
#include "../model/experiment_run_report.h"
#include "../model/run_experiment_request.h"
#include "../model/run_judge_experiment_request.h"


experiment_run_report_t*
ExperimentsAPI_experimentsRunDeterministicExperiment(apiClient_t *apiClient, char *tenant_id, char *project_id, char *dataset_id, char *version_id, run_experiment_request_t *run_experiment_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


experiment_run_report_t*
ExperimentsAPI_experimentsRunJudgeExperiment(apiClient_t *apiClient, char *tenant_id, char *project_id, char *dataset_id, char *version_id, run_judge_experiment_request_t *run_judge_experiment_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


