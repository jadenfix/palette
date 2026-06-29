#include <stdlib.h>
#include <stdio.h>
#include "../include/apiClient.h"
#include "../include/list.h"
#include "../external/cJSON.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"
#include "../model/add_prompt_version_request.h"
#include "../model/create_prompt_request.h"
#include "../model/created_prompt.h"
#include "../model/error_response.h"
#include "../model/prompt.h"
#include "../model/prompt_list_response.h"
#include "../model/prompt_version.h"
#include "../model/prompt_version_diff.h"
#include "../model/prompt_version_list_response.h"


prompt_version_t*
PromptsAPI_addPromptVersion(apiClient_t *apiClient, char *tenant_id, char *project_id, char *prompt_id, add_prompt_version_request_t *add_prompt_version_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


created_prompt_t*
PromptsAPI_createPrompt(apiClient_t *apiClient, char *tenant_id, char *project_id, create_prompt_request_t *create_prompt_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


prompt_version_diff_t*
PromptsAPI_diffPromptVersions(apiClient_t *apiClient, char *tenant_id, char *project_id, char *prompt_id, char *from, char *to, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


prompt_t*
PromptsAPI_getPrompt(apiClient_t *apiClient, char *tenant_id, char *project_id, char *prompt_id, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


prompt_version_list_response_t*
PromptsAPI_listPromptVersions(apiClient_t *apiClient, char *tenant_id, char *project_id, char *prompt_id, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


prompt_list_response_t*
PromptsAPI_listPrompts(apiClient_t *apiClient, char *tenant_id, char *project_id, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


