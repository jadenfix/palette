# ScenariosAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**ScenariosAPI_scenariosCreateScenario**](ScenariosAPI.md#ScenariosAPI_scenariosCreateScenario) | **POST** /v1/scenarios/{tenant_id}/{project_id} |
[**ScenariosAPI_scenariosGetScenario**](ScenariosAPI.md#ScenariosAPI_scenariosGetScenario) | **GET** /v1/scenarios/{tenant_id}/{project_id}/{scenario_id} |
[**ScenariosAPI_scenariosListScenarios**](ScenariosAPI.md#ScenariosAPI_scenariosListScenarios) | **GET** /v1/scenarios/{tenant_id}/{project_id} |
[**ScenariosAPI_scenariosMineScenarios**](ScenariosAPI.md#ScenariosAPI_scenariosMineScenarios) | **POST** /v1/scenarios/{tenant_id}/{project_id}/mine |


# **ScenariosAPI_scenariosCreateScenario**
```c
scenario_t* ScenariosAPI_scenariosCreateScenario(apiClient_t *apiClient, char *tenant_id, char *project_id, create_scenario_request_t *create_scenario_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
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

# **ScenariosAPI_scenariosGetScenario**
```c
scenario_t* ScenariosAPI_scenariosGetScenario(apiClient_t *apiClient, char *tenant_id, char *project_id, char *scenario_id, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
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

# **ScenariosAPI_scenariosListScenarios**
```c
list_scenarios_response_t* ScenariosAPI_scenariosListScenarios(apiClient_t *apiClient, char *tenant_id, char *project_id, int *limit, char *cursor, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
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

# **ScenariosAPI_scenariosMineScenarios**
```c
mine_scenarios_response_t* ScenariosAPI_scenariosMineScenarios(apiClient_t *apiClient, char *tenant_id, char *project_id, mine_scenarios_request_t *mine_scenarios_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
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
