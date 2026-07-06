# BeaterClient::IngestOutcome

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **ack** | [**WriteAck**](WriteAck.md) |  |  |
| **downstream_queued** | **Boolean** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::IngestOutcome.new(
  ack: null,
  downstream_queued: null
)
```
