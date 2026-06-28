# EvaluatorKindOneOf7

Browser world-state success: asserts the final step's observed page (url and/or DOM) matches the configured target — NOT the agent's self-reported \"done\". Reads `trace.browser_steps`.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**dom_contains** | **str** |  | [optional] 
**type** | **str** |  | 
**url_contains** | **str** |  | [optional] 

## Example

```python
from beater_client.models.evaluator_kind_one_of7 import EvaluatorKindOneOf7

# TODO update the JSON string below
json = "{}"
# create an instance of EvaluatorKindOneOf7 from a JSON string
evaluator_kind_one_of7_instance = EvaluatorKindOneOf7.from_json(json)
# print the JSON string representation of the object
print(EvaluatorKindOneOf7.to_json())

# convert the object into a dict
evaluator_kind_one_of7_dict = evaluator_kind_one_of7_instance.to_dict()
# create an instance of EvaluatorKindOneOf7 from a dict
evaluator_kind_one_of7_from_dict = EvaluatorKindOneOf7.from_dict(evaluator_kind_one_of7_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


