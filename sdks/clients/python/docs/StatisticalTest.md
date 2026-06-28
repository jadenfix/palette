# StatisticalTest

The statistical test that produced an [`ExperimentComparison`]. These mirror `beater_stats::TestKind`; the gate records which method was actually used so a reader can tell a t-test result from an exact McNemar one. The old single `PairedNormalApproximation` (a hard-coded-z normal approximation with no p-value) is gone — see `beater-stats`.

## Enum

* `PAIRED_T` (value: `'paired_t'`)

* `MCNEMAR_EXACT` (value: `'mcnemar_exact'`)

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


