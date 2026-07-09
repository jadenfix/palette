# IngestOutcome


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**ack** | [**WriteAck**](WriteAck.md) |  |
**downstream_queued** | **bool** |  |

## Example

```python
from beater_client.models.ingest_outcome import IngestOutcome

# TODO update the JSON string below
json = "{}"
# create an instance of IngestOutcome from a JSON string
ingest_outcome_instance = IngestOutcome.from_json(json)
# print the JSON string representation of the object
print(IngestOutcome.to_json())

# convert the object into a dict
ingest_outcome_dict = ingest_outcome_instance.to_dict()
# create an instance of IngestOutcome from a dict
ingest_outcome_from_dict = IngestOutcome.from_dict(ingest_outcome_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
