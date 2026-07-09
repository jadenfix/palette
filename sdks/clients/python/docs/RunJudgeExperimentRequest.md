# RunJudgeExperimentRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**baseline_outputs** | [**List[CaseOutputOverrideRequest]**](CaseOutputOverrideRequest.md) |  |
**baseline_release_id** | **str** |  |
**candidate_outputs** | [**List[CaseOutputOverrideRequest]**](CaseOutputOverrideRequest.md) |  |
**candidate_release_id** | **str** |  |
**evaluator_id** | **str** |  |
**evaluator_version_id** | **str** |  |
**gate_policy** | [**GatePolicy**](GatePolicy.md) |  | [optional]
**kind** | [**EvaluatorKind**](EvaluatorKind.md) |  |
**provider_secret_id** | **str** |  |

## Example

```python
from beater_client.models.run_judge_experiment_request import RunJudgeExperimentRequest

# TODO update the JSON string below
json = "{}"
# create an instance of RunJudgeExperimentRequest from a JSON string
run_judge_experiment_request_instance = RunJudgeExperimentRequest.from_json(json)
# print the JSON string representation of the object
print(RunJudgeExperimentRequest.to_json())

# convert the object into a dict
run_judge_experiment_request_dict = run_judge_experiment_request_instance.to_dict()
# create an instance of RunJudgeExperimentRequest from a dict
run_judge_experiment_request_from_dict = RunJudgeExperimentRequest.from_dict(run_judge_experiment_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
