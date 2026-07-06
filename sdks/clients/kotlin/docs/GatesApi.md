# GatesApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------------- | ------------- | ------------- |
| [**createGate**](GatesApi.md#createGate) | **POST** /v1/gates/{tenant_id}/{project_id} |  |
| [**runGate**](GatesApi.md#runGate) | **POST** /v1/gates/{tenant_id}/{project_id}/{gate_id}/run |  |


<a id="createGate"></a>
# **createGate**
> GateDefinition createGate(tenantId, projectId, createGateRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = GatesApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val createGateRequest : CreateGateRequest =  // CreateGateRequest |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : GateDefinition = apiInstance.createGate(tenantId, projectId, createGateRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling GatesApi#createGate")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling GatesApi#createGate")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **createGateRequest** | [**CreateGateRequest**](CreateGateRequest.md)|  | |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**GateDefinition**](GateDefinition.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

<a id="runGate"></a>
# **runGate**
> GateRunReport runGate(tenantId, projectId, gateId, runGateRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = GatesApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val gateId : kotlin.String = gateId_example // kotlin.String | gate_id
val runGateRequest : RunGateRequest =  // RunGateRequest |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : GateRunReport = apiInstance.runGate(tenantId, projectId, gateId, runGateRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling GatesApi#runGate")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling GatesApi#runGate")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **gateId** | **kotlin.String**| gate_id | |
| **runGateRequest** | [**RunGateRequest**](RunGateRequest.md)|  | |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**GateRunReport**](GateRunReport.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json
