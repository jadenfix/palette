# EvaluationCase


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**input** | **object** |  |
**output** | **object** |  |
**reference** | **object** |  | [optional]
**trace** | **object** |  | [optional]

## Example

```python
from beater_client.models.evaluation_case import EvaluationCase

# TODO update the JSON string below
json = "{}"
# create an instance of EvaluationCase from a JSON string
evaluation_case_instance = EvaluationCase.from_json(json)
# print the JSON string representation of the object
print(EvaluationCase.to_json())

# convert the object into a dict
evaluation_case_dict = evaluation_case_instance.to_dict()
# create an instance of EvaluationCase from a dict
evaluation_case_from_dict = EvaluationCase.from_dict(evaluation_case_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
