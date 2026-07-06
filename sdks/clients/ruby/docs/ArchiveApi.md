# BeaterClient::ArchiveApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------ | ------------ | ----------- |
| [**archive_trace**](ArchiveApi.md#archive_trace) | **POST** /v1/archive/{tenant_id}/{project_id}/{trace_id} |  |
| [**query_archive_spans**](ArchiveApi.md#query_archive_spans) | **GET** /v1/archive/{tenant_id}/{project_id}/spans |  |


## archive_trace

> <ArchiveManifest> archive_trace(tenant_id, project_id, trace_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ArchiveApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
trace_id = 'trace_id_example' # String | trace_id
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.archive_trace(tenant_id, project_id, trace_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ArchiveApi->archive_trace: #{e}"
end
```

#### Using the archive_trace_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<ArchiveManifest>, Integer, Hash)> archive_trace_with_http_info(tenant_id, project_id, trace_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.archive_trace_with_http_info(tenant_id, project_id, trace_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <ArchiveManifest>
rescue BeaterClient::ApiError => e
  puts "Error when calling ArchiveApi->archive_trace_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **trace_id** | **String** | trace_id |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**ArchiveManifest**](ArchiveManifest.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json


## query_archive_spans

> <ArchiveQueryResponse> query_archive_spans(tenant_id, project_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ArchiveApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
opts = {
  environment_id: 'environment_id_example', # String |
  trace_id: 'trace_id_example', # String |
  span_id: 'span_id_example', # String |
  kind: 'kind_example', # String |
  status: 'status_example', # String |
  limit: 56, # Integer |
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.query_archive_spans(tenant_id, project_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ArchiveApi->query_archive_spans: #{e}"
end
```

#### Using the query_archive_spans_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<ArchiveQueryResponse>, Integer, Hash)> query_archive_spans_with_http_info(tenant_id, project_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.query_archive_spans_with_http_info(tenant_id, project_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <ArchiveQueryResponse>
rescue BeaterClient::ApiError => e
  puts "Error when calling ArchiveApi->query_archive_spans_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **environment_id** | **String** |  | [optional] |
| **trace_id** | **String** |  | [optional] |
| **span_id** | **String** |  | [optional] |
| **kind** | **String** |  | [optional] |
| **status** | **String** |  | [optional] |
| **limit** | **Integer** |  | [optional] |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**ArchiveQueryResponse**](ArchiveQueryResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json
