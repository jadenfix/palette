# BeaterClient::WriteAck

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **accepted_raw** | **Integer** |  |  |
| **accepted_spans** | **Integer** |  |  |
| **duplicate_raw** | **Integer** |  |  |
| **duplicate_spans** | **Integer** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::WriteAck.new(
  accepted_raw: null,
  accepted_spans: null,
  duplicate_raw: null,
  duplicate_spans: null
)
```
