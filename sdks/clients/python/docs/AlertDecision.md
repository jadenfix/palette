# AlertDecision


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**delivery** | [**WebhookDelivery**](WebhookDelivery.md) |  | [optional]
**emitted** | **bool** |  |
**suppressed_reason** | **str** |  | [optional]

## Example

```python
from beater_client.models.alert_decision import AlertDecision

# TODO update the JSON string below
json = "{}"
# create an instance of AlertDecision from a JSON string
alert_decision_instance = AlertDecision.from_json(json)
# print the JSON string representation of the object
print(AlertDecision.to_json())

# convert the object into a dict
alert_decision_dict = alert_decision_instance.to_dict()
# create an instance of AlertDecision from a dict
alert_decision_from_dict = AlertDecision.from_dict(alert_decision_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
