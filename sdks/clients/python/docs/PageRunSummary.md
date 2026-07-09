# PageRunSummary


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**items** | [**List[PageRunSummaryItemsInner]**](PageRunSummaryItemsInner.md) |  |
**next_cursor** | **str** |  | [optional]

## Example

```python
from beater_client.models.page_run_summary import PageRunSummary

# TODO update the JSON string below
json = "{}"
# create an instance of PageRunSummary from a JSON string
page_run_summary_instance = PageRunSummary.from_json(json)
# print the JSON string representation of the object
print(PageRunSummary.to_json())

# convert the object into a dict
page_run_summary_dict = page_run_summary_instance.to_dict()
# create an instance of PageRunSummary from a dict
page_run_summary_from_dict = PageRunSummary.from_dict(page_run_summary_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
