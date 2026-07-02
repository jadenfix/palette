# CalibrationReport

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**brier_score** | **f64** |  | 
**calibration_report_id** | **String** |  | 
**cohen_kappa** | **f64** |  | 
**cohen_kappa_ci_high** | Option<**f64**> |  | [optional]
**cohen_kappa_ci_low** | Option<**f64**> | Percentile-bootstrap 95% confidence interval for `cohen_kappa` (multinomial resampling of the confusion table, deterministic seed). Kappa over small calibration samples is high-variance; a bare point estimate invites over-reading. Absent on pre-uncertainty reports. | [optional]
**confusion** | [**models::CalibrationConfusion**](CalibrationConfusion.md) |  | 
**created_at** | **String** |  | 
**dataset_id** | **String** |  | 
**dataset_version_id** | **String** |  | 
**eval_report_id** | **String** |  | 
**evaluator_version_id** | **String** |  | 
**expected_agreement** | **f64** |  | 
**expected_calibration_error** | **f64** |  | 
**items** | [**Vec<models::CalibrationItem>**](CalibrationItem.md) |  | 
**observed_agreement** | **f64** |  | 
**observed_agreement_ci_high** | Option<**f64**> |  | [optional]
**observed_agreement_ci_low** | Option<**f64**> | Wilson 95% confidence interval for `observed_agreement` — the honest width of an agreement estimate over a (typically small) human-labelled sample. Absent on reports persisted before uncertainty was reported. | [optional]
**policy** | [**models::CalibrationPolicy**](CalibrationPolicy.md) |  | 
**project_id** | **String** |  | 
**reliability_bins** | [**Vec<models::ReliabilityBin>**](ReliabilityBin.md) |  | 
**sample_count** | **i32** |  | 
**tenant_id** | **String** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


