# ExperimentRunReport


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**baseline_release_id** | **str** |  |
**candidate_release_id** | **str** |  |
**case_scores** | [**List[CaseExperimentScore]**](CaseExperimentScore.md) |  |
**comparison** | [**ExperimentComparison**](ExperimentComparison.md) |  |
**created_at** | **datetime** |  |
**dataset_id** | **str** |  |
**dataset_version_id** | **str** |  |
**decision** | [**GateDecision**](GateDecision.md) |  |
**evaluator_version_id** | **str** |  |
**experiment_run_id** | **str** |  |
**gate_policy** | [**GatePolicy**](GatePolicy.md) |  | [optional]
**project_id** | **str** |  |
**tenant_id** | **str** |  |

## Example

```python
from beater_client.models.experiment_run_report import ExperimentRunReport

# TODO update the JSON string below
json = "{}"
# create an instance of ExperimentRunReport from a JSON string
experiment_run_report_instance = ExperimentRunReport.from_json(json)
# print the JSON string representation of the object
print(ExperimentRunReport.to_json())

# convert the object into a dict
experiment_run_report_dict = experiment_run_report_instance.to_dict()
# create an instance of ExperimentRunReport from a dict
experiment_run_report_from_dict = ExperimentRunReport.from_dict(experiment_run_report_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
