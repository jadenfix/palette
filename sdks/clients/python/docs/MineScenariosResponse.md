# MineScenariosResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**clusters** | [**List[ScenarioCluster]**](ScenarioCluster.md) |  | 

## Example

```python
from beater_client.models.mine_scenarios_response import MineScenariosResponse

# TODO update the JSON string below
json = "{}"
# create an instance of MineScenariosResponse from a JSON string
mine_scenarios_response_instance = MineScenariosResponse.from_json(json)
# print the JSON string representation of the object
print(MineScenariosResponse.to_json())

# convert the object into a dict
mine_scenarios_response_dict = mine_scenarios_response_instance.to_dict()
# create an instance of MineScenariosResponse from a dict
mine_scenarios_response_from_dict = MineScenariosResponse.from_dict(mine_scenarios_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


