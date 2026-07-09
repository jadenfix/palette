# SpanIoResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**input** | [**SpanIoValue**](SpanIoValue.md) |  |
**output** | [**SpanIoValue**](SpanIoValue.md) |  |
**span_id** | **str** |  |
**tenant_id** | **str** |  |
**trace_id** | **str** |  |

## Example

```python
from beater_client.models.span_io_response import SpanIoResponse

# TODO update the JSON string below
json = "{}"
# create an instance of SpanIoResponse from a JSON string
span_io_response_instance = SpanIoResponse.from_json(json)
# print the JSON string representation of the object
print(SpanIoResponse.to_json())

# convert the object into a dict
span_io_response_dict = span_io_response_instance.to_dict()
# create an instance of SpanIoResponse from a dict
span_io_response_from_dict = SpanIoResponse.from_dict(span_io_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
