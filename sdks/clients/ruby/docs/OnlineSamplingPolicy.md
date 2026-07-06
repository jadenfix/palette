# BeaterClient::OnlineSamplingPolicy

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **high_cost_micros_threshold** | **Integer** |  | [optional] |
| **keep_errors** | **Boolean** |  |  |
| **sample_rate_per_mille** | **Integer** |  |  |
| **slow_ms_threshold** | **Integer** |  | [optional] |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::OnlineSamplingPolicy.new(
  high_cost_micros_threshold: null,
  keep_errors: null,
  sample_rate_per_mille: null,
  slow_ms_threshold: null
)
```
