# CreateGateRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**dataset_id** | **str** |  | [optional]
**evaluator_version_id** | **str** |  | [optional]
**gate_id** | **str** |  |
**inconclusive_policy** | [**InconclusivePolicy**](InconclusivePolicy.md) |  | [optional]
**name** | **str** |  |

## Example

```python
from beater_client.models.create_gate_request import CreateGateRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreateGateRequest from a JSON string
create_gate_request_instance = CreateGateRequest.from_json(json)
# print the JSON string representation of the object
print(CreateGateRequest.to_json())

# convert the object into a dict
create_gate_request_dict = create_gate_request_instance.to_dict()
# create an instance of CreateGateRequest from a dict
create_gate_request_from_dict = CreateGateRequest.from_dict(create_gate_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
