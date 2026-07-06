# Beater.Client.Api.TracesApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|--------|--------------|-------------|
| [**GetTrace**](TracesApi.md#gettrace) | **GET** /v1/traces/{tenant_id}/{trace_id} |  |
| [**ListTraces**](TracesApi.md#listtraces) | **GET** /v1/traces/{tenant_id} |  |

<a id="gettrace"></a>
# **GetTrace**
> TraceView GetTrace (string tenantId, string traceId, bool? unmask = null, string? reason = null, string? authorization = null, string? xBeaterApiKey = null, string? xBeaterProjectId = null, string? xBeaterEnvironmentId = null)



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
    public class GetTraceExample
    {
        public static void Main()
        {
            Configuration config = new Configuration();
            config.BasePath = "http://localhost";
            // create instances of HttpClient, HttpClientHandler to be reused later with different Api classes
            HttpClient httpClient = new HttpClient();
            HttpClientHandler httpClientHandler = new HttpClientHandler();
            var apiInstance = new TracesApi(httpClient, config, httpClientHandler);
            var tenantId = "tenantId_example";  // string | tenant_id
            var traceId = "traceId_example";  // string | trace_id
            var unmask = true;  // bool? |  (optional)
            var reason = "reason_example";  // string? |  (optional)
            var authorization = "authorization_example";  // string? | Bearer API token for strict auth (optional)
            var xBeaterApiKey = "xBeaterApiKey_example";  // string? | API key alternative for strict auth (optional)
            var xBeaterProjectId = "xBeaterProjectId_example";  // string? | Strict-auth project scope (optional)
            var xBeaterEnvironmentId = "xBeaterEnvironmentId_example";  // string? | Strict-auth environment scope (optional)

            try
            {
                TraceView result = apiInstance.GetTrace(tenantId, traceId, unmask, reason, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
                Debug.WriteLine(result);
            }
            catch (ApiException  e)
            {
                Debug.Print("Exception when calling TracesApi.GetTrace: " + e.Message);
                Debug.Print("Status Code: " + e.ErrorCode);
                Debug.Print(e.StackTrace);
            }
        }
    }
}
```

#### Using the GetTraceWithHttpInfo variant
This returns an ApiResponse object which contains the response data, status code and headers.

```csharp
try
{
    ApiResponse<TraceView> response = apiInstance.GetTraceWithHttpInfo(tenantId, traceId, unmask, reason, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
    Debug.Write("Status Code: " + response.StatusCode);
    Debug.Write("Response Headers: " + response.Headers);
    Debug.Write("Response Body: " + response.Data);
}
catch (ApiException e)
{
    Debug.Print("Exception when calling TracesApi.GetTraceWithHttpInfo: " + e.Message);
    Debug.Print("Status Code: " + e.ErrorCode);
    Debug.Print(e.StackTrace);
}
```

### Parameters

| Name | Type | Description | Notes |
|------|------|-------------|-------|
| **tenantId** | **string** | tenant_id |  |
| **traceId** | **string** | trace_id |  |
| **unmask** | **bool?** |  | [optional]  |
| **reason** | **string?** |  | [optional]  |
| **authorization** | **string?** | Bearer API token for strict auth | [optional]  |
| **xBeaterApiKey** | **string?** | API key alternative for strict auth | [optional]  |
| **xBeaterProjectId** | **string?** | Strict-auth project scope | [optional]  |
| **xBeaterEnvironmentId** | **string?** | Strict-auth environment scope | [optional]  |

### Return type

[**TraceView**](TraceView.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Get a canonical trace |  -  |
| **400** | Invalid request, scope, or filter |  -  |
| **401** | Missing or invalid credentials |  -  |
| **403** | Credentials lack the required scope |  -  |
| **404** | Resource not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

<a id="listtraces"></a>
# **ListTraces**
> PageRunSummary ListTraces (string tenantId, string? projectId = null, string? environmentId = null, string? traceId = null, string? kind = null, string? status = null, string? startedAfter = null, string? startedBefore = null, string? model = null, string? release = null, long? minCostMicros = null, long? maxCostMicros = null, long? minLatencyMs = null, long? maxLatencyMs = null, int? limit = null, string? cursor = null, string? authorization = null, string? xBeaterApiKey = null, string? xBeaterProjectId = null, string? xBeaterEnvironmentId = null)



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
    public class ListTracesExample
    {
        public static void Main()
        {
            Configuration config = new Configuration();
            config.BasePath = "http://localhost";
            // create instances of HttpClient, HttpClientHandler to be reused later with different Api classes
            HttpClient httpClient = new HttpClient();
            HttpClientHandler httpClientHandler = new HttpClientHandler();
            var apiInstance = new TracesApi(httpClient, config, httpClientHandler);
            var tenantId = "tenantId_example";  // string | tenant_id
            var projectId = "projectId_example";  // string? |  (optional)
            var environmentId = "environmentId_example";  // string? |  (optional)
            var traceId = "traceId_example";  // string? |  (optional)
            var kind = "kind_example";  // string? |  (optional)
            var status = "status_example";  // string? |  (optional)
            var startedAfter = "startedAfter_example";  // string? |  (optional)
            var startedBefore = "startedBefore_example";  // string? |  (optional)
            var model = "model_example";  // string? |  (optional)
            var release = "release_example";  // string? |  (optional)
            var minCostMicros = 789L;  // long? |  (optional)
            var maxCostMicros = 789L;  // long? |  (optional)
            var minLatencyMs = 789L;  // long? |  (optional)
            var maxLatencyMs = 789L;  // long? |  (optional)
            var limit = 56;  // int? |  (optional)
            var cursor = "cursor_example";  // string? |  (optional)
            var authorization = "authorization_example";  // string? | Bearer API token for strict auth (optional)
            var xBeaterApiKey = "xBeaterApiKey_example";  // string? | API key alternative for strict auth (optional)
            var xBeaterProjectId = "xBeaterProjectId_example";  // string? | Strict-auth project scope (optional)
            var xBeaterEnvironmentId = "xBeaterEnvironmentId_example";  // string? | Strict-auth environment scope (optional)

            try
            {
                PageRunSummary result = apiInstance.ListTraces(tenantId, projectId, environmentId, traceId, kind, status, startedAfter, startedBefore, model, release, minCostMicros, maxCostMicros, minLatencyMs, maxLatencyMs, limit, cursor, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
                Debug.WriteLine(result);
            }
            catch (ApiException  e)
            {
                Debug.Print("Exception when calling TracesApi.ListTraces: " + e.Message);
                Debug.Print("Status Code: " + e.ErrorCode);
                Debug.Print(e.StackTrace);
            }
        }
    }
}
```

#### Using the ListTracesWithHttpInfo variant
This returns an ApiResponse object which contains the response data, status code and headers.

```csharp
try
{
    ApiResponse<PageRunSummary> response = apiInstance.ListTracesWithHttpInfo(tenantId, projectId, environmentId, traceId, kind, status, startedAfter, startedBefore, model, release, minCostMicros, maxCostMicros, minLatencyMs, maxLatencyMs, limit, cursor, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
    Debug.Write("Status Code: " + response.StatusCode);
    Debug.Write("Response Headers: " + response.Headers);
    Debug.Write("Response Body: " + response.Data);
}
catch (ApiException e)
{
    Debug.Print("Exception when calling TracesApi.ListTracesWithHttpInfo: " + e.Message);
    Debug.Print("Status Code: " + e.ErrorCode);
    Debug.Print(e.StackTrace);
}
```

### Parameters

| Name | Type | Description | Notes |
|------|------|-------------|-------|
| **tenantId** | **string** | tenant_id |  |
| **projectId** | **string?** |  | [optional]  |
| **environmentId** | **string?** |  | [optional]  |
| **traceId** | **string?** |  | [optional]  |
| **kind** | **string?** |  | [optional]  |
| **status** | **string?** |  | [optional]  |
| **startedAfter** | **string?** |  | [optional]  |
| **startedBefore** | **string?** |  | [optional]  |
| **model** | **string?** |  | [optional]  |
| **release** | **string?** |  | [optional]  |
| **minCostMicros** | **long?** |  | [optional]  |
| **maxCostMicros** | **long?** |  | [optional]  |
| **minLatencyMs** | **long?** |  | [optional]  |
| **maxLatencyMs** | **long?** |  | [optional]  |
| **limit** | **int?** |  | [optional]  |
| **cursor** | **string?** |  | [optional]  |
| **authorization** | **string?** | Bearer API token for strict auth | [optional]  |
| **xBeaterApiKey** | **string?** | API key alternative for strict auth | [optional]  |
| **xBeaterProjectId** | **string?** | Strict-auth project scope | [optional]  |
| **xBeaterEnvironmentId** | **string?** | Strict-auth environment scope | [optional]  |

### Return type

[**PageRunSummary**](PageRunSummary.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | List trace run summaries |  -  |
| **400** | Invalid request, scope, or filter |  -  |
| **401** | Missing or invalid credentials |  -  |
| **403** | Credentials lack the required scope |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
