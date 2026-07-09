# DatasetEvalReport


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**aggregate_score** | **float** |  |
**created_at** | **datetime** |  |
**dataset_id** | **str** |  |
**dataset_version_id** | **str** |  |
**evaluator_version_id** | **str** |  |
**project_id** | **str** |  |
**report_id** | **str** |  |
**result_count** | **int** |  |
**results** | [**List[EvalResult]**](EvalResult.md) |  |
**tenant_id** | **str** |  |

## Example

```python
from beater_client.models.dataset_eval_report import DatasetEvalReport

# TODO update the JSON string below
json = "{}"
# create an instance of DatasetEvalReport from a JSON string
dataset_eval_report_instance = DatasetEvalReport.from_json(json)
# print the JSON string representation of the object
print(DatasetEvalReport.to_json())

# convert the object into a dict
dataset_eval_report_dict = dataset_eval_report_instance.to_dict()
# create an instance of DatasetEvalReport from a dict
dataset_eval_report_from_dict = DatasetEvalReport.from_dict(dataset_eval_report_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
