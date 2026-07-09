# SearchApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**searchSearchSpans**](SearchApi.md#searchSearchSpans) | **GET** /v1/search/{tenant_id}/spans |  |
| [**searchSearchSpansWithHttpInfo**](SearchApi.md#searchSearchSpansWithHttpInfo) | **GET** /v1/search/{tenant_id}/spans |  |



## searchSearchSpans

> SearchResponse searchSearchSpans(tenantId, q, projectId, environmentId, traceId, spanId, kind, status, model, tool, limit, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example

```java
// Import classes:
import ai.beater.client.ApiClient;
import ai.beater.client.ApiException;
import ai.beater.client.Configuration;
import ai.beater.client.models.*;
import ai.beater.client.api.SearchApi;

public class Example {
    public static void main(String[] args) {
        ApiClient defaultClient = Configuration.getDefaultApiClient();
        defaultClient.setBasePath("http://localhost");

        SearchApi apiInstance = new SearchApi(defaultClient);
        String tenantId = "tenantId_example"; // String | tenant_id
        String q = "q_example"; // String |
        String projectId = "projectId_example"; // String |
        String environmentId = "environmentId_example"; // String |
        String traceId = "traceId_example"; // String |
        String spanId = "spanId_example"; // String |
        String kind = "kind_example"; // String |
        String status = "status_example"; // String |
        String model = "model_example"; // String |
        String tool = "tool_example"; // String |
        Integer limit = 56; // Integer |
        String authorization = "authorization_example"; // String | Bearer API token for strict auth
        String xBeaterApiKey = "xBeaterApiKey_example"; // String | API key alternative for strict auth
        String xBeaterProjectId = "xBeaterProjectId_example"; // String | Strict-auth project scope
        String xBeaterEnvironmentId = "xBeaterEnvironmentId_example"; // String | Strict-auth environment scope
        try {
            SearchResponse result = apiInstance.searchSearchSpans(tenantId, q, projectId, environmentId, traceId, spanId, kind, status, model, tool, limit, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
            System.out.println(result);
        } catch (ApiException e) {
            System.err.println("Exception when calling SearchApi#searchSearchSpans");
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
| **q** | **String**|  | [optional] |
| **projectId** | **String**|  | [optional] |
| **environmentId** | **String**|  | [optional] |
| **traceId** | **String**|  | [optional] |
| **spanId** | **String**|  | [optional] |
| **kind** | **String**|  | [optional] |
| **status** | **String**|  | [optional] |
| **model** | **String**|  | [optional] |
| **tool** | **String**|  | [optional] |
| **limit** | **Integer**|  | [optional] |
| **authorization** | **String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **String**| Strict-auth project scope | [optional] |
| **xBeaterEnvironmentId** | **String**| Strict-auth environment scope | [optional] |

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

## searchSearchSpansWithHttpInfo

> ApiResponse<SearchResponse> searchSearchSpans searchSearchSpansWithHttpInfo(tenantId, q, projectId, environmentId, traceId, spanId, kind, status, model, tool, limit, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example

```java
// Import classes:
import ai.beater.client.ApiClient;
import ai.beater.client.ApiException;
import ai.beater.client.ApiResponse;
import ai.beater.client.Configuration;
import ai.beater.client.models.*;
import ai.beater.client.api.SearchApi;

public class Example {
    public static void main(String[] args) {
        ApiClient defaultClient = Configuration.getDefaultApiClient();
        defaultClient.setBasePath("http://localhost");

        SearchApi apiInstance = new SearchApi(defaultClient);
        String tenantId = "tenantId_example"; // String | tenant_id
        String q = "q_example"; // String |
        String projectId = "projectId_example"; // String |
        String environmentId = "environmentId_example"; // String |
        String traceId = "traceId_example"; // String |
        String spanId = "spanId_example"; // String |
        String kind = "kind_example"; // String |
        String status = "status_example"; // String |
        String model = "model_example"; // String |
        String tool = "tool_example"; // String |
        Integer limit = 56; // Integer |
        String authorization = "authorization_example"; // String | Bearer API token for strict auth
        String xBeaterApiKey = "xBeaterApiKey_example"; // String | API key alternative for strict auth
        String xBeaterProjectId = "xBeaterProjectId_example"; // String | Strict-auth project scope
        String xBeaterEnvironmentId = "xBeaterEnvironmentId_example"; // String | Strict-auth environment scope
        try {
            ApiResponse<SearchResponse> response = apiInstance.searchSearchSpansWithHttpInfo(tenantId, q, projectId, environmentId, traceId, spanId, kind, status, model, tool, limit, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
            System.out.println("Status code: " + response.getStatusCode());
            System.out.println("Response headers: " + response.getHeaders());
            System.out.println("Response body: " + response.getData());
        } catch (ApiException e) {
            System.err.println("Exception when calling SearchApi#searchSearchSpans");
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
| **q** | **String**|  | [optional] |
| **projectId** | **String**|  | [optional] |
| **environmentId** | **String**|  | [optional] |
| **traceId** | **String**|  | [optional] |
| **spanId** | **String**|  | [optional] |
| **kind** | **String**|  | [optional] |
| **status** | **String**|  | [optional] |
| **model** | **String**|  | [optional] |
| **tool** | **String**|  | [optional] |
| **limit** | **Integer**|  | [optional] |
| **authorization** | **String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **String**| Strict-auth project scope | [optional] |
| **xBeaterEnvironmentId** | **String**| Strict-auth environment scope | [optional] |

### Return type

ApiResponse<[**SearchResponse**](SearchResponse.md)>


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
