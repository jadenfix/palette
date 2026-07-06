# TracesApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------------- | ------------- | ------------- |
| [**getTrace**](TracesApi.md#getTrace) | **GET** /v1/traces/{tenant_id}/{trace_id} |  |
| [**listTraces**](TracesApi.md#listTraces) | **GET** /v1/traces/{tenant_id} |  |


<a id="getTrace"></a>
# **getTrace**
> TraceView getTrace(tenantId, traceId, unmask, reason, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = TracesApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val traceId : kotlin.String = traceId_example // kotlin.String | trace_id
val unmask : kotlin.Boolean = true // kotlin.Boolean |
val reason : kotlin.String = reason_example // kotlin.String |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : TraceView = apiInstance.getTrace(tenantId, traceId, unmask, reason, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling TracesApi#getTrace")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling TracesApi#getTrace")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **traceId** | **kotlin.String**| trace_id | |
| **unmask** | **kotlin.Boolean**|  | [optional] |
| **reason** | **kotlin.String**|  | [optional] |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**TraceView**](TraceView.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

<a id="listTraces"></a>
# **listTraces**
> PageRunSummary listTraces(tenantId, projectId, environmentId, traceId, kind, status, startedAfter, startedBefore, model, release, minCostMicros, maxCostMicros, minLatencyMs, maxLatencyMs, limit, cursor, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = TracesApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String |
val environmentId : kotlin.String = environmentId_example // kotlin.String |
val traceId : kotlin.String = traceId_example // kotlin.String |
val kind : kotlin.String = kind_example // kotlin.String |
val status : kotlin.String = status_example // kotlin.String |
val startedAfter : kotlin.String = startedAfter_example // kotlin.String |
val startedBefore : kotlin.String = startedBefore_example // kotlin.String |
val model : kotlin.String = model_example // kotlin.String |
val release : kotlin.String = release_example // kotlin.String |
val minCostMicros : kotlin.Long = 789 // kotlin.Long |
val maxCostMicros : kotlin.Long = 789 // kotlin.Long |
val minLatencyMs : kotlin.Long = 789 // kotlin.Long |
val maxLatencyMs : kotlin.Long = 789 // kotlin.Long |
val limit : kotlin.Int = 56 // kotlin.Int |
val cursor : kotlin.String = cursor_example // kotlin.String |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : PageRunSummary = apiInstance.listTraces(tenantId, projectId, environmentId, traceId, kind, status, startedAfter, startedBefore, model, release, minCostMicros, maxCostMicros, minLatencyMs, maxLatencyMs, limit, cursor, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling TracesApi#listTraces")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling TracesApi#listTraces")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**|  | [optional] |
| **environmentId** | **kotlin.String**|  | [optional] |
| **traceId** | **kotlin.String**|  | [optional] |
| **kind** | **kotlin.String**|  | [optional] |
| **status** | **kotlin.String**|  | [optional] |
| **startedAfter** | **kotlin.String**|  | [optional] |
| **startedBefore** | **kotlin.String**|  | [optional] |
| **model** | **kotlin.String**|  | [optional] |
| **release** | **kotlin.String**|  | [optional] |
| **minCostMicros** | **kotlin.Long**|  | [optional] |
| **maxCostMicros** | **kotlin.Long**|  | [optional] |
| **minLatencyMs** | **kotlin.Long**|  | [optional] |
| **maxLatencyMs** | **kotlin.Long**|  | [optional] |
| **limit** | **kotlin.Int**|  | [optional] |
| **cursor** | **kotlin.String**|  | [optional] |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**PageRunSummary**](PageRunSummary.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json
