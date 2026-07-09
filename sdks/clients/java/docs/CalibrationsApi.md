# CalibrationsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**calibrationsRunCalibration**](CalibrationsApi.md#calibrationsRunCalibration) | **POST** /v1/calibrations/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id} |  |
| [**calibrationsRunCalibrationWithHttpInfo**](CalibrationsApi.md#calibrationsRunCalibrationWithHttpInfo) | **POST** /v1/calibrations/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id} |  |



## calibrationsRunCalibration

> CalibrationReport calibrationsRunCalibration(tenantId, projectId, datasetId, versionId, runCalibrationHttpRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example

```java
// Import classes:
import ai.beater.client.ApiClient;
import ai.beater.client.ApiException;
import ai.beater.client.Configuration;
import ai.beater.client.models.*;
import ai.beater.client.api.CalibrationsApi;

public class Example {
    public static void main(String[] args) {
        ApiClient defaultClient = Configuration.getDefaultApiClient();
        defaultClient.setBasePath("http://localhost");

        CalibrationsApi apiInstance = new CalibrationsApi(defaultClient);
        String tenantId = "tenantId_example"; // String | tenant_id
        String projectId = "projectId_example"; // String | project_id
        String datasetId = "datasetId_example"; // String | dataset_id
        String versionId = "versionId_example"; // String | version_id
        RunCalibrationHttpRequest runCalibrationHttpRequest = new RunCalibrationHttpRequest(); // RunCalibrationHttpRequest |
        String authorization = "authorization_example"; // String | Bearer API token for strict auth
        String xBeaterApiKey = "xBeaterApiKey_example"; // String | API key alternative for strict auth
        String xBeaterProjectId = "xBeaterProjectId_example"; // String | Strict-auth project scope
        String xBeaterEnvironmentId = "xBeaterEnvironmentId_example"; // String | Strict-auth environment scope
        try {
            CalibrationReport result = apiInstance.calibrationsRunCalibration(tenantId, projectId, datasetId, versionId, runCalibrationHttpRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
            System.out.println(result);
        } catch (ApiException e) {
            System.err.println("Exception when calling CalibrationsApi#calibrationsRunCalibration");
            System.err.println("Status code: " + e.getCode());
            System.err.println("Reason: " + e.getResponseBody());
            System.err.println("Response headers: " + e.getResponseHeaders());
            e.printStackTrace();
        }
    }
}
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **tenantId** | **String**| tenant_id | |
| **projectId** | **String**| project_id | |
| **datasetId** | **String**| dataset_id | |
| **versionId** | **String**| version_id | |
| **runCalibrationHttpRequest** | [**RunCalibrationHttpRequest**](RunCalibrationHttpRequest.md)|  | |
| **authorization** | **String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **String**| Strict-auth project scope | [optional] |
| **xBeaterEnvironmentId** | **String**| Strict-auth environment scope | [optional] |

### Return type

[**CalibrationReport**](CalibrationReport.md)


### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Run a calibration over an eval report |  -  |
| **400** | Invalid request, scope, or filter |  -  |
| **401** | Missing or invalid credentials |  -  |
| **403** | Credentials lack the required scope |  -  |
| **404** | Resource not found |  -  |

## calibrationsRunCalibrationWithHttpInfo

> ApiResponse<CalibrationReport> calibrationsRunCalibration calibrationsRunCalibrationWithHttpInfo(tenantId, projectId, datasetId, versionId, runCalibrationHttpRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example

```java
// Import classes:
import ai.beater.client.ApiClient;
import ai.beater.client.ApiException;
import ai.beater.client.ApiResponse;
import ai.beater.client.Configuration;
import ai.beater.client.models.*;
import ai.beater.client.api.CalibrationsApi;

public class Example {
    public static void main(String[] args) {
        ApiClient defaultClient = Configuration.getDefaultApiClient();
        defaultClient.setBasePath("http://localhost");

        CalibrationsApi apiInstance = new CalibrationsApi(defaultClient);
        String tenantId = "tenantId_example"; // String | tenant_id
        String projectId = "projectId_example"; // String | project_id
        String datasetId = "datasetId_example"; // String | dataset_id
        String versionId = "versionId_example"; // String | version_id
        RunCalibrationHttpRequest runCalibrationHttpRequest = new RunCalibrationHttpRequest(); // RunCalibrationHttpRequest |
        String authorization = "authorization_example"; // String | Bearer API token for strict auth
        String xBeaterApiKey = "xBeaterApiKey_example"; // String | API key alternative for strict auth
        String xBeaterProjectId = "xBeaterProjectId_example"; // String | Strict-auth project scope
        String xBeaterEnvironmentId = "xBeaterEnvironmentId_example"; // String | Strict-auth environment scope
        try {
            ApiResponse<CalibrationReport> response = apiInstance.calibrationsRunCalibrationWithHttpInfo(tenantId, projectId, datasetId, versionId, runCalibrationHttpRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
            System.out.println("Status code: " + response.getStatusCode());
            System.out.println("Response headers: " + response.getHeaders());
            System.out.println("Response body: " + response.getData());
        } catch (ApiException e) {
            System.err.println("Exception when calling CalibrationsApi#calibrationsRunCalibration");
            System.err.println("Status code: " + e.getCode());
            System.err.println("Response headers: " + e.getResponseHeaders());
            System.err.println("Reason: " + e.getResponseBody());
            e.printStackTrace();
        }
    }
}
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **tenantId** | **String**| tenant_id | |
| **projectId** | **String**| project_id | |
| **datasetId** | **String**| dataset_id | |
| **versionId** | **String**| version_id | |
| **runCalibrationHttpRequest** | [**RunCalibrationHttpRequest**](RunCalibrationHttpRequest.md)|  | |
| **authorization** | **String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **String**| Strict-auth project scope | [optional] |
| **xBeaterEnvironmentId** | **String**| Strict-auth environment scope | [optional] |

### Return type

ApiResponse<[**CalibrationReport**](CalibrationReport.md)>


### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Run a calibration over an eval report |  -  |
| **400** | Invalid request, scope, or filter |  -  |
| **401** | Missing or invalid credentials |  -  |
| **403** | Credentials lack the required scope |  -  |
| **404** | Resource not found |  -  |
