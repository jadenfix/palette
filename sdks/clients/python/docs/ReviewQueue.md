# ReviewQueue


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**annotation_schema** | **object** |  |
**created_at** | **datetime** |  |
**name** | **str** |  |
**project_id** | **str** |  |
**queue_id** | **str** |  |
**tenant_id** | **str** |  |

## Example

```python
from beater_client.models.review_queue import ReviewQueue

# TODO update the JSON string below
json = "{}"
# create an instance of ReviewQueue from a JSON string
review_queue_instance = ReviewQueue.from_json(json)
# print the JSON string representation of the object
print(ReviewQueue.to_json())

# convert the object into a dict
review_queue_dict = review_queue_instance.to_dict()
# create an instance of ReviewQueue from a dict
review_queue_from_dict = ReviewQueue.from_dict(review_queue_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
