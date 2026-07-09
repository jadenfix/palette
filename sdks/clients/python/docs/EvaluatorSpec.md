# EvaluatorSpec


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **str** |  |
**kind** | [**EvaluatorKind**](EvaluatorKind.md) |  |
**lane** | [**EvaluatorLane**](EvaluatorLane.md) |  |

## Example

```python
from beater_client.models.evaluator_spec import EvaluatorSpec

# TODO update the JSON string below
json = "{}"
# create an instance of EvaluatorSpec from a JSON string
evaluator_spec_instance = EvaluatorSpec.from_json(json)
# print the JSON string representation of the object
print(EvaluatorSpec.to_json())

# convert the object into a dict
evaluator_spec_dict = evaluator_spec_instance.to_dict()
# create an instance of EvaluatorSpec from a dict
evaluator_spec_from_dict = EvaluatorSpec.from_dict(evaluator_spec_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
