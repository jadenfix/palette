# OtlpIngestOutcome


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**accepted_raw** | **int** |  |
**accepted_spans** | **int** |  |
**downstream_queued** | **bool** |  |
**duplicate_raw** | **int** |  |
**duplicate_spans** | **int** |  |

## Example

```python
from beater_client.models.otlp_ingest_outcome import OtlpIngestOutcome

# TODO update the JSON string below
json = "{}"
# create an instance of OtlpIngestOutcome from a JSON string
otlp_ingest_outcome_instance = OtlpIngestOutcome.from_json(json)
# print the JSON string representation of the object
print(OtlpIngestOutcome.to_json())

# convert the object into a dict
otlp_ingest_outcome_dict = otlp_ingest_outcome_instance.to_dict()
# create an instance of OtlpIngestOutcome from a dict
otlp_ingest_outcome_from_dict = OtlpIngestOutcome.from_dict(otlp_ingest_outcome_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
