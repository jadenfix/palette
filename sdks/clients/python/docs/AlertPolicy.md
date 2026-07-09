# AlertPolicy


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**dedupe_window_seconds** | **int** |  |
**endpoint_url** | **str** |  |
**fire_when_score_at_or_below** | **float** |  |
**maintenance_windows** | [**List[MaintenanceWindow]**](MaintenanceWindow.md) |  |
**policy_id** | **str** |  |
**severity** | [**AlertSeverity**](AlertSeverity.md) |  |
**signing_secret** | **str** |  |

## Example

```python
from beater_client.models.alert_policy import AlertPolicy

# TODO update the JSON string below
json = "{}"
# create an instance of AlertPolicy from a JSON string
alert_policy_instance = AlertPolicy.from_json(json)
# print the JSON string representation of the object
print(AlertPolicy.to_json())

# convert the object into a dict
alert_policy_dict = alert_policy_instance.to_dict()
# create an instance of AlertPolicy from a dict
alert_policy_from_dict = AlertPolicy.from_dict(alert_policy_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
