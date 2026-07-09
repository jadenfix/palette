# ArchivedSpanRow


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**attributes_json** | **str** |  |
**cost_amount_micros** | **str** |  | [optional]
**cost_currency** | **str** |  | [optional]
**end_time** | **str** |  | [optional]
**environment_id** | **str** |  |
**input_tokens** | **str** |  | [optional]
**input_uri** | **str** |  | [optional]
**kind** | **str** |  |
**model_name** | **str** |  | [optional]
**model_provider** | **str** |  | [optional]
**name** | **str** |  |
**output_tokens** | **str** |  | [optional]
**output_uri** | **str** |  | [optional]
**parent_span_id** | **str** |  | [optional]
**project_id** | **str** |  |
**raw_uri** | **str** |  |
**reasoning_tokens** | **str** |  | [optional]
**seq** | **int** |  |
**span_id** | **str** |  |
**start_time** | **str** |  |
**status** | **str** |  |
**tenant_id** | **str** |  |
**trace_id** | **str** |  |
**unmapped_json** | **str** |  |

## Example

```python
from beater_client.models.archived_span_row import ArchivedSpanRow

# TODO update the JSON string below
json = "{}"
# create an instance of ArchivedSpanRow from a JSON string
archived_span_row_instance = ArchivedSpanRow.from_json(json)
# print the JSON string representation of the object
print(ArchivedSpanRow.to_json())

# convert the object into a dict
archived_span_row_dict = archived_span_row_instance.to_dict()
# create an instance of ArchivedSpanRow from a dict
archived_span_row_from_dict = ArchivedSpanRow.from_dict(archived_span_row_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
