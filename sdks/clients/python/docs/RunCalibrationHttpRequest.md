# RunCalibrationHttpRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**eval_report_id** | **str** |  | [optional]
**evaluator_version_id** | **str** |  | [optional]
**pass_threshold** | **float** |  | [optional]

## Example

```python
from beater_client.models.run_calibration_http_request import RunCalibrationHttpRequest

# TODO update the JSON string below
json = "{}"
# create an instance of RunCalibrationHttpRequest from a JSON string
run_calibration_http_request_instance = RunCalibrationHttpRequest.from_json(json)
# print the JSON string representation of the object
print(RunCalibrationHttpRequest.to_json())

# convert the object into a dict
run_calibration_http_request_dict = run_calibration_http_request_instance.to_dict()
# create an instance of RunCalibrationHttpRequest from a dict
run_calibration_http_request_from_dict = RunCalibrationHttpRequest.from_dict(run_calibration_http_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
