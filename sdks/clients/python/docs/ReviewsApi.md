# beater_client.ReviewsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**reviews_create_review_queue**](ReviewsApi.md#reviews_create_review_queue) | **POST** /v1/review-queues/{tenant_id}/{project_id} |
[**reviews_enqueue_review_task_from_trace**](ReviewsApi.md#reviews_enqueue_review_task_from_trace) | **POST** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks/from-trace |
[**reviews_list_review_tasks**](ReviewsApi.md#reviews_list_review_tasks) | **GET** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks |
[**reviews_promote_review_annotation**](ReviewsApi.md#reviews_promote_review_annotation) | **POST** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks/{task_id}/annotations/{annotation_id}/promote |
[**reviews_submit_review_annotation**](ReviewsApi.md#reviews_submit_review_annotation) | **POST** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks/{task_id}/annotations |


# **reviews_create_review_queue**
> ReviewQueue reviews_create_review_queue(tenant_id, project_id, create_review_queue_http_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.create_review_queue_http_request import CreateReviewQueueHttpRequest
from beater_client.models.review_queue import ReviewQueue
from beater_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = beater_client.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with beater_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = beater_client.ReviewsApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    create_review_queue_http_request = beater_client.CreateReviewQueueHttpRequest() # CreateReviewQueueHttpRequest |
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.reviews_create_review_queue(tenant_id, project_id, create_review_queue_http_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of ReviewsApi->reviews_create_review_queue:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ReviewsApi->reviews_create_review_queue: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id |
 **project_id** | **str**| project_id |
 **create_review_queue_http_request** | [**CreateReviewQueueHttpRequest**](CreateReviewQueueHttpRequest.md)|  |
 **authorization** | **str**| Bearer API token for strict auth | [optional]
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional]
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional]
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional]

### Return type

[**ReviewQueue**](ReviewQueue.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Create a human review queue |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **reviews_enqueue_review_task_from_trace**
> ReviewTask reviews_enqueue_review_task_from_trace(tenant_id, project_id, queue_id, enqueue_review_task_from_trace_http_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.enqueue_review_task_from_trace_http_request import EnqueueReviewTaskFromTraceHttpRequest
from beater_client.models.review_task import ReviewTask
from beater_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = beater_client.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with beater_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = beater_client.ReviewsApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    queue_id = 'queue_id_example' # str | queue_id
    enqueue_review_task_from_trace_http_request = beater_client.EnqueueReviewTaskFromTraceHttpRequest() # EnqueueReviewTaskFromTraceHttpRequest |
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.reviews_enqueue_review_task_from_trace(tenant_id, project_id, queue_id, enqueue_review_task_from_trace_http_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of ReviewsApi->reviews_enqueue_review_task_from_trace:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ReviewsApi->reviews_enqueue_review_task_from_trace: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id |
 **project_id** | **str**| project_id |
 **queue_id** | **str**| queue_id |
 **enqueue_review_task_from_trace_http_request** | [**EnqueueReviewTaskFromTraceHttpRequest**](EnqueueReviewTaskFromTraceHttpRequest.md)|  |
 **authorization** | **str**| Bearer API token for strict auth | [optional]
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional]
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional]
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional]

### Return type

[**ReviewTask**](ReviewTask.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Enqueue a review task from a trace |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |
**404** | Resource not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **reviews_list_review_tasks**
> List[ReviewTask] reviews_list_review_tasks(tenant_id, project_id, queue_id, state=state, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.review_task import ReviewTask
from beater_client.models.review_task_state import ReviewTaskState
from beater_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = beater_client.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with beater_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = beater_client.ReviewsApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    queue_id = 'queue_id_example' # str | queue_id
    state = beater_client.ReviewTaskState() # ReviewTaskState |  (optional)
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.reviews_list_review_tasks(tenant_id, project_id, queue_id, state=state, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of ReviewsApi->reviews_list_review_tasks:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ReviewsApi->reviews_list_review_tasks: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id |
 **project_id** | **str**| project_id |
 **queue_id** | **str**| queue_id |
 **state** | [**ReviewTaskState**](.md)|  | [optional]
 **authorization** | **str**| Bearer API token for strict auth | [optional]
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional]
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional]
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional]

### Return type

[**List[ReviewTask]**](ReviewTask.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | List review tasks |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |
**404** | Resource not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **reviews_promote_review_annotation**
> DatasetCase reviews_promote_review_annotation(tenant_id, project_id, queue_id, task_id, annotation_id, promote_review_annotation_http_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.dataset_case import DatasetCase
from beater_client.models.promote_review_annotation_http_request import PromoteReviewAnnotationHttpRequest
from beater_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = beater_client.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with beater_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = beater_client.ReviewsApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    queue_id = 'queue_id_example' # str | queue_id
    task_id = 'task_id_example' # str | task_id
    annotation_id = 'annotation_id_example' # str | annotation_id
    promote_review_annotation_http_request = beater_client.PromoteReviewAnnotationHttpRequest() # PromoteReviewAnnotationHttpRequest |
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.reviews_promote_review_annotation(tenant_id, project_id, queue_id, task_id, annotation_id, promote_review_annotation_http_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of ReviewsApi->reviews_promote_review_annotation:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ReviewsApi->reviews_promote_review_annotation: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id |
 **project_id** | **str**| project_id |
 **queue_id** | **str**| queue_id |
 **task_id** | **str**| task_id |
 **annotation_id** | **str**| annotation_id |
 **promote_review_annotation_http_request** | [**PromoteReviewAnnotationHttpRequest**](PromoteReviewAnnotationHttpRequest.md)|  |
 **authorization** | **str**| Bearer API token for strict auth | [optional]
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional]
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional]
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional]

### Return type

[**DatasetCase**](DatasetCase.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Promote a review annotation to a dataset case |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |
**404** | Resource not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **reviews_submit_review_annotation**
> ReviewAnnotation reviews_submit_review_annotation(tenant_id, project_id, queue_id, task_id, submit_review_annotation_http_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.review_annotation import ReviewAnnotation
from beater_client.models.submit_review_annotation_http_request import SubmitReviewAnnotationHttpRequest
from beater_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = beater_client.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with beater_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = beater_client.ReviewsApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    queue_id = 'queue_id_example' # str | queue_id
    task_id = 'task_id_example' # str | task_id
    submit_review_annotation_http_request = beater_client.SubmitReviewAnnotationHttpRequest() # SubmitReviewAnnotationHttpRequest |
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.reviews_submit_review_annotation(tenant_id, project_id, queue_id, task_id, submit_review_annotation_http_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of ReviewsApi->reviews_submit_review_annotation:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ReviewsApi->reviews_submit_review_annotation: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id |
 **project_id** | **str**| project_id |
 **queue_id** | **str**| queue_id |
 **task_id** | **str**| task_id |
 **submit_review_annotation_http_request** | [**SubmitReviewAnnotationHttpRequest**](SubmitReviewAnnotationHttpRequest.md)|  |
 **authorization** | **str**| Bearer API token for strict auth | [optional]
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional]
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional]
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional]

### Return type

[**ReviewAnnotation**](ReviewAnnotation.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Submit a review annotation |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |
**404** | Resource not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
