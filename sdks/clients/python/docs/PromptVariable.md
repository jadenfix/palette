# PromptVariable


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**default** | **str** |  | [optional] 
**description** | **str** |  | [optional] 
**name** | **str** |  | 
**required** | **bool** |  | 

## Example

```python
from beater_client.models.prompt_variable import PromptVariable

# TODO update the JSON string below
json = "{}"
# create an instance of PromptVariable from a JSON string
prompt_variable_instance = PromptVariable.from_json(json)
# print the JSON string representation of the object
print(PromptVariable.to_json())

# convert the object into a dict
prompt_variable_dict = prompt_variable_instance.to_dict()
# create an instance of PromptVariable from a dict
prompt_variable_from_dict = PromptVariable.from_dict(prompt_variable_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


