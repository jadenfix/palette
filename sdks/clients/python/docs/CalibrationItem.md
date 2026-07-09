# CalibrationItem


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**agreed** | **bool** |  |
**dataset_case_id** | **str** |  |
**evidence** | **object** |  |
**human_label** | [**CalibrationLabel**](CalibrationLabel.md) |  |
**judge_label** | [**CalibrationLabel**](CalibrationLabel.md) |  |
**judge_result_label** | **str** |  | [optional]
**judge_score** | **float** |  |

## Example

```python
from beater_client.models.calibration_item import CalibrationItem

# TODO update the JSON string below
json = "{}"
# create an instance of CalibrationItem from a JSON string
calibration_item_instance = CalibrationItem.from_json(json)
# print the JSON string representation of the object
print(CalibrationItem.to_json())

# convert the object into a dict
calibration_item_dict = calibration_item_instance.to_dict()
# create an instance of CalibrationItem from a dict
calibration_item_from_dict = CalibrationItem.from_dict(calibration_item_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
