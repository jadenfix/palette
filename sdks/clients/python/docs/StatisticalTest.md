# StatisticalTest

The statistical test that produced an [`ExperimentComparison`]. The gate records which method was **actually executed** so a reader can tell a t-test result from an exact McNemar, Wilcoxon, bootstrap, cluster-robust, or anytime-valid sequential one. The old single `PairedNormalApproximation` (a hard-coded-z normal approximation with no p-value) is gone — see `beater-stats`.

## Enum

* `PAIRED_T` (value: `'paired_t'`)

* `MCNEMAR_EXACT` (value: `'mcnemar_exact'`)

* `WILCOXON_SIGNED_RANK` (value: `'wilcoxon_signed_rank'`)

* `PAIRED_BOOTSTRAP` (value: `'paired_bootstrap'`)

* `CLUSTERED_PAIRED_T` (value: `'clustered_paired_t'`)

* `SEQUENTIAL_E_VALUE` (value: `'sequential_e_value'`)

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
