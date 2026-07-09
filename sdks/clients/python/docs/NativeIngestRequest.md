# NativeIngestRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**attributes** | **Dict[str, object]** |  |
**auth_context** | [**AuthContext**](AuthContext.md) |  | [optional]
**cost** | [**Money**](Money.md) |  | [optional]
**end_time** | **datetime** |  | [optional]
**idempotency_key** | **str** |  | [optional]
**input** | **object** |  | [optional]
**kind** | **str** | Canonical agent span kind such as agent.run or llm.call |
**model** | [**ModelRef**](ModelRef.md) |  | [optional]
**name** | **str** |  |
**output** | **object** |  | [optional]
**parent_span_id** | **str** |  | [optional]
**redaction_class** | [**RedactionClass**](RedactionClass.md) |  |
**scope** | [**TenantScope**](TenantScope.md) |  |
**seq** | **int** |  |
**span_id** | **str** |  |
**start_time** | **datetime** |  | [optional]
**status** | [**SpanStatus**](SpanStatus.md) |  |
**tokens** | [**TokenCounts**](TokenCounts.md) |  | [optional]
**trace_id** | **str** |  |

## Example

```python
from beater_client.models.native_ingest_request import NativeIngestRequest

# TODO update the JSON string below
json = "{}"
# create an instance of NativeIngestRequest from a JSON string
native_ingest_request_instance = NativeIngestRequest.from_json(json)
# print the JSON string representation of the object
print(NativeIngestRequest.to_json())

# convert the object into a dict
native_ingest_request_dict = native_ingest_request_instance.to_dict()
# create an instance of NativeIngestRequest from a dict
native_ingest_request_from_dict = NativeIngestRequest.from_dict(native_ingest_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
