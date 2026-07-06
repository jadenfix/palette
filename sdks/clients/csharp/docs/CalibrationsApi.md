# Beater.Client.Api.CalibrationsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|--------|--------------|-------------|
| [**RunCalibration**](CalibrationsApi.md#runcalibration) | **POST** /v1/calibrations/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id} |  |

<a id="runcalibration"></a>
# **RunCalibration**
> CalibrationReport RunCalibration (string tenantId, string projectId, string datasetId, string versionId, RunCalibrationHttpRequest runCalibrationHttpRequest, string? authorization = null, string? xBeaterApiKey = null, string? xBeaterProjectId = null, string? xBeaterEnvironmentId = null)



### Example
```csharp
using System.Collections.Generic;
using System.Diagnostics;
using System.Net.Http;
using Beater.Client.Api;
using Beater.Client.Client;
using Beater.Client.Model;

namespace Example
{
    public class RunCalibrationExample
    {
        public static void Main()
        {
            Configuration config = new Configuration();
            config.BasePath = "http://localhost";
            // create instances of HttpClient, HttpClientHandler to be reused later with different Api classes
            HttpClient httpClient = new HttpClient();
            HttpClientHandler httpClientHandler = new HttpClientHandler();
            var apiInstance = new CalibrationsApi(httpClient, config, httpClientHandler);
            var tenantId = "tenantId_example";  // string | tenant_id
            var projectId = "projectId_example";  // string | project_id
            var datasetId = "datasetId_example";  // string | dataset_id
            var versionId = "versionId_example";  // string | version_id
            var runCalibrationHttpRequest = new RunCalibrationHttpRequest(); // RunCalibrationHttpRequest |
            var authorization = "authorization_example";  // string? | Bearer API token for strict auth (optional)
            var xBeaterApiKey = "xBeaterApiKey_example";  // string? | API key alternative for strict auth (optional)
            var xBeaterProjectId = "xBeaterProjectId_example";  // string? | Strict-auth project scope (optional)
            var xBeaterEnvironmentId = "xBeaterEnvironmentId_example";  // string? | Strict-auth environment scope (optional)

            try
            {
                CalibrationReport result = apiInstance.RunCalibration(tenantId, projectId, datasetId, versionId, runCalibrationHttpRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
                Debug.WriteLine(result);
            }
            catch (ApiException  e)
            {
                Debug.Print("Exception when calling CalibrationsApi.RunCalibration: " + e.Message);
                Debug.Print("Status Code: " + e.ErrorCode);
                Debug.Print(e.StackTrace);
            }
        }
    }
}
```

#### Using the RunCalibrationWithHttpInfo variant
This returns an ApiResponse object which contains the response data, status code and headers.

```csharp
try
{
    ApiResponse<CalibrationReport> response = apiInstance.RunCalibrationWithHttpInfo(tenantId, projectId, datasetId, versionId, runCalibrationHttpRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
    Debug.Write("Status Code: " + response.StatusCode);
    Debug.Write("Response Headers: " + response.Headers);
    Debug.Write("Response Body: " + response.Data);
}
catch (ApiException e)
{
    Debug.Print("Exception when calling CalibrationsApi.RunCalibrationWithHttpInfo: " + e.Message);
    Debug.Print("Status Code: " + e.ErrorCode);
    Debug.Print(e.StackTrace);
}
```

### Parameters

| Name | Type | Description | Notes |
|------|------|-------------|-------|
| **tenantId** | **string** | tenant_id |  |
| **projectId** | **string** | project_id |  |
| **datasetId** | **string** | dataset_id |  |
| **versionId** | **string** | version_id |  |
| **runCalibrationHttpRequest** | [**RunCalibrationHttpRequest**](RunCalibrationHttpRequest.md) |  |  |
| **authorization** | **string?** | Bearer API token for strict auth | [optional]  |
| **xBeaterApiKey** | **string?** | API key alternative for strict auth | [optional]  |
| **xBeaterProjectId** | **string?** | Strict-auth project scope | [optional]  |
| **xBeaterEnvironmentId** | **string?** | Strict-auth environment scope | [optional]  |

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

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
