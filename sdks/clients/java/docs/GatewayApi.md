# GatewayApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**gatewayChatCompletions**](GatewayApi.md#gatewayChatCompletions) | **POST** /v1/gateway/{tenant_id}/{project_id}/{environment_id}/chat/completions |  |
| [**gatewayChatCompletionsWithHttpInfo**](GatewayApi.md#gatewayChatCompletionsWithHttpInfo) | **POST** /v1/gateway/{tenant_id}/{project_id}/{environment_id}/chat/completions |  |



## gatewayChatCompletions

> GatewayOutcome gatewayChatCompletions(tenantId, projectId, environmentId, gatewayChatHttpRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example

```java
// Import classes:
import ai.beater.client.ApiClient;
import ai.beater.client.ApiException;
import ai.beater.client.Configuration;
import ai.beater.client.models.*;
import ai.beater.client.api.GatewayApi;

public class Example {
    public static void main(String[] args) {
        ApiClient defaultClient = Configuration.getDefaultApiClient();
        defaultClient.setBasePath("http://localhost");

        GatewayApi apiInstance = new GatewayApi(defaultClient);
        String tenantId = "tenantId_example"; // String | tenant_id
        String projectId = "projectId_example"; // String | project_id
        String environmentId = "environmentId_example"; // String | environment_id
        GatewayChatHttpRequest gatewayChatHttpRequest = new GatewayChatHttpRequest(); // GatewayChatHttpRequest | 
        String authorization = "authorization_example"; // String | Bearer API token for strict auth
        String xBeaterApiKey = "xBeaterApiKey_example"; // String | API key alternative for strict auth
        String xBeaterProjectId = "xBeaterProjectId_example"; // String | Strict-auth project scope
        String xBeaterEnvironmentId = "xBeaterEnvironmentId_example"; // String | Strict-auth environment scope
        try {
            GatewayOutcome result = apiInstance.gatewayChatCompletions(tenantId, projectId, environmentId, gatewayChatHttpRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
            System.out.println(result);
        } catch (ApiException e) {
            System.err.println("Exception when calling GatewayApi#gatewayChatCompletions");
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
| **environmentId** | **String**| environment_id | |
| **gatewayChatHttpRequest** | [**GatewayChatHttpRequest**](GatewayChatHttpRequest.md)|  | |
| **authorization** | **String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **String**| Strict-auth project scope | [optional] |
| **xBeaterEnvironmentId** | **String**| Strict-auth environment scope | [optional] |

### Return type

[**GatewayOutcome**](GatewayOutcome.md)


### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Proxied OpenAI-compatible chat completion |  -  |
| **400** | Invalid request, scope, or no key + no managed default |  -  |
| **401** | Missing or invalid credentials |  -  |
| **402** | Per-tenant budget exceeded |  -  |
| **403** | Credentials lack the required scope |  -  |
| **404** | Provider secret not found or inactive |  -  |
| **502** | All providers/keys failed |  -  |
| **504** | Gateway request timed out |  -  |

## gatewayChatCompletionsWithHttpInfo

> ApiResponse<GatewayOutcome> gatewayChatCompletions gatewayChatCompletionsWithHttpInfo(tenantId, projectId, environmentId, gatewayChatHttpRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example

```java
// Import classes:
import ai.beater.client.ApiClient;
import ai.beater.client.ApiException;
import ai.beater.client.ApiResponse;
import ai.beater.client.Configuration;
import ai.beater.client.models.*;
import ai.beater.client.api.GatewayApi;

public class Example {
    public static void main(String[] args) {
        ApiClient defaultClient = Configuration.getDefaultApiClient();
        defaultClient.setBasePath("http://localhost");

        GatewayApi apiInstance = new GatewayApi(defaultClient);
        String tenantId = "tenantId_example"; // String | tenant_id
        String projectId = "projectId_example"; // String | project_id
        String environmentId = "environmentId_example"; // String | environment_id
        GatewayChatHttpRequest gatewayChatHttpRequest = new GatewayChatHttpRequest(); // GatewayChatHttpRequest | 
        String authorization = "authorization_example"; // String | Bearer API token for strict auth
        String xBeaterApiKey = "xBeaterApiKey_example"; // String | API key alternative for strict auth
        String xBeaterProjectId = "xBeaterProjectId_example"; // String | Strict-auth project scope
        String xBeaterEnvironmentId = "xBeaterEnvironmentId_example"; // String | Strict-auth environment scope
        try {
            ApiResponse<GatewayOutcome> response = apiInstance.gatewayChatCompletionsWithHttpInfo(tenantId, projectId, environmentId, gatewayChatHttpRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
            System.out.println("Status code: " + response.getStatusCode());
            System.out.println("Response headers: " + response.getHeaders());
            System.out.println("Response body: " + response.getData());
        } catch (ApiException e) {
            System.err.println("Exception when calling GatewayApi#gatewayChatCompletions");
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
| **environmentId** | **String**| environment_id | |
| **gatewayChatHttpRequest** | [**GatewayChatHttpRequest**](GatewayChatHttpRequest.md)|  | |
| **authorization** | **String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **String**| Strict-auth project scope | [optional] |
| **xBeaterEnvironmentId** | **String**| Strict-auth environment scope | [optional] |

### Return type

ApiResponse<[**GatewayOutcome**](GatewayOutcome.md)>


### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Proxied OpenAI-compatible chat completion |  -  |
| **400** | Invalid request, scope, or no key + no managed default |  -  |
| **401** | Missing or invalid credentials |  -  |
| **402** | Per-tenant budget exceeded |  -  |
| **403** | Credentials lack the required scope |  -  |
| **404** | Provider secret not found or inactive |  -  |
| **502** | All providers/keys failed |  -  |
| **504** | Gateway request timed out |  -  |

