# CanonicalSpan


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**attributes** | **Dict[str, object]** |  |
**cost** | [**Money**](Money.md) |  | [optional]
**end_time** | **datetime** |  | [optional]
**environment_id** | **str** |  |
**input_ref** | [**ArtifactRef**](ArtifactRef.md) |  | [optional]
**kind** | **str** | Canonical agent span kind such as agent.run or llm.call |
**model** | [**ModelRef**](ModelRef.md) |  | [optional]
**name** | **str** |  |
**normalizer_version** | **str** |  |
**output_ref** | [**ArtifactRef**](ArtifactRef.md) |  | [optional]
**parent_span_id** | **str** |  | [optional]
**project_id** | **str** |  |
**raw_ref** | [**ArtifactRef**](ArtifactRef.md) |  |
**schema_version** | **int** |  |
**seq** | **int** |  |
**span_id** | **str** |  |
**start_time** | **datetime** |  |
**status** | [**SpanStatus**](SpanStatus.md) |  |
**tenant_id** | **str** |  |
**tokens** | [**TokenCounts**](TokenCounts.md) |  | [optional]
**trace_id** | **str** |  |
**unmapped_attrs** | **object** |  |

## Example

```python
from beater_client.models.canonical_span import CanonicalSpan

# TODO update the JSON string below
json = "{}"
# create an instance of CanonicalSpan from a JSON string
canonical_span_instance = CanonicalSpan.from_json(json)
# print the JSON string representation of the object
print(CanonicalSpan.to_json())

# convert the object into a dict
canonical_span_dict = canonical_span_instance.to_dict()
# create an instance of CanonicalSpan from a dict
canonical_span_from_dict = CanonicalSpan.from_dict(canonical_span_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
