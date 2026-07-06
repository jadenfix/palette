# BeaterClient::ToolExecution

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **data** | **Object** | Tool output payload (shape is tool-specific). | [optional] |
| **error** | **String** | Error message when &#x60;successful&#x60; is false. | [optional] |
| **log_id** | **String** | Composio execution log id, for tracing. | [optional] |
| **successful** | **Boolean** | Whether the tool reported success. |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::ToolExecution.new(
  data: null,
  error: null,
  log_id: null,
  successful: null
)
```
