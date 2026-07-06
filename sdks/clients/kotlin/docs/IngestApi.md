# IngestApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------------- | ------------- | ------------- |
| [**drainTraceIngested**](IngestApi.md#drainTraceIngested) | **POST** /v1/ingest/{tenant_id}/{project_id}/trace-ingested/drain |  |
| [**drainTraceWrites**](IngestApi.md#drainTraceWrites) | **POST** /v1/ingest/{tenant_id}/{project_id}/trace-writes/drain |  |
| [**getIngestQueueStatus**](IngestApi.md#getIngestQueueStatus) | **GET** /v1/ingest/{tenant_id}/{project_id}/queue |  |
| [**importSource**](IngestApi.md#importSource) | **POST** /v1/import/{tenant_id}/{project_id}/{environment_id} |  |
| [**ingestNative**](IngestApi.md#ingestNative) | **POST** /v1/traces/native |  |
| [**ingestOtlp**](IngestApi.md#ingestOtlp) | **POST** /v1/otlp/{tenant_id}/{project_id}/{environment_id}/v1/traces |  |
| [**reconcileTrace**](IngestApi.md#reconcileTrace) | **POST** /v1/ingest/{tenant_id}/{project_id}/traces/{trace_id}/reconcile |  |
| [**replayDeadLetter**](IngestApi.md#replayDeadLetter) | **POST** /v1/ingest/{tenant_id}/{project_id}/dead-letters/{message_id}/replay |  |


<a id="drainTraceIngested"></a>
# **drainTraceIngested**
> TraceIngestedDrainReport drainTraceIngested(tenantId, projectId, limit, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = IngestApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val limit : kotlin.Int = 56 // kotlin.Int |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : TraceIngestedDrainReport = apiInstance.drainTraceIngested(tenantId, projectId, limit, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling IngestApi#drainTraceIngested")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling IngestApi#drainTraceIngested")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **limit** | **kotlin.Int**|  | [optional] |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**TraceIngestedDrainReport**](TraceIngestedDrainReport.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

<a id="drainTraceWrites"></a>
# **drainTraceWrites**
> TraceWriteDrainReport drainTraceWrites(tenantId, projectId, limit, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = IngestApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val limit : kotlin.Int = 56 // kotlin.Int |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : TraceWriteDrainReport = apiInstance.drainTraceWrites(tenantId, projectId, limit, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling IngestApi#drainTraceWrites")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling IngestApi#drainTraceWrites")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **limit** | **kotlin.Int**|  | [optional] |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**TraceWriteDrainReport**](TraceWriteDrainReport.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

<a id="getIngestQueueStatus"></a>
# **getIngestQueueStatus**
> IngestQueueStatus getIngestQueueStatus(tenantId, projectId, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = IngestApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : IngestQueueStatus = apiInstance.getIngestQueueStatus(tenantId, projectId, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling IngestApi#getIngestQueueStatus")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling IngestApi#getIngestQueueStatus")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**IngestQueueStatus**](IngestQueueStatus.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

<a id="importSource"></a>
# **importSource**
> IngestOutcome importSource(tenantId, projectId, environmentId, importSourceHttpRequest, durability, authorization, xBeaterApiKey)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = IngestApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val environmentId : kotlin.String = environmentId_example // kotlin.String | environment_id
val importSourceHttpRequest : ImportSourceHttpRequest =  // ImportSourceHttpRequest |
val durability : kotlin.String = durability_example // kotlin.String |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
try {
    val result : IngestOutcome = apiInstance.importSource(tenantId, projectId, environmentId, importSourceHttpRequest, durability, authorization, xBeaterApiKey)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling IngestApi#importSource")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling IngestApi#importSource")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **environmentId** | **kotlin.String**| environment_id | |
| **importSourceHttpRequest** | [**ImportSourceHttpRequest**](ImportSourceHttpRequest.md)|  | |
| **durability** | **kotlin.String**|  | [optional] |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |

### Return type

[**IngestOutcome**](IngestOutcome.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

<a id="ingestNative"></a>
# **ingestNative**
> IngestOutcome ingestNative(nativeIngestRequest, durability, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = IngestApi()
val nativeIngestRequest : NativeIngestRequest =  // NativeIngestRequest |
val durability : kotlin.String = durability_example // kotlin.String |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : IngestOutcome = apiInstance.ingestNative(nativeIngestRequest, durability, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling IngestApi#ingestNative")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling IngestApi#ingestNative")
    e.printStackTrace()
}
```

### Parameters
| **nativeIngestRequest** | [**NativeIngestRequest**](NativeIngestRequest.md)|  | |
| **durability** | **kotlin.String**|  | [optional] |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**IngestOutcome**](IngestOutcome.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

<a id="ingestOtlp"></a>
# **ingestOtlp**
> OtlpIngestOutcome ingestOtlp(tenantId, projectId, environmentId, durability, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = IngestApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val environmentId : kotlin.String = environmentId_example // kotlin.String | environment_id
val durability : kotlin.String = durability_example // kotlin.String |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : OtlpIngestOutcome = apiInstance.ingestOtlp(tenantId, projectId, environmentId, durability, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling IngestApi#ingestOtlp")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling IngestApi#ingestOtlp")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **environmentId** | **kotlin.String**| environment_id | |
| **durability** | **kotlin.String**|  | [optional] |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**OtlpIngestOutcome**](OtlpIngestOutcome.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

<a id="reconcileTrace"></a>
# **reconcileTrace**
> TraceIngestedReconcileReport reconcileTrace(tenantId, projectId, traceId, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = IngestApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val traceId : kotlin.String = traceId_example // kotlin.String | trace_id
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : TraceIngestedReconcileReport = apiInstance.reconcileTrace(tenantId, projectId, traceId, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling IngestApi#reconcileTrace")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling IngestApi#reconcileTrace")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **traceId** | **kotlin.String**| trace_id | |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**TraceIngestedReconcileReport**](TraceIngestedReconcileReport.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

<a id="replayDeadLetter"></a>
# **replayDeadLetter**
> DeadLetterReplayReport replayDeadLetter(tenantId, projectId, messageId, resetAttempts, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = IngestApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val messageId : kotlin.String = messageId_example // kotlin.String | message_id
val resetAttempts : kotlin.Boolean = true // kotlin.Boolean |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : DeadLetterReplayReport = apiInstance.replayDeadLetter(tenantId, projectId, messageId, resetAttempts, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling IngestApi#replayDeadLetter")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling IngestApi#replayDeadLetter")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **messageId** | **kotlin.String**| message_id | |
| **resetAttempts** | **kotlin.Boolean**|  | [optional] |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**DeadLetterReplayReport**](DeadLetterReplayReport.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json
