# Beater.Client.Api.ApiKeysApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|--------|--------------|-------------|
| [**CreateApiKey**](ApiKeysApi.md#createapikey) | **POST** /v1/api-keys/{tenant_id}/{project_id}/{environment_id} |  |
| [**RevokeApiKey**](ApiKeysApi.md#revokeapikey) | **POST** /v1/api-keys/{tenant_id}/{project_id}/{environment_id}/{api_key_id}/revoke |  |

<a id="createapikey"></a>
# **CreateApiKey**
> ApiKeyCreatedResponse CreateApiKey (string tenantId, string projectId, string environmentId, CreateApiKeyHttpRequest createApiKeyHttpRequest, string? authorization = null, string? xBeaterApiKey = null, string? xBeaterProjectId = null, string? xBeaterEnvironmentId = null)



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
    public class CreateApiKeyExample
    {
        public static void Main()
        {
            Configuration config = new Configuration();
            config.BasePath = "http://localhost";
            // create instances of HttpClient, HttpClientHandler to be reused later with different Api classes
            HttpClient httpClient = new HttpClient();
            HttpClientHandler httpClientHandler = new HttpClientHandler();
            var apiInstance = new ApiKeysApi(httpClient, config, httpClientHandler);
            var tenantId = "tenantId_example";  // string | tenant_id
            var projectId = "projectId_example";  // string | project_id
            var environmentId = "environmentId_example";  // string | environment_id
            var createApiKeyHttpRequest = new CreateApiKeyHttpRequest(); // CreateApiKeyHttpRequest |
            var authorization = "authorization_example";  // string? | Bearer API token for strict auth (optional)
            var xBeaterApiKey = "xBeaterApiKey_example";  // string? | API key alternative for strict auth (optional)
            var xBeaterProjectId = "xBeaterProjectId_example";  // string? | Strict-auth project scope (optional)
            var xBeaterEnvironmentId = "xBeaterEnvironmentId_example";  // string? | Strict-auth environment scope (optional)

            try
            {
                ApiKeyCreatedResponse result = apiInstance.CreateApiKey(tenantId, projectId, environmentId, createApiKeyHttpRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
                Debug.WriteLine(result);
            }
            catch (ApiException  e)
            {
                Debug.Print("Exception when calling ApiKeysApi.CreateApiKey: " + e.Message);
                Debug.Print("Status Code: " + e.ErrorCode);
                Debug.Print(e.StackTrace);
            }
        }
    }
}
```

#### Using the CreateApiKeyWithHttpInfo variant
This returns an ApiResponse object which contains the response data, status code and headers.

```csharp
try
{
    ApiResponse<ApiKeyCreatedResponse> response = apiInstance.CreateApiKeyWithHttpInfo(tenantId, projectId, environmentId, createApiKeyHttpRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
    Debug.Write("Status Code: " + response.StatusCode);
    Debug.Write("Response Headers: " + response.Headers);
    Debug.Write("Response Body: " + response.Data);
}
catch (ApiException e)
{
    Debug.Print("Exception when calling ApiKeysApi.CreateApiKeyWithHttpInfo: " + e.Message);
    Debug.Print("Status Code: " + e.ErrorCode);
    Debug.Print(e.StackTrace);
}
```

### Parameters

| Name | Type | Description | Notes |
|------|------|-------------|-------|
| **tenantId** | **string** | tenant_id |  |
| **projectId** | **string** | project_id |  |
| **environmentId** | **string** | environment_id |  |
| **createApiKeyHttpRequest** | [**CreateApiKeyHttpRequest**](CreateApiKeyHttpRequest.md) |  |  |
| **authorization** | **string?** | Bearer API token for strict auth | [optional]  |
| **xBeaterApiKey** | **string?** | API key alternative for strict auth | [optional]  |
| **xBeaterProjectId** | **string?** | Strict-auth project scope | [optional]  |
| **xBeaterEnvironmentId** | **string?** | Strict-auth environment scope | [optional]  |

### Return type

[**ApiKeyCreatedResponse**](ApiKeyCreatedResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Create a scoped API key |  -  |
| **400** | Invalid request, scope, or filter |  -  |
| **401** | Missing or invalid credentials |  -  |
| **403** | Credentials lack the required scope |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

<a id="revokeapikey"></a>
# **RevokeApiKey**
> RevokedApiKey RevokeApiKey (string tenantId, string projectId, string environmentId, string apiKeyId, string? authorization = null, string? xBeaterApiKey = null, string? xBeaterProjectId = null, string? xBeaterEnvironmentId = null)



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
    public class RevokeApiKeyExample
    {
        public static void Main()
        {
            Configuration config = new Configuration();
            config.BasePath = "http://localhost";
            // create instances of HttpClient, HttpClientHandler to be reused later with different Api classes
            HttpClient httpClient = new HttpClient();
            HttpClientHandler httpClientHandler = new HttpClientHandler();
            var apiInstance = new ApiKeysApi(httpClient, config, httpClientHandler);
            var tenantId = "tenantId_example";  // string | tenant_id
            var projectId = "projectId_example";  // string | project_id
            var environmentId = "environmentId_example";  // string | environment_id
            var apiKeyId = "apiKeyId_example";  // string | api_key_id
            var authorization = "authorization_example";  // string? | Bearer API token for strict auth (optional)
            var xBeaterApiKey = "xBeaterApiKey_example";  // string? | API key alternative for strict auth (optional)
            var xBeaterProjectId = "xBeaterProjectId_example";  // string? | Strict-auth project scope (optional)
            var xBeaterEnvironmentId = "xBeaterEnvironmentId_example";  // string? | Strict-auth environment scope (optional)

            try
            {
                RevokedApiKey result = apiInstance.RevokeApiKey(tenantId, projectId, environmentId, apiKeyId, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
                Debug.WriteLine(result);
            }
            catch (ApiException  e)
            {
                Debug.Print("Exception when calling ApiKeysApi.RevokeApiKey: " + e.Message);
                Debug.Print("Status Code: " + e.ErrorCode);
                Debug.Print(e.StackTrace);
            }
        }
    }
}
```

#### Using the RevokeApiKeyWithHttpInfo variant
This returns an ApiResponse object which contains the response data, status code and headers.

```csharp
try
{
    ApiResponse<RevokedApiKey> response = apiInstance.RevokeApiKeyWithHttpInfo(tenantId, projectId, environmentId, apiKeyId, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
    Debug.Write("Status Code: " + response.StatusCode);
    Debug.Write("Response Headers: " + response.Headers);
    Debug.Write("Response Body: " + response.Data);
}
catch (ApiException e)
{
    Debug.Print("Exception when calling ApiKeysApi.RevokeApiKeyWithHttpInfo: " + e.Message);
    Debug.Print("Status Code: " + e.ErrorCode);
    Debug.Print(e.StackTrace);
}
```

### Parameters

| Name | Type | Description | Notes |
|------|------|-------------|-------|
| **tenantId** | **string** | tenant_id |  |
| **projectId** | **string** | project_id |  |
| **environmentId** | **string** | environment_id |  |
| **apiKeyId** | **string** | api_key_id |  |
| **authorization** | **string?** | Bearer API token for strict auth | [optional]  |
| **xBeaterApiKey** | **string?** | API key alternative for strict auth | [optional]  |
| **xBeaterProjectId** | **string?** | Strict-auth project scope | [optional]  |
| **xBeaterEnvironmentId** | **string?** | Strict-auth environment scope | [optional]  |

### Return type

[**RevokedApiKey**](RevokedApiKey.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Revoke an API key |  -  |
| **400** | Invalid request, scope, or filter |  -  |
| **401** | Missing or invalid credentials |  -  |
| **403** | Credentials lack the required scope |  -  |
| **404** | Resource not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
