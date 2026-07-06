# BeaterClient::JudgeBrokerOutcome

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **audit** | [**JudgeAuditRecord**](JudgeAuditRecord.md) |  |  |
| **remaining_budget** | [**Money**](Money.md) |  |  |
| **result** | [**ScoreResult**](ScoreResult.md) |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::JudgeBrokerOutcome.new(
  audit: null,
  remaining_budget: null,
  result: null
)
```
