# EvaluatorKind


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**type** | **str** |  |
**pattern** | **str** |  |
**abs** | **float** |  |
**rel** | **float** |  |
**max_micros** | **int** |  |
**max_ms** | **int** |  |
**model** | **str** |  |
**rubric** | **str** |  |
**dom_contains** | **str** |  | [optional]
**url_contains** | **str** |  | [optional]
**max_steps** | **int** |  |
**min_ratio** | **float** |  |

## Example

```python
from beater_client.models.evaluator_kind import EvaluatorKind

# TODO update the JSON string below
json = "{}"
# create an instance of EvaluatorKind from a JSON string
evaluator_kind_instance = EvaluatorKind.from_json(json)
# print the JSON string representation of the object
print(EvaluatorKind.to_json())

# convert the object into a dict
evaluator_kind_dict = evaluator_kind_instance.to_dict()
# create an instance of EvaluatorKind from a dict
evaluator_kind_from_dict = EvaluatorKind.from_dict(evaluator_kind_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
