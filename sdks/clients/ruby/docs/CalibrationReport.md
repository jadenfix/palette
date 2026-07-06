# BeaterClient::CalibrationReport

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **brier_score** | **Float** |  |  |
| **calibration_report_id** | **String** |  |  |
| **cohen_kappa** | **Float** |  |  |
| **cohen_kappa_ci_high** | **Float** |  | [optional] |
| **cohen_kappa_ci_low** | **Float** | Percentile-bootstrap 95% confidence interval for &#x60;cohen_kappa&#x60; (multinomial resampling of the confusion table, deterministic seed). Kappa over small calibration samples is high-variance; a bare point estimate invites over-reading. Absent on pre-uncertainty reports. | [optional] |
| **confusion** | [**CalibrationConfusion**](CalibrationConfusion.md) |  |  |
| **created_at** | **Time** |  |  |
| **dataset_id** | **String** |  |  |
| **dataset_version_id** | **String** |  |  |
| **eval_report_id** | **String** |  |  |
| **evaluator_version_id** | **String** |  |  |
| **expected_agreement** | **Float** |  |  |
| **expected_calibration_error** | **Float** |  |  |
| **items** | [**Array&lt;CalibrationItem&gt;**](CalibrationItem.md) |  |  |
| **observed_agreement** | **Float** |  |  |
| **observed_agreement_ci_high** | **Float** |  | [optional] |
| **observed_agreement_ci_low** | **Float** | Wilson 95% confidence interval for &#x60;observed_agreement&#x60; — the honest width of an agreement estimate over a (typically small) human-labelled sample. Absent on reports persisted before uncertainty was reported. | [optional] |
| **policy** | [**CalibrationPolicy**](CalibrationPolicy.md) |  |  |
| **project_id** | **String** |  |  |
| **reliability_bins** | [**Array&lt;ReliabilityBin&gt;**](ReliabilityBin.md) |  |  |
| **sample_count** | **Integer** |  |  |
| **tenant_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::CalibrationReport.new(
  brier_score: null,
  calibration_report_id: null,
  cohen_kappa: null,
  cohen_kappa_ci_high: null,
  cohen_kappa_ci_low: null,
  confusion: null,
  created_at: null,
  dataset_id: null,
  dataset_version_id: null,
  eval_report_id: null,
  evaluator_version_id: null,
  expected_agreement: null,
  expected_calibration_error: null,
  items: null,
  observed_agreement: null,
  observed_agreement_ci_high: null,
  observed_agreement_ci_low: null,
  policy: null,
  project_id: null,
  reliability_bins: null,
  sample_count: null,
  tenant_id: null
)
```
