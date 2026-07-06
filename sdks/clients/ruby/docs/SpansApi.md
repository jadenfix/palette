# BeaterClient::SpansApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------ | ------------ | ----------- |
| [**get_span**](SpansApi.md#get_span) | **GET** /v1/spans/{tenant_id}/{trace_id}/{span_id} |  |
| [**get_span_io**](SpansApi.md#get_span_io) | **GET** /v1/spans/{tenant_id}/{trace_id}/{span_id}/io |  |


## get_span

> <CanonicalSpan> get_span(tenant_id, trace_id, span_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::SpansApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
trace_id = 'trace_id_example' # String | trace_id
span_id = 'span_id_example' # String | span_id
opts = {
  unmask: true, # Boolean |
  reason: 'reason_example', # String |
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.get_span(tenant_id, trace_id, span_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling SpansApi->get_span: #{e}"
end
```

#### Using the get_span_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<CanonicalSpan>, Integer, Hash)> get_span_with_http_info(tenant_id, trace_id, span_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.get_span_with_http_info(tenant_id, trace_id, span_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <CanonicalSpan>
rescue BeaterClient::ApiError => e
  puts "Error when calling SpansApi->get_span_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **trace_id** | **String** | trace_id |  |
| **span_id** | **String** | span_id |  |
| **unmask** | **Boolean** |  | [optional] |
| **reason** | **String** |  | [optional] |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**CanonicalSpan**](CanonicalSpan.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json


## get_span_io

> <SpanIoResponse> get_span_io(tenant_id, trace_id, span_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::SpansApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
trace_id = 'trace_id_example' # String | trace_id
span_id = 'span_id_example' # String | span_id
opts = {
  unmask: true, # Boolean |
  reason: 'reason_example', # String |
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.get_span_io(tenant_id, trace_id, span_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling SpansApi->get_span_io: #{e}"
end
```

#### Using the get_span_io_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<SpanIoResponse>, Integer, Hash)> get_span_io_with_http_info(tenant_id, trace_id, span_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.get_span_io_with_http_info(tenant_id, trace_id, span_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <SpanIoResponse>
rescue BeaterClient::ApiError => e
  puts "Error when calling SpansApi->get_span_io_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **trace_id** | **String** | trace_id |  |
| **span_id** | **String** | span_id |  |
| **unmask** | **Boolean** |  | [optional] |
| **reason** | **String** |  | [optional] |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**SpanIoResponse**](SpanIoResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json
