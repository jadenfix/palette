# TraceWriteDrainReport


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**consumed** | **int** |  |
**dead_lettered** | **int** |  |
**downstream_published** | **int** |  |
**duplicate_raw** | **int** |  |
**duplicate_spans** | **int** |  |
**failed_downstream_publishes** | **int** |  |
**failed_writes** | **int** |  |
**invalid_messages** | **int** |  |
**retried** | **int** |  |
**trace_ids** | **List[str]** |  |
**trace_refs** | [**List[QueuedTraceWork]**](QueuedTraceWork.md) |  |
**written_raw** | **int** |  |
**written_spans** | **int** |  |

## Example

```python
from beater_client.models.trace_write_drain_report import TraceWriteDrainReport

# TODO update the JSON string below
json = "{}"
# create an instance of TraceWriteDrainReport from a JSON string
trace_write_drain_report_instance = TraceWriteDrainReport.from_json(json)
# print the JSON string representation of the object
print(TraceWriteDrainReport.to_json())

# convert the object into a dict
trace_write_drain_report_dict = trace_write_drain_report_instance.to_dict()
# create an instance of TraceWriteDrainReport from a dict
trace_write_drain_report_from_dict = TraceWriteDrainReport.from_dict(trace_write_drain_report_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
