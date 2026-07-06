# BeaterClient::EvalsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------ | ------------ | ----------- |
| [**run_deterministic_eval**](EvalsApi.md#run_deterministic_eval) | **POST** /v1/datasets/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id}/evals/deterministic |  |
| [**run_judge_eval**](EvalsApi.md#run_judge_eval) | **POST** /v1/datasets/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id}/evals/judge |  |


## run_deterministic_eval

> <DatasetEvalReport> run_deterministic_eval(tenant_id, project_id, dataset_id, version_id, run_deterministic_eval_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::EvalsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
dataset_id = 'dataset_id_example' # String | dataset_id
version_id = 'version_id_example' # String | version_id
run_deterministic_eval_request = BeaterClient::RunDeterministicEvalRequest.new({agent_release_id: 'agent_release_id_example', evaluator_id: 'evaluator_id_example', evaluator_version_id: 'evaluator_version_id_example', kind: BeaterClient::EvaluatorKindOneOf.new({type: 'exact_match'})}) # RunDeterministicEvalRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.run_deterministic_eval(tenant_id, project_id, dataset_id, version_id, run_deterministic_eval_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling EvalsApi->run_deterministic_eval: #{e}"
end
```

#### Using the run_deterministic_eval_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<DatasetEvalReport>, Integer, Hash)> run_deterministic_eval_with_http_info(tenant_id, project_id, dataset_id, version_id, run_deterministic_eval_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.run_deterministic_eval_with_http_info(tenant_id, project_id, dataset_id, version_id, run_deterministic_eval_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <DatasetEvalReport>
rescue BeaterClient::ApiError => e
  puts "Error when calling EvalsApi->run_deterministic_eval_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **dataset_id** | **String** | dataset_id |  |
| **version_id** | **String** | version_id |  |
| **run_deterministic_eval_request** | [**RunDeterministicEvalRequest**](RunDeterministicEvalRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**DatasetEvalReport**](DatasetEvalReport.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json


## run_judge_eval

> <DatasetEvalReport> run_judge_eval(tenant_id, project_id, dataset_id, version_id, run_judge_dataset_eval_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::EvalsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
dataset_id = 'dataset_id_example' # String | dataset_id
version_id = 'version_id_example' # String | version_id
run_judge_dataset_eval_request = BeaterClient::RunJudgeDatasetEvalRequest.new({agent_release_id: 'agent_release_id_example', evaluator_id: 'evaluator_id_example', evaluator_version_id: 'evaluator_version_id_example', kind: BeaterClient::EvaluatorKindOneOf.new({type: 'exact_match'}), provider_secret_id: 'provider_secret_id_example'}) # RunJudgeDatasetEvalRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.run_judge_eval(tenant_id, project_id, dataset_id, version_id, run_judge_dataset_eval_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling EvalsApi->run_judge_eval: #{e}"
end
```

#### Using the run_judge_eval_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<DatasetEvalReport>, Integer, Hash)> run_judge_eval_with_http_info(tenant_id, project_id, dataset_id, version_id, run_judge_dataset_eval_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.run_judge_eval_with_http_info(tenant_id, project_id, dataset_id, version_id, run_judge_dataset_eval_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <DatasetEvalReport>
rescue BeaterClient::ApiError => e
  puts "Error when calling EvalsApi->run_judge_eval_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **dataset_id** | **String** | dataset_id |  |
| **version_id** | **String** | version_id |  |
| **run_judge_dataset_eval_request** | [**RunJudgeDatasetEvalRequest**](RunJudgeDatasetEvalRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**DatasetEvalReport**](DatasetEvalReport.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json
