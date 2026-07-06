# BeaterClient::RunJudgeEvalHttpRequest

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **cache_namespace** | **String** | Calibration-map / judge-instrument version folded into the judge cache key; bumping it on recalibration invalidates stale cached scores. | [optional] |
| **_case** | [**EvaluationCase**](EvaluationCase.md) |  |  |
| **evaluator** | [**EvaluatorSpec**](EvaluatorSpec.md) |  |  |
| **provider_secret_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::RunJudgeEvalHttpRequest.new(
  cache_namespace: null,
  _case: null,
  evaluator: null,
  provider_secret_id: null
)
```
