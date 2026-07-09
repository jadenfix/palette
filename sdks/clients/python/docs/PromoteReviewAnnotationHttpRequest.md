# PromoteReviewAnnotationHttpRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**dataset_id** | **str** |  |
**reference** | **object** |  | [optional]

## Example

```python
from beater_client.models.promote_review_annotation_http_request import PromoteReviewAnnotationHttpRequest

# TODO update the JSON string below
json = "{}"
# create an instance of PromoteReviewAnnotationHttpRequest from a JSON string
promote_review_annotation_http_request_instance = PromoteReviewAnnotationHttpRequest.from_json(json)
# print the JSON string representation of the object
print(PromoteReviewAnnotationHttpRequest.to_json())

# convert the object into a dict
promote_review_annotation_http_request_dict = promote_review_annotation_http_request_instance.to_dict()
# create an instance of PromoteReviewAnnotationHttpRequest from a dict
promote_review_annotation_http_request_from_dict = PromoteReviewAnnotationHttpRequest.from_dict(promote_review_annotation_http_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
