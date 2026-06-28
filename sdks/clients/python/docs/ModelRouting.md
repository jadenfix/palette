# ModelRouting

Per-request credential routing policy.  This enum is the encoding of *\"any model, optionally BYOK, else managed default\"*:  * [`ModelRouting::Byok`] — use the tenant's own opaque provider secret. The   model named in the [`ChatCompletionRequest`] is used as-is, so any model the   key can reach is reachable. * [`ModelRouting::Managed`] — use Beater's managed credentials with the given   default model. Only available when the gateway is built with a   [`ManagedDefault`] (hosted edition).  A [`GatewayRequest`] carries an *ordered* `Vec<ModelRouting>` so the gateway can fail over from one key/provider to the next. An empty list means *\"I did not bring a key\"*: the gateway then uses the managed default if configured, or returns [`GatewayError::NoKeyAndNoManagedDefault`] in OSS.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**kind** | **str** |  | 
**provider_secret_id** | **str** |  | 
**default_model** | [**ModelRef**](ModelRef.md) |  | 

## Example

```python
from beater_client.models.model_routing import ModelRouting

# TODO update the JSON string below
json = "{}"
# create an instance of ModelRouting from a JSON string
model_routing_instance = ModelRouting.from_json(json)
# print the JSON string representation of the object
print(ModelRouting.to_json())

# convert the object into a dict
model_routing_dict = model_routing_instance.to_dict()
# create an instance of ModelRouting from a dict
model_routing_from_dict = ModelRouting.from_dict(model_routing_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


