# Beater.Client.Api.SearchApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|--------|--------------|-------------|
| [**SearchSpans**](SearchApi.md#searchspans) | **GET** /v1/search/{tenant_id}/spans |  |

<a id="searchspans"></a>
# **SearchSpans**
> SearchResponse SearchSpans (string tenantId, string? q = null, string? projectId = null, string? environmentId = null, string? traceId = null, string? spanId = null, string? kind = null, string? status = null, string? model = null, string? tool = null, int? limit = null, string? authorization = null, string? xBeaterApiKey = null, string? xBeaterProjectId = null, string? xBeaterEnvironmentId = null)



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
    public class SearchSpansExample
    {
        public static void Main()
        {
            Configuration config = new Configuration();
            config.BasePath = "http://localhost";
            // create instances of HttpClient, HttpClientHandler to be reused later with different Api classes
            HttpClient httpClient = new HttpClient();
            HttpClientHandler httpClientHandler = new HttpClientHandler();
            var apiInstance = new SearchApi(httpClient, config, httpClientHandler);
            var tenantId = "tenantId_example";  // string | tenant_id
            var q = "q_example";  // string? |  (optional)
            var projectId = "projectId_example";  // string? |  (optional)
            var environmentId = "environmentId_example";  // string? |  (optional)
            var traceId = "traceId_example";  // string? |  (optional)
            var spanId = "spanId_example";  // string? |  (optional)
            var kind = "kind_example";  // string? |  (optional)
            var status = "status_example";  // string? |  (optional)
            var model = "model_example";  // string? |  (optional)
            var tool = "tool_example";  // string? |  (optional)
            var limit = 56;  // int? |  (optional)
            var authorization = "authorization_example";  // string? | Bearer API token for strict auth (optional)
            var xBeaterApiKey = "xBeaterApiKey_example";  // string? | API key alternative for strict auth (optional)
            var xBeaterProjectId = "xBeaterProjectId_example";  // string? | Strict-auth project scope (optional)
            var xBeaterEnvironmentId = "xBeaterEnvironmentId_example";  // string? | Strict-auth environment scope (optional)

            try
            {
                SearchResponse result = apiInstance.SearchSpans(tenantId, q, projectId, environmentId, traceId, spanId, kind, status, model, tool, limit, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
                Debug.WriteLine(result);
            }
            catch (ApiException  e)
            {
                Debug.Print("Exception when calling SearchApi.SearchSpans: " + e.Message);
                Debug.Print("Status Code: " + e.ErrorCode);
                Debug.Print(e.StackTrace);
            }
        }
    }
}
```

#### Using the SearchSpansWithHttpInfo variant
This returns an ApiResponse object which contains the response data, status code and headers.

```csharp
try
{
    ApiResponse<SearchResponse> response = apiInstance.SearchSpansWithHttpInfo(tenantId, q, projectId, environmentId, traceId, spanId, kind, status, model, tool, limit, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
    Debug.Write("Status Code: " + response.StatusCode);
    Debug.Write("Response Headers: " + response.Headers);
    Debug.Write("Response Body: " + response.Data);
}
catch (ApiException e)
{
    Debug.Print("Exception when calling SearchApi.SearchSpansWithHttpInfo: " + e.Message);
    Debug.Print("Status Code: " + e.ErrorCode);
    Debug.Print(e.StackTrace);
}
```

### Parameters

| Name | Type | Description | Notes |
|------|------|-------------|-------|
| **tenantId** | **string** | tenant_id |  |
| **q** | **string?** |  | [optional]  |
| **projectId** | **string?** |  | [optional]  |
| **environmentId** | **string?** |  | [optional]  |
| **traceId** | **string?** |  | [optional]  |
| **spanId** | **string?** |  | [optional]  |
| **kind** | **string?** |  | [optional]  |
| **status** | **string?** |  | [optional]  |
| **model** | **string?** |  | [optional]  |
| **tool** | **string?** |  | [optional]  |
| **limit** | **int?** |  | [optional]  |
| **authorization** | **string?** | Bearer API token for strict auth | [optional]  |
| **xBeaterApiKey** | **string?** | API key alternative for strict auth | [optional]  |
| **xBeaterProjectId** | **string?** | Strict-auth project scope | [optional]  |
| **xBeaterEnvironmentId** | **string?** | Strict-auth environment scope | [optional]  |

### Return type

[**SearchResponse**](SearchResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Search spans |  -  |
| **400** | Invalid request, scope, or filter |  -  |
| **401** | Missing or invalid credentials |  -  |
| **403** | Credentials lack the required scope |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
