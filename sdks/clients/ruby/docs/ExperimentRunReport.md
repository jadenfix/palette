# BeaterClient::ExperimentRunReport

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **baseline_release_id** | **String** |  |  |
| **candidate_release_id** | **String** |  |  |
| **case_scores** | [**Array&lt;CaseExperimentScore&gt;**](CaseExperimentScore.md) |  |  |
| **comparison** | [**ExperimentComparison**](ExperimentComparison.md) |  |  |
| **created_at** | **Time** |  |  |
| **dataset_id** | **String** |  |  |
| **dataset_version_id** | **String** |  |  |
| **decision** | [**GateDecision**](GateDecision.md) |  |  |
| **evaluator_version_id** | **String** |  |  |
| **experiment_run_id** | **String** |  |  |
| **gate_policy** | [**GatePolicy**](GatePolicy.md) |  | [optional] |
| **project_id** | **String** |  |  |
| **tenant_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::ExperimentRunReport.new(
  baseline_release_id: null,
  candidate_release_id: null,
  case_scores: null,
  comparison: null,
  created_at: null,
  dataset_id: null,
  dataset_version_id: null,
  decision: null,
  evaluator_version_id: null,
  experiment_run_id: null,
  gate_policy: null,
  project_id: null,
  tenant_id: null
)
```
