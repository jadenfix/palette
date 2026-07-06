# BeaterClient::CalibrationConfusion

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **human_fail_judge_fail** | **Integer** |  |  |
| **human_fail_judge_pass** | **Integer** |  |  |
| **human_pass_judge_fail** | **Integer** |  |  |
| **human_pass_judge_pass** | **Integer** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::CalibrationConfusion.new(
  human_fail_judge_fail: null,
  human_fail_judge_pass: null,
  human_pass_judge_fail: null,
  human_pass_judge_pass: null
)
```
