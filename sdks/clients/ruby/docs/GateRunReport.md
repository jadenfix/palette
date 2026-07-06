# BeaterClient::GateRunReport

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **baseline_release_id** | **String** |  |  |
| **candidate_release_id** | **String** |  |  |
| **comparison** | [**ExperimentComparison**](ExperimentComparison.md) |  |  |
| **created_at** | **Time** |  |  |
| **dataset_id** | **String** |  |  |
| **evaluator_version_id** | **String** |  |  |
| **experiment_created_at** | **Time** |  |  |
| **experiment_decision** | [**GateDecision**](GateDecision.md) |  |  |
| **experiment_gate_policy** | [**GatePolicy**](GatePolicy.md) |  |  |
| **experiment_run_id** | **String** |  |  |
| **gate_dataset_id** | **String** |  | [optional] |
| **gate_evaluator_version_id** | **String** |  | [optional] |
| **gate_id** | **String** |  |  |
| **gate_name** | **String** |  |  |
| **gate_run_id** | **String** |  |  |
| **inconclusive_policy** | [**InconclusivePolicy**](InconclusivePolicy.md) |  |  |
| **passed** | **Boolean** |  |  |
| **project_id** | **String** |  |  |
| **reason** | **String** |  |  |
| **tenant_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::GateRunReport.new(
  baseline_release_id: null,
  candidate_release_id: null,
  comparison: null,
  created_at: null,
  dataset_id: null,
  evaluator_version_id: null,
  experiment_created_at: null,
  experiment_decision: null,
  experiment_gate_policy: null,
  experiment_run_id: null,
  gate_dataset_id: null,
  gate_evaluator_version_id: null,
  gate_id: null,
  gate_name: null,
  gate_run_id: null,
  inconclusive_policy: null,
  passed: null,
  project_id: null,
  reason: null,
  tenant_id: null
)
```
