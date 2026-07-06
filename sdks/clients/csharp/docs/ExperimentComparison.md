# Beater.Client.Model.ExperimentComparison

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AdjustedAlpha** | **double** |  |
**BaselineMean** | **double** |  |
**CandidateMean** | **double** |  |
**CiHigh** | **double** |  |
**CiLow** | **double** |  |
**Decision** | **GateDecision** |  |
**Delta** | **double** |  |
**Mde** | **double?** | Minimum detectable effect at the current sample size, in the metric&#39;s own units, at the gate&#39;s (adjusted) alpha and the standard power of 0.8 (§10.3 #5). Populated only when &#x60;decision&#x60; is &#x60;Inconclusive&#x60; — the comparison lacked the power to resolve the regression bound, and regressions smaller than this are invisible at this N. &#x60;None&#x60; on a conclusive decision (or when the paired differences have zero spread, so no effect-scale is defined). This replaces a bare \&quot;underpowered\&quot; flag with the actionable \&quot;how small an effect could we even have seen\&quot; number. | [optional]
**PValue** | **double** | Real two-sided p-value from &#x60;test&#x60;. The previous normal-approximation path reported no p-value at all. |
**RequiredN** | **int?** | Number of paired observations that would be required to detect the *observed* effect at the gate&#39;s (adjusted) alpha and power 0.8 (§10.3 #5). Populated only when &#x60;decision&#x60; is &#x60;Inconclusive&#x60; and the observed effect is non-degenerate (non-zero delta over non-zero difference spread). &#x60;None&#x60; otherwise. This answers \&quot;how many more cases would have made this conclusive?\&quot;. | [optional]
**SampleSize** | **int** |  |
**Test** | **StatisticalTest** |  |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
