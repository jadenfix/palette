# QueuedTraceWork


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**project_id** | **str** |  |
**tenant_id** | **str** |  |
**trace_id** | **str** |  |

## Example

```python
from beater_client.models.queued_trace_work import QueuedTraceWork

# TODO update the JSON string below
json = "{}"
# create an instance of QueuedTraceWork from a JSON string
queued_trace_work_instance = QueuedTraceWork.from_json(json)
# print the JSON string representation of the object
print(QueuedTraceWork.to_json())

# convert the object into a dict
queued_trace_work_dict = queued_trace_work_instance.to_dict()
# create an instance of QueuedTraceWork from a dict
queued_trace_work_from_dict = QueuedTraceWork.from_dict(queued_trace_work_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
