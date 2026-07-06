# BeaterClient::ReviewTask

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **created_at** | **Time** |  |  |
| **dataset_case_id** | **String** |  | [optional] |
| **dataset_id** | **String** |  | [optional] |
| **priority** | **Integer** |  |  |
| **project_id** | **String** |  |  |
| **queue_id** | **String** |  |  |
| **span_id** | **String** |  | [optional] |
| **state** | [**ReviewTaskState**](ReviewTaskState.md) |  |  |
| **task_id** | **String** |  |  |
| **tenant_id** | **String** |  |  |
| **trace_id** | **String** |  |  |
| **updated_at** | **Time** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::ReviewTask.new(
  created_at: null,
  dataset_case_id: null,
  dataset_id: null,
  priority: null,
  project_id: null,
  queue_id: null,
  span_id: null,
  state: null,
  task_id: null,
  tenant_id: null,
  trace_id: null,
  updated_at: null
)
```
