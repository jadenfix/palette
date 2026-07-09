# \SpansApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**spans_period_get_span**](SpansApi.md#spans_period_get_span) | **GET** /v1/spans/{tenant_id}/{trace_id}/{span_id} |
[**spans_period_get_span_io**](SpansApi.md#spans_period_get_span_io) | **GET** /v1/spans/{tenant_id}/{trace_id}/{span_id}/io |



## spans_period_get_span

> models::CanonicalSpan spans_period_get_span(tenant_id, trace_id, span_id, unmask, reason, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**trace_id** | **String** | trace_id | [required] |
**span_id** | **String** | span_id | [required] |
**unmask** | Option<**bool**> |  |  |
**reason** | Option<**String**> |  |  |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::CanonicalSpan**](CanonicalSpan.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## spans_period_get_span_io

> models::SpanIoResponse spans_period_get_span_io(tenant_id, trace_id, span_id, unmask, reason, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**trace_id** | **String** | trace_id | [required] |
**span_id** | **String** | span_id | [required] |
**unmask** | Option<**bool**> |  |  |
**reason** | Option<**String**> |  |  |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::SpanIoResponse**](SpanIoResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
