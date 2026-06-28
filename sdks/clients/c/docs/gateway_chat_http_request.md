# gateway_chat_http_request_t

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**budget_micros** | **long** | Per-tenant budget ceiling in micros; defaults when omitted. | [optional] 
**completion** | [**chat_completion_request_t**](chat_completion_request.md) \* | The OpenAI-compatible chat completion request. | 
**routing** | [**list_t**](model_routing.md) \* | Ordered failover list of credential policies (BYOK or managed). | [optional] 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


