# AlertInput


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**baseline_score** | **float** |  | [optional]
**group_key** | **str** |  |
**links** | [**AlertLinks**](AlertLinks.md) |  |
**now** | **datetime** |  |
**project_id** | **str** |  |
**score** | **float** |  |
**tenant_id** | **str** |  |
**title** | **str** |  |
**trace_id** | **str** |  |

## Example

```python
from beater_client.models.alert_input import AlertInput

# TODO update the JSON string below
json = "{}"
# create an instance of AlertInput from a JSON string
alert_input_instance = AlertInput.from_json(json)
# print the JSON string representation of the object
print(AlertInput.to_json())

# convert the object into a dict
alert_input_dict = alert_input_instance.to_dict()
# create an instance of AlertInput from a dict
alert_input_from_dict = AlertInput.from_dict(alert_input_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
