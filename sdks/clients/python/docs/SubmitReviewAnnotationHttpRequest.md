# SubmitReviewAnnotationHttpRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**annotation_id** | **str** |  | [optional]
**payload** | **object** |  |
**reviewer_id** | **str** |  |
**verdict** | [**ReviewVerdict**](ReviewVerdict.md) |  |

## Example

```python
from beater_client.models.submit_review_annotation_http_request import SubmitReviewAnnotationHttpRequest

# TODO update the JSON string below
json = "{}"
# create an instance of SubmitReviewAnnotationHttpRequest from a JSON string
submit_review_annotation_http_request_instance = SubmitReviewAnnotationHttpRequest.from_json(json)
# print the JSON string representation of the object
print(SubmitReviewAnnotationHttpRequest.to_json())

# convert the object into a dict
submit_review_annotation_http_request_dict = submit_review_annotation_http_request_instance.to_dict()
# create an instance of SubmitReviewAnnotationHttpRequest from a dict
submit_review_annotation_http_request_from_dict = SubmitReviewAnnotationHttpRequest.from_dict(submit_review_annotation_http_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
