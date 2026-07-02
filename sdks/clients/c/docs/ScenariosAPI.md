# ScenariosAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**ScenariosAPI_createScenario**](ScenariosAPI.md#ScenariosAPI_createScenario) | **POST** /v1/scenarios/{tenant_id}/{project_id} | 
[**ScenariosAPI_getScenario**](ScenariosAPI.md#ScenariosAPI_getScenario) | **GET** /v1/scenarios/{tenant_id}/{project_id}/{scenario_id} | 
[**ScenariosAPI_listScenarios**](ScenariosAPI.md#ScenariosAPI_listScenarios) | **GET** /v1/scenarios/{tenant_id}/{project_id} | 
[**ScenariosAPI_mineScenarios**](ScenariosAPI.md#ScenariosAPI_mineScenarios) | **POST** /v1/scenarios/{tenant_id}/{project_id}/mine | 


# **ScenariosAPI_createScenario**
```c
scenario_t* ScenariosAPI_createScenario(apiClient_t *apiClient, char *tenant_id, char *project_id, create_scenario_request_t *create_scenario_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id | 
**project_id** | **char \*** | project_id | 
**create_scenario_request** | **[create_scenario_request_t](create_scenario_request.md) \*** |  | 
**authorization** | **char \*** | Bearer API token for strict auth | [optional] 
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional] 
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional] 
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional] 

### Return type

[scenario_t](scenario.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **ScenariosAPI_getScenario**
```c
scenario_t* ScenariosAPI_getScenario(apiClient_t *apiClient, char *tenant_id, char *project_id, char *scenario_id, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id | 
**project_id** | **char \*** | project_id | 
**scenario_id** | **char \*** | scenario_id | 
**authorization** | **char \*** | Bearer API token for strict auth | [optional] 
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional] 
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional] 
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional] 

### Return type

[scenario_t](scenario.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **ScenariosAPI_listScenarios**
```c
list_scenarios_response_t* ScenariosAPI_listScenarios(apiClient_t *apiClient, char *tenant_id, char *project_id, int *limit, char *cursor, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id | 
**project_id** | **char \*** | project_id | 
**limit** | **int \*** |  | [optional] 
**cursor** | **char \*** |  | [optional] 
**authorization** | **char \*** | Bearer API token for strict auth | [optional] 
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional] 
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional] 
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional] 

### Return type

[list_scenarios_response_t](list_scenarios_response.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **ScenariosAPI_mineScenarios**
```c
mine_scenarios_response_t* ScenariosAPI_mineScenarios(apiClient_t *apiClient, char *tenant_id, char *project_id, mine_scenarios_request_t *mine_scenarios_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id | 
**project_id** | **char \*** | project_id | 
**mine_scenarios_request** | **[mine_scenarios_request_t](mine_scenarios_request.md) \*** |  | 
**authorization** | **char \*** | Bearer API token for strict auth | [optional] 
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional] 
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional] 
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional] 

### Return type

[mine_scenarios_response_t](mine_scenarios_response.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

