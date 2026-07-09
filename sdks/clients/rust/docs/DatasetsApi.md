# \DatasetsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**datasets_period_create_dataset**](DatasetsApi.md#datasets_period_create_dataset) | **POST** /v1/datasets/{tenant_id}/{project_id} |
[**datasets_period_create_dataset_version**](DatasetsApi.md#datasets_period_create_dataset_version) | **POST** /v1/datasets/{tenant_id}/{project_id}/{dataset_id}/versions |
[**datasets_period_promote_dataset_case_from_trace**](DatasetsApi.md#datasets_period_promote_dataset_case_from_trace) | **POST** /v1/datasets/{tenant_id}/{project_id}/{dataset_id}/cases/from-trace |



## datasets_period_create_dataset

> models::Dataset datasets_period_create_dataset(tenant_id, project_id, create_dataset_request, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**project_id** | **String** | project_id | [required] |
**create_dataset_request** | [**CreateDatasetRequest**](CreateDatasetRequest.md) |  | [required] |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::Dataset**](Dataset.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## datasets_period_create_dataset_version

> models::DatasetVersionSnapshot datasets_period_create_dataset_version(tenant_id, project_id, dataset_id, create_dataset_version_request, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**project_id** | **String** | project_id | [required] |
**dataset_id** | **String** | dataset_id | [required] |
**create_dataset_version_request** | [**CreateDatasetVersionRequest**](CreateDatasetVersionRequest.md) |  | [required] |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::DatasetVersionSnapshot**](DatasetVersionSnapshot.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## datasets_period_promote_dataset_case_from_trace

> models::DatasetCase datasets_period_promote_dataset_case_from_trace(tenant_id, project_id, dataset_id, promote_trace_case_request, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**project_id** | **String** | project_id | [required] |
**dataset_id** | **String** | dataset_id | [required] |
**promote_trace_case_request** | [**PromoteTraceCaseRequest**](PromoteTraceCaseRequest.md) |  | [required] |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::DatasetCase**](DatasetCase.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
