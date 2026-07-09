# EvaluateAlertRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**input** | [**AlertInput**](AlertInput.md) |  |
**policy** | [**AlertPolicy**](AlertPolicy.md) |  |

## Example

```python
from beater_client.models.evaluate_alert_request import EvaluateAlertRequest

# TODO update the JSON string below
json = "{}"
# create an instance of EvaluateAlertRequest from a JSON string
evaluate_alert_request_instance = EvaluateAlertRequest.from_json(json)
# print the JSON string representation of the object
print(EvaluateAlertRequest.to_json())

# convert the object into a dict
evaluate_alert_request_dict = evaluate_alert_request_instance.to_dict()
# create an instance of EvaluateAlertRequest from a dict
evaluate_alert_request_from_dict = EvaluateAlertRequest.from_dict(evaluate_alert_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
