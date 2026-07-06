# BeaterClient::AlertDecision

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **delivery** | [**WebhookDelivery**](WebhookDelivery.md) |  | [optional] |
| **emitted** | **Boolean** |  |  |
| **suppressed_reason** | **String** |  | [optional] |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::AlertDecision.new(
  delivery: null,
  emitted: null,
  suppressed_reason: null
)
```
