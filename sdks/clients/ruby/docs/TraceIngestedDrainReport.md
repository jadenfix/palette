# BeaterClient::TraceIngestedDrainReport

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **completed** | **Integer** |  |  |
| **consumed** | **Integer** |  |  |
| **dead_lettered** | **Integer** |  |  |
| **failed_work** | **Integer** |  |  |
| **invalid_messages** | **Integer** |  |  |
| **retried** | **Integer** |  |  |
| **trace_refs** | [**Array&lt;QueuedTraceWork&gt;**](QueuedTraceWork.md) |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::TraceIngestedDrainReport.new(
  completed: null,
  consumed: null,
  dead_lettered: null,
  failed_work: null,
  invalid_messages: null,
  retried: null,
  trace_refs: null
)
```
