#include <stdlib.h>
#include <stdio.h>
#include "../include/apiClient.h"
#include "../include/list.h"
#include "../external/cJSON.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"
#include "../model/api_key_created_response.h"
#include "../model/create_api_key_http_request.h"
#include "../model/error_response.h"
#include "../model/revoked_api_key.h"


api_key_created_response_t*
ApiKeysAPI_apiKeysCreateApiKey(apiClient_t *apiClient, char *tenant_id, char *project_id, char *environment_id, create_api_key_http_request_t *create_api_key_http_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


revoked_api_key_t*
ApiKeysAPI_apiKeysRevokeApiKey(apiClient_t *apiClient, char *tenant_id, char *project_id, char *environment_id, char *api_key_id, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


