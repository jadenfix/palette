# ListScenariosResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**next_cursor** | **str** |  | [optional]
**scenarios** | [**List[Scenario]**](Scenario.md) |  |

## Example

```python
from beater_client.models.list_scenarios_response import ListScenariosResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ListScenariosResponse from a JSON string
list_scenarios_response_instance = ListScenariosResponse.from_json(json)
# print the JSON string representation of the object
print(ListScenariosResponse.to_json())

# convert the object into a dict
list_scenarios_response_dict = list_scenarios_response_instance.to_dict()
# create an instance of ListScenariosResponse from a dict
list_scenarios_response_from_dict = ListScenariosResponse.from_dict(list_scenarios_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
