# Beater.Client.Model.CalibrationReport

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**BrierScore** | **double** |  |
**CalibrationReportId** | **string** |  |
**CohenKappa** | **double** |  |
**CohenKappaCiHigh** | **double?** |  | [optional]
**CohenKappaCiLow** | **double?** | Percentile-bootstrap 95% confidence interval for &#x60;cohen_kappa&#x60; (multinomial resampling of the confusion table, deterministic seed). Kappa over small calibration samples is high-variance; a bare point estimate invites over-reading. Absent on pre-uncertainty reports. | [optional]
**Confusion** | [**CalibrationConfusion**](CalibrationConfusion.md) |  |
**CreatedAt** | **DateTime** |  |
**DatasetId** | **string** |  |
**DatasetVersionId** | **string** |  |
**EvalReportId** | **string** |  |
**EvaluatorVersionId** | **string** |  |
**ExpectedAgreement** | **double** |  |
**ExpectedCalibrationError** | **double** |  |
**Items** | [**List&lt;CalibrationItem&gt;**](CalibrationItem.md) |  |
**ObservedAgreement** | **double** |  |
**ObservedAgreementCiHigh** | **double?** |  | [optional]
**ObservedAgreementCiLow** | **double?** | Wilson 95% confidence interval for &#x60;observed_agreement&#x60; — the honest width of an agreement estimate over a (typically small) human-labelled sample. Absent on reports persisted before uncertainty was reported. | [optional]
**Policy** | [**CalibrationPolicy**](CalibrationPolicy.md) |  |
**ProjectId** | **string** |  |
**ReliabilityBins** | [**List&lt;ReliabilityBin&gt;**](ReliabilityBin.md) |  |
**SampleCount** | **int** |  |
**TenantId** | **string** |  |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
