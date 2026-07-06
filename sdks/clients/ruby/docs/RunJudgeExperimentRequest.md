# BeaterClient::RunJudgeExperimentRequest

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **baseline_outputs** | [**Array&lt;CaseOutputOverrideRequest&gt;**](CaseOutputOverrideRequest.md) |  |  |
| **baseline_release_id** | **String** |  |  |
| **candidate_outputs** | [**Array&lt;CaseOutputOverrideRequest&gt;**](CaseOutputOverrideRequest.md) |  |  |
| **candidate_release_id** | **String** |  |  |
| **evaluator_id** | **String** |  |  |
| **evaluator_version_id** | **String** |  |  |
| **gate_policy** | [**GatePolicy**](GatePolicy.md) |  | [optional] |
| **kind** | [**EvaluatorKind**](EvaluatorKind.md) |  |  |
| **provider_secret_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::RunJudgeExperimentRequest.new(
  baseline_outputs: null,
  baseline_release_id: null,
  candidate_outputs: null,
  candidate_release_id: null,
  evaluator_id: null,
  evaluator_version_id: null,
  gate_policy: null,
  kind: null,
  provider_secret_id: null
)
```
