
# CalibrationReport

## Properties
| Name | Type | Description | Notes |
| ------------ | ------------- | ------------- | ------------- |
| **brierScore** | **kotlin.Double** |  |  |
| **calibrationReportId** | **kotlin.String** |  |  |
| **cohenKappa** | **kotlin.Double** |  |  |
| **confusion** | [**CalibrationConfusion**](CalibrationConfusion.md) |  |  |
| **createdAt** | [**java.time.OffsetDateTime**](java.time.OffsetDateTime.md) |  |  |
| **datasetId** | **kotlin.String** |  |  |
| **datasetVersionId** | **kotlin.String** |  |  |
| **evalReportId** | **kotlin.String** |  |  |
| **evaluatorVersionId** | **kotlin.String** |  |  |
| **expectedAgreement** | **kotlin.Double** |  |  |
| **expectedCalibrationError** | **kotlin.Double** |  |  |
| **items** | [**kotlin.collections.List&lt;CalibrationItem&gt;**](CalibrationItem.md) |  |  |
| **observedAgreement** | **kotlin.Double** |  |  |
| **policy** | [**CalibrationPolicy**](CalibrationPolicy.md) |  |  |
| **projectId** | **kotlin.String** |  |  |
| **reliabilityBins** | [**kotlin.collections.List&lt;ReliabilityBin&gt;**](ReliabilityBin.md) |  |  |
| **sampleCount** | **kotlin.Int** |  |  |
| **tenantId** | **kotlin.String** |  |  |
| **cohenKappaCiHigh** | **kotlin.Double** |  |  [optional] |
| **cohenKappaCiLow** | **kotlin.Double** | Percentile-bootstrap 95% confidence interval for &#x60;cohen_kappa&#x60; (multinomial resampling of the confusion table, deterministic seed). Kappa over small calibration samples is high-variance; a bare point estimate invites over-reading. Absent on pre-uncertainty reports. |  [optional] |
| **observedAgreementCiHigh** | **kotlin.Double** |  |  [optional] |
| **observedAgreementCiLow** | **kotlin.Double** | Wilson 95% confidence interval for &#x60;observed_agreement&#x60; — the honest width of an agreement estimate over a (typically small) human-labelled sample. Absent on reports persisted before uncertainty was reported. |  [optional] |
