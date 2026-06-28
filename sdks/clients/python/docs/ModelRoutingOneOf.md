# ModelRoutingOneOf

Bring-your-own-key: resolve the tenant's opaque provider secret.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**kind** | **str** |  | 
**provider_secret_id** | **str** |  | 

## Example

```python
from beater_client.models.model_routing_one_of import ModelRoutingOneOf

# TODO update the JSON string below
json = "{}"
# create an instance of ModelRoutingOneOf from a JSON string
model_routing_one_of_instance = ModelRoutingOneOf.from_json(json)
# print the JSON string representation of the object
print(ModelRoutingOneOf.to_json())

# convert the object into a dict
model_routing_one_of_dict = model_routing_one_of_instance.to_dict()
# create an instance of ModelRoutingOneOf from a dict
model_routing_one_of_from_dict = ModelRoutingOneOf.from_dict(model_routing_one_of_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


