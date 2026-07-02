#include <stdlib.h>
#include <stdio.h>
#include "../include/apiClient.h"
#include "../include/list.h"
#include "../external/cJSON.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"
#include "../model/create_scenario_request.h"
#include "../model/error_response.h"
#include "../model/list_scenarios_response.h"
#include "../model/mine_scenarios_request.h"
#include "../model/mine_scenarios_response.h"
#include "../model/scenario.h"


scenario_t*
ScenariosAPI_createScenario(apiClient_t *apiClient, char *tenant_id, char *project_id, create_scenario_request_t *create_scenario_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


scenario_t*
ScenariosAPI_getScenario(apiClient_t *apiClient, char *tenant_id, char *project_id, char *scenario_id, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


list_scenarios_response_t*
ScenariosAPI_listScenarios(apiClient_t *apiClient, char *tenant_id, char *project_id, int *limit, char *cursor, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


mine_scenarios_response_t*
ScenariosAPI_mineScenarios(apiClient_t *apiClient, char *tenant_id, char *project_id, mine_scenarios_request_t *mine_scenarios_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


