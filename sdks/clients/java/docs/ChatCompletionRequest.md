

# ChatCompletionRequest

An OpenAI-compatible chat completion request. The `model` field is a free-form model identifier, making the gateway model-agnostic: any provider/model can be named per request, paired with a [`ModelRouting`] credential policy.

## Properties

| Name | Type | Description | Notes |
|------------ | ------------- | ------------- | -------------|
|**maxTokens** | **Long** |  |  [optional] |
|**messages** | [**List&lt;ChatMessage&gt;**](ChatMessage.md) |  |  |
|**model** | **String** |  |  |
|**reasoningEffort** | **String** | OpenAI-style reasoning effort hint (&#x60;low&#x60; | &#x60;medium&#x60; | &#x60;high&#x60;), forwarded to providers that understand it. |  [optional] |
|**temperature** | **Double** |  |  [optional] |
|**topP** | **Double** |  |  [optional] |



