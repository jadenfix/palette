# SearchApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------------- | ------------- | ------------- |
| [**searchSpans**](SearchApi.md#searchSpans) | **GET** /v1/search/{tenant_id}/spans |  |


<a id="searchSpans"></a>
# **searchSpans**
> SearchResponse searchSpans(tenantId, q, projectId, environmentId, traceId, spanId, kind, status, model, tool, limit, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = SearchApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val q : kotlin.String = q_example // kotlin.String |
val projectId : kotlin.String = projectId_example // kotlin.String |
val environmentId : kotlin.String = environmentId_example // kotlin.String |
val traceId : kotlin.String = traceId_example // kotlin.String |
val spanId : kotlin.String = spanId_example // kotlin.String |
val kind : kotlin.String = kind_example // kotlin.String |
val status : kotlin.String = status_example // kotlin.String |
val model : kotlin.String = model_example // kotlin.String |
val tool : kotlin.String = tool_example // kotlin.String |
val limit : kotlin.Int = 56 // kotlin.Int |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : SearchResponse = apiInstance.searchSpans(tenantId, q, projectId, environmentId, traceId, spanId, kind, status, model, tool, limit, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling SearchApi#searchSpans")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling SearchApi#searchSpans")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **q** | **kotlin.String**|  | [optional] |
| **projectId** | **kotlin.String**|  | [optional] |
| **environmentId** | **kotlin.String**|  | [optional] |
| **traceId** | **kotlin.String**|  | [optional] |
| **spanId** | **kotlin.String**|  | [optional] |
| **kind** | **kotlin.String**|  | [optional] |
| **status** | **kotlin.String**|  | [optional] |
| **model** | **kotlin.String**|  | [optional] |
| **tool** | **kotlin.String**|  | [optional] |
| **limit** | **kotlin.Int**|  | [optional] |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**SearchResponse**](SearchResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json
