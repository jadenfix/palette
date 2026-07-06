# BeaterClient::GatesApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------ | ------------ | ----------- |
| [**create_gate**](GatesApi.md#create_gate) | **POST** /v1/gates/{tenant_id}/{project_id} |  |
| [**run_gate**](GatesApi.md#run_gate) | **POST** /v1/gates/{tenant_id}/{project_id}/{gate_id}/run |  |


## create_gate

> <GateDefinition> create_gate(tenant_id, project_id, create_gate_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::GatesApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
create_gate_request = BeaterClient::CreateGateRequest.new({gate_id: 'gate_id_example', name: 'name_example'}) # CreateGateRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.create_gate(tenant_id, project_id, create_gate_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling GatesApi->create_gate: #{e}"
end
```

#### Using the create_gate_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<GateDefinition>, Integer, Hash)> create_gate_with_http_info(tenant_id, project_id, create_gate_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.create_gate_with_http_info(tenant_id, project_id, create_gate_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <GateDefinition>
rescue BeaterClient::ApiError => e
  puts "Error when calling GatesApi->create_gate_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **create_gate_request** | [**CreateGateRequest**](CreateGateRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**GateDefinition**](GateDefinition.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json


## run_gate

> <GateRunReport> run_gate(tenant_id, project_id, gate_id, run_gate_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::GatesApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
gate_id = 'gate_id_example' # String | gate_id
run_gate_request = BeaterClient::RunGateRequest.new # RunGateRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.run_gate(tenant_id, project_id, gate_id, run_gate_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling GatesApi->run_gate: #{e}"
end
```

#### Using the run_gate_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<GateRunReport>, Integer, Hash)> run_gate_with_http_info(tenant_id, project_id, gate_id, run_gate_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.run_gate_with_http_info(tenant_id, project_id, gate_id, run_gate_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <GateRunReport>
rescue BeaterClient::ApiError => e
  puts "Error when calling GatesApi->run_gate_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **gate_id** | **String** | gate_id |  |
| **run_gate_request** | [**RunGateRequest**](RunGateRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**GateRunReport**](GateRunReport.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json
