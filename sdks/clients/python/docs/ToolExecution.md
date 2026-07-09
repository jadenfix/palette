# ToolExecution

Result of executing a tool — Composio's `{successful, data, error}` envelope.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**data** | **object** | Tool output payload (shape is tool-specific). | [optional]
**error** | **str** | Error message when &#x60;successful&#x60; is false. | [optional]
**log_id** | **str** | Composio execution log id, for tracing. | [optional]
**successful** | **bool** | Whether the tool reported success. |

## Example

```python
from beater_client.models.tool_execution import ToolExecution

# TODO update the JSON string below
json = "{}"
# create an instance of ToolExecution from a JSON string
tool_execution_instance = ToolExecution.from_json(json)
# print the JSON string representation of the object
print(ToolExecution.to_json())

# convert the object into a dict
tool_execution_dict = tool_execution_instance.to_dict()
# create an instance of ToolExecution from a dict
tool_execution_from_dict = ToolExecution.from_dict(tool_execution_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
