# GateDefinition


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **datetime** |  |
**dataset_id** | **str** |  | [optional]
**evaluator_version_id** | **str** |  | [optional]
**gate_id** | **str** |  |
**inconclusive_policy** | [**InconclusivePolicy**](InconclusivePolicy.md) |  | [optional]
**name** | **str** |  |
**project_id** | **str** |  |
**tenant_id** | **str** |  |

## Example

```python
from beater_client.models.gate_definition import GateDefinition

# TODO update the JSON string below
json = "{}"
# create an instance of GateDefinition from a JSON string
gate_definition_instance = GateDefinition.from_json(json)
# print the JSON string representation of the object
print(GateDefinition.to_json())

# convert the object into a dict
gate_definition_dict = gate_definition_instance.to_dict()
# create an instance of GateDefinition from a dict
gate_definition_from_dict = GateDefinition.from_dict(gate_definition_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
