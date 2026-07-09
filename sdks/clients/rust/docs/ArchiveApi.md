# \ArchiveApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**archive_period_archive_trace**](ArchiveApi.md#archive_period_archive_trace) | **POST** /v1/archive/{tenant_id}/{project_id}/{trace_id} |
[**archive_period_query_archive_spans**](ArchiveApi.md#archive_period_query_archive_spans) | **GET** /v1/archive/{tenant_id}/{project_id}/spans |



## archive_period_archive_trace

> models::ArchiveManifest archive_period_archive_trace(tenant_id, project_id, trace_id, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**project_id** | **String** | project_id | [required] |
**trace_id** | **String** | trace_id | [required] |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::ArchiveManifest**](ArchiveManifest.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## archive_period_query_archive_spans

> models::ArchiveQueryResponse archive_period_query_archive_spans(tenant_id, project_id, environment_id, trace_id, span_id, kind, status, limit, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**project_id** | **String** | project_id | [required] |
**environment_id** | Option<**String**> |  |  |
**trace_id** | Option<**String**> |  |  |
**span_id** | Option<**String**> |  |  |
**kind** | Option<**String**> |  |  |
**status** | Option<**String**> |  |  |
**limit** | Option<**i32**> |  |  |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::ArchiveQueryResponse**](ArchiveQueryResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
