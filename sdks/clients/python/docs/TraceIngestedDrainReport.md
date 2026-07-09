# TraceIngestedDrainReport


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**completed** | **int** |  |
**consumed** | **int** |  |
**dead_lettered** | **int** |  |
**failed_work** | **int** |  |
**invalid_messages** | **int** |  |
**retried** | **int** |  |
**trace_refs** | [**List[QueuedTraceWork]**](QueuedTraceWork.md) |  |

## Example

```python
from beater_client.models.trace_ingested_drain_report import TraceIngestedDrainReport

# TODO update the JSON string below
json = "{}"
# create an instance of TraceIngestedDrainReport from a JSON string
trace_ingested_drain_report_instance = TraceIngestedDrainReport.from_json(json)
# print the JSON string representation of the object
print(TraceIngestedDrainReport.to_json())

# convert the object into a dict
trace_ingested_drain_report_dict = trace_ingested_drain_report_instance.to_dict()
# create an instance of TraceIngestedDrainReport from a dict
trace_ingested_drain_report_from_dict = TraceIngestedDrainReport.from_dict(trace_ingested_drain_report_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
