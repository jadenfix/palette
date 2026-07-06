# BeaterClient::ProviderSecretsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------ | ------------ | ----------- |
| [**create_provider_secret**](ProviderSecretsApi.md#create_provider_secret) | **POST** /v1/provider-secrets/{tenant_id}/{project_id} |  |
| [**list_provider_secrets**](ProviderSecretsApi.md#list_provider_secrets) | **GET** /v1/provider-secrets/{tenant_id}/{project_id} |  |
| [**revoke_provider_secret**](ProviderSecretsApi.md#revoke_provider_secret) | **POST** /v1/provider-secrets/{tenant_id}/{project_id}/{provider_secret_id}/revoke |  |


## create_provider_secret

> <ProviderSecretMetadata> create_provider_secret(tenant_id, project_id, create_provider_secret_http_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ProviderSecretsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
create_provider_secret_http_request = BeaterClient::CreateProviderSecretHttpRequest.new({display_name: 'display_name_example', provider: 'provider_example', secret_value: 'secret_value_example'}) # CreateProviderSecretHttpRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.create_provider_secret(tenant_id, project_id, create_provider_secret_http_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ProviderSecretsApi->create_provider_secret: #{e}"
end
```

#### Using the create_provider_secret_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<ProviderSecretMetadata>, Integer, Hash)> create_provider_secret_with_http_info(tenant_id, project_id, create_provider_secret_http_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.create_provider_secret_with_http_info(tenant_id, project_id, create_provider_secret_http_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <ProviderSecretMetadata>
rescue BeaterClient::ApiError => e
  puts "Error when calling ProviderSecretsApi->create_provider_secret_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **create_provider_secret_http_request** | [**CreateProviderSecretHttpRequest**](CreateProviderSecretHttpRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**ProviderSecretMetadata**](ProviderSecretMetadata.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json


## list_provider_secrets

> <Array<ProviderSecretMetadata>> list_provider_secrets(tenant_id, project_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ProviderSecretsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.list_provider_secrets(tenant_id, project_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ProviderSecretsApi->list_provider_secrets: #{e}"
end
```

#### Using the list_provider_secrets_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<Array<ProviderSecretMetadata>>, Integer, Hash)> list_provider_secrets_with_http_info(tenant_id, project_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.list_provider_secrets_with_http_info(tenant_id, project_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <Array<ProviderSecretMetadata>>
rescue BeaterClient::ApiError => e
  puts "Error when calling ProviderSecretsApi->list_provider_secrets_with_http_info: #{e}"
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

[**Array&lt;ProviderSecretMetadata&gt;**](ProviderSecretMetadata.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json


## revoke_provider_secret

> <RevokedProviderSecret> revoke_provider_secret(tenant_id, project_id, provider_secret_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ProviderSecretsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
provider_secret_id = 'provider_secret_id_example' # String | provider_secret_id
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.revoke_provider_secret(tenant_id, project_id, provider_secret_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ProviderSecretsApi->revoke_provider_secret: #{e}"
end
```

#### Using the revoke_provider_secret_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<RevokedProviderSecret>, Integer, Hash)> revoke_provider_secret_with_http_info(tenant_id, project_id, provider_secret_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.revoke_provider_secret_with_http_info(tenant_id, project_id, provider_secret_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <RevokedProviderSecret>
rescue BeaterClient::ApiError => e
  puts "Error when calling ProviderSecretsApi->revoke_provider_secret_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **provider_secret_id** | **String** | provider_secret_id |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**RevokedProviderSecret**](RevokedProviderSecret.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json
