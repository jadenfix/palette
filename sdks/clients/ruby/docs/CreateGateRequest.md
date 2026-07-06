# BeaterClient::CreateGateRequest

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **dataset_id** | **String** |  | [optional] |
| **evaluator_version_id** | **String** |  | [optional] |
| **gate_id** | **String** |  |  |
| **inconclusive_policy** | [**InconclusivePolicy**](InconclusivePolicy.md) |  | [optional] |
| **name** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::CreateGateRequest.new(
  dataset_id: null,
  evaluator_version_id: null,
  gate_id: null,
  inconclusive_policy: null,
  name: null
)
```
