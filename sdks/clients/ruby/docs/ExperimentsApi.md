# BeaterClient::ExperimentsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------ | ------------ | ----------- |
| [**run_deterministic_experiment**](ExperimentsApi.md#run_deterministic_experiment) | **POST** /v1/experiments/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id}/deterministic |  |
| [**run_judge_experiment**](ExperimentsApi.md#run_judge_experiment) | **POST** /v1/experiments/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id}/judge |  |


## run_deterministic_experiment

> <ExperimentRunReport> run_deterministic_experiment(tenant_id, project_id, dataset_id, version_id, run_experiment_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ExperimentsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
dataset_id = 'dataset_id_example' # String | dataset_id
version_id = 'version_id_example' # String | version_id
run_experiment_request = BeaterClient::RunExperimentRequest.new({baseline_outputs: [BeaterClient::CaseOutputOverrideRequest.new({case_id: 'case_id_example', output: 3.56})], baseline_release_id: 'baseline_release_id_example', candidate_outputs: [BeaterClient::CaseOutputOverrideRequest.new({case_id: 'case_id_example', output: 3.56})], candidate_release_id: 'candidate_release_id_example', evaluator_id: 'evaluator_id_example', evaluator_version_id: 'evaluator_version_id_example', kind: BeaterClient::EvaluatorKindOneOf.new({type: 'exact_match'})}) # RunExperimentRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.run_deterministic_experiment(tenant_id, project_id, dataset_id, version_id, run_experiment_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ExperimentsApi->run_deterministic_experiment: #{e}"
end
```

#### Using the run_deterministic_experiment_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<ExperimentRunReport>, Integer, Hash)> run_deterministic_experiment_with_http_info(tenant_id, project_id, dataset_id, version_id, run_experiment_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.run_deterministic_experiment_with_http_info(tenant_id, project_id, dataset_id, version_id, run_experiment_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <ExperimentRunReport>
rescue BeaterClient::ApiError => e
  puts "Error when calling ExperimentsApi->run_deterministic_experiment_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **dataset_id** | **String** | dataset_id |  |
| **version_id** | **String** | version_id |  |
| **run_experiment_request** | [**RunExperimentRequest**](RunExperimentRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**ExperimentRunReport**](ExperimentRunReport.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json


## run_judge_experiment

> <ExperimentRunReport> run_judge_experiment(tenant_id, project_id, dataset_id, version_id, run_judge_experiment_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ExperimentsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
dataset_id = 'dataset_id_example' # String | dataset_id
version_id = 'version_id_example' # String | version_id
run_judge_experiment_request = BeaterClient::RunJudgeExperimentRequest.new({baseline_outputs: [BeaterClient::CaseOutputOverrideRequest.new({case_id: 'case_id_example', output: 3.56})], baseline_release_id: 'baseline_release_id_example', candidate_outputs: [BeaterClient::CaseOutputOverrideRequest.new({case_id: 'case_id_example', output: 3.56})], candidate_release_id: 'candidate_release_id_example', evaluator_id: 'evaluator_id_example', evaluator_version_id: 'evaluator_version_id_example', kind: BeaterClient::EvaluatorKindOneOf.new({type: 'exact_match'}), provider_secret_id: 'provider_secret_id_example'}) # RunJudgeExperimentRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.run_judge_experiment(tenant_id, project_id, dataset_id, version_id, run_judge_experiment_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ExperimentsApi->run_judge_experiment: #{e}"
end
```

#### Using the run_judge_experiment_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<ExperimentRunReport>, Integer, Hash)> run_judge_experiment_with_http_info(tenant_id, project_id, dataset_id, version_id, run_judge_experiment_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.run_judge_experiment_with_http_info(tenant_id, project_id, dataset_id, version_id, run_judge_experiment_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <ExperimentRunReport>
rescue BeaterClient::ApiError => e
  puts "Error when calling ExperimentsApi->run_judge_experiment_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **dataset_id** | **String** | dataset_id |  |
| **version_id** | **String** | version_id |  |
| **run_judge_experiment_request** | [**RunJudgeExperimentRequest**](RunJudgeExperimentRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**ExperimentRunReport**](ExperimentRunReport.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json
