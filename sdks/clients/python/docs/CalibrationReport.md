# CalibrationReport


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**brier_score** | **float** |  | 
**calibration_report_id** | **str** |  | 
**cohen_kappa** | **float** |  | 
**cohen_kappa_ci_high** | **float** |  | [optional] 
**cohen_kappa_ci_low** | **float** | Percentile-bootstrap 95% confidence interval for &#x60;cohen_kappa&#x60; (multinomial resampling of the confusion table, deterministic seed). Kappa over small calibration samples is high-variance; a bare point estimate invites over-reading. Absent on pre-uncertainty reports. | [optional] 
**confusion** | [**CalibrationConfusion**](CalibrationConfusion.md) |  | 
**created_at** | **datetime** |  | 
**dataset_id** | **str** |  | 
**dataset_version_id** | **str** |  | 
**eval_report_id** | **str** |  | 
**evaluator_version_id** | **str** |  | 
**expected_agreement** | **float** |  | 
**expected_calibration_error** | **float** |  | 
**items** | [**List[CalibrationItem]**](CalibrationItem.md) |  | 
**observed_agreement** | **float** |  | 
**observed_agreement_ci_high** | **float** |  | [optional] 
**observed_agreement_ci_low** | **float** | Wilson 95% confidence interval for &#x60;observed_agreement&#x60; — the honest width of an agreement estimate over a (typically small) human-labelled sample. Absent on reports persisted before uncertainty was reported. | [optional] 
**policy** | [**CalibrationPolicy**](CalibrationPolicy.md) |  | 
**project_id** | **str** |  | 
**reliability_bins** | [**List[ReliabilityBin]**](ReliabilityBin.md) |  | 
**sample_count** | **int** |  | 
**tenant_id** | **str** |  | 

## Example

```python
from beater_client.models.calibration_report import CalibrationReport

# TODO update the JSON string below
json = "{}"
# create an instance of CalibrationReport from a JSON string
calibration_report_instance = CalibrationReport.from_json(json)
# print the JSON string representation of the object
print(CalibrationReport.to_json())

# convert the object into a dict
calibration_report_dict = calibration_report_instance.to_dict()
# create an instance of CalibrationReport from a dict
calibration_report_from_dict = CalibrationReport.from_dict(calibration_report_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


