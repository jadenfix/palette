# \SearchApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**search_period_search_spans**](SearchApi.md#search_period_search_spans) | **GET** /v1/search/{tenant_id}/spans |



## search_period_search_spans

> models::SearchResponse search_period_search_spans(tenant_id, q, project_id, environment_id, trace_id, span_id, kind, status, model, tool, limit, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**q** | Option<**String**> |  |  |
**project_id** | Option<**String**> |  |  |
**environment_id** | Option<**String**> |  |  |
**trace_id** | Option<**String**> |  |  |
**span_id** | Option<**String**> |  |  |
**kind** | Option<**String**> |  |  |
**status** | Option<**String**> |  |  |
**model** | Option<**String**> |  |  |
**tool** | Option<**String**> |  |  |
**limit** | Option<**i32**> |  |  |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::SearchResponse**](SearchResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
