# Beater.Client.Model.StatisticalTest
The statistical test that produced an [`ExperimentComparison`]. The gate records which method was **actually executed** so a reader can tell a t-test result from an exact McNemar, Wilcoxon, bootstrap, cluster-robust, or anytime-valid sequential one. The old single `PairedNormalApproximation` (a hard-coded-z normal approximation with no p-value) is gone — see `beater-stats`.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
