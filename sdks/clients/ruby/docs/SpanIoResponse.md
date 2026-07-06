# BeaterClient::SpanIoResponse

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **input** | [**SpanIoValue**](SpanIoValue.md) |  |  |
| **output** | [**SpanIoValue**](SpanIoValue.md) |  |  |
| **span_id** | **String** |  |  |
| **tenant_id** | **String** |  |  |
| **trace_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::SpanIoResponse.new(
  input: null,
  output: null,
  span_id: null,
  tenant_id: null,
  trace_id: null
)
```
