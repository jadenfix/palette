# DatasetsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------------- | ------------- | ------------- |
| [**createDataset**](DatasetsApi.md#createDataset) | **POST** /v1/datasets/{tenant_id}/{project_id} |  |
| [**createDatasetVersion**](DatasetsApi.md#createDatasetVersion) | **POST** /v1/datasets/{tenant_id}/{project_id}/{dataset_id}/versions |  |
| [**promoteDatasetCaseFromTrace**](DatasetsApi.md#promoteDatasetCaseFromTrace) | **POST** /v1/datasets/{tenant_id}/{project_id}/{dataset_id}/cases/from-trace |  |


<a id="createDataset"></a>
# **createDataset**
> Dataset createDataset(tenantId, projectId, createDatasetRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = DatasetsApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val createDatasetRequest : CreateDatasetRequest =  // CreateDatasetRequest |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : Dataset = apiInstance.createDataset(tenantId, projectId, createDatasetRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling DatasetsApi#createDataset")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling DatasetsApi#createDataset")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **createDatasetRequest** | [**CreateDatasetRequest**](CreateDatasetRequest.md)|  | |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**Dataset**](Dataset.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

<a id="createDatasetVersion"></a>
# **createDatasetVersion**
> DatasetVersionSnapshot createDatasetVersion(tenantId, projectId, datasetId, createDatasetVersionRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = DatasetsApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val datasetId : kotlin.String = datasetId_example // kotlin.String | dataset_id
val createDatasetVersionRequest : CreateDatasetVersionRequest =  // CreateDatasetVersionRequest |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : DatasetVersionSnapshot = apiInstance.createDatasetVersion(tenantId, projectId, datasetId, createDatasetVersionRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling DatasetsApi#createDatasetVersion")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling DatasetsApi#createDatasetVersion")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **datasetId** | **kotlin.String**| dataset_id | |
| **createDatasetVersionRequest** | [**CreateDatasetVersionRequest**](CreateDatasetVersionRequest.md)|  | |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**DatasetVersionSnapshot**](DatasetVersionSnapshot.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

<a id="promoteDatasetCaseFromTrace"></a>
# **promoteDatasetCaseFromTrace**
> DatasetCase promoteDatasetCaseFromTrace(tenantId, projectId, datasetId, promoteTraceCaseRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = DatasetsApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val datasetId : kotlin.String = datasetId_example // kotlin.String | dataset_id
val promoteTraceCaseRequest : PromoteTraceCaseRequest =  // PromoteTraceCaseRequest |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : DatasetCase = apiInstance.promoteDatasetCaseFromTrace(tenantId, projectId, datasetId, promoteTraceCaseRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling DatasetsApi#promoteDatasetCaseFromTrace")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling DatasetsApi#promoteDatasetCaseFromTrace")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **datasetId** | **kotlin.String**| dataset_id | |
| **promoteTraceCaseRequest** | [**PromoteTraceCaseRequest**](PromoteTraceCaseRequest.md)|  | |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**DatasetCase**](DatasetCase.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json
