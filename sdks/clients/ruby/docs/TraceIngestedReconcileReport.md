# BeaterClient::TraceIngestedReconcileReport

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **downstream_accepted** | **Integer** |  |  |
| **downstream_duplicate** | **Integer** |  |  |
| **downstream_queued** | **Boolean** |  |  |
| **project_id** | **String** |  |  |
| **span_count** | **Integer** |  |  |
| **tenant_id** | **String** |  |  |
| **trace_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::TraceIngestedReconcileReport.new(
  downstream_accepted: null,
  downstream_duplicate: null,
  downstream_queued: null,
  project_id: null,
  span_count: null,
  tenant_id: null,
  trace_id: null
)
```
