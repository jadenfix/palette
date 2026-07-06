# Beater.Client.Api.EvalsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|--------|--------------|-------------|
| [**RunDeterministicEval**](EvalsApi.md#rundeterministiceval) | **POST** /v1/datasets/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id}/evals/deterministic |  |
| [**RunJudgeEval**](EvalsApi.md#runjudgeeval) | **POST** /v1/datasets/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id}/evals/judge |  |

<a id="rundeterministiceval"></a>
# **RunDeterministicEval**
> DatasetEvalReport RunDeterministicEval (string tenantId, string projectId, string datasetId, string versionId, RunDeterministicEvalRequest runDeterministicEvalRequest, string? authorization = null, string? xBeaterApiKey = null, string? xBeaterProjectId = null, string? xBeaterEnvironmentId = null)



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
    public class RunDeterministicEvalExample
    {
        public static void Main()
        {
            Configuration config = new Configuration();
            config.BasePath = "http://localhost";
            // create instances of HttpClient, HttpClientHandler to be reused later with different Api classes
            HttpClient httpClient = new HttpClient();
            HttpClientHandler httpClientHandler = new HttpClientHandler();
            var apiInstance = new EvalsApi(httpClient, config, httpClientHandler);
            var tenantId = "tenantId_example";  // string | tenant_id
            var projectId = "projectId_example";  // string | project_id
            var datasetId = "datasetId_example";  // string | dataset_id
            var versionId = "versionId_example";  // string | version_id
            var runDeterministicEvalRequest = new RunDeterministicEvalRequest(); // RunDeterministicEvalRequest |
            var authorization = "authorization_example";  // string? | Bearer API token for strict auth (optional)
            var xBeaterApiKey = "xBeaterApiKey_example";  // string? | API key alternative for strict auth (optional)
            var xBeaterProjectId = "xBeaterProjectId_example";  // string? | Strict-auth project scope (optional)
            var xBeaterEnvironmentId = "xBeaterEnvironmentId_example";  // string? | Strict-auth environment scope (optional)

            try
            {
                DatasetEvalReport result = apiInstance.RunDeterministicEval(tenantId, projectId, datasetId, versionId, runDeterministicEvalRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
                Debug.WriteLine(result);
            }
            catch (ApiException  e)
            {
                Debug.Print("Exception when calling EvalsApi.RunDeterministicEval: " + e.Message);
                Debug.Print("Status Code: " + e.ErrorCode);
                Debug.Print(e.StackTrace);
            }
        }
    }
}
```

#### Using the RunDeterministicEvalWithHttpInfo variant
This returns an ApiResponse object which contains the response data, status code and headers.

```csharp
try
{
    ApiResponse<DatasetEvalReport> response = apiInstance.RunDeterministicEvalWithHttpInfo(tenantId, projectId, datasetId, versionId, runDeterministicEvalRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
    Debug.Write("Status Code: " + response.StatusCode);
    Debug.Write("Response Headers: " + response.Headers);
    Debug.Write("Response Body: " + response.Data);
}
catch (ApiException e)
{
    Debug.Print("Exception when calling EvalsApi.RunDeterministicEvalWithHttpInfo: " + e.Message);
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
| **runDeterministicEvalRequest** | [**RunDeterministicEvalRequest**](RunDeterministicEvalRequest.md) |  |  |
| **authorization** | **string?** | Bearer API token for strict auth | [optional]  |
| **xBeaterApiKey** | **string?** | API key alternative for strict auth | [optional]  |
| **xBeaterProjectId** | **string?** | Strict-auth project scope | [optional]  |
| **xBeaterEnvironmentId** | **string?** | Strict-auth environment scope | [optional]  |

### Return type

[**DatasetEvalReport**](DatasetEvalReport.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Run a deterministic dataset evaluation |  -  |
| **400** | Invalid request, scope, or filter |  -  |
| **401** | Missing or invalid credentials |  -  |
| **403** | Credentials lack the required scope |  -  |
| **404** | Resource not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

<a id="runjudgeeval"></a>
# **RunJudgeEval**
> DatasetEvalReport RunJudgeEval (string tenantId, string projectId, string datasetId, string versionId, RunJudgeDatasetEvalRequest runJudgeDatasetEvalRequest, string? authorization = null, string? xBeaterApiKey = null, string? xBeaterProjectId = null, string? xBeaterEnvironmentId = null)



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
    public class RunJudgeEvalExample
    {
        public static void Main()
        {
            Configuration config = new Configuration();
            config.BasePath = "http://localhost";
            // create instances of HttpClient, HttpClientHandler to be reused later with different Api classes
            HttpClient httpClient = new HttpClient();
            HttpClientHandler httpClientHandler = new HttpClientHandler();
            var apiInstance = new EvalsApi(httpClient, config, httpClientHandler);
            var tenantId = "tenantId_example";  // string | tenant_id
            var projectId = "projectId_example";  // string | project_id
            var datasetId = "datasetId_example";  // string | dataset_id
            var versionId = "versionId_example";  // string | version_id
            var runJudgeDatasetEvalRequest = new RunJudgeDatasetEvalRequest(); // RunJudgeDatasetEvalRequest |
            var authorization = "authorization_example";  // string? | Bearer API token for strict auth (optional)
            var xBeaterApiKey = "xBeaterApiKey_example";  // string? | API key alternative for strict auth (optional)
            var xBeaterProjectId = "xBeaterProjectId_example";  // string? | Strict-auth project scope (optional)
            var xBeaterEnvironmentId = "xBeaterEnvironmentId_example";  // string? | Strict-auth environment scope (optional)

            try
            {
                DatasetEvalReport result = apiInstance.RunJudgeEval(tenantId, projectId, datasetId, versionId, runJudgeDatasetEvalRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
                Debug.WriteLine(result);
            }
            catch (ApiException  e)
            {
                Debug.Print("Exception when calling EvalsApi.RunJudgeEval: " + e.Message);
                Debug.Print("Status Code: " + e.ErrorCode);
                Debug.Print(e.StackTrace);
            }
        }
    }
}
```

#### Using the RunJudgeEvalWithHttpInfo variant
This returns an ApiResponse object which contains the response data, status code and headers.

```csharp
try
{
    ApiResponse<DatasetEvalReport> response = apiInstance.RunJudgeEvalWithHttpInfo(tenantId, projectId, datasetId, versionId, runJudgeDatasetEvalRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
    Debug.Write("Status Code: " + response.StatusCode);
    Debug.Write("Response Headers: " + response.Headers);
    Debug.Write("Response Body: " + response.Data);
}
catch (ApiException e)
{
    Debug.Print("Exception when calling EvalsApi.RunJudgeEvalWithHttpInfo: " + e.Message);
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
| **runJudgeDatasetEvalRequest** | [**RunJudgeDatasetEvalRequest**](RunJudgeDatasetEvalRequest.md) |  |  |
| **authorization** | **string?** | Bearer API token for strict auth | [optional]  |
| **xBeaterApiKey** | **string?** | API key alternative for strict auth | [optional]  |
| **xBeaterProjectId** | **string?** | Strict-auth project scope | [optional]  |
| **xBeaterEnvironmentId** | **string?** | Strict-auth environment scope | [optional]  |

### Return type

[**DatasetEvalReport**](DatasetEvalReport.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Run a judge dataset evaluation |  -  |
| **400** | Invalid request, scope, or filter |  -  |
| **401** | Missing or invalid credentials |  -  |
| **403** | Credentials lack the required scope |  -  |
| **404** | Resource not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
