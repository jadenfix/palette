# CaseExperimentScore


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**baseline_cached** | **bool** |  | [optional]
**baseline_cost** | [**Money**](Money.md) |  | [optional]
**baseline_evidence** | **object** |  |
**baseline_judge_call_id** | **str** |  | [optional]
**baseline_output** | **object** |  |
**baseline_score** | **float** |  |
**baseline_trace** | **object** |  | [optional]
**candidate_cached** | **bool** |  | [optional]
**candidate_cost** | [**Money**](Money.md) |  | [optional]
**candidate_evidence** | **object** |  |
**candidate_judge_call_id** | **str** |  | [optional]
**candidate_output** | **object** |  |
**candidate_score** | **float** |  |
**candidate_trace** | **object** |  | [optional]
**case_id** | **str** |  |
**delta** | **float** |  |
**reference** | **object** |  | [optional]

## Example

```python
from beater_client.models.case_experiment_score import CaseExperimentScore

# TODO update the JSON string below
json = "{}"
# create an instance of CaseExperimentScore from a JSON string
case_experiment_score_instance = CaseExperimentScore.from_json(json)
# print the JSON string representation of the object
print(CaseExperimentScore.to_json())

# convert the object into a dict
case_experiment_score_dict = case_experiment_score_instance.to_dict()
# create an instance of CaseExperimentScore from a dict
case_experiment_score_from_dict = CaseExperimentScore.from_dict(case_experiment_score_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
