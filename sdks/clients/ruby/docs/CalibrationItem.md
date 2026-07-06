# BeaterClient::CalibrationItem

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **agreed** | **Boolean** |  |  |
| **dataset_case_id** | **String** |  |  |
| **evidence** | **Object** |  |  |
| **human_label** | [**CalibrationLabel**](CalibrationLabel.md) |  |  |
| **judge_label** | [**CalibrationLabel**](CalibrationLabel.md) |  |  |
| **judge_result_label** | **String** |  | [optional] |
| **judge_score** | **Float** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::CalibrationItem.new(
  agreed: null,
  dataset_case_id: null,
  evidence: null,
  human_label: null,
  judge_label: null,
  judge_result_label: null,
  judge_score: null
)
```
