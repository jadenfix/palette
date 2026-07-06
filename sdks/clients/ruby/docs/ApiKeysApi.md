# BeaterClient::ApiKeysApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------ | ------------ | ----------- |
| [**create_api_key**](ApiKeysApi.md#create_api_key) | **POST** /v1/api-keys/{tenant_id}/{project_id}/{environment_id} |  |
| [**revoke_api_key**](ApiKeysApi.md#revoke_api_key) | **POST** /v1/api-keys/{tenant_id}/{project_id}/{environment_id}/{api_key_id}/revoke |  |


## create_api_key

> <ApiKeyCreatedResponse> create_api_key(tenant_id, project_id, environment_id, create_api_key_http_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ApiKeysApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
environment_id = 'environment_id_example' # String | environment_id
create_api_key_http_request = BeaterClient::CreateApiKeyHttpRequest.new({scopes: [BeaterClient::ApiScope::TRACE_WRITE]}) # CreateApiKeyHttpRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.create_api_key(tenant_id, project_id, environment_id, create_api_key_http_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ApiKeysApi->create_api_key: #{e}"
end
```

#### Using the create_api_key_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<ApiKeyCreatedResponse>, Integer, Hash)> create_api_key_with_http_info(tenant_id, project_id, environment_id, create_api_key_http_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.create_api_key_with_http_info(tenant_id, project_id, environment_id, create_api_key_http_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <ApiKeyCreatedResponse>
rescue BeaterClient::ApiError => e
  puts "Error when calling ApiKeysApi->create_api_key_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **environment_id** | **String** | environment_id |  |
| **create_api_key_http_request** | [**CreateApiKeyHttpRequest**](CreateApiKeyHttpRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**ApiKeyCreatedResponse**](ApiKeyCreatedResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json


## revoke_api_key

> <RevokedApiKey> revoke_api_key(tenant_id, project_id, environment_id, api_key_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ApiKeysApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
environment_id = 'environment_id_example' # String | environment_id
api_key_id = 'api_key_id_example' # String | api_key_id
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.revoke_api_key(tenant_id, project_id, environment_id, api_key_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ApiKeysApi->revoke_api_key: #{e}"
end
```

#### Using the revoke_api_key_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<RevokedApiKey>, Integer, Hash)> revoke_api_key_with_http_info(tenant_id, project_id, environment_id, api_key_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.revoke_api_key_with_http_info(tenant_id, project_id, environment_id, api_key_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <RevokedApiKey>
rescue BeaterClient::ApiError => e
  puts "Error when calling ApiKeysApi->revoke_api_key_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **environment_id** | **String** | environment_id |  |
| **api_key_id** | **String** | api_key_id |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**RevokedApiKey**](RevokedApiKey.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json
