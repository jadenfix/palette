#include <stdlib.h>
#include <stdio.h>
#include "../include/apiClient.h"
#include "../include/list.h"
#include "../external/cJSON.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"
#include "../model/create_provider_secret_http_request.h"
#include "../model/error_response.h"
#include "../model/provider_secret_metadata.h"
#include "../model/revoked_provider_secret.h"


provider_secret_metadata_t*
ProviderSecretsAPI_providerSecretsCreateProviderSecret(apiClient_t *apiClient, char *tenant_id, char *project_id, create_provider_secret_http_request_t *create_provider_secret_http_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


list_t*
ProviderSecretsAPI_providerSecretsListProviderSecrets(apiClient_t *apiClient, char *tenant_id, char *project_id, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


revoked_provider_secret_t*
ProviderSecretsAPI_providerSecretsRevokeProviderSecret(apiClient_t *apiClient, char *tenant_id, char *project_id, char *provider_secret_id, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


