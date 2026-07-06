# BeaterClient::DeadLetterReplayReport

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **ack** | [**PublishAck**](PublishAck.md) |  |  |
| **message_id** | **String** |  |  |
| **project_id** | **String** |  |  |
| **reset_attempts** | **Boolean** |  |  |
| **tenant_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::DeadLetterReplayReport.new(
  ack: null,
  message_id: null,
  project_id: null,
  reset_attempts: null,
  tenant_id: null
)
```
