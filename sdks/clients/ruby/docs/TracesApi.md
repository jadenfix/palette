# BeaterClient::TracesApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------ | ------------ | ----------- |
| [**get_trace**](TracesApi.md#get_trace) | **GET** /v1/traces/{tenant_id}/{trace_id} |  |
| [**list_traces**](TracesApi.md#list_traces) | **GET** /v1/traces/{tenant_id} |  |


## get_trace

> <TraceView> get_trace(tenant_id, trace_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::TracesApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
trace_id = 'trace_id_example' # String | trace_id
opts = {
  unmask: true, # Boolean |
  reason: 'reason_example', # String |
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.get_trace(tenant_id, trace_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling TracesApi->get_trace: #{e}"
end
```

#### Using the get_trace_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<TraceView>, Integer, Hash)> get_trace_with_http_info(tenant_id, trace_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.get_trace_with_http_info(tenant_id, trace_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <TraceView>
rescue BeaterClient::ApiError => e
  puts "Error when calling TracesApi->get_trace_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **trace_id** | **String** | trace_id |  |
| **unmask** | **Boolean** |  | [optional] |
| **reason** | **String** |  | [optional] |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**TraceView**](TraceView.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json


## list_traces

> <PageRunSummary> list_traces(tenant_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::TracesApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
opts = {
  project_id: 'project_id_example', # String |
  environment_id: 'environment_id_example', # String |
  trace_id: 'trace_id_example', # String |
  kind: 'kind_example', # String |
  status: 'status_example', # String |
  started_after: 'started_after_example', # String |
  started_before: 'started_before_example', # String |
  model: 'model_example', # String |
  release: 'release_example', # String |
  min_cost_micros: 789, # Integer |
  max_cost_micros: 789, # Integer |
  min_latency_ms: 789, # Integer |
  max_latency_ms: 789, # Integer |
  limit: 56, # Integer |
  cursor: 'cursor_example', # String |
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.list_traces(tenant_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling TracesApi->list_traces: #{e}"
end
```

#### Using the list_traces_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<PageRunSummary>, Integer, Hash)> list_traces_with_http_info(tenant_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.list_traces_with_http_info(tenant_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <PageRunSummary>
rescue BeaterClient::ApiError => e
  puts "Error when calling TracesApi->list_traces_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** |  | [optional] |
| **environment_id** | **String** |  | [optional] |
| **trace_id** | **String** |  | [optional] |
| **kind** | **String** |  | [optional] |
| **status** | **String** |  | [optional] |
| **started_after** | **String** |  | [optional] |
| **started_before** | **String** |  | [optional] |
| **model** | **String** |  | [optional] |
| **release** | **String** |  | [optional] |
| **min_cost_micros** | **Integer** |  | [optional] |
| **max_cost_micros** | **Integer** |  | [optional] |
| **min_latency_ms** | **Integer** |  | [optional] |
| **max_latency_ms** | **Integer** |  | [optional] |
| **limit** | **Integer** |  | [optional] |
| **cursor** | **String** |  | [optional] |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**PageRunSummary**](PageRunSummary.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json
