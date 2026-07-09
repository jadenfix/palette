# ReviewAnnotation


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**annotation_id** | **str** |  |
**created_at** | **datetime** |  |
**payload** | **object** |  |
**project_id** | **str** |  |
**queue_id** | **str** |  |
**reviewer_id** | **str** |  |
**task_id** | **str** |  |
**tenant_id** | **str** |  |
**verdict** | [**ReviewVerdict**](ReviewVerdict.md) |  |

## Example

```python
from beater_client.models.review_annotation import ReviewAnnotation

# TODO update the JSON string below
json = "{}"
# create an instance of ReviewAnnotation from a JSON string
review_annotation_instance = ReviewAnnotation.from_json(json)
# print the JSON string representation of the object
print(ReviewAnnotation.to_json())

# convert the object into a dict
review_annotation_dict = review_annotation_instance.to_dict()
# create an instance of ReviewAnnotation from a dict
review_annotation_from_dict = ReviewAnnotation.from_dict(review_annotation_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
