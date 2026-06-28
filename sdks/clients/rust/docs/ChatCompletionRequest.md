# ChatCompletionRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**max_tokens** | Option<**i64**> |  | [optional]
**messages** | [**Vec<models::ChatMessage>**](ChatMessage.md) |  | 
**model** | **String** |  | 
**reasoning_effort** | Option<**String**> | OpenAI-style reasoning effort hint (`low` | `medium` | `high`), forwarded to providers that understand it. | [optional]
**temperature** | Option<**f64**> |  | [optional]
**top_p** | Option<**f64**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


