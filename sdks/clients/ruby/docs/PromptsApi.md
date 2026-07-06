# BeaterClient::PromptsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------ | ------------ | ----------- |
| [**add_prompt_version**](PromptsApi.md#add_prompt_version) | **POST** /v1/prompts/{tenant_id}/{project_id}/{prompt_id}/versions |  |
| [**create_prompt**](PromptsApi.md#create_prompt) | **POST** /v1/prompts/{tenant_id}/{project_id} |  |
| [**diff_prompt_versions**](PromptsApi.md#diff_prompt_versions) | **GET** /v1/prompts/{tenant_id}/{project_id}/{prompt_id}/diff |  |
| [**get_prompt**](PromptsApi.md#get_prompt) | **GET** /v1/prompts/{tenant_id}/{project_id}/{prompt_id} |  |
| [**list_prompt_versions**](PromptsApi.md#list_prompt_versions) | **GET** /v1/prompts/{tenant_id}/{project_id}/{prompt_id}/versions |  |
| [**list_prompts**](PromptsApi.md#list_prompts) | **GET** /v1/prompts/{tenant_id}/{project_id} |  |


## add_prompt_version

> <PromptVersion> add_prompt_version(tenant_id, project_id, prompt_id, add_prompt_version_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::PromptsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
prompt_id = 'prompt_id_example' # String | prompt_id
add_prompt_version_request = BeaterClient::AddPromptVersionRequest.new({template: BeaterClient::PromptTemplate.new({body: 'body_example', tags: ['tags_example'], variables: [BeaterClient::PromptVariable.new({name: 'name_example', required: false})]})}) # AddPromptVersionRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.add_prompt_version(tenant_id, project_id, prompt_id, add_prompt_version_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling PromptsApi->add_prompt_version: #{e}"
end
```

#### Using the add_prompt_version_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<PromptVersion>, Integer, Hash)> add_prompt_version_with_http_info(tenant_id, project_id, prompt_id, add_prompt_version_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.add_prompt_version_with_http_info(tenant_id, project_id, prompt_id, add_prompt_version_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <PromptVersion>
rescue BeaterClient::ApiError => e
  puts "Error when calling PromptsApi->add_prompt_version_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **prompt_id** | **String** | prompt_id |  |
| **add_prompt_version_request** | [**AddPromptVersionRequest**](AddPromptVersionRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**PromptVersion**](PromptVersion.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json


## create_prompt

> <CreatedPrompt> create_prompt(tenant_id, project_id, create_prompt_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::PromptsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
create_prompt_request = BeaterClient::CreatePromptRequest.new({name: 'name_example', template: BeaterClient::PromptTemplate.new({body: 'body_example', tags: ['tags_example'], variables: [BeaterClient::PromptVariable.new({name: 'name_example', required: false})]})}) # CreatePromptRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.create_prompt(tenant_id, project_id, create_prompt_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling PromptsApi->create_prompt: #{e}"
end
```

#### Using the create_prompt_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<CreatedPrompt>, Integer, Hash)> create_prompt_with_http_info(tenant_id, project_id, create_prompt_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.create_prompt_with_http_info(tenant_id, project_id, create_prompt_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <CreatedPrompt>
rescue BeaterClient::ApiError => e
  puts "Error when calling PromptsApi->create_prompt_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **create_prompt_request** | [**CreatePromptRequest**](CreatePromptRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**CreatedPrompt**](CreatedPrompt.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json


## diff_prompt_versions

> <PromptVersionDiff> diff_prompt_versions(tenant_id, project_id, prompt_id, from, to, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::PromptsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
prompt_id = 'prompt_id_example' # String | prompt_id
from = 'from_example' # String |
to = 'to_example' # String |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.diff_prompt_versions(tenant_id, project_id, prompt_id, from, to, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling PromptsApi->diff_prompt_versions: #{e}"
end
```

#### Using the diff_prompt_versions_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<PromptVersionDiff>, Integer, Hash)> diff_prompt_versions_with_http_info(tenant_id, project_id, prompt_id, from, to, opts)

```ruby
begin

  data, status_code, headers = api_instance.diff_prompt_versions_with_http_info(tenant_id, project_id, prompt_id, from, to, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <PromptVersionDiff>
rescue BeaterClient::ApiError => e
  puts "Error when calling PromptsApi->diff_prompt_versions_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **prompt_id** | **String** | prompt_id |  |
| **from** | **String** |  |  |
| **to** | **String** |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**PromptVersionDiff**](PromptVersionDiff.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json


## get_prompt

> <Prompt> get_prompt(tenant_id, project_id, prompt_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::PromptsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
prompt_id = 'prompt_id_example' # String | prompt_id
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.get_prompt(tenant_id, project_id, prompt_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling PromptsApi->get_prompt: #{e}"
end
```

#### Using the get_prompt_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<Prompt>, Integer, Hash)> get_prompt_with_http_info(tenant_id, project_id, prompt_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.get_prompt_with_http_info(tenant_id, project_id, prompt_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <Prompt>
rescue BeaterClient::ApiError => e
  puts "Error when calling PromptsApi->get_prompt_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **prompt_id** | **String** | prompt_id |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**Prompt**](Prompt.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json


## list_prompt_versions

> <PromptVersionListResponse> list_prompt_versions(tenant_id, project_id, prompt_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::PromptsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
prompt_id = 'prompt_id_example' # String | prompt_id
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.list_prompt_versions(tenant_id, project_id, prompt_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling PromptsApi->list_prompt_versions: #{e}"
end
```

#### Using the list_prompt_versions_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<PromptVersionListResponse>, Integer, Hash)> list_prompt_versions_with_http_info(tenant_id, project_id, prompt_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.list_prompt_versions_with_http_info(tenant_id, project_id, prompt_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <PromptVersionListResponse>
rescue BeaterClient::ApiError => e
  puts "Error when calling PromptsApi->list_prompt_versions_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **prompt_id** | **String** | prompt_id |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**PromptVersionListResponse**](PromptVersionListResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json


## list_prompts

> <PromptListResponse> list_prompts(tenant_id, project_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::PromptsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.list_prompts(tenant_id, project_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling PromptsApi->list_prompts: #{e}"
end
```

#### Using the list_prompts_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<PromptListResponse>, Integer, Hash)> list_prompts_with_http_info(tenant_id, project_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.list_prompts_with_http_info(tenant_id, project_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <PromptListResponse>
rescue BeaterClient::ApiError => e
  puts "Error when calling PromptsApi->list_prompts_with_http_info: #{e}"
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

[**PromptListResponse**](PromptListResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json
