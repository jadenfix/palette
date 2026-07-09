# TraceView


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**spans** | [**List[CanonicalSpan]**](CanonicalSpan.md) |  |
**tenant_id** | **str** |  |
**trace_id** | **str** |  |

## Example

```python
from beater_client.models.trace_view import TraceView

# TODO update the JSON string below
json = "{}"
# create an instance of TraceView from a JSON string
trace_view_instance = TraceView.from_json(json)
# print the JSON string representation of the object
print(TraceView.to_json())

# convert the object into a dict
trace_view_dict = trace_view_instance.to_dict()
# create an instance of TraceView from a dict
trace_view_from_dict = TraceView.from_dict(trace_view_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
