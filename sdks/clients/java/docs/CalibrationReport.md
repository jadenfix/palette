

# CalibrationReport


## Properties

| Name | Type | Description | Notes |
|------------ | ------------- | ------------- | -------------|
|**brierScore** | **Double** |  |  |
|**calibrationReportId** | **String** |  |  |
|**cohenKappa** | **Double** |  |  |
|**cohenKappaCiHigh** | **Double** |  |  [optional] |
|**cohenKappaCiLow** | **Double** | Percentile-bootstrap 95% confidence interval for &#x60;cohen_kappa&#x60; (multinomial resampling of the confusion table, deterministic seed). Kappa over small calibration samples is high-variance; a bare point estimate invites over-reading. Absent on pre-uncertainty reports. |  [optional] |
|**confusion** | [**CalibrationConfusion**](CalibrationConfusion.md) |  |  |
|**createdAt** | **OffsetDateTime** |  |  |
|**datasetId** | **String** |  |  |
|**datasetVersionId** | **String** |  |  |
|**evalReportId** | **String** |  |  |
|**evaluatorVersionId** | **String** |  |  |
|**expectedAgreement** | **Double** |  |  |
|**expectedCalibrationError** | **Double** |  |  |
|**items** | [**List&lt;CalibrationItem&gt;**](CalibrationItem.md) |  |  |
|**observedAgreement** | **Double** |  |  |
|**observedAgreementCiHigh** | **Double** |  |  [optional] |
|**observedAgreementCiLow** | **Double** | Wilson 95% confidence interval for &#x60;observed_agreement&#x60; — the honest width of an agreement estimate over a (typically small) human-labelled sample. Absent on reports persisted before uncertainty was reported. |  [optional] |
|**policy** | [**CalibrationPolicy**](CalibrationPolicy.md) |  |  |
|**projectId** | **String** |  |  |
|**reliabilityBins** | [**List&lt;ReliabilityBin&gt;**](ReliabilityBin.md) |  |  |
|**sampleCount** | **Integer** |  |  |
|**tenantId** | **String** |  |  |
