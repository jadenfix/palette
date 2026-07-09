# CaseOutputOverrideRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**case_id** | **str** |  |
**output** | **object** |  |
**trace** | **object** |  | [optional]

## Example

```python
from beater_client.models.case_output_override_request import CaseOutputOverrideRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CaseOutputOverrideRequest from a JSON string
case_output_override_request_instance = CaseOutputOverrideRequest.from_json(json)
# print the JSON string representation of the object
print(CaseOutputOverrideRequest.to_json())

# convert the object into a dict
case_output_override_request_dict = case_output_override_request_instance.to_dict()
# create an instance of CaseOutputOverrideRequest from a dict
case_output_override_request_from_dict = CaseOutputOverrideRequest.from_dict(case_output_override_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
