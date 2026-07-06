# SpansApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------------- | ------------- | ------------- |
| [**getSpan**](SpansApi.md#getSpan) | **GET** /v1/spans/{tenant_id}/{trace_id}/{span_id} |  |
| [**getSpanIo**](SpansApi.md#getSpanIo) | **GET** /v1/spans/{tenant_id}/{trace_id}/{span_id}/io |  |


<a id="getSpan"></a>
# **getSpan**
> CanonicalSpan getSpan(tenantId, traceId, spanId, unmask, reason, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = SpansApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val traceId : kotlin.String = traceId_example // kotlin.String | trace_id
val spanId : kotlin.String = spanId_example // kotlin.String | span_id
val unmask : kotlin.Boolean = true // kotlin.Boolean |
val reason : kotlin.String = reason_example // kotlin.String |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : CanonicalSpan = apiInstance.getSpan(tenantId, traceId, spanId, unmask, reason, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling SpansApi#getSpan")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling SpansApi#getSpan")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **traceId** | **kotlin.String**| trace_id | |
| **spanId** | **kotlin.String**| span_id | |
| **unmask** | **kotlin.Boolean**|  | [optional] |
| **reason** | **kotlin.String**|  | [optional] |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**CanonicalSpan**](CanonicalSpan.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

<a id="getSpanIo"></a>
# **getSpanIo**
> SpanIoResponse getSpanIo(tenantId, traceId, spanId, unmask, reason, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = SpansApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val traceId : kotlin.String = traceId_example // kotlin.String | trace_id
val spanId : kotlin.String = spanId_example // kotlin.String | span_id
val unmask : kotlin.Boolean = true // kotlin.Boolean |
val reason : kotlin.String = reason_example // kotlin.String |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : SpanIoResponse = apiInstance.getSpanIo(tenantId, traceId, spanId, unmask, reason, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling SpansApi#getSpanIo")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling SpansApi#getSpanIo")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **traceId** | **kotlin.String**| trace_id | |
| **spanId** | **kotlin.String**| span_id | |
| **unmask** | **kotlin.Boolean**|  | [optional] |
| **reason** | **kotlin.String**|  | [optional] |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**SpanIoResponse**](SpanIoResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json
