# SpanIoValue


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**kind** | **str** |  |
**value** | **object** |  |
**artifact_ref** | [**ArtifactRef**](ArtifactRef.md) |  |
**reason** | **str** |  |

## Example

```python
from beater_client.models.span_io_value import SpanIoValue

# TODO update the JSON string below
json = "{}"
# create an instance of SpanIoValue from a JSON string
span_io_value_instance = SpanIoValue.from_json(json)
# print the JSON string representation of the object
print(SpanIoValue.to_json())

# convert the object into a dict
span_io_value_dict = span_io_value_instance.to_dict()
# create an instance of SpanIoValue from a dict
span_io_value_from_dict = SpanIoValue.from_dict(span_io_value_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
