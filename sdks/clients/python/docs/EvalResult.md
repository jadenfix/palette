# EvalResult


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**cost** | [**Money**](Money.md) |  | [optional]
**created_at** | **datetime** |  |
**eval_result_id** | **str** |  |
**evidence** | **object** |  |
**label** | **str** |  | [optional]
**non_reproducible_reason** | **str** |  | [optional]
**project_id** | **str** |  |
**reproducibility** | [**EvalReproducibility**](EvalReproducibility.md) |  |
**score** | **float** |  |
**span_id** | **str** |  | [optional]
**tenant_id** | **str** |  |
**tokens** | [**TokenCounts**](TokenCounts.md) |  | [optional]
**trace_id** | **str** |  |

## Example

```python
from beater_client.models.eval_result import EvalResult

# TODO update the JSON string below
json = "{}"
# create an instance of EvalResult from a JSON string
eval_result_instance = EvalResult.from_json(json)
# print the JSON string representation of the object
print(EvalResult.to_json())

# convert the object into a dict
eval_result_dict = eval_result_instance.to_dict()
# create an instance of EvalResult from a dict
eval_result_from_dict = EvalResult.from_dict(eval_result_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
