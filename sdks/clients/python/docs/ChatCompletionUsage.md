# ChatCompletionUsage

Token usage in the OpenAI-compatible response shape.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**completion_tokens** | **int** |  | 
**prompt_tokens** | **int** |  | 
**total_tokens** | **int** |  | 

## Example

```python
from beater_client.models.chat_completion_usage import ChatCompletionUsage

# TODO update the JSON string below
json = "{}"
# create an instance of ChatCompletionUsage from a JSON string
chat_completion_usage_instance = ChatCompletionUsage.from_json(json)
# print the JSON string representation of the object
print(ChatCompletionUsage.to_json())

# convert the object into a dict
chat_completion_usage_dict = chat_completion_usage_instance.to_dict()
# create an instance of ChatCompletionUsage from a dict
chat_completion_usage_from_dict = ChatCompletionUsage.from_dict(chat_completion_usage_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


