# BeaterClient::CreateScenarioRequest

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **exemplar_trace_id** | **String** |  | [optional] |
| **expected_outcome** | **String** |  | [optional] |
| **failure_mode** | [**FailureMode**](FailureMode.md) |  | [optional] |
| **source_trace_ids** | **Array&lt;String&gt;** |  |  |
| **title** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::CreateScenarioRequest.new(
  exemplar_trace_id: null,
  expected_outcome: null,
  failure_mode: null,
  source_trace_ids: null,
  title: null
)
```
