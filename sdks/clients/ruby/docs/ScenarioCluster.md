# BeaterClient::ScenarioCluster

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **dominant_failure_mode** | [**FailureMode**](FailureMode.md) | The most common failure mode across members. |  |
| **exemplar_trace_id** | **String** |  |  |
| **member_trace_ids** | **Array&lt;String&gt;** | All member trace ids, sorted ascending. |  |
| **signature** | [**Signature**](Signature.md) | The signature of the cluster&#39;s exemplar. |  |
| **size** | **Integer** | Number of member traces. |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::ScenarioCluster.new(
  dominant_failure_mode: null,
  exemplar_trace_id: null,
  member_trace_ids: null,
  signature: null,
  size: null
)
```
