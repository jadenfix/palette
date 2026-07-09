# PromoteTraceCaseRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**reference** | **object** |  | [optional]
**span_id** | **str** |  | [optional]
**trace_id** | **str** |  |

## Example

```python
from beater_client.models.promote_trace_case_request import PromoteTraceCaseRequest

# TODO update the JSON string below
json = "{}"
# create an instance of PromoteTraceCaseRequest from a JSON string
promote_trace_case_request_instance = PromoteTraceCaseRequest.from_json(json)
# print the JSON string representation of the object
print(PromoteTraceCaseRequest.to_json())

# convert the object into a dict
promote_trace_case_request_dict = promote_trace_case_request_instance.to_dict()
# create an instance of PromoteTraceCaseRequest from a dict
promote_trace_case_request_from_dict = PromoteTraceCaseRequest.from_dict(promote_trace_case_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
