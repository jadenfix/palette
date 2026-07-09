# EvaluatorKindOneOf8

Browser step efficiency: passes when the run used at most `max_steps` browser steps (catches looping/backtracking). Reads `trace.browser_steps`.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**max_steps** | **int** |  |
**type** | **str** |  |

## Example

```python
from beater_client.models.evaluator_kind_one_of8 import EvaluatorKindOneOf8

# TODO update the JSON string below
json = "{}"
# create an instance of EvaluatorKindOneOf8 from a JSON string
evaluator_kind_one_of8_instance = EvaluatorKindOneOf8.from_json(json)
# print the JSON string representation of the object
print(EvaluatorKindOneOf8.to_json())

# convert the object into a dict
evaluator_kind_one_of8_dict = evaluator_kind_one_of8_instance.to_dict()
# create an instance of EvaluatorKindOneOf8 from a dict
evaluator_kind_one_of8_from_dict = EvaluatorKindOneOf8.from_dict(evaluator_kind_one_of8_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
