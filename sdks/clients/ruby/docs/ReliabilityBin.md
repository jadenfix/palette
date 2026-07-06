# BeaterClient::ReliabilityBin

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **accuracy** | **Float** |  | [optional] |
| **bin_index** | **Integer** |  |  |
| **calibration_gap** | **Float** |  | [optional] |
| **lower_bound** | **Float** |  |  |
| **mean_confidence** | **Float** |  | [optional] |
| **sample_count** | **Integer** |  |  |
| **upper_bound** | **Float** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::ReliabilityBin.new(
  accuracy: null,
  bin_index: null,
  calibration_gap: null,
  lower_bound: null,
  mean_confidence: null,
  sample_count: null,
  upper_bound: null
)
```
