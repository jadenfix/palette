# PageRunSummaryItemsInner


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
from beater_client.models.page_run_summary_items_inner import PageRunSummaryItemsInner

# TODO update the JSON string below
json = "{}"
# create an instance of PageRunSummaryItemsInner from a JSON string
page_run_summary_items_inner_instance = PageRunSummaryItemsInner.from_json(json)
# print the JSON string representation of the object
print(PageRunSummaryItemsInner.to_json())

# convert the object into a dict
page_run_summary_items_inner_dict = page_run_summary_items_inner_instance.to_dict()
# create an instance of PageRunSummaryItemsInner from a dict
page_run_summary_items_inner_from_dict = PageRunSummaryItemsInner.from_dict(page_run_summary_items_inner_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
