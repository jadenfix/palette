# AlertsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------------- | ------------- | ------------- |
| [**evaluateAlert**](AlertsApi.md#evaluateAlert) | **POST** /v1/alerts/{tenant_id}/{project_id}/traces/{trace_id}/webhook |  |


<a id="evaluateAlert"></a>
# **evaluateAlert**
> AlertDecision evaluateAlert(tenantId, projectId, traceId, evaluateAlertRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = AlertsApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val traceId : kotlin.String = traceId_example // kotlin.String | trace_id
val evaluateAlertRequest : EvaluateAlertRequest =  // EvaluateAlertRequest |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : AlertDecision = apiInstance.evaluateAlert(tenantId, projectId, traceId, evaluateAlertRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling AlertsApi#evaluateAlert")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling AlertsApi#evaluateAlert")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **traceId** | **kotlin.String**| trace_id | |
| **evaluateAlertRequest** | [**EvaluateAlertRequest**](EvaluateAlertRequest.md)|  | |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**AlertDecision**](AlertDecision.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json
