# calibration_report_t

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**brier_score** | **double** |  |
**calibration_report_id** | **char \*** |  |
**cohen_kappa** | **double** |  |
**cohen_kappa_ci_high** | **double** |  | [optional]
**cohen_kappa_ci_low** | **double** | Percentile-bootstrap 95% confidence interval for &#x60;cohen_kappa&#x60; (multinomial resampling of the confusion table, deterministic seed). Kappa over small calibration samples is high-variance; a bare point estimate invites over-reading. Absent on pre-uncertainty reports. | [optional]
**confusion** | [**calibration_confusion_t**](calibration_confusion.md) \* |  |
**created_at** | **char \*** |  |
**dataset_id** | **char \*** |  |
**dataset_version_id** | **char \*** |  |
**eval_report_id** | **char \*** |  |
**evaluator_version_id** | **char \*** |  |
**expected_agreement** | **double** |  |
**expected_calibration_error** | **double** |  |
**items** | [**list_t**](calibration_item.md) \* |  |
**observed_agreement** | **double** |  |
**observed_agreement_ci_high** | **double** |  | [optional]
**observed_agreement_ci_low** | **double** | Wilson 95% confidence interval for &#x60;observed_agreement&#x60; — the honest width of an agreement estimate over a (typically small) human-labelled sample. Absent on reports persisted before uncertainty was reported. | [optional]
**policy** | [**calibration_policy_t**](calibration_policy.md) \* |  |
**project_id** | **char \*** |  |
**reliability_bins** | [**list_t**](reliability_bin.md) \* |  |
**sample_count** | **int** |  |
**tenant_id** | **char \*** |  |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
