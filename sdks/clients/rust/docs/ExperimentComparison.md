# ExperimentComparison

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**adjusted_alpha** | **f64** |  |
**baseline_mean** | **f64** |  |
**candidate_mean** | **f64** |  |
**ci_high** | **f64** |  |
**ci_low** | **f64** |  |
**decision** | [**models::GateDecision**](GateDecision.md) |  |
**delta** | **f64** |  |
**mde** | Option<**f64**> | Minimum detectable effect at the current sample size, in the metric's own units, at the gate's (adjusted) alpha and the standard power of 0.8 (§10.3 #5). Populated only when `decision` is `Inconclusive` — the comparison lacked the power to resolve the regression bound, and regressions smaller than this are invisible at this N. `None` on a conclusive decision (or when the paired differences have zero spread, so no effect-scale is defined). This replaces a bare \"underpowered\" flag with the actionable \"how small an effect could we even have seen\" number. | [optional]
**p_value** | **f64** | Real two-sided p-value from `test`. The previous normal-approximation path reported no p-value at all. |
**required_n** | Option<**i32**> | Number of paired observations that would be required to detect the *observed* effect at the gate's (adjusted) alpha and power 0.8 (§10.3 #5). Populated only when `decision` is `Inconclusive` and the observed effect is non-degenerate (non-zero delta over non-zero difference spread). `None` otherwise. This answers \"how many more cases would have made this conclusive?\". | [optional]
**sample_size** | **i32** |  |
**test** | [**models::StatisticalTest**](StatisticalTest.md) |  |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
