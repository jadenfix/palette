# RunJudgeEvalHttpRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**cache_namespace** | **str** | Calibration-map / judge-instrument version folded into the judge cache key; bumping it on recalibration invalidates stale cached scores. | [optional]
**case** | [**EvaluationCase**](EvaluationCase.md) |  |
**evaluator** | [**EvaluatorSpec**](EvaluatorSpec.md) |  |
**provider_secret_id** | **str** |  |

## Example

```python
from beater_client.models.run_judge_eval_http_request import RunJudgeEvalHttpRequest

# TODO update the JSON string below
json = "{}"
# create an instance of RunJudgeEvalHttpRequest from a JSON string
run_judge_eval_http_request_instance = RunJudgeEvalHttpRequest.from_json(json)
# print the JSON string representation of the object
print(RunJudgeEvalHttpRequest.to_json())

# convert the object into a dict
run_judge_eval_http_request_dict = run_judge_eval_http_request_instance.to_dict()
# create an instance of RunJudgeEvalHttpRequest from a dict
run_judge_eval_http_request_from_dict = RunJudgeEvalHttpRequest.from_dict(run_judge_eval_http_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
