# PromptTemplate


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**body** | **str** |  | 
**tags** | **List[str]** |  | 
**variables** | [**List[PromptVariable]**](PromptVariable.md) |  | 

## Example

```python
from beater_client.models.prompt_template import PromptTemplate

# TODO update the JSON string below
json = "{}"
# create an instance of PromptTemplate from a JSON string
prompt_template_instance = PromptTemplate.from_json(json)
# print the JSON string representation of the object
print(PromptTemplate.to_json())

# convert the object into a dict
prompt_template_dict = prompt_template_instance.to_dict()
# create an instance of PromptTemplate from a dict
prompt_template_from_dict = PromptTemplate.from_dict(prompt_template_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


