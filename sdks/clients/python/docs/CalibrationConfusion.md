# CalibrationConfusion


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**human_fail_judge_fail** | **int** |  |
**human_fail_judge_pass** | **int** |  |
**human_pass_judge_fail** | **int** |  |
**human_pass_judge_pass** | **int** |  |

## Example

```python
from beater_client.models.calibration_confusion import CalibrationConfusion

# TODO update the JSON string below
json = "{}"
# create an instance of CalibrationConfusion from a JSON string
calibration_confusion_instance = CalibrationConfusion.from_json(json)
# print the JSON string representation of the object
print(CalibrationConfusion.to_json())

# convert the object into a dict
calibration_confusion_dict = calibration_confusion_instance.to_dict()
# create an instance of CalibrationConfusion from a dict
calibration_confusion_from_dict = CalibrationConfusion.from_dict(calibration_confusion_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
