# GatesAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**GatesAPI_gatesCreateGate**](GatesAPI.md#GatesAPI_gatesCreateGate) | **POST** /v1/gates/{tenant_id}/{project_id} |
[**GatesAPI_gatesRunGate**](GatesAPI.md#GatesAPI_gatesRunGate) | **POST** /v1/gates/{tenant_id}/{project_id}/{gate_id}/run |


# **GatesAPI_gatesCreateGate**
```c
gate_definition_t* GatesAPI_gatesCreateGate(apiClient_t *apiClient, char *tenant_id, char *project_id, create_gate_request_t *create_gate_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**create_gate_request** | **[create_gate_request_t](create_gate_request.md) \*** |  |
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[gate_definition_t](gate_definition.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **GatesAPI_gatesRunGate**
```c
gate_run_report_t* GatesAPI_gatesRunGate(apiClient_t *apiClient, char *tenant_id, char *project_id, char *gate_id, run_gate_request_t *run_gate_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**gate_id** | **char \*** | gate_id |
**run_gate_request** | **[run_gate_request_t](run_gate_request.md) \*** |  |
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[gate_run_report_t](gate_run_report.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
