# BeaterClient::ReviewsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------ | ------------ | ----------- |
| [**create_review_queue**](ReviewsApi.md#create_review_queue) | **POST** /v1/review-queues/{tenant_id}/{project_id} |  |
| [**enqueue_review_task_from_trace**](ReviewsApi.md#enqueue_review_task_from_trace) | **POST** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks/from-trace |  |
| [**list_review_tasks**](ReviewsApi.md#list_review_tasks) | **GET** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks |  |
| [**promote_review_annotation**](ReviewsApi.md#promote_review_annotation) | **POST** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks/{task_id}/annotations/{annotation_id}/promote |  |
| [**submit_review_annotation**](ReviewsApi.md#submit_review_annotation) | **POST** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks/{task_id}/annotations |  |


## create_review_queue

> <ReviewQueue> create_review_queue(tenant_id, project_id, create_review_queue_http_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ReviewsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
create_review_queue_http_request = BeaterClient::CreateReviewQueueHttpRequest.new({annotation_schema: 3.56, name: 'name_example'}) # CreateReviewQueueHttpRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.create_review_queue(tenant_id, project_id, create_review_queue_http_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ReviewsApi->create_review_queue: #{e}"
end
```

#### Using the create_review_queue_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<ReviewQueue>, Integer, Hash)> create_review_queue_with_http_info(tenant_id, project_id, create_review_queue_http_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.create_review_queue_with_http_info(tenant_id, project_id, create_review_queue_http_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <ReviewQueue>
rescue BeaterClient::ApiError => e
  puts "Error when calling ReviewsApi->create_review_queue_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **create_review_queue_http_request** | [**CreateReviewQueueHttpRequest**](CreateReviewQueueHttpRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**ReviewQueue**](ReviewQueue.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json


## enqueue_review_task_from_trace

> <ReviewTask> enqueue_review_task_from_trace(tenant_id, project_id, queue_id, enqueue_review_task_from_trace_http_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ReviewsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
queue_id = 'queue_id_example' # String | queue_id
enqueue_review_task_from_trace_http_request = BeaterClient::EnqueueReviewTaskFromTraceHttpRequest.new({trace_id: 'trace_id_example'}) # EnqueueReviewTaskFromTraceHttpRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.enqueue_review_task_from_trace(tenant_id, project_id, queue_id, enqueue_review_task_from_trace_http_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ReviewsApi->enqueue_review_task_from_trace: #{e}"
end
```

#### Using the enqueue_review_task_from_trace_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<ReviewTask>, Integer, Hash)> enqueue_review_task_from_trace_with_http_info(tenant_id, project_id, queue_id, enqueue_review_task_from_trace_http_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.enqueue_review_task_from_trace_with_http_info(tenant_id, project_id, queue_id, enqueue_review_task_from_trace_http_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <ReviewTask>
rescue BeaterClient::ApiError => e
  puts "Error when calling ReviewsApi->enqueue_review_task_from_trace_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **queue_id** | **String** | queue_id |  |
| **enqueue_review_task_from_trace_http_request** | [**EnqueueReviewTaskFromTraceHttpRequest**](EnqueueReviewTaskFromTraceHttpRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**ReviewTask**](ReviewTask.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json


## list_review_tasks

> <Array<ReviewTask>> list_review_tasks(tenant_id, project_id, queue_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ReviewsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
queue_id = 'queue_id_example' # String | queue_id
opts = {
  state: BeaterClient::ReviewTaskState::OPEN, # ReviewTaskState |
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.list_review_tasks(tenant_id, project_id, queue_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ReviewsApi->list_review_tasks: #{e}"
end
```

#### Using the list_review_tasks_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<Array<ReviewTask>>, Integer, Hash)> list_review_tasks_with_http_info(tenant_id, project_id, queue_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.list_review_tasks_with_http_info(tenant_id, project_id, queue_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <Array<ReviewTask>>
rescue BeaterClient::ApiError => e
  puts "Error when calling ReviewsApi->list_review_tasks_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **queue_id** | **String** | queue_id |  |
| **state** | [**ReviewTaskState**](.md) |  | [optional] |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**Array&lt;ReviewTask&gt;**](ReviewTask.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json


## promote_review_annotation

> <DatasetCase> promote_review_annotation(tenant_id, project_id, queue_id, task_id, annotation_id, promote_review_annotation_http_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ReviewsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
queue_id = 'queue_id_example' # String | queue_id
task_id = 'task_id_example' # String | task_id
annotation_id = 'annotation_id_example' # String | annotation_id
promote_review_annotation_http_request = BeaterClient::PromoteReviewAnnotationHttpRequest.new({dataset_id: 'dataset_id_example'}) # PromoteReviewAnnotationHttpRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.promote_review_annotation(tenant_id, project_id, queue_id, task_id, annotation_id, promote_review_annotation_http_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ReviewsApi->promote_review_annotation: #{e}"
end
```

#### Using the promote_review_annotation_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<DatasetCase>, Integer, Hash)> promote_review_annotation_with_http_info(tenant_id, project_id, queue_id, task_id, annotation_id, promote_review_annotation_http_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.promote_review_annotation_with_http_info(tenant_id, project_id, queue_id, task_id, annotation_id, promote_review_annotation_http_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <DatasetCase>
rescue BeaterClient::ApiError => e
  puts "Error when calling ReviewsApi->promote_review_annotation_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **queue_id** | **String** | queue_id |  |
| **task_id** | **String** | task_id |  |
| **annotation_id** | **String** | annotation_id |  |
| **promote_review_annotation_http_request** | [**PromoteReviewAnnotationHttpRequest**](PromoteReviewAnnotationHttpRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**DatasetCase**](DatasetCase.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json


## submit_review_annotation

> <ReviewAnnotation> submit_review_annotation(tenant_id, project_id, queue_id, task_id, submit_review_annotation_http_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ReviewsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
queue_id = 'queue_id_example' # String | queue_id
task_id = 'task_id_example' # String | task_id
submit_review_annotation_http_request = BeaterClient::SubmitReviewAnnotationHttpRequest.new({payload: 3.56, reviewer_id: 'reviewer_id_example', verdict: BeaterClient::ReviewVerdict::PASS}) # SubmitReviewAnnotationHttpRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.submit_review_annotation(tenant_id, project_id, queue_id, task_id, submit_review_annotation_http_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ReviewsApi->submit_review_annotation: #{e}"
end
```

#### Using the submit_review_annotation_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<ReviewAnnotation>, Integer, Hash)> submit_review_annotation_with_http_info(tenant_id, project_id, queue_id, task_id, submit_review_annotation_http_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.submit_review_annotation_with_http_info(tenant_id, project_id, queue_id, task_id, submit_review_annotation_http_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <ReviewAnnotation>
rescue BeaterClient::ApiError => e
  puts "Error when calling ReviewsApi->submit_review_annotation_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **queue_id** | **String** | queue_id |  |
| **task_id** | **String** | task_id |  |
| **submit_review_annotation_http_request** | [**SubmitReviewAnnotationHttpRequest**](SubmitReviewAnnotationHttpRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**ReviewAnnotation**](ReviewAnnotation.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json
