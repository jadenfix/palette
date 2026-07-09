# TraceIngestedReconcileReport


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**downstream_accepted** | **int** |  |
**downstream_duplicate** | **int** |  |
**downstream_queued** | **bool** |  |
**project_id** | **str** |  |
**span_count** | **int** |  |
**tenant_id** | **str** |  |
**trace_id** | **str** |  |

## Example

```python
from beater_client.models.trace_ingested_reconcile_report import TraceIngestedReconcileReport

# TODO update the JSON string below
json = "{}"
# create an instance of TraceIngestedReconcileReport from a JSON string
trace_ingested_reconcile_report_instance = TraceIngestedReconcileReport.from_json(json)
# print the JSON string representation of the object
print(TraceIngestedReconcileReport.to_json())

# convert the object into a dict
trace_ingested_reconcile_report_dict = trace_ingested_reconcile_report_instance.to_dict()
# create an instance of TraceIngestedReconcileReport from a dict
trace_ingested_reconcile_report_from_dict = TraceIngestedReconcileReport.from_dict(trace_ingested_reconcile_report_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
