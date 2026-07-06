# BeaterClient::SamplingDecision

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **reason** | [**SamplingReason**](SamplingReason.md) |  |  |
| **selected** | **Boolean** |  |  |
| **stable_score_per_mille** | **Integer** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::SamplingDecision.new(
  reason: null,
  selected: null,
  stable_score_per_mille: null
)
```
