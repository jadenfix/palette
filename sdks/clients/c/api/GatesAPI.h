#include <stdlib.h>
#include <stdio.h>
#include "../include/apiClient.h"
#include "../include/list.h"
#include "../external/cJSON.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"
#include "../model/create_gate_request.h"
#include "../model/error_response.h"
#include "../model/gate_definition.h"
#include "../model/gate_run_report.h"
#include "../model/run_gate_request.h"


gate_definition_t*
GatesAPI_gatesCreateGate(apiClient_t *apiClient, char *tenant_id, char *project_id, create_gate_request_t *create_gate_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


gate_run_report_t*
GatesAPI_gatesRunGate(apiClient_t *apiClient, char *tenant_id, char *project_id, char *gate_id, run_gate_request_t *run_gate_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


