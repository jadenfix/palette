# BeaterClient::DatasetsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------ | ------------ | ----------- |
| [**create_dataset**](DatasetsApi.md#create_dataset) | **POST** /v1/datasets/{tenant_id}/{project_id} |  |
| [**create_dataset_version**](DatasetsApi.md#create_dataset_version) | **POST** /v1/datasets/{tenant_id}/{project_id}/{dataset_id}/versions |  |
| [**promote_dataset_case_from_trace**](DatasetsApi.md#promote_dataset_case_from_trace) | **POST** /v1/datasets/{tenant_id}/{project_id}/{dataset_id}/cases/from-trace |  |


## create_dataset

> <Dataset> create_dataset(tenant_id, project_id, create_dataset_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::DatasetsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
create_dataset_request = BeaterClient::CreateDatasetRequest.new({name: 'name_example'}) # CreateDatasetRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.create_dataset(tenant_id, project_id, create_dataset_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling DatasetsApi->create_dataset: #{e}"
end
```

#### Using the create_dataset_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<Dataset>, Integer, Hash)> create_dataset_with_http_info(tenant_id, project_id, create_dataset_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.create_dataset_with_http_info(tenant_id, project_id, create_dataset_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <Dataset>
rescue BeaterClient::ApiError => e
  puts "Error when calling DatasetsApi->create_dataset_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **create_dataset_request** | [**CreateDatasetRequest**](CreateDatasetRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**Dataset**](Dataset.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json


## create_dataset_version

> <DatasetVersionSnapshot> create_dataset_version(tenant_id, project_id, dataset_id, create_dataset_version_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::DatasetsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
dataset_id = 'dataset_id_example' # String | dataset_id
create_dataset_version_request = BeaterClient::CreateDatasetVersionRequest.new # CreateDatasetVersionRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.create_dataset_version(tenant_id, project_id, dataset_id, create_dataset_version_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling DatasetsApi->create_dataset_version: #{e}"
end
```

#### Using the create_dataset_version_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<DatasetVersionSnapshot>, Integer, Hash)> create_dataset_version_with_http_info(tenant_id, project_id, dataset_id, create_dataset_version_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.create_dataset_version_with_http_info(tenant_id, project_id, dataset_id, create_dataset_version_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <DatasetVersionSnapshot>
rescue BeaterClient::ApiError => e
  puts "Error when calling DatasetsApi->create_dataset_version_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **dataset_id** | **String** | dataset_id |  |
| **create_dataset_version_request** | [**CreateDatasetVersionRequest**](CreateDatasetVersionRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**DatasetVersionSnapshot**](DatasetVersionSnapshot.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json


## promote_dataset_case_from_trace

> <DatasetCase> promote_dataset_case_from_trace(tenant_id, project_id, dataset_id, promote_trace_case_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::DatasetsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
dataset_id = 'dataset_id_example' # String | dataset_id
promote_trace_case_request = BeaterClient::PromoteTraceCaseRequest.new({trace_id: 'trace_id_example'}) # PromoteTraceCaseRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.promote_dataset_case_from_trace(tenant_id, project_id, dataset_id, promote_trace_case_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling DatasetsApi->promote_dataset_case_from_trace: #{e}"
end
```

#### Using the promote_dataset_case_from_trace_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<DatasetCase>, Integer, Hash)> promote_dataset_case_from_trace_with_http_info(tenant_id, project_id, dataset_id, promote_trace_case_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.promote_dataset_case_from_trace_with_http_info(tenant_id, project_id, dataset_id, promote_trace_case_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <DatasetCase>
rescue BeaterClient::ApiError => e
  puts "Error when calling DatasetsApi->promote_dataset_case_from_trace_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **dataset_id** | **String** | dataset_id |  |
| **promote_trace_case_request** | [**PromoteTraceCaseRequest**](PromoteTraceCaseRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**DatasetCase**](DatasetCase.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json
