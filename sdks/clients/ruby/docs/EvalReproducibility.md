# BeaterClient::EvalReproducibility

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **agent_release_id** | **String** |  |  |
| **code_hash** | **String** |  | [optional] |
| **dataset_case_id** | **String** |  |  |
| **dataset_version_id** | **String** |  |  |
| **evaluator_version_id** | **String** |  |  |
| **input_artifact_hashes** | **Array&lt;String&gt;** |  |  |
| **judge_model_id** | **String** |  | [optional] |
| **judge_parameters** | **Object** |  |  |
| **judge_provider** | **String** |  | [optional] |
| **judge_rubric_version** | **String** |  | [optional] |
| **judge_seed** | **Integer** |  | [optional] |
| **normalizer_version** | **String** |  |  |
| **prompt_version_id** | **String** |  | [optional] |
| **trace_schema_version** | **Integer** |  |  |
| **wasi_abi_version** | **String** |  | [optional] |
| **wasm_hash** | **String** |  | [optional] |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::EvalReproducibility.new(
  agent_release_id: null,
  code_hash: null,
  dataset_case_id: null,
  dataset_version_id: null,
  evaluator_version_id: null,
  input_artifact_hashes: null,
  judge_model_id: null,
  judge_parameters: null,
  judge_provider: null,
  judge_rubric_version: null,
  judge_seed: null,
  normalizer_version: null,
  prompt_version_id: null,
  trace_schema_version: null,
  wasi_abi_version: null,
  wasm_hash: null
)
```
