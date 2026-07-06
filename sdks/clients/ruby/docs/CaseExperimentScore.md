# BeaterClient::CaseExperimentScore

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **baseline_cached** | **Boolean** |  | [optional] |
| **baseline_cost** | [**Money**](Money.md) |  | [optional] |
| **baseline_evidence** | **Object** |  |  |
| **baseline_judge_call_id** | **String** |  | [optional] |
| **baseline_output** | **Object** |  |  |
| **baseline_score** | **Float** |  |  |
| **baseline_trace** | **Object** |  | [optional] |
| **candidate_cached** | **Boolean** |  | [optional] |
| **candidate_cost** | [**Money**](Money.md) |  | [optional] |
| **candidate_evidence** | **Object** |  |  |
| **candidate_judge_call_id** | **String** |  | [optional] |
| **candidate_output** | **Object** |  |  |
| **candidate_score** | **Float** |  |  |
| **candidate_trace** | **Object** |  | [optional] |
| **case_id** | **String** |  |  |
| **delta** | **Float** |  |  |
| **reference** | **Object** |  | [optional] |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::CaseExperimentScore.new(
  baseline_cached: null,
  baseline_cost: null,
  baseline_evidence: null,
  baseline_judge_call_id: null,
  baseline_output: null,
  baseline_score: null,
  baseline_trace: null,
  candidate_cached: null,
  candidate_cost: null,
  candidate_evidence: null,
  candidate_judge_call_id: null,
  candidate_output: null,
  candidate_score: null,
  candidate_trace: null,
  case_id: null,
  delta: null,
  reference: null
)
```
