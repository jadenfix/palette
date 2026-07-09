# PaletteConnectStatusResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**first_eval_run** | **bool** |  |
**first_trace_received** | **bool** |  |
**ok** | **bool** |  |
**project_id** | **str** |  |
**status** | [**PaletteConnectStatus**](PaletteConnectStatus.md) |  |
**tenant_id** | **str** |  |
**totals** | [**Dict[str, UsageTotal]**](UsageTotal.md) |  |
**usage_configured** | **bool** |  |

## Example

```python
from beater_client.models.palette_connect_status_response import PaletteConnectStatusResponse

# TODO update the JSON string below
json = "{}"
# create an instance of PaletteConnectStatusResponse from a JSON string
palette_connect_status_response_instance = PaletteConnectStatusResponse.from_json(json)
# print the JSON string representation of the object
print(PaletteConnectStatusResponse.to_json())

# convert the object into a dict
palette_connect_status_response_dict = palette_connect_status_response_instance.to_dict()
# create an instance of PaletteConnectStatusResponse from a dict
palette_connect_status_response_from_dict = PaletteConnectStatusResponse.from_dict(palette_connect_status_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
