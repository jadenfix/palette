# \ReviewsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**reviews_period_create_review_queue**](ReviewsApi.md#reviews_period_create_review_queue) | **POST** /v1/review-queues/{tenant_id}/{project_id} |
[**reviews_period_enqueue_review_task_from_trace**](ReviewsApi.md#reviews_period_enqueue_review_task_from_trace) | **POST** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks/from-trace |
[**reviews_period_list_review_tasks**](ReviewsApi.md#reviews_period_list_review_tasks) | **GET** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks |
[**reviews_period_promote_review_annotation**](ReviewsApi.md#reviews_period_promote_review_annotation) | **POST** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks/{task_id}/annotations/{annotation_id}/promote |
[**reviews_period_submit_review_annotation**](ReviewsApi.md#reviews_period_submit_review_annotation) | **POST** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks/{task_id}/annotations |



## reviews_period_create_review_queue

> models::ReviewQueue reviews_period_create_review_queue(tenant_id, project_id, create_review_queue_http_request, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**project_id** | **String** | project_id | [required] |
**create_review_queue_http_request** | [**CreateReviewQueueHttpRequest**](CreateReviewQueueHttpRequest.md) |  | [required] |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::ReviewQueue**](ReviewQueue.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## reviews_period_enqueue_review_task_from_trace

> models::ReviewTask reviews_period_enqueue_review_task_from_trace(tenant_id, project_id, queue_id, enqueue_review_task_from_trace_http_request, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**project_id** | **String** | project_id | [required] |
**queue_id** | **String** | queue_id | [required] |
**enqueue_review_task_from_trace_http_request** | [**EnqueueReviewTaskFromTraceHttpRequest**](EnqueueReviewTaskFromTraceHttpRequest.md) |  | [required] |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::ReviewTask**](ReviewTask.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## reviews_period_list_review_tasks

> Vec<models::ReviewTask> reviews_period_list_review_tasks(tenant_id, project_id, queue_id, state, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**project_id** | **String** | project_id | [required] |
**queue_id** | **String** | queue_id | [required] |
**state** | Option<[**ReviewTaskState**](.md)> |  |  |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**Vec<models::ReviewTask>**](ReviewTask.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## reviews_period_promote_review_annotation

> models::DatasetCase reviews_period_promote_review_annotation(tenant_id, project_id, queue_id, task_id, annotation_id, promote_review_annotation_http_request, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**project_id** | **String** | project_id | [required] |
**queue_id** | **String** | queue_id | [required] |
**task_id** | **String** | task_id | [required] |
**annotation_id** | **String** | annotation_id | [required] |
**promote_review_annotation_http_request** | [**PromoteReviewAnnotationHttpRequest**](PromoteReviewAnnotationHttpRequest.md) |  | [required] |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::DatasetCase**](DatasetCase.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## reviews_period_submit_review_annotation

> models::ReviewAnnotation reviews_period_submit_review_annotation(tenant_id, project_id, queue_id, task_id, submit_review_annotation_http_request, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**project_id** | **String** | project_id | [required] |
**queue_id** | **String** | queue_id | [required] |
**task_id** | **String** | task_id | [required] |
**submit_review_annotation_http_request** | [**SubmitReviewAnnotationHttpRequest**](SubmitReviewAnnotationHttpRequest.md) |  | [required] |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::ReviewAnnotation**](ReviewAnnotation.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
