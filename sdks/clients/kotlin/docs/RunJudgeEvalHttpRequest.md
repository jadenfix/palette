
# RunJudgeEvalHttpRequest

## Properties
| Name | Type | Description | Notes |
| ------------ | ------------- | ------------- | ------------- |
| **case** | [**EvaluationCase**](EvaluationCase.md) |  |  |
| **evaluator** | [**EvaluatorSpec**](EvaluatorSpec.md) |  |  |
| **providerSecretId** | **kotlin.String** |  |  |
| **cacheNamespace** | **kotlin.String** | Calibration-map / judge-instrument version folded into the judge cache key; bumping it on recalibration invalidates stale cached scores. |  [optional] |
