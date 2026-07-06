# BeaterClient::NativeIngestRequest

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **attributes** | **Hash&lt;String, Object&gt;** |  |  |
| **auth_context** | [**AuthContext**](AuthContext.md) |  | [optional] |
| **cost** | [**Money**](Money.md) |  | [optional] |
| **end_time** | **Time** |  | [optional] |
| **idempotency_key** | **String** |  | [optional] |
| **input** | **Object** |  | [optional] |
| **kind** | **String** | Canonical agent span kind such as agent.run or llm.call |  |
| **model** | [**ModelRef**](ModelRef.md) |  | [optional] |
| **name** | **String** |  |  |
| **output** | **Object** |  | [optional] |
| **parent_span_id** | **String** |  | [optional] |
| **redaction_class** | [**RedactionClass**](RedactionClass.md) |  |  |
| **scope** | [**TenantScope**](TenantScope.md) |  |  |
| **seq** | **Integer** |  |  |
| **span_id** | **String** |  |  |
| **start_time** | **Time** |  | [optional] |
| **status** | [**SpanStatus**](SpanStatus.md) |  |  |
| **tokens** | [**TokenCounts**](TokenCounts.md) |  | [optional] |
| **trace_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::NativeIngestRequest.new(
  attributes: null,
  auth_context: null,
  cost: null,
  end_time: null,
  idempotency_key: null,
  input: null,
  kind: null,
  model: null,
  name: null,
  output: null,
  parent_span_id: null,
  redaction_class: null,
  scope: null,
  seq: null,
  span_id: null,
  start_time: null,
  status: null,
  tokens: null,
  trace_id: null
)
```
