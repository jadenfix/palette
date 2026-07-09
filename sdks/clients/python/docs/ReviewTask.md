# ReviewTask


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **datetime** |  |
**dataset_case_id** | **str** |  | [optional]
**dataset_id** | **str** |  | [optional]
**priority** | **int** |  |
**project_id** | **str** |  |
**queue_id** | **str** |  |
**span_id** | **str** |  | [optional]
**state** | [**ReviewTaskState**](ReviewTaskState.md) |  |
**task_id** | **str** |  |
**tenant_id** | **str** |  |
**trace_id** | **str** |  |
**updated_at** | **datetime** |  |

## Example

```python
from beater_client.models.review_task import ReviewTask

# TODO update the JSON string below
json = "{}"
# create an instance of ReviewTask from a JSON string
review_task_instance = ReviewTask.from_json(json)
# print the JSON string representation of the object
print(ReviewTask.to_json())

# convert the object into a dict
review_task_dict = review_task_instance.to_dict()
# create an instance of ReviewTask from a dict
review_task_from_dict = ReviewTask.from_dict(review_task_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
