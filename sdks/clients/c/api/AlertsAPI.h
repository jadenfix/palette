#include <stdlib.h>
#include <stdio.h>
#include "../include/apiClient.h"
#include "../include/list.h"
#include "../external/cJSON.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"
#include "../model/alert_decision.h"
#include "../model/error_response.h"
#include "../model/evaluate_alert_request.h"


alert_decision_t*
AlertsAPI_alertsEvaluateAlert(apiClient_t *apiClient, char *tenant_id, char *project_id, char *trace_id, evaluate_alert_request_t *evaluate_alert_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


