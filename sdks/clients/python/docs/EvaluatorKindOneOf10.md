# EvaluatorKindOneOf10

Browser recovery: passes when the run either hit no errors or recovered to a successful final step (catches death spirals after a failed action).

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**type** | **str** |  | 

## Example

```python
from beater_client.models.evaluator_kind_one_of10 import EvaluatorKindOneOf10

# TODO update the JSON string below
json = "{}"
# create an instance of EvaluatorKindOneOf10 from a JSON string
evaluator_kind_one_of10_instance = EvaluatorKindOneOf10.from_json(json)
# print the JSON string representation of the object
print(EvaluatorKindOneOf10.to_json())

# convert the object into a dict
evaluator_kind_one_of10_dict = evaluator_kind_one_of10_instance.to_dict()
# create an instance of EvaluatorKindOneOf10 from a dict
evaluator_kind_one_of10_from_dict = EvaluatorKindOneOf10.from_dict(evaluator_kind_one_of10_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


