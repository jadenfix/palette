# PromptsAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**PromptsAPI_promptsAddPromptVersion**](PromptsAPI.md#PromptsAPI_promptsAddPromptVersion) | **POST** /v1/prompts/{tenant_id}/{project_id}/{prompt_id}/versions |
[**PromptsAPI_promptsCreatePrompt**](PromptsAPI.md#PromptsAPI_promptsCreatePrompt) | **POST** /v1/prompts/{tenant_id}/{project_id} |
[**PromptsAPI_promptsDiffPromptVersions**](PromptsAPI.md#PromptsAPI_promptsDiffPromptVersions) | **GET** /v1/prompts/{tenant_id}/{project_id}/{prompt_id}/diff |
[**PromptsAPI_promptsGetPrompt**](PromptsAPI.md#PromptsAPI_promptsGetPrompt) | **GET** /v1/prompts/{tenant_id}/{project_id}/{prompt_id} |
[**PromptsAPI_promptsListPromptVersions**](PromptsAPI.md#PromptsAPI_promptsListPromptVersions) | **GET** /v1/prompts/{tenant_id}/{project_id}/{prompt_id}/versions |
[**PromptsAPI_promptsListPrompts**](PromptsAPI.md#PromptsAPI_promptsListPrompts) | **GET** /v1/prompts/{tenant_id}/{project_id} |


# **PromptsAPI_promptsAddPromptVersion**
```c
prompt_version_t* PromptsAPI_promptsAddPromptVersion(apiClient_t *apiClient, char *tenant_id, char *project_id, char *prompt_id, add_prompt_version_request_t *add_prompt_version_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**prompt_id** | **char \*** | prompt_id |
**add_prompt_version_request** | **[add_prompt_version_request_t](add_prompt_version_request.md) \*** |  |
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[prompt_version_t](prompt_version.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **PromptsAPI_promptsCreatePrompt**
```c
created_prompt_t* PromptsAPI_promptsCreatePrompt(apiClient_t *apiClient, char *tenant_id, char *project_id, create_prompt_request_t *create_prompt_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**create_prompt_request** | **[create_prompt_request_t](create_prompt_request.md) \*** |  |
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[created_prompt_t](created_prompt.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **PromptsAPI_promptsDiffPromptVersions**
```c
prompt_version_diff_t* PromptsAPI_promptsDiffPromptVersions(apiClient_t *apiClient, char *tenant_id, char *project_id, char *prompt_id, char *from, char *to, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**prompt_id** | **char \*** | prompt_id |
**from** | **char \*** |  |
**to** | **char \*** |  |
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[prompt_version_diff_t](prompt_version_diff.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **PromptsAPI_promptsGetPrompt**
```c
prompt_t* PromptsAPI_promptsGetPrompt(apiClient_t *apiClient, char *tenant_id, char *project_id, char *prompt_id, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**prompt_id** | **char \*** | prompt_id |
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[prompt_t](prompt.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **PromptsAPI_promptsListPromptVersions**
```c
prompt_version_list_response_t* PromptsAPI_promptsListPromptVersions(apiClient_t *apiClient, char *tenant_id, char *project_id, char *prompt_id, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**prompt_id** | **char \*** | prompt_id |
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[prompt_version_list_response_t](prompt_version_list_response.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **PromptsAPI_promptsListPrompts**
```c
prompt_list_response_t* PromptsAPI_promptsListPrompts(apiClient_t *apiClient, char *tenant_id, char *project_id, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[prompt_list_response_t](prompt_list_response.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
