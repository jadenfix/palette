# BeaterClient::DatasetCase

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **case_id** | **String** |  |  |
| **created_at** | **Time** |  |  |
| **dataset_id** | **String** |  |  |
| **input** | **Object** |  |  |
| **input_artifact_hashes** | **Array&lt;String&gt;** |  |  |
| **normalizer_version** | **String** |  |  |
| **output** | **Object** |  |  |
| **project_id** | **String** |  |  |
| **reference** | **Object** |  | [optional] |
| **source_environment_id** | **String** |  |  |
| **source_span_id** | **String** |  |  |
| **source_trace_id** | **String** |  |  |
| **tenant_id** | **String** |  |  |
| **trace** | **Object** |  |  |
| **trace_schema_version** | **Integer** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::DatasetCase.new(
  case_id: null,
  created_at: null,
  dataset_id: null,
  input: null,
  input_artifact_hashes: null,
  normalizer_version: null,
  output: null,
  project_id: null,
  reference: null,
  source_environment_id: null,
  source_span_id: null,
  source_trace_id: null,
  tenant_id: null,
  trace: null,
  trace_schema_version: null
)
```
