# Beater.Client.Api.ProviderSecretsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|--------|--------------|-------------|
| [**CreateProviderSecret**](ProviderSecretsApi.md#createprovidersecret) | **POST** /v1/provider-secrets/{tenant_id}/{project_id} |  |
| [**ListProviderSecrets**](ProviderSecretsApi.md#listprovidersecrets) | **GET** /v1/provider-secrets/{tenant_id}/{project_id} |  |
| [**RevokeProviderSecret**](ProviderSecretsApi.md#revokeprovidersecret) | **POST** /v1/provider-secrets/{tenant_id}/{project_id}/{provider_secret_id}/revoke |  |

<a id="createprovidersecret"></a>
# **CreateProviderSecret**
> ProviderSecretMetadata CreateProviderSecret (string tenantId, string projectId, CreateProviderSecretHttpRequest createProviderSecretHttpRequest, string? authorization = null, string? xBeaterApiKey = null, string? xBeaterProjectId = null, string? xBeaterEnvironmentId = null)



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
    public class CreateProviderSecretExample
    {
        public static void Main()
        {
            Configuration config = new Configuration();
            config.BasePath = "http://localhost";
            // create instances of HttpClient, HttpClientHandler to be reused later with different Api classes
            HttpClient httpClient = new HttpClient();
            HttpClientHandler httpClientHandler = new HttpClientHandler();
            var apiInstance = new ProviderSecretsApi(httpClient, config, httpClientHandler);
            var tenantId = "tenantId_example";  // string | tenant_id
            var projectId = "projectId_example";  // string | project_id
            var createProviderSecretHttpRequest = new CreateProviderSecretHttpRequest(); // CreateProviderSecretHttpRequest |
            var authorization = "authorization_example";  // string? | Bearer API token for strict auth (optional)
            var xBeaterApiKey = "xBeaterApiKey_example";  // string? | API key alternative for strict auth (optional)
            var xBeaterProjectId = "xBeaterProjectId_example";  // string? | Strict-auth project scope (optional)
            var xBeaterEnvironmentId = "xBeaterEnvironmentId_example";  // string? | Strict-auth environment scope (optional)

            try
            {
                ProviderSecretMetadata result = apiInstance.CreateProviderSecret(tenantId, projectId, createProviderSecretHttpRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
                Debug.WriteLine(result);
            }
            catch (ApiException  e)
            {
                Debug.Print("Exception when calling ProviderSecretsApi.CreateProviderSecret: " + e.Message);
                Debug.Print("Status Code: " + e.ErrorCode);
                Debug.Print(e.StackTrace);
            }
        }
    }
}
```

#### Using the CreateProviderSecretWithHttpInfo variant
This returns an ApiResponse object which contains the response data, status code and headers.

```csharp
try
{
    ApiResponse<ProviderSecretMetadata> response = apiInstance.CreateProviderSecretWithHttpInfo(tenantId, projectId, createProviderSecretHttpRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
    Debug.Write("Status Code: " + response.StatusCode);
    Debug.Write("Response Headers: " + response.Headers);
    Debug.Write("Response Body: " + response.Data);
}
catch (ApiException e)
{
    Debug.Print("Exception when calling ProviderSecretsApi.CreateProviderSecretWithHttpInfo: " + e.Message);
    Debug.Print("Status Code: " + e.ErrorCode);
    Debug.Print(e.StackTrace);
}
```

### Parameters

| Name | Type | Description | Notes |
|------|------|-------------|-------|
| **tenantId** | **string** | tenant_id |  |
| **projectId** | **string** | project_id |  |
| **createProviderSecretHttpRequest** | [**CreateProviderSecretHttpRequest**](CreateProviderSecretHttpRequest.md) |  |  |
| **authorization** | **string?** | Bearer API token for strict auth | [optional]  |
| **xBeaterApiKey** | **string?** | API key alternative for strict auth | [optional]  |
| **xBeaterProjectId** | **string?** | Strict-auth project scope | [optional]  |
| **xBeaterEnvironmentId** | **string?** | Strict-auth environment scope | [optional]  |

### Return type

[**ProviderSecretMetadata**](ProviderSecretMetadata.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Store an encrypted provider secret |  -  |
| **400** | Invalid request, scope, or filter |  -  |
| **401** | Missing or invalid credentials |  -  |
| **403** | Credentials lack the required scope |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

<a id="listprovidersecrets"></a>
# **ListProviderSecrets**
> List&lt;ProviderSecretMetadata&gt; ListProviderSecrets (string tenantId, string projectId, string? authorization = null, string? xBeaterApiKey = null, string? xBeaterProjectId = null, string? xBeaterEnvironmentId = null)



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
    public class ListProviderSecretsExample
    {
        public static void Main()
        {
            Configuration config = new Configuration();
            config.BasePath = "http://localhost";
            // create instances of HttpClient, HttpClientHandler to be reused later with different Api classes
            HttpClient httpClient = new HttpClient();
            HttpClientHandler httpClientHandler = new HttpClientHandler();
            var apiInstance = new ProviderSecretsApi(httpClient, config, httpClientHandler);
            var tenantId = "tenantId_example";  // string | tenant_id
            var projectId = "projectId_example";  // string | project_id
            var authorization = "authorization_example";  // string? | Bearer API token for strict auth (optional)
            var xBeaterApiKey = "xBeaterApiKey_example";  // string? | API key alternative for strict auth (optional)
            var xBeaterProjectId = "xBeaterProjectId_example";  // string? | Strict-auth project scope (optional)
            var xBeaterEnvironmentId = "xBeaterEnvironmentId_example";  // string? | Strict-auth environment scope (optional)

            try
            {
                List<ProviderSecretMetadata> result = apiInstance.ListProviderSecrets(tenantId, projectId, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
                Debug.WriteLine(result);
            }
            catch (ApiException  e)
            {
                Debug.Print("Exception when calling ProviderSecretsApi.ListProviderSecrets: " + e.Message);
                Debug.Print("Status Code: " + e.ErrorCode);
                Debug.Print(e.StackTrace);
            }
        }
    }
}
```

#### Using the ListProviderSecretsWithHttpInfo variant
This returns an ApiResponse object which contains the response data, status code and headers.

```csharp
try
{
    ApiResponse<List<ProviderSecretMetadata>> response = apiInstance.ListProviderSecretsWithHttpInfo(tenantId, projectId, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
    Debug.Write("Status Code: " + response.StatusCode);
    Debug.Write("Response Headers: " + response.Headers);
    Debug.Write("Response Body: " + response.Data);
}
catch (ApiException e)
{
    Debug.Print("Exception when calling ProviderSecretsApi.ListProviderSecretsWithHttpInfo: " + e.Message);
    Debug.Print("Status Code: " + e.ErrorCode);
    Debug.Print(e.StackTrace);
}
```

### Parameters

| Name | Type | Description | Notes |
|------|------|-------------|-------|
| **tenantId** | **string** | tenant_id |  |
| **projectId** | **string** | project_id |  |
| **authorization** | **string?** | Bearer API token for strict auth | [optional]  |
| **xBeaterApiKey** | **string?** | API key alternative for strict auth | [optional]  |
| **xBeaterProjectId** | **string?** | Strict-auth project scope | [optional]  |
| **xBeaterEnvironmentId** | **string?** | Strict-auth environment scope | [optional]  |

### Return type

[**List&lt;ProviderSecretMetadata&gt;**](ProviderSecretMetadata.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | List provider secret metadata |  -  |
| **400** | Invalid request, scope, or filter |  -  |
| **401** | Missing or invalid credentials |  -  |
| **403** | Credentials lack the required scope |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

<a id="revokeprovidersecret"></a>
# **RevokeProviderSecret**
> RevokedProviderSecret RevokeProviderSecret (string tenantId, string projectId, string providerSecretId, string? authorization = null, string? xBeaterApiKey = null, string? xBeaterProjectId = null, string? xBeaterEnvironmentId = null)



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
    public class RevokeProviderSecretExample
    {
        public static void Main()
        {
            Configuration config = new Configuration();
            config.BasePath = "http://localhost";
            // create instances of HttpClient, HttpClientHandler to be reused later with different Api classes
            HttpClient httpClient = new HttpClient();
            HttpClientHandler httpClientHandler = new HttpClientHandler();
            var apiInstance = new ProviderSecretsApi(httpClient, config, httpClientHandler);
            var tenantId = "tenantId_example";  // string | tenant_id
            var projectId = "projectId_example";  // string | project_id
            var providerSecretId = "providerSecretId_example";  // string | provider_secret_id
            var authorization = "authorization_example";  // string? | Bearer API token for strict auth (optional)
            var xBeaterApiKey = "xBeaterApiKey_example";  // string? | API key alternative for strict auth (optional)
            var xBeaterProjectId = "xBeaterProjectId_example";  // string? | Strict-auth project scope (optional)
            var xBeaterEnvironmentId = "xBeaterEnvironmentId_example";  // string? | Strict-auth environment scope (optional)

            try
            {
                RevokedProviderSecret result = apiInstance.RevokeProviderSecret(tenantId, projectId, providerSecretId, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
                Debug.WriteLine(result);
            }
            catch (ApiException  e)
            {
                Debug.Print("Exception when calling ProviderSecretsApi.RevokeProviderSecret: " + e.Message);
                Debug.Print("Status Code: " + e.ErrorCode);
                Debug.Print(e.StackTrace);
            }
        }
    }
}
```

#### Using the RevokeProviderSecretWithHttpInfo variant
This returns an ApiResponse object which contains the response data, status code and headers.

```csharp
try
{
    ApiResponse<RevokedProviderSecret> response = apiInstance.RevokeProviderSecretWithHttpInfo(tenantId, projectId, providerSecretId, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
    Debug.Write("Status Code: " + response.StatusCode);
    Debug.Write("Response Headers: " + response.Headers);
    Debug.Write("Response Body: " + response.Data);
}
catch (ApiException e)
{
    Debug.Print("Exception when calling ProviderSecretsApi.RevokeProviderSecretWithHttpInfo: " + e.Message);
    Debug.Print("Status Code: " + e.ErrorCode);
    Debug.Print(e.StackTrace);
}
```

### Parameters

| Name | Type | Description | Notes |
|------|------|-------------|-------|
| **tenantId** | **string** | tenant_id |  |
| **projectId** | **string** | project_id |  |
| **providerSecretId** | **string** | provider_secret_id |  |
| **authorization** | **string?** | Bearer API token for strict auth | [optional]  |
| **xBeaterApiKey** | **string?** | API key alternative for strict auth | [optional]  |
| **xBeaterProjectId** | **string?** | Strict-auth project scope | [optional]  |
| **xBeaterEnvironmentId** | **string?** | Strict-auth environment scope | [optional]  |

### Return type

[**RevokedProviderSecret**](RevokedProviderSecret.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Revoke a provider secret |  -  |
| **400** | Invalid request, scope, or filter |  -  |
| **401** | Missing or invalid credentials |  -  |
| **403** | Credentials lack the required scope |  -  |
| **404** | Resource not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
