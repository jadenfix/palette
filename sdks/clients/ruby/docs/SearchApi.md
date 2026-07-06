# BeaterClient::SearchApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------ | ------------ | ----------- |
| [**search_spans**](SearchApi.md#search_spans) | **GET** /v1/search/{tenant_id}/spans |  |


## search_spans

> <SearchResponse> search_spans(tenant_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::SearchApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
opts = {
  q: 'q_example', # String |
  project_id: 'project_id_example', # String |
  environment_id: 'environment_id_example', # String |
  trace_id: 'trace_id_example', # String |
  span_id: 'span_id_example', # String |
  kind: 'kind_example', # String |
  status: 'status_example', # String |
  model: 'model_example', # String |
  tool: 'tool_example', # String |
  limit: 56, # Integer |
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.search_spans(tenant_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling SearchApi->search_spans: #{e}"
end
```

#### Using the search_spans_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<SearchResponse>, Integer, Hash)> search_spans_with_http_info(tenant_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.search_spans_with_http_info(tenant_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <SearchResponse>
rescue BeaterClient::ApiError => e
  puts "Error when calling SearchApi->search_spans_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **q** | **String** |  | [optional] |
| **project_id** | **String** |  | [optional] |
| **environment_id** | **String** |  | [optional] |
| **trace_id** | **String** |  | [optional] |
| **span_id** | **String** |  | [optional] |
| **kind** | **String** |  | [optional] |
| **status** | **String** |  | [optional] |
| **model** | **String** |  | [optional] |
| **tool** | **String** |  | [optional] |
| **limit** | **Integer** |  | [optional] |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**SearchResponse**](SearchResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json
