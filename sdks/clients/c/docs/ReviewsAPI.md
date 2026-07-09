# ReviewsAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**ReviewsAPI_reviewsCreateReviewQueue**](ReviewsAPI.md#ReviewsAPI_reviewsCreateReviewQueue) | **POST** /v1/review-queues/{tenant_id}/{project_id} |
[**ReviewsAPI_reviewsEnqueueReviewTaskFromTrace**](ReviewsAPI.md#ReviewsAPI_reviewsEnqueueReviewTaskFromTrace) | **POST** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks/from-trace |
[**ReviewsAPI_reviewsListReviewTasks**](ReviewsAPI.md#ReviewsAPI_reviewsListReviewTasks) | **GET** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks |
[**ReviewsAPI_reviewsPromoteReviewAnnotation**](ReviewsAPI.md#ReviewsAPI_reviewsPromoteReviewAnnotation) | **POST** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks/{task_id}/annotations/{annotation_id}/promote |
[**ReviewsAPI_reviewsSubmitReviewAnnotation**](ReviewsAPI.md#ReviewsAPI_reviewsSubmitReviewAnnotation) | **POST** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks/{task_id}/annotations |


# **ReviewsAPI_reviewsCreateReviewQueue**
```c
review_queue_t* ReviewsAPI_reviewsCreateReviewQueue(apiClient_t *apiClient, char *tenant_id, char *project_id, create_review_queue_http_request_t *create_review_queue_http_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**create_review_queue_http_request** | **[create_review_queue_http_request_t](create_review_queue_http_request.md) \*** |  |
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[review_queue_t](review_queue.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **ReviewsAPI_reviewsEnqueueReviewTaskFromTrace**
```c
review_task_t* ReviewsAPI_reviewsEnqueueReviewTaskFromTrace(apiClient_t *apiClient, char *tenant_id, char *project_id, char *queue_id, enqueue_review_task_from_trace_http_request_t *enqueue_review_task_from_trace_http_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**queue_id** | **char \*** | queue_id |
**enqueue_review_task_from_trace_http_request** | **[enqueue_review_task_from_trace_http_request_t](enqueue_review_task_from_trace_http_request.md) \*** |  |
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[review_task_t](review_task.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **ReviewsAPI_reviewsListReviewTasks**
```c
list_t* ReviewsAPI_reviewsListReviewTasks(apiClient_t *apiClient, char *tenant_id, char *project_id, char *queue_id, review_task_state_e state, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**queue_id** | **char \*** | queue_id |
**state** | **review_task_state_e** |  | [optional]
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[list_t](review_task.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **ReviewsAPI_reviewsPromoteReviewAnnotation**
```c
dataset_case_t* ReviewsAPI_reviewsPromoteReviewAnnotation(apiClient_t *apiClient, char *tenant_id, char *project_id, char *queue_id, char *task_id, char *annotation_id, promote_review_annotation_http_request_t *promote_review_annotation_http_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**queue_id** | **char \*** | queue_id |
**task_id** | **char \*** | task_id |
**annotation_id** | **char \*** | annotation_id |
**promote_review_annotation_http_request** | **[promote_review_annotation_http_request_t](promote_review_annotation_http_request.md) \*** |  |
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[dataset_case_t](dataset_case.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **ReviewsAPI_reviewsSubmitReviewAnnotation**
```c
review_annotation_t* ReviewsAPI_reviewsSubmitReviewAnnotation(apiClient_t *apiClient, char *tenant_id, char *project_id, char *queue_id, char *task_id, submit_review_annotation_http_request_t *submit_review_annotation_http_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**queue_id** | **char \*** | queue_id |
**task_id** | **char \*** | task_id |
**submit_review_annotation_http_request** | **[submit_review_annotation_http_request_t](submit_review_annotation_http_request.md) \*** |  |
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[review_annotation_t](review_annotation.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
