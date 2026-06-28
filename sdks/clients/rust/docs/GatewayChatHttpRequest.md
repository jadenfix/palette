# GatewayChatHttpRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**budget_micros** | Option<**i64**> | Per-tenant budget ceiling in micros; defaults when omitted. | [optional]
**completion** | [**models::ChatCompletionRequest**](ChatCompletionRequest.md) | The OpenAI-compatible chat completion request. | 
**routing** | Option<[**Vec<models::ModelRouting>**](ModelRouting.md)> | Ordered failover list of credential policies (BYOK or managed). | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


