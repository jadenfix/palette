# CreateScenarioRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**exemplar_trace_id** | **str** |  | [optional]
**expected_outcome** | **str** |  | [optional]
**failure_mode** | [**FailureMode**](FailureMode.md) |  | [optional]
**source_trace_ids** | **List[str]** |  |
**title** | **str** |  |

## Example

```python
from beater_client.models.create_scenario_request import CreateScenarioRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreateScenarioRequest from a JSON string
create_scenario_request_instance = CreateScenarioRequest.from_json(json)
# print the JSON string representation of the object
print(CreateScenarioRequest.to_json())

# convert the object into a dict
create_scenario_request_dict = create_scenario_request_instance.to_dict()
# create an instance of CreateScenarioRequest from a dict
create_scenario_request_from_dict = CreateScenarioRequest.from_dict(create_scenario_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
