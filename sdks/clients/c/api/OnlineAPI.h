#include <stdlib.h>
#include <stdio.h>
#include "../include/apiClient.h"
#include "../include/list.h"
#include "../external/cJSON.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"
#include "../model/error_response.h"
#include "../model/online_sampling_policy.h"
#include "../model/sampling_decision.h"


sampling_decision_t*
OnlineAPI_onlineDecideOnlineSampling(apiClient_t *apiClient, char *tenant_id, char *project_id, char *trace_id, online_sampling_policy_t *online_sampling_policy, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


