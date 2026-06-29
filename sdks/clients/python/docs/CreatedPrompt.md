# CreatedPrompt


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**prompt** | [**Prompt**](Prompt.md) |  | 
**version** | [**PromptVersion**](PromptVersion.md) |  | 

## Example

```python
from beater_client.models.created_prompt import CreatedPrompt

# TODO update the JSON string below
json = "{}"
# create an instance of CreatedPrompt from a JSON string
created_prompt_instance = CreatedPrompt.from_json(json)
# print the JSON string representation of the object
print(CreatedPrompt.to_json())

# convert the object into a dict
created_prompt_dict = created_prompt_instance.to_dict()
# create an instance of CreatedPrompt from a dict
created_prompt_from_dict = CreatedPrompt.from_dict(created_prompt_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


