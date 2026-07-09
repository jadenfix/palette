#include <stdlib.h>
#include <stdio.h>
#include "../include/apiClient.h"
#include "../include/list.h"
#include "../external/cJSON.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"
#include "../model/error_response.h"
#include "../model/judge_audit_record.h"
#include "../model/judge_broker_outcome.h"
#include "../model/run_judge_eval_http_request.h"


judge_broker_outcome_t*
JudgeAPI_judgeEvaluateJudge(apiClient_t *apiClient, char *tenant_id, char *project_id, run_judge_eval_http_request_t *run_judge_eval_http_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


list_t*
JudgeAPI_judgeListJudgeLedger(apiClient_t *apiClient, char *tenant_id, char *project_id, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


