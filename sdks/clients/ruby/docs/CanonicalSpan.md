# BeaterClient::CanonicalSpan

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **attributes** | **Hash&lt;String, Object&gt;** |  |  |
| **cost** | [**Money**](Money.md) |  | [optional] |
| **end_time** | **Time** |  | [optional] |
| **environment_id** | **String** |  |  |
| **input_ref** | [**ArtifactRef**](ArtifactRef.md) |  | [optional] |
| **kind** | **String** | Canonical agent span kind such as agent.run or llm.call |  |
| **model** | [**ModelRef**](ModelRef.md) |  | [optional] |
| **name** | **String** |  |  |
| **normalizer_version** | **String** |  |  |
| **output_ref** | [**ArtifactRef**](ArtifactRef.md) |  | [optional] |
| **parent_span_id** | **String** |  | [optional] |
| **project_id** | **String** |  |  |
| **raw_ref** | [**ArtifactRef**](ArtifactRef.md) |  |  |
| **schema_version** | **Integer** |  |  |
| **seq** | **Integer** |  |  |
| **span_id** | **String** |  |  |
| **start_time** | **Time** |  |  |
| **status** | [**SpanStatus**](SpanStatus.md) |  |  |
| **tenant_id** | **String** |  |  |
| **tokens** | [**TokenCounts**](TokenCounts.md) |  | [optional] |
| **trace_id** | **String** |  |  |
| **unmapped_attrs** | **Object** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::CanonicalSpan.new(
  attributes: null,
  cost: null,
  end_time: null,
  environment_id: null,
  input_ref: null,
  kind: null,
  model: null,
  name: null,
  normalizer_version: null,
  output_ref: null,
  parent_span_id: null,
  project_id: null,
  raw_ref: null,
  schema_version: null,
  seq: null,
  span_id: null,
  start_time: null,
  status: null,
  tenant_id: null,
  tokens: null,
  trace_id: null,
  unmapped_attrs: null
)
```
