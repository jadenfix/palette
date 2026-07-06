# BeaterClient::EvalResult

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **cost** | [**Money**](Money.md) |  | [optional] |
| **created_at** | **Time** |  |  |
| **eval_result_id** | **String** |  |  |
| **evidence** | **Object** |  |  |
| **label** | **String** |  | [optional] |
| **non_reproducible_reason** | **String** |  | [optional] |
| **project_id** | **String** |  |  |
| **reproducibility** | [**EvalReproducibility**](EvalReproducibility.md) |  |  |
| **score** | **Float** |  |  |
| **span_id** | **String** |  | [optional] |
| **tenant_id** | **String** |  |  |
| **tokens** | [**TokenCounts**](TokenCounts.md) |  | [optional] |
| **trace_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::EvalResult.new(
  cost: null,
  created_at: null,
  eval_result_id: null,
  evidence: null,
  label: null,
  non_reproducible_reason: null,
  project_id: null,
  reproducibility: null,
  score: null,
  span_id: null,
  tenant_id: null,
  tokens: null,
  trace_id: null
)
```
