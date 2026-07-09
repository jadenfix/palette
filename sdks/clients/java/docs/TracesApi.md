# TracesApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**tracesGetTrace**](TracesApi.md#tracesGetTrace) | **GET** /v1/traces/{tenant_id}/{trace_id} |  |
| [**tracesGetTraceWithHttpInfo**](TracesApi.md#tracesGetTraceWithHttpInfo) | **GET** /v1/traces/{tenant_id}/{trace_id} |  |
| [**tracesListTraces**](TracesApi.md#tracesListTraces) | **GET** /v1/traces/{tenant_id} |  |
| [**tracesListTracesWithHttpInfo**](TracesApi.md#tracesListTracesWithHttpInfo) | **GET** /v1/traces/{tenant_id} |  |



## tracesGetTrace

> TraceView tracesGetTrace(tenantId, traceId, unmask, reason, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example

```java
// Import classes:
import ai.beater.client.ApiClient;
import ai.beater.client.ApiException;
import ai.beater.client.Configuration;
import ai.beater.client.models.*;
import ai.beater.client.api.TracesApi;

public class Example {
    public static void main(String[] args) {
        ApiClient defaultClient = Configuration.getDefaultApiClient();
        defaultClient.setBasePath("http://localhost");

        TracesApi apiInstance = new TracesApi(defaultClient);
        String tenantId = "tenantId_example"; // String | tenant_id
        String traceId = "traceId_example"; // String | trace_id
        Boolean unmask = true; // Boolean |
        String reason = "reason_example"; // String |
        String authorization = "authorization_example"; // String | Bearer API token for strict auth
        String xBeaterApiKey = "xBeaterApiKey_example"; // String | API key alternative for strict auth
        String xBeaterProjectId = "xBeaterProjectId_example"; // String | Strict-auth project scope
        String xBeaterEnvironmentId = "xBeaterEnvironmentId_example"; // String | Strict-auth environment scope
        try {
            TraceView result = apiInstance.tracesGetTrace(tenantId, traceId, unmask, reason, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
            System.out.println(result);
        } catch (ApiException e) {
            System.err.println("Exception when calling TracesApi#tracesGetTrace");
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
| **traceId** | **String**| trace_id | |
| **unmask** | **Boolean**|  | [optional] |
| **reason** | **String**|  | [optional] |
| **authorization** | **String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **String**| Strict-auth project scope | [optional] |
| **xBeaterEnvironmentId** | **String**| Strict-auth environment scope | [optional] |

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

## tracesGetTraceWithHttpInfo

> ApiResponse<TraceView> tracesGetTrace tracesGetTraceWithHttpInfo(tenantId, traceId, unmask, reason, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example

```java
// Import classes:
import ai.beater.client.ApiClient;
import ai.beater.client.ApiException;
import ai.beater.client.ApiResponse;
import ai.beater.client.Configuration;
import ai.beater.client.models.*;
import ai.beater.client.api.TracesApi;

public class Example {
    public static void main(String[] args) {
        ApiClient defaultClient = Configuration.getDefaultApiClient();
        defaultClient.setBasePath("http://localhost");

        TracesApi apiInstance = new TracesApi(defaultClient);
        String tenantId = "tenantId_example"; // String | tenant_id
        String traceId = "traceId_example"; // String | trace_id
        Boolean unmask = true; // Boolean |
        String reason = "reason_example"; // String |
        String authorization = "authorization_example"; // String | Bearer API token for strict auth
        String xBeaterApiKey = "xBeaterApiKey_example"; // String | API key alternative for strict auth
        String xBeaterProjectId = "xBeaterProjectId_example"; // String | Strict-auth project scope
        String xBeaterEnvironmentId = "xBeaterEnvironmentId_example"; // String | Strict-auth environment scope
        try {
            ApiResponse<TraceView> response = apiInstance.tracesGetTraceWithHttpInfo(tenantId, traceId, unmask, reason, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
            System.out.println("Status code: " + response.getStatusCode());
            System.out.println("Response headers: " + response.getHeaders());
            System.out.println("Response body: " + response.getData());
        } catch (ApiException e) {
            System.err.println("Exception when calling TracesApi#tracesGetTrace");
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
| **traceId** | **String**| trace_id | |
| **unmask** | **Boolean**|  | [optional] |
| **reason** | **String**|  | [optional] |
| **authorization** | **String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **String**| Strict-auth project scope | [optional] |
| **xBeaterEnvironmentId** | **String**| Strict-auth environment scope | [optional] |

### Return type

ApiResponse<[**TraceView**](TraceView.md)>


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


## tracesListTraces

> PageRunSummary tracesListTraces(tenantId, projectId, environmentId, traceId, kind, status, startedAfter, startedBefore, model, release, minCostMicros, maxCostMicros, minLatencyMs, maxLatencyMs, limit, cursor, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example

```java
// Import classes:
import ai.beater.client.ApiClient;
import ai.beater.client.ApiException;
import ai.beater.client.Configuration;
import ai.beater.client.models.*;
import ai.beater.client.api.TracesApi;

public class Example {
    public static void main(String[] args) {
        ApiClient defaultClient = Configuration.getDefaultApiClient();
        defaultClient.setBasePath("http://localhost");

        TracesApi apiInstance = new TracesApi(defaultClient);
        String tenantId = "tenantId_example"; // String | tenant_id
        String projectId = "projectId_example"; // String |
        String environmentId = "environmentId_example"; // String |
        String traceId = "traceId_example"; // String |
        String kind = "kind_example"; // String |
        String status = "status_example"; // String |
        String startedAfter = "startedAfter_example"; // String |
        String startedBefore = "startedBefore_example"; // String |
        String model = "model_example"; // String |
        String release = "release_example"; // String |
        Long minCostMicros = 56L; // Long |
        Long maxCostMicros = 56L; // Long |
        Long minLatencyMs = 56L; // Long |
        Long maxLatencyMs = 56L; // Long |
        Integer limit = 56; // Integer |
        String cursor = "cursor_example"; // String |
        String authorization = "authorization_example"; // String | Bearer API token for strict auth
        String xBeaterApiKey = "xBeaterApiKey_example"; // String | API key alternative for strict auth
        String xBeaterProjectId = "xBeaterProjectId_example"; // String | Strict-auth project scope
        String xBeaterEnvironmentId = "xBeaterEnvironmentId_example"; // String | Strict-auth environment scope
        try {
            PageRunSummary result = apiInstance.tracesListTraces(tenantId, projectId, environmentId, traceId, kind, status, startedAfter, startedBefore, model, release, minCostMicros, maxCostMicros, minLatencyMs, maxLatencyMs, limit, cursor, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
            System.out.println(result);
        } catch (ApiException e) {
            System.err.println("Exception when calling TracesApi#tracesListTraces");
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
| **projectId** | **String**|  | [optional] |
| **environmentId** | **String**|  | [optional] |
| **traceId** | **String**|  | [optional] |
| **kind** | **String**|  | [optional] |
| **status** | **String**|  | [optional] |
| **startedAfter** | **String**|  | [optional] |
| **startedBefore** | **String**|  | [optional] |
| **model** | **String**|  | [optional] |
| **release** | **String**|  | [optional] |
| **minCostMicros** | **Long**|  | [optional] |
| **maxCostMicros** | **Long**|  | [optional] |
| **minLatencyMs** | **Long**|  | [optional] |
| **maxLatencyMs** | **Long**|  | [optional] |
| **limit** | **Integer**|  | [optional] |
| **cursor** | **String**|  | [optional] |
| **authorization** | **String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **String**| Strict-auth project scope | [optional] |
| **xBeaterEnvironmentId** | **String**| Strict-auth environment scope | [optional] |

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

## tracesListTracesWithHttpInfo

> ApiResponse<PageRunSummary> tracesListTraces tracesListTracesWithHttpInfo(tenantId, projectId, environmentId, traceId, kind, status, startedAfter, startedBefore, model, release, minCostMicros, maxCostMicros, minLatencyMs, maxLatencyMs, limit, cursor, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example

```java
// Import classes:
import ai.beater.client.ApiClient;
import ai.beater.client.ApiException;
import ai.beater.client.ApiResponse;
import ai.beater.client.Configuration;
import ai.beater.client.models.*;
import ai.beater.client.api.TracesApi;

public class Example {
    public static void main(String[] args) {
        ApiClient defaultClient = Configuration.getDefaultApiClient();
        defaultClient.setBasePath("http://localhost");

        TracesApi apiInstance = new TracesApi(defaultClient);
        String tenantId = "tenantId_example"; // String | tenant_id
        String projectId = "projectId_example"; // String |
        String environmentId = "environmentId_example"; // String |
        String traceId = "traceId_example"; // String |
        String kind = "kind_example"; // String |
        String status = "status_example"; // String |
        String startedAfter = "startedAfter_example"; // String |
        String startedBefore = "startedBefore_example"; // String |
        String model = "model_example"; // String |
        String release = "release_example"; // String |
        Long minCostMicros = 56L; // Long |
        Long maxCostMicros = 56L; // Long |
        Long minLatencyMs = 56L; // Long |
        Long maxLatencyMs = 56L; // Long |
        Integer limit = 56; // Integer |
        String cursor = "cursor_example"; // String |
        String authorization = "authorization_example"; // String | Bearer API token for strict auth
        String xBeaterApiKey = "xBeaterApiKey_example"; // String | API key alternative for strict auth
        String xBeaterProjectId = "xBeaterProjectId_example"; // String | Strict-auth project scope
        String xBeaterEnvironmentId = "xBeaterEnvironmentId_example"; // String | Strict-auth environment scope
        try {
            ApiResponse<PageRunSummary> response = apiInstance.tracesListTracesWithHttpInfo(tenantId, projectId, environmentId, traceId, kind, status, startedAfter, startedBefore, model, release, minCostMicros, maxCostMicros, minLatencyMs, maxLatencyMs, limit, cursor, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
            System.out.println("Status code: " + response.getStatusCode());
            System.out.println("Response headers: " + response.getHeaders());
            System.out.println("Response body: " + response.getData());
        } catch (ApiException e) {
            System.err.println("Exception when calling TracesApi#tracesListTraces");
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
| **projectId** | **String**|  | [optional] |
| **environmentId** | **String**|  | [optional] |
| **traceId** | **String**|  | [optional] |
| **kind** | **String**|  | [optional] |
| **status** | **String**|  | [optional] |
| **startedAfter** | **String**|  | [optional] |
| **startedBefore** | **String**|  | [optional] |
| **model** | **String**|  | [optional] |
| **release** | **String**|  | [optional] |
| **minCostMicros** | **Long**|  | [optional] |
| **maxCostMicros** | **Long**|  | [optional] |
| **minLatencyMs** | **Long**|  | [optional] |
| **maxLatencyMs** | **Long**|  | [optional] |
| **limit** | **Integer**|  | [optional] |
| **cursor** | **String**|  | [optional] |
| **authorization** | **String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **String**| Strict-auth project scope | [optional] |
| **xBeaterEnvironmentId** | **String**| Strict-auth environment scope | [optional] |

### Return type

ApiResponse<[**PageRunSummary**](PageRunSummary.md)>


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
