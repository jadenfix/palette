# BeaterClient::JudgeAuditRecord

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **cached** | **Boolean** |  |  |
| **charged_cost** | [**Money**](Money.md) |  |  |
| **created_at** | **Time** |  |  |
| **evaluator_id** | **String** |  |  |
| **judge_call_id** | **String** |  |  |
| **model** | **String** |  |  |
| **project_id** | **String** |  |  |
| **provider** | **String** |  |  |
| **provider_cost** | [**Money**](Money.md) |  |  |
| **provider_secret_id** | **String** |  |  |
| **request_hash** | **String** |  |  |
| **response_hash** | **String** |  |  |
| **score** | **Float** |  |  |
| **tenant_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::JudgeAuditRecord.new(
  cached: null,
  charged_cost: null,
  created_at: null,
  evaluator_id: null,
  judge_call_id: null,
  model: null,
  project_id: null,
  provider: null,
  provider_cost: null,
  provider_secret_id: null,
  request_hash: null,
  response_hash: null,
  score: null,
  tenant_id: null
)
```
