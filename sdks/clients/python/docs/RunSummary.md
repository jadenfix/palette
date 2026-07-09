# RunSummary


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**duration_ms** | **int** |  | [optional]
**ended_at** | **datetime** |  | [optional]
**first_span_name** | **str** |  |
**models** | [**List[ModelRef]**](ModelRef.md) |  |
**project_id** | **str** |  |
**release_ids** | **List[str]** |  |
**span_count** | **int** |  |
**started_at** | **datetime** |  |
**status** | [**SpanStatus**](SpanStatus.md) |  |
**tenant_id** | **str** |  |
**total_cost** | [**Money**](Money.md) |  | [optional]
**trace_id** | **str** |  |

## Example

```python
from beater_client.models.run_summary import RunSummary

# TODO update the JSON string below
json = "{}"
# create an instance of RunSummary from a JSON string
run_summary_instance = RunSummary.from_json(json)
# print the JSON string representation of the object
print(RunSummary.to_json())

# convert the object into a dict
run_summary_dict = run_summary_instance.to_dict()
# create an instance of RunSummary from a dict
run_summary_from_dict = RunSummary.from_dict(run_summary_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
