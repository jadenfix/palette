# IngestQueueStatus


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**dead_letters** | [**List[DeadLetter]**](DeadLetter.md) |  |
**project_id** | **str** |  |
**tenant_id** | **str** |  |
**total_depth** | **int** |  |
**trace_ingested_depth** | **int** |  |
**trace_write_depth** | **int** |  |

## Example

```python
from beater_client.models.ingest_queue_status import IngestQueueStatus

# TODO update the JSON string below
json = "{}"
# create an instance of IngestQueueStatus from a JSON string
ingest_queue_status_instance = IngestQueueStatus.from_json(json)
# print the JSON string representation of the object
print(IngestQueueStatus.to_json())

# convert the object into a dict
ingest_queue_status_dict = ingest_queue_status_instance.to_dict()
# create an instance of IngestQueueStatus from a dict
ingest_queue_status_from_dict = IngestQueueStatus.from_dict(ingest_queue_status_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
