# BeaterClient::GatePolicy

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **alpha** | **Float** |  |  |
| **comparison_count** | **Integer** |  |  |
| **max_regression** | **Float** |  |  |
| **min_sample_size** | **Integer** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::GatePolicy.new(
  alpha: null,
  comparison_count: null,
  max_regression: null,
  min_sample_size: null
)
```
