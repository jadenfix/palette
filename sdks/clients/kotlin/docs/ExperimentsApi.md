# ExperimentsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------------- | ------------- | ------------- |
| [**runDeterministicExperiment**](ExperimentsApi.md#runDeterministicExperiment) | **POST** /v1/experiments/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id}/deterministic |  |
| [**runJudgeExperiment**](ExperimentsApi.md#runJudgeExperiment) | **POST** /v1/experiments/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id}/judge |  |


<a id="runDeterministicExperiment"></a>
# **runDeterministicExperiment**
> ExperimentRunReport runDeterministicExperiment(tenantId, projectId, datasetId, versionId, runExperimentRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = ExperimentsApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val datasetId : kotlin.String = datasetId_example // kotlin.String | dataset_id
val versionId : kotlin.String = versionId_example // kotlin.String | version_id
val runExperimentRequest : RunExperimentRequest =  // RunExperimentRequest |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : ExperimentRunReport = apiInstance.runDeterministicExperiment(tenantId, projectId, datasetId, versionId, runExperimentRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling ExperimentsApi#runDeterministicExperiment")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling ExperimentsApi#runDeterministicExperiment")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **datasetId** | **kotlin.String**| dataset_id | |
| **versionId** | **kotlin.String**| version_id | |
| **runExperimentRequest** | [**RunExperimentRequest**](RunExperimentRequest.md)|  | |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**ExperimentRunReport**](ExperimentRunReport.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

<a id="runJudgeExperiment"></a>
# **runJudgeExperiment**
> ExperimentRunReport runJudgeExperiment(tenantId, projectId, datasetId, versionId, runJudgeExperimentRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = ExperimentsApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val datasetId : kotlin.String = datasetId_example // kotlin.String | dataset_id
val versionId : kotlin.String = versionId_example // kotlin.String | version_id
val runJudgeExperimentRequest : RunJudgeExperimentRequest =  // RunJudgeExperimentRequest |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : ExperimentRunReport = apiInstance.runJudgeExperiment(tenantId, projectId, datasetId, versionId, runJudgeExperimentRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling ExperimentsApi#runJudgeExperiment")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling ExperimentsApi#runJudgeExperiment")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **datasetId** | **kotlin.String**| dataset_id | |
| **versionId** | **kotlin.String**| version_id | |
| **runJudgeExperimentRequest** | [**RunJudgeExperimentRequest**](RunJudgeExperimentRequest.md)|  | |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**ExperimentRunReport**](ExperimentRunReport.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json
