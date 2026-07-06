# UsageApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------------- | ------------- | ------------- |
| [**getUsageSummary**](UsageApi.md#getUsageSummary) | **GET** /v1/usage/{tenant_id}/{project_id} |  |


<a id="getUsageSummary"></a>
# **getUsageSummary**
> UsageSummary getUsageSummary(tenantId, projectId, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = UsageApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : UsageSummary = apiInstance.getUsageSummary(tenantId, projectId, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling UsageApi#getUsageSummary")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling UsageApi#getUsageSummary")
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

[**UsageSummary**](UsageSummary.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json
