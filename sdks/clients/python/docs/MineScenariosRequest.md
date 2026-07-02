# MineScenariosRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**jaccard_threshold** | **float** |  | [optional] 
**trace_ids** | **List[str]** |  | 

## Example

```python
from beater_client.models.mine_scenarios_request import MineScenariosRequest

# TODO update the JSON string below
json = "{}"
# create an instance of MineScenariosRequest from a JSON string
mine_scenarios_request_instance = MineScenariosRequest.from_json(json)
# print the JSON string representation of the object
print(MineScenariosRequest.to_json())

# convert the object into a dict
mine_scenarios_request_dict = mine_scenarios_request_instance.to_dict()
# create an instance of MineScenariosRequest from a dict
mine_scenarios_request_from_dict = MineScenariosRequest.from_dict(mine_scenarios_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


