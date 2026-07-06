# BeaterClient::DeadLetter

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **failed_at** | **Time** |  |  |
| **message** | [**BusMessage**](BusMessage.md) |  |  |
| **reason** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::DeadLetter.new(
  failed_at: null,
  message: null,
  reason: null
)
```
