# BeaterClient::ReviewAnnotation

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **annotation_id** | **String** |  |  |
| **created_at** | **Time** |  |  |
| **payload** | **Object** |  |  |
| **project_id** | **String** |  |  |
| **queue_id** | **String** |  |  |
| **reviewer_id** | **String** |  |  |
| **task_id** | **String** |  |  |
| **tenant_id** | **String** |  |  |
| **verdict** | [**ReviewVerdict**](ReviewVerdict.md) |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::ReviewAnnotation.new(
  annotation_id: null,
  created_at: null,
  payload: null,
  project_id: null,
  queue_id: null,
  reviewer_id: null,
  task_id: null,
  tenant_id: null,
  verdict: null
)
```
