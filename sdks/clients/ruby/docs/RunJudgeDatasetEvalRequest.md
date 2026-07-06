# BeaterClient::RunJudgeDatasetEvalRequest

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **agent_release_id** | **String** |  |  |
| **code_hash** | **String** |  | [optional] |
| **evaluator_id** | **String** |  |  |
| **evaluator_version_id** | **String** |  |  |
| **kind** | [**EvaluatorKind**](EvaluatorKind.md) |  |  |
| **prompt_version_id** | **String** |  | [optional] |
| **provider_secret_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::RunJudgeDatasetEvalRequest.new(
  agent_release_id: null,
  code_hash: null,
  evaluator_id: null,
  evaluator_version_id: null,
  kind: null,
  prompt_version_id: null,
  provider_secret_id: null
)
```
