# BeaterClient::IngestQueueStatus

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **dead_letters** | [**Array&lt;DeadLetter&gt;**](DeadLetter.md) |  |  |
| **project_id** | **String** |  |  |
| **tenant_id** | **String** |  |  |
| **total_depth** | **Integer** |  |  |
| **trace_ingested_depth** | **Integer** |  |  |
| **trace_write_depth** | **Integer** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::IngestQueueStatus.new(
  dead_letters: null,
  project_id: null,
  tenant_id: null,
  total_depth: null,
  trace_ingested_depth: null,
  trace_write_depth: null
)
```
