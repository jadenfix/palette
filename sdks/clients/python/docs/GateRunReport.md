# GateRunReport


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**baseline_release_id** | **str** |  |
**candidate_release_id** | **str** |  |
**comparison** | [**ExperimentComparison**](ExperimentComparison.md) |  |
**created_at** | **datetime** |  |
**dataset_id** | **str** |  |
**evaluator_version_id** | **str** |  |
**experiment_created_at** | **datetime** |  |
**experiment_decision** | [**GateDecision**](GateDecision.md) |  |
**experiment_gate_policy** | [**GatePolicy**](GatePolicy.md) |  |
**experiment_run_id** | **str** |  |
**gate_dataset_id** | **str** |  | [optional]
**gate_evaluator_version_id** | **str** |  | [optional]
**gate_id** | **str** |  |
**gate_name** | **str** |  |
**gate_run_id** | **str** |  |
**inconclusive_policy** | [**InconclusivePolicy**](InconclusivePolicy.md) |  |
**passed** | **bool** |  |
**project_id** | **str** |  |
**reason** | **str** |  |
**tenant_id** | **str** |  |

## Example

```python
from beater_client.models.gate_run_report import GateRunReport

# TODO update the JSON string below
json = "{}"
# create an instance of GateRunReport from a JSON string
gate_run_report_instance = GateRunReport.from_json(json)
# print the JSON string representation of the object
print(GateRunReport.to_json())

# convert the object into a dict
gate_run_report_dict = gate_run_report_instance.to_dict()
# create an instance of GateRunReport from a dict
gate_run_report_from_dict = GateRunReport.from_dict(gate_run_report_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
