# BeaterClient::AlertInput

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **baseline_score** | **Float** |  | [optional] |
| **group_key** | **String** |  |  |
| **links** | [**AlertLinks**](AlertLinks.md) |  |  |
| **now** | **Time** |  |  |
| **project_id** | **String** |  |  |
| **score** | **Float** |  |  |
| **tenant_id** | **String** |  |  |
| **title** | **String** |  |  |
| **trace_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::AlertInput.new(
  baseline_score: null,
  group_key: null,
  links: null,
  now: null,
  project_id: null,
  score: null,
  tenant_id: null,
  title: null,
  trace_id: null
)
```
