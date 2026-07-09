#include <stdlib.h>
#include <stdio.h>
#include "../include/apiClient.h"
#include "../include/list.h"
#include "../external/cJSON.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"
#include "../model/error_response.h"
#include "../model/palette_connect_status_response.h"


palette_connect_status_response_t*
ConnectAPI_connectGetPaletteConnectStatus(apiClient_t *apiClient, char *tenant_id, char *project_id, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


