# BeaterClient::BusMessage

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **attempts** | **Integer** |  |  |
| **enqueued_at** | **Time** |  |  |
| **idempotency_key** | **String** |  |  |
| **kind** | **String** |  |  |
| **max_attempts** | **Integer** |  |  |
| **message_id** | **String** |  |  |
| **payload** | **Array&lt;Integer&gt;** |  |  |
| **project_id** | **String** |  |  |
| **tenant_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::BusMessage.new(
  attempts: null,
  enqueued_at: null,
  idempotency_key: null,
  kind: null,
  max_attempts: null,
  message_id: null,
  payload: null,
  project_id: null,
  tenant_id: null
)
```
