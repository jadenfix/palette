# TracesAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**TracesAPI_tracesGetTrace**](TracesAPI.md#TracesAPI_tracesGetTrace) | **GET** /v1/traces/{tenant_id}/{trace_id} |
[**TracesAPI_tracesListTraces**](TracesAPI.md#TracesAPI_tracesListTraces) | **GET** /v1/traces/{tenant_id} |


# **TracesAPI_tracesGetTrace**
```c
trace_view_t* TracesAPI_tracesGetTrace(apiClient_t *apiClient, char *tenant_id, char *trace_id, int *unmask, char *reason, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**trace_id** | **char \*** | trace_id |
**unmask** | **int \*** |  | [optional]
**reason** | **char \*** |  | [optional]
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[trace_view_t](trace_view.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **TracesAPI_tracesListTraces**
```c
page_run_summary_t* TracesAPI_tracesListTraces(apiClient_t *apiClient, char *tenant_id, char *project_id, char *environment_id, char *trace_id, char *kind, char *status, char *started_after, char *started_before, char *model, char *release, long min_cost_micros, long max_cost_micros, long min_latency_ms, long max_latency_ms, int *limit, char *cursor, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** |  | [optional]
**environment_id** | **char \*** |  | [optional]
**trace_id** | **char \*** |  | [optional]
**kind** | **char \*** |  | [optional]
**status** | **char \*** |  | [optional]
**started_after** | **char \*** |  | [optional]
**started_before** | **char \*** |  | [optional]
**model** | **char \*** |  | [optional]
**release** | **char \*** |  | [optional]
**min_cost_micros** | **long** |  | [optional]
**max_cost_micros** | **long** |  | [optional]
**min_latency_ms** | **long** |  | [optional]
**max_latency_ms** | **long** |  | [optional]
**limit** | **int \*** |  | [optional]
**cursor** | **char \*** |  | [optional]
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[page_run_summary_t](page_run_summary.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
