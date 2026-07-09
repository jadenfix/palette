# RunJudgeDatasetEvalRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**agent_release_id** | **str** |  |
**code_hash** | **str** |  | [optional]
**evaluator_id** | **str** |  |
**evaluator_version_id** | **str** |  |
**kind** | [**EvaluatorKind**](EvaluatorKind.md) |  |
**prompt_version_id** | **str** |  | [optional]
**provider_secret_id** | **str** |  |

## Example

```python
from beater_client.models.run_judge_dataset_eval_request import RunJudgeDatasetEvalRequest

# TODO update the JSON string below
json = "{}"
# create an instance of RunJudgeDatasetEvalRequest from a JSON string
run_judge_dataset_eval_request_instance = RunJudgeDatasetEvalRequest.from_json(json)
# print the JSON string representation of the object
print(RunJudgeDatasetEvalRequest.to_json())

# convert the object into a dict
run_judge_dataset_eval_request_dict = run_judge_dataset_eval_request_instance.to_dict()
# create an instance of RunJudgeDatasetEvalRequest from a dict
run_judge_dataset_eval_request_from_dict = RunJudgeDatasetEvalRequest.from_dict(run_judge_dataset_eval_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
