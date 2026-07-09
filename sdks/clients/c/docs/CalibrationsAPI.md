# CalibrationsAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**CalibrationsAPI_calibrationsRunCalibration**](CalibrationsAPI.md#CalibrationsAPI_calibrationsRunCalibration) | **POST** /v1/calibrations/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id} |


# **CalibrationsAPI_calibrationsRunCalibration**
```c
calibration_report_t* CalibrationsAPI_calibrationsRunCalibration(apiClient_t *apiClient, char *tenant_id, char *project_id, char *dataset_id, char *version_id, run_calibration_http_request_t *run_calibration_http_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**dataset_id** | **char \*** | dataset_id |
**version_id** | **char \*** | version_id |
**run_calibration_http_request** | **[run_calibration_http_request_t](run_calibration_http_request.md) \*** |  |
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[calibration_report_t](calibration_report.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
