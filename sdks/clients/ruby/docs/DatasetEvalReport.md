# BeaterClient::DatasetEvalReport

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **aggregate_score** | **Float** |  |  |
| **created_at** | **Time** |  |  |
| **dataset_id** | **String** |  |  |
| **dataset_version_id** | **String** |  |  |
| **evaluator_version_id** | **String** |  |  |
| **project_id** | **String** |  |  |
| **report_id** | **String** |  |  |
| **result_count** | **Integer** |  |  |
| **results** | [**Array&lt;EvalResult&gt;**](EvalResult.md) |  |  |
| **tenant_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::DatasetEvalReport.new(
  aggregate_score: null,
  created_at: null,
  dataset_id: null,
  dataset_version_id: null,
  evaluator_version_id: null,
  project_id: null,
  report_id: null,
  result_count: null,
  results: null,
  tenant_id: null
)
```
