# CreateReviewQueueHttpRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**annotation_schema** | **object** |  |
**name** | **str** |  |
**queue_id** | **str** |  | [optional]

## Example

```python
from beater_client.models.create_review_queue_http_request import CreateReviewQueueHttpRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreateReviewQueueHttpRequest from a JSON string
create_review_queue_http_request_instance = CreateReviewQueueHttpRequest.from_json(json)
# print the JSON string representation of the object
print(CreateReviewQueueHttpRequest.to_json())

# convert the object into a dict
create_review_queue_http_request_dict = create_review_queue_http_request_instance.to_dict()
# create an instance of CreateReviewQueueHttpRequest from a dict
create_review_queue_http_request_from_dict = CreateReviewQueueHttpRequest.from_dict(create_review_queue_http_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
