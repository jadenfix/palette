# ScenarioCluster

A cluster of failing traces that share a similar failure signature.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**dominant_failure_mode** | [**FailureMode**](FailureMode.md) | The most common failure mode across members. |
**exemplar_trace_id** | **str** |  |
**member_trace_ids** | **List[str]** | All member trace ids, sorted ascending. |
**signature** | [**Signature**](Signature.md) | The signature of the cluster&#39;s exemplar. |
**size** | **int** | Number of member traces. |

## Example

```python
from beater_client.models.scenario_cluster import ScenarioCluster

# TODO update the JSON string below
json = "{}"
# create an instance of ScenarioCluster from a JSON string
scenario_cluster_instance = ScenarioCluster.from_json(json)
# print the JSON string representation of the object
print(ScenarioCluster.to_json())

# convert the object into a dict
scenario_cluster_dict = scenario_cluster_instance.to_dict()
# create an instance of ScenarioCluster from a dict
scenario_cluster_from_dict = ScenarioCluster.from_dict(scenario_cluster_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
