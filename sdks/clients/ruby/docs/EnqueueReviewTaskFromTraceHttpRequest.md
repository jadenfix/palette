# BeaterClient::EnqueueReviewTaskFromTraceHttpRequest

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **dataset_case_id** | **String** |  | [optional] |
| **dataset_id** | **String** |  | [optional] |
| **priority** | **Integer** |  | [optional] |
| **span_id** | **String** |  | [optional] |
| **task_id** | **String** |  | [optional] |
| **trace_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::EnqueueReviewTaskFromTraceHttpRequest.new(
  dataset_case_id: null,
  dataset_id: null,
  priority: null,
  span_id: null,
  task_id: null,
  trace_id: null
)
```
