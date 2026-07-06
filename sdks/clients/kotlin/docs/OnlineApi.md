# OnlineApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------------- | ------------- | ------------- |
| [**decideOnlineSampling**](OnlineApi.md#decideOnlineSampling) | **POST** /v1/online/{tenant_id}/{project_id}/traces/{trace_id}/sampling |  |


<a id="decideOnlineSampling"></a>
# **decideOnlineSampling**
> SamplingDecision decideOnlineSampling(tenantId, projectId, traceId, onlineSamplingPolicy, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = OnlineApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val traceId : kotlin.String = traceId_example // kotlin.String | trace_id
val onlineSamplingPolicy : OnlineSamplingPolicy =  // OnlineSamplingPolicy |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : SamplingDecision = apiInstance.decideOnlineSampling(tenantId, projectId, traceId, onlineSamplingPolicy, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling OnlineApi#decideOnlineSampling")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling OnlineApi#decideOnlineSampling")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **traceId** | **kotlin.String**| trace_id | |
| **onlineSamplingPolicy** | [**OnlineSamplingPolicy**](OnlineSamplingPolicy.md)|  | |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**SamplingDecision**](SamplingDecision.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json
