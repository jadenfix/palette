

# GatewayChatHttpRequest

Body for [`gateway_chat_completions_route`]: an OpenAI-compatible chat completion request plus the per-request routing policy and an optional budget ceiling. An empty `routing` means \"I did not bring a key\" — the gateway falls back to its managed default model if one is configured (hosted), otherwise it returns a typed no-key error (OSS).

## Properties

| Name | Type | Description | Notes |
|------------ | ------------- | ------------- | -------------|
|**budgetMicros** | **Long** | Per-tenant budget ceiling in micros; defaults when omitted. |  [optional] |
|**completion** | [**ChatCompletionRequest**](ChatCompletionRequest.md) | The OpenAI-compatible chat completion request. |  |
|**routing** | [**List&lt;ModelRouting&gt;**](ModelRouting.md) | Ordered failover list of credential policies (BYOK or managed). |  [optional] |



