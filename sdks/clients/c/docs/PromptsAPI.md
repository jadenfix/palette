# PromptsAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**PromptsAPI_addPromptVersion**](PromptsAPI.md#PromptsAPI_addPromptVersion) | **POST** /v1/prompts/{tenant_id}/{project_id}/{prompt_id}/versions | 
[**PromptsAPI_createPrompt**](PromptsAPI.md#PromptsAPI_createPrompt) | **POST** /v1/prompts/{tenant_id}/{project_id} | 
[**PromptsAPI_diffPromptVersions**](PromptsAPI.md#PromptsAPI_diffPromptVersions) | **GET** /v1/prompts/{tenant_id}/{project_id}/{prompt_id}/diff | 
[**PromptsAPI_getPrompt**](PromptsAPI.md#PromptsAPI_getPrompt) | **GET** /v1/prompts/{tenant_id}/{project_id}/{prompt_id} | 
[**PromptsAPI_listPromptVersions**](PromptsAPI.md#PromptsAPI_listPromptVersions) | **GET** /v1/prompts/{tenant_id}/{project_id}/{prompt_id}/versions | 
[**PromptsAPI_listPrompts**](PromptsAPI.md#PromptsAPI_listPrompts) | **GET** /v1/prompts/{tenant_id}/{project_id} | 


# **PromptsAPI_addPromptVersion**
```c
prompt_version_t* PromptsAPI_addPromptVersion(apiClient_t *apiClient, char *tenant_id, char *project_id, char *prompt_id, add_prompt_version_request_t *add_prompt_version_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
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

# **PromptsAPI_createPrompt**
```c
created_prompt_t* PromptsAPI_createPrompt(apiClient_t *apiClient, char *tenant_id, char *project_id, create_prompt_request_t *create_prompt_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
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

# **PromptsAPI_diffPromptVersions**
```c
prompt_version_diff_t* PromptsAPI_diffPromptVersions(apiClient_t *apiClient, char *tenant_id, char *project_id, char *prompt_id, char *from, char *to, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
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

# **PromptsAPI_getPrompt**
```c
prompt_t* PromptsAPI_getPrompt(apiClient_t *apiClient, char *tenant_id, char *project_id, char *prompt_id, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
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

# **PromptsAPI_listPromptVersions**
```c
prompt_version_list_response_t* PromptsAPI_listPromptVersions(apiClient_t *apiClient, char *tenant_id, char *project_id, char *prompt_id, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
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

# **PromptsAPI_listPrompts**
```c
prompt_list_response_t* PromptsAPI_listPrompts(apiClient_t *apiClient, char *tenant_id, char *project_id, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
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

