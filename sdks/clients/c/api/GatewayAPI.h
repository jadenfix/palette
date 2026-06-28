#include <stdlib.h>
#include <stdio.h>
#include "../include/apiClient.h"
#include "../include/list.h"
#include "../external/cJSON.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"
#include "../model/error_response.h"
#include "../model/gateway_chat_http_request.h"
#include "../model/gateway_outcome.h"


gateway_outcome_t*
GatewayAPI_gatewayChatCompletions(apiClient_t *apiClient, char *tenant_id, char *project_id, char *environment_id, gateway_chat_http_request_t *gateway_chat_http_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


