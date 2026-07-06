# BeaterClient::TraceView

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **spans** | [**Array&lt;CanonicalSpan&gt;**](CanonicalSpan.md) |  |  |
| **tenant_id** | **String** |  |  |
| **trace_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::TraceView.new(
  spans: null,
  tenant_id: null,
  trace_id: null
)
```
