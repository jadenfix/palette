# CalibrationPolicy


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**pass_threshold** | **float** |  |

## Example

```python
from beater_client.models.calibration_policy import CalibrationPolicy

# TODO update the JSON string below
json = "{}"
# create an instance of CalibrationPolicy from a JSON string
calibration_policy_instance = CalibrationPolicy.from_json(json)
# print the JSON string representation of the object
print(CalibrationPolicy.to_json())

# convert the object into a dict
calibration_policy_dict = calibration_policy_instance.to_dict()
# create an instance of CalibrationPolicy from a dict
calibration_policy_from_dict = CalibrationPolicy.from_dict(calibration_policy_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
