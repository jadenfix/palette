# BeaterClient::OnlineApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------ | ------------ | ----------- |
| [**decide_online_sampling**](OnlineApi.md#decide_online_sampling) | **POST** /v1/online/{tenant_id}/{project_id}/traces/{trace_id}/sampling |  |


## decide_online_sampling

> <SamplingDecision> decide_online_sampling(tenant_id, project_id, trace_id, online_sampling_policy, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::OnlineApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
trace_id = 'trace_id_example' # String | trace_id
online_sampling_policy = BeaterClient::OnlineSamplingPolicy.new({keep_errors: false, sample_rate_per_mille: 37}) # OnlineSamplingPolicy |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.decide_online_sampling(tenant_id, project_id, trace_id, online_sampling_policy, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling OnlineApi->decide_online_sampling: #{e}"
end
```

#### Using the decide_online_sampling_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<SamplingDecision>, Integer, Hash)> decide_online_sampling_with_http_info(tenant_id, project_id, trace_id, online_sampling_policy, opts)

```ruby
begin

  data, status_code, headers = api_instance.decide_online_sampling_with_http_info(tenant_id, project_id, trace_id, online_sampling_policy, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <SamplingDecision>
rescue BeaterClient::ApiError => e
  puts "Error when calling OnlineApi->decide_online_sampling_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **trace_id** | **String** | trace_id |  |
| **online_sampling_policy** | [**OnlineSamplingPolicy**](OnlineSamplingPolicy.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**SamplingDecision**](SamplingDecision.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json
