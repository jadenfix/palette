# ChatCompletionRequest

An OpenAI-compatible chat completion request. The `model` field is a free-form model identifier, making the gateway model-agnostic: any provider/model can be named per request, paired with a [`ModelRouting`] credential policy.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**max_tokens** | **int** |  | [optional] 
**messages** | [**List[ChatMessage]**](ChatMessage.md) |  | 
**model** | **str** |  | 
**reasoning_effort** | **str** | OpenAI-style reasoning effort hint (&#x60;low&#x60; | &#x60;medium&#x60; | &#x60;high&#x60;), forwarded to providers that understand it. | [optional] 
**temperature** | **float** |  | [optional] 
**top_p** | **float** |  | [optional] 

## Example

```python
from beater_client.models.chat_completion_request import ChatCompletionRequest

# TODO update the JSON string below
json = "{}"
# create an instance of ChatCompletionRequest from a JSON string
chat_completion_request_instance = ChatCompletionRequest.from_json(json)
# print the JSON string representation of the object
print(ChatCompletionRequest.to_json())

# convert the object into a dict
chat_completion_request_dict = chat_completion_request_instance.to_dict()
# create an instance of ChatCompletionRequest from a dict
chat_completion_request_from_dict = ChatCompletionRequest.from_dict(chat_completion_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


