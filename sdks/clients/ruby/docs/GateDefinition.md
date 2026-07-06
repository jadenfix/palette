# BeaterClient::GateDefinition

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **created_at** | **Time** |  |  |
| **dataset_id** | **String** |  | [optional] |
| **evaluator_version_id** | **String** |  | [optional] |
| **gate_id** | **String** |  |  |
| **inconclusive_policy** | [**InconclusivePolicy**](InconclusivePolicy.md) |  | [optional] |
| **name** | **String** |  |  |
| **project_id** | **String** |  |  |
| **tenant_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::GateDefinition.new(
  created_at: null,
  dataset_id: null,
  evaluator_version_id: null,
  gate_id: null,
  inconclusive_policy: null,
  name: null,
  project_id: null,
  tenant_id: null
)
```
