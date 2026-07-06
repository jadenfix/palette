# ReviewsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------------- | ------------- | ------------- |
| [**createReviewQueue**](ReviewsApi.md#createReviewQueue) | **POST** /v1/review-queues/{tenant_id}/{project_id} |  |
| [**enqueueReviewTaskFromTrace**](ReviewsApi.md#enqueueReviewTaskFromTrace) | **POST** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks/from-trace |  |
| [**listReviewTasks**](ReviewsApi.md#listReviewTasks) | **GET** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks |  |
| [**promoteReviewAnnotation**](ReviewsApi.md#promoteReviewAnnotation) | **POST** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks/{task_id}/annotations/{annotation_id}/promote |  |
| [**submitReviewAnnotation**](ReviewsApi.md#submitReviewAnnotation) | **POST** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks/{task_id}/annotations |  |


<a id="createReviewQueue"></a>
# **createReviewQueue**
> ReviewQueue createReviewQueue(tenantId, projectId, createReviewQueueHttpRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = ReviewsApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val createReviewQueueHttpRequest : CreateReviewQueueHttpRequest =  // CreateReviewQueueHttpRequest |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : ReviewQueue = apiInstance.createReviewQueue(tenantId, projectId, createReviewQueueHttpRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling ReviewsApi#createReviewQueue")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling ReviewsApi#createReviewQueue")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **createReviewQueueHttpRequest** | [**CreateReviewQueueHttpRequest**](CreateReviewQueueHttpRequest.md)|  | |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**ReviewQueue**](ReviewQueue.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

<a id="enqueueReviewTaskFromTrace"></a>
# **enqueueReviewTaskFromTrace**
> ReviewTask enqueueReviewTaskFromTrace(tenantId, projectId, queueId, enqueueReviewTaskFromTraceHttpRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = ReviewsApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val queueId : kotlin.String = queueId_example // kotlin.String | queue_id
val enqueueReviewTaskFromTraceHttpRequest : EnqueueReviewTaskFromTraceHttpRequest =  // EnqueueReviewTaskFromTraceHttpRequest |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : ReviewTask = apiInstance.enqueueReviewTaskFromTrace(tenantId, projectId, queueId, enqueueReviewTaskFromTraceHttpRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling ReviewsApi#enqueueReviewTaskFromTrace")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling ReviewsApi#enqueueReviewTaskFromTrace")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **queueId** | **kotlin.String**| queue_id | |
| **enqueueReviewTaskFromTraceHttpRequest** | [**EnqueueReviewTaskFromTraceHttpRequest**](EnqueueReviewTaskFromTraceHttpRequest.md)|  | |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**ReviewTask**](ReviewTask.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

<a id="listReviewTasks"></a>
# **listReviewTasks**
> kotlin.collections.List&lt;ReviewTask&gt; listReviewTasks(tenantId, projectId, queueId, state, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = ReviewsApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val queueId : kotlin.String = queueId_example // kotlin.String | queue_id
val state : ReviewTaskState =  // ReviewTaskState |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : kotlin.collections.List<ReviewTask> = apiInstance.listReviewTasks(tenantId, projectId, queueId, state, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling ReviewsApi#listReviewTasks")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling ReviewsApi#listReviewTasks")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **queueId** | **kotlin.String**| queue_id | |
| **state** | [**ReviewTaskState**](.md)|  | [optional] [enum: open, submitted, cancelled] |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**kotlin.collections.List&lt;ReviewTask&gt;**](ReviewTask.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

<a id="promoteReviewAnnotation"></a>
# **promoteReviewAnnotation**
> DatasetCase promoteReviewAnnotation(tenantId, projectId, queueId, taskId, annotationId, promoteReviewAnnotationHttpRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = ReviewsApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val queueId : kotlin.String = queueId_example // kotlin.String | queue_id
val taskId : kotlin.String = taskId_example // kotlin.String | task_id
val annotationId : kotlin.String = annotationId_example // kotlin.String | annotation_id
val promoteReviewAnnotationHttpRequest : PromoteReviewAnnotationHttpRequest =  // PromoteReviewAnnotationHttpRequest |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : DatasetCase = apiInstance.promoteReviewAnnotation(tenantId, projectId, queueId, taskId, annotationId, promoteReviewAnnotationHttpRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling ReviewsApi#promoteReviewAnnotation")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling ReviewsApi#promoteReviewAnnotation")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **queueId** | **kotlin.String**| queue_id | |
| **taskId** | **kotlin.String**| task_id | |
| **annotationId** | **kotlin.String**| annotation_id | |
| **promoteReviewAnnotationHttpRequest** | [**PromoteReviewAnnotationHttpRequest**](PromoteReviewAnnotationHttpRequest.md)|  | |
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

<a id="submitReviewAnnotation"></a>
# **submitReviewAnnotation**
> ReviewAnnotation submitReviewAnnotation(tenantId, projectId, queueId, taskId, submitReviewAnnotationHttpRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = ReviewsApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val queueId : kotlin.String = queueId_example // kotlin.String | queue_id
val taskId : kotlin.String = taskId_example // kotlin.String | task_id
val submitReviewAnnotationHttpRequest : SubmitReviewAnnotationHttpRequest =  // SubmitReviewAnnotationHttpRequest |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : ReviewAnnotation = apiInstance.submitReviewAnnotation(tenantId, projectId, queueId, taskId, submitReviewAnnotationHttpRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling ReviewsApi#submitReviewAnnotation")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling ReviewsApi#submitReviewAnnotation")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **queueId** | **kotlin.String**| queue_id | |
| **taskId** | **kotlin.String**| task_id | |
| **submitReviewAnnotationHttpRequest** | [**SubmitReviewAnnotationHttpRequest**](SubmitReviewAnnotationHttpRequest.md)|  | |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**ReviewAnnotation**](ReviewAnnotation.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json
