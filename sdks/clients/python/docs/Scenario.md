# Scenario

A reusable failure scenario mined from production traces.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **datetime** | When the scenario was created. |
**exemplar_trace_id** | **str** |  |
**expected_outcome** | **str** | Expected outcome for replay assertions, if known. | [optional]
**failure_mode** | [**FailureMode**](FailureMode.md) | The dominant failure mode this scenario reproduces. |
**perturbation_knobs** | [**PerturbationKnobs**](PerturbationKnobs.md) | Suggested perturbation knobs for replay. |
**recurrence_count** | **int** | How many traces exhibited this scenario. |
**redaction_class** | [**RedactionClass**](RedactionClass.md) | Redaction classification of the scenario payload. |
**scenario_id** | **str** | Stable, deterministic identifier for the scenario. |
**scope** | [**TenantScope**](TenantScope.md) | Tenant/project/environment scope this scenario belongs to. |
**source_trace_ids** | **List[str]** | Trace ids the scenario was mined from, sorted ascending. |
**title** | **str** | Human-readable title. |

## Example

```python
from beater_client.models.scenario import Scenario

# TODO update the JSON string below
json = "{}"
# create an instance of Scenario from a JSON string
scenario_instance = Scenario.from_json(json)
# print the JSON string representation of the object
print(Scenario.to_json())

# convert the object into a dict
scenario_dict = scenario_instance.to_dict()
# create an instance of Scenario from a dict
scenario_from_dict = Scenario.from_dict(scenario_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
