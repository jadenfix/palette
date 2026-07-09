# \TracesApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**traces_period_get_trace**](TracesApi.md#traces_period_get_trace) | **GET** /v1/traces/{tenant_id}/{trace_id} |
[**traces_period_list_traces**](TracesApi.md#traces_period_list_traces) | **GET** /v1/traces/{tenant_id} |



## traces_period_get_trace

> models::TraceView traces_period_get_trace(tenant_id, trace_id, unmask, reason, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**trace_id** | **String** | trace_id | [required] |
**unmask** | Option<**bool**> |  |  |
**reason** | Option<**String**> |  |  |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::TraceView**](TraceView.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## traces_period_list_traces

> models::PageRunSummary traces_period_list_traces(tenant_id, project_id, environment_id, trace_id, kind, status, started_after, started_before, model, release, min_cost_micros, max_cost_micros, min_latency_ms, max_latency_ms, limit, cursor, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**project_id** | Option<**String**> |  |  |
**environment_id** | Option<**String**> |  |  |
**trace_id** | Option<**String**> |  |  |
**kind** | Option<**String**> |  |  |
**status** | Option<**String**> |  |  |
**started_after** | Option<**String**> |  |  |
**started_before** | Option<**String**> |  |  |
**model** | Option<**String**> |  |  |
**release** | Option<**String**> |  |  |
**min_cost_micros** | Option<**i64**> |  |  |
**max_cost_micros** | Option<**i64**> |  |  |
**min_latency_ms** | Option<**i64**> |  |  |
**max_latency_ms** | Option<**i64**> |  |  |
**limit** | Option<**i32**> |  |  |
**cursor** | Option<**String**> |  |  |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::PageRunSummary**](Page_RunSummary.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
