# GatewayOutcome

The result of a successfully proxied completion.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**attempts** | **int** |  | 
**cached** | **bool** |  | 
**cost** | [**Money**](Money.md) |  | 
**model** | [**ModelRef**](ModelRef.md) |  | 
**provider** | **str** |  | 
**request_hash** | **str** |  | 
**response** | [**ChatCompletionResponse**](ChatCompletionResponse.md) |  | 
**tokens** | [**TokenCounts**](TokenCounts.md) |  | 

## Example

```python
from beater_client.models.gateway_outcome import GatewayOutcome

# TODO update the JSON string below
json = "{}"
# create an instance of GatewayOutcome from a JSON string
gateway_outcome_instance = GatewayOutcome.from_json(json)
# print the JSON string representation of the object
print(GatewayOutcome.to_json())

# convert the object into a dict
gateway_outcome_dict = gateway_outcome_instance.to_dict()
# create an instance of GatewayOutcome from a dict
gateway_outcome_from_dict = GatewayOutcome.from_dict(gateway_outcome_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


