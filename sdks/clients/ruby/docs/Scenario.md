# BeaterClient::Scenario

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **created_at** | **Time** | When the scenario was created. |  |
| **exemplar_trace_id** | **String** |  |  |
| **expected_outcome** | **String** | Expected outcome for replay assertions, if known. | [optional] |
| **failure_mode** | [**FailureMode**](FailureMode.md) | The dominant failure mode this scenario reproduces. |  |
| **perturbation_knobs** | [**PerturbationKnobs**](PerturbationKnobs.md) | Suggested perturbation knobs for replay. |  |
| **recurrence_count** | **Integer** | How many traces exhibited this scenario. |  |
| **redaction_class** | [**RedactionClass**](RedactionClass.md) | Redaction classification of the scenario payload. |  |
| **scenario_id** | **String** | Stable, deterministic identifier for the scenario. |  |
| **scope** | [**TenantScope**](TenantScope.md) | Tenant/project/environment scope this scenario belongs to. |  |
| **source_trace_ids** | **Array&lt;String&gt;** | Trace ids the scenario was mined from, sorted ascending. |  |
| **title** | **String** | Human-readable title. |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::Scenario.new(
  created_at: null,
  exemplar_trace_id: null,
  expected_outcome: null,
  failure_mode: null,
  perturbation_knobs: null,
  recurrence_count: null,
  redaction_class: null,
  scenario_id: null,
  scope: null,
  source_trace_ids: null,
  title: null
)
```
