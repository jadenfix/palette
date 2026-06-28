# GatewayChatHttpRequest

Body for [`gateway_chat_completions_route`]: an OpenAI-compatible chat completion request plus the per-request routing policy and an optional budget ceiling. An empty `routing` means \"I did not bring a key\" — the gateway falls back to its managed default model if one is configured (hosted), otherwise it returns a typed no-key error (OSS).

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**budget_micros** | **int** | Per-tenant budget ceiling in micros; defaults when omitted. | [optional] 
**completion** | [**ChatCompletionRequest**](ChatCompletionRequest.md) | The OpenAI-compatible chat completion request. | 
**routing** | [**List[ModelRouting]**](ModelRouting.md) | Ordered failover list of credential policies (BYOK or managed). | [optional] 

## Example

```python
from beater_client.models.gateway_chat_http_request import GatewayChatHttpRequest

# TODO update the JSON string below
json = "{}"
# create an instance of GatewayChatHttpRequest from a JSON string
gateway_chat_http_request_instance = GatewayChatHttpRequest.from_json(json)
# print the JSON string representation of the object
print(GatewayChatHttpRequest.to_json())

# convert the object into a dict
gateway_chat_http_request_dict = gateway_chat_http_request_instance.to_dict()
# create an instance of GatewayChatHttpRequest from a dict
gateway_chat_http_request_from_dict = GatewayChatHttpRequest.from_dict(gateway_chat_http_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


