# ArchiveApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------------- | ------------- | ------------- |
| [**archiveTrace**](ArchiveApi.md#archiveTrace) | **POST** /v1/archive/{tenant_id}/{project_id}/{trace_id} |  |
| [**queryArchiveSpans**](ArchiveApi.md#queryArchiveSpans) | **GET** /v1/archive/{tenant_id}/{project_id}/spans |  |


<a id="archiveTrace"></a>
# **archiveTrace**
> ArchiveManifest archiveTrace(tenantId, projectId, traceId, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = ArchiveApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val traceId : kotlin.String = traceId_example // kotlin.String | trace_id
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : ArchiveManifest = apiInstance.archiveTrace(tenantId, projectId, traceId, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling ArchiveApi#archiveTrace")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling ArchiveApi#archiveTrace")
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

[**ArchiveManifest**](ArchiveManifest.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

<a id="queryArchiveSpans"></a>
# **queryArchiveSpans**
> ArchiveQueryResponse queryArchiveSpans(tenantId, projectId, environmentId, traceId, spanId, kind, status, limit, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = ArchiveApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val environmentId : kotlin.String = environmentId_example // kotlin.String |
val traceId : kotlin.String = traceId_example // kotlin.String |
val spanId : kotlin.String = spanId_example // kotlin.String |
val kind : kotlin.String = kind_example // kotlin.String |
val status : kotlin.String = status_example // kotlin.String |
val limit : kotlin.Int = 56 // kotlin.Int |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : ArchiveQueryResponse = apiInstance.queryArchiveSpans(tenantId, projectId, environmentId, traceId, spanId, kind, status, limit, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling ArchiveApi#queryArchiveSpans")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling ArchiveApi#queryArchiveSpans")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **environmentId** | **kotlin.String**|  | [optional] |
| **traceId** | **kotlin.String**|  | [optional] |
| **spanId** | **kotlin.String**|  | [optional] |
| **kind** | **kotlin.String**|  | [optional] |
| **status** | **kotlin.String**|  | [optional] |
| **limit** | **kotlin.Int**|  | [optional] |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**ArchiveQueryResponse**](ArchiveQueryResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json
