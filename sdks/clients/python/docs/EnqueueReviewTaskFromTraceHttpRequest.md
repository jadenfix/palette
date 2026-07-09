# EnqueueReviewTaskFromTraceHttpRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**dataset_case_id** | **str** |  | [optional]
**dataset_id** | **str** |  | [optional]
**priority** | **int** |  | [optional]
**span_id** | **str** |  | [optional]
**task_id** | **str** |  | [optional]
**trace_id** | **str** |  |

## Example

```python
from beater_client.models.enqueue_review_task_from_trace_http_request import EnqueueReviewTaskFromTraceHttpRequest

# TODO update the JSON string below
json = "{}"
# create an instance of EnqueueReviewTaskFromTraceHttpRequest from a JSON string
enqueue_review_task_from_trace_http_request_instance = EnqueueReviewTaskFromTraceHttpRequest.from_json(json)
# print the JSON string representation of the object
print(EnqueueReviewTaskFromTraceHttpRequest.to_json())

# convert the object into a dict
enqueue_review_task_from_trace_http_request_dict = enqueue_review_task_from_trace_http_request_instance.to_dict()
# create an instance of EnqueueReviewTaskFromTraceHttpRequest from a dict
enqueue_review_task_from_trace_http_request_from_dict = EnqueueReviewTaskFromTraceHttpRequest.from_dict(enqueue_review_task_from_trace_http_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
