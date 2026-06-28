# ModelRouting

## Enum Variants

| Name | Description |
|---- | -----|
| ModelRoutingOneOf | Per-request credential routing policy.  This enum is the encoding of *\&quot;any model, optionally BYOK, else managed default\&quot;*:  * [&#x60;ModelRouting::Byok&#x60;] — use the tenant&#39;s own opaque provider secret. The   model named in the [&#x60;ChatCompletionRequest&#x60;] is used as-is, so any model the   key can reach is reachable. * [&#x60;ModelRouting::Managed&#x60;] — use Beater&#39;s managed credentials with the given   default model. Only available when the gateway is built with a   [&#x60;ManagedDefault&#x60;] (hosted edition).  A [&#x60;GatewayRequest&#x60;] carries an *ordered* &#x60;Vec&lt;ModelRouting&gt;&#x60; so the gateway can fail over from one key/provider to the next. An empty list means *\&quot;I did not bring a key\&quot;*: the gateway then uses the managed default if configured, or returns [&#x60;GatewayError::NoKeyAndNoManagedDefault&#x60;] in OSS. |
| ModelRoutingOneOf1 | Per-request credential routing policy.  This enum is the encoding of *\&quot;any model, optionally BYOK, else managed default\&quot;*:  * [&#x60;ModelRouting::Byok&#x60;] — use the tenant&#39;s own opaque provider secret. The   model named in the [&#x60;ChatCompletionRequest&#x60;] is used as-is, so any model the   key can reach is reachable. * [&#x60;ModelRouting::Managed&#x60;] — use Beater&#39;s managed credentials with the given   default model. Only available when the gateway is built with a   [&#x60;ManagedDefault&#x60;] (hosted edition).  A [&#x60;GatewayRequest&#x60;] carries an *ordered* &#x60;Vec&lt;ModelRouting&gt;&#x60; so the gateway can fail over from one key/provider to the next. An empty list means *\&quot;I did not bring a key\&quot;*: the gateway then uses the managed default if configured, or returns [&#x60;GatewayError::NoKeyAndNoManagedDefault&#x60;] in OSS. |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


