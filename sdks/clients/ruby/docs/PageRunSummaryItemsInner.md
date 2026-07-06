# BeaterClient::PageRunSummaryItemsInner

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **duration_ms** | **Integer** |  | [optional] |
| **ended_at** | **Time** |  | [optional] |
| **first_span_name** | **String** |  |  |
| **models** | [**Array&lt;ModelRef&gt;**](ModelRef.md) |  |  |
| **project_id** | **String** |  |  |
| **release_ids** | **Array&lt;String&gt;** |  |  |
| **span_count** | **Integer** |  |  |
| **started_at** | **Time** |  |  |
| **status** | [**SpanStatus**](SpanStatus.md) |  |  |
| **tenant_id** | **String** |  |  |
| **total_cost** | [**Money**](Money.md) |  | [optional] |
| **trace_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::PageRunSummaryItemsInner.new(
  duration_ms: null,
  ended_at: null,
  first_span_name: null,
  models: null,
  project_id: null,
  release_ids: null,
  span_count: null,
  started_at: null,
  status: null,
  tenant_id: null,
  total_cost: null,
  trace_id: null
)
```
