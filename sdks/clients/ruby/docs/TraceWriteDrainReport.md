# BeaterClient::TraceWriteDrainReport

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **consumed** | **Integer** |  |  |
| **dead_lettered** | **Integer** |  |  |
| **downstream_published** | **Integer** |  |  |
| **duplicate_raw** | **Integer** |  |  |
| **duplicate_spans** | **Integer** |  |  |
| **failed_downstream_publishes** | **Integer** |  |  |
| **failed_writes** | **Integer** |  |  |
| **invalid_messages** | **Integer** |  |  |
| **retried** | **Integer** |  |  |
| **trace_ids** | **Array&lt;String&gt;** |  |  |
| **trace_refs** | [**Array&lt;QueuedTraceWork&gt;**](QueuedTraceWork.md) |  |  |
| **written_raw** | **Integer** |  |  |
| **written_spans** | **Integer** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::TraceWriteDrainReport.new(
  consumed: null,
  dead_lettered: null,
  downstream_published: null,
  duplicate_raw: null,
  duplicate_spans: null,
  failed_downstream_publishes: null,
  failed_writes: null,
  invalid_messages: null,
  retried: null,
  trace_ids: null,
  trace_refs: null,
  written_raw: null,
  written_spans: null
)
```
