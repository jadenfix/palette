# BeaterClient::JudgeApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------ | ------------ | ----------- |
| [**evaluate_judge**](JudgeApi.md#evaluate_judge) | **POST** /v1/judge/{tenant_id}/{project_id}/evaluate |  |
| [**list_judge_ledger**](JudgeApi.md#list_judge_ledger) | **GET** /v1/judge/{tenant_id}/{project_id}/ledger |  |


## evaluate_judge

> <JudgeBrokerOutcome> evaluate_judge(tenant_id, project_id, run_judge_eval_http_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::JudgeApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
run_judge_eval_http_request = BeaterClient::RunJudgeEvalHttpRequest.new({_case: BeaterClient::EvaluationCase.new({input: 3.56, output: 3.56}), evaluator: BeaterClient::EvaluatorSpec.new({id: 'id_example', kind: BeaterClient::EvaluatorKindOneOf.new({type: 'exact_match'}), lane: BeaterClient::EvaluatorLane::DETERMINISTIC_WASI}), provider_secret_id: 'provider_secret_id_example'}) # RunJudgeEvalHttpRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.evaluate_judge(tenant_id, project_id, run_judge_eval_http_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling JudgeApi->evaluate_judge: #{e}"
end
```

#### Using the evaluate_judge_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<JudgeBrokerOutcome>, Integer, Hash)> evaluate_judge_with_http_info(tenant_id, project_id, run_judge_eval_http_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.evaluate_judge_with_http_info(tenant_id, project_id, run_judge_eval_http_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <JudgeBrokerOutcome>
rescue BeaterClient::ApiError => e
  puts "Error when calling JudgeApi->evaluate_judge_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **run_judge_eval_http_request** | [**RunJudgeEvalHttpRequest**](RunJudgeEvalHttpRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**JudgeBrokerOutcome**](JudgeBrokerOutcome.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json


## list_judge_ledger

> <Array<JudgeAuditRecord>> list_judge_ledger(tenant_id, project_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::JudgeApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.list_judge_ledger(tenant_id, project_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling JudgeApi->list_judge_ledger: #{e}"
end
```

#### Using the list_judge_ledger_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<Array<JudgeAuditRecord>>, Integer, Hash)> list_judge_ledger_with_http_info(tenant_id, project_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.list_judge_ledger_with_http_info(tenant_id, project_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <Array<JudgeAuditRecord>>
rescue BeaterClient::ApiError => e
  puts "Error when calling JudgeApi->list_judge_ledger_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**Array&lt;JudgeAuditRecord&gt;**](JudgeAuditRecord.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json
