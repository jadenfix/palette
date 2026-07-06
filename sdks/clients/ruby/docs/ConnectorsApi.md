# BeaterClient::ConnectorsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------ | ------------ | ----------- |
| [**connect_connector**](ConnectorsApi.md#connect_connector) | **POST** /v1/connectors/{tenant_id}/{project_id}/connect |  |
| [**connector_status**](ConnectorsApi.md#connector_status) | **GET** /v1/connectors/{tenant_id}/{project_id}/status |  |
| [**get_connector_skills**](ConnectorsApi.md#get_connector_skills) | **GET** /v1/connectors/{tenant_id}/{project_id}/skills |  |
| [**invoke_connector_tool**](ConnectorsApi.md#invoke_connector_tool) | **POST** /v1/connectors/{tenant_id}/{project_id}/invoke |  |
| [**list_connector_tools**](ConnectorsApi.md#list_connector_tools) | **GET** /v1/connectors/{tenant_id}/{project_id}/tools |  |
| [**list_connectors**](ConnectorsApi.md#list_connectors) | **GET** /v1/connectors/{tenant_id}/{project_id} |  |


## connect_connector

> <ConnectionLink> connect_connector(tenant_id, project_id, connect_connector_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ConnectorsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
connect_connector_request = BeaterClient::ConnectConnectorRequest.new({toolkit: 'toolkit_example'}) # ConnectConnectorRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.connect_connector(tenant_id, project_id, connect_connector_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ConnectorsApi->connect_connector: #{e}"
end
```

#### Using the connect_connector_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<ConnectionLink>, Integer, Hash)> connect_connector_with_http_info(tenant_id, project_id, connect_connector_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.connect_connector_with_http_info(tenant_id, project_id, connect_connector_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <ConnectionLink>
rescue BeaterClient::ApiError => e
  puts "Error when calling ConnectorsApi->connect_connector_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **connect_connector_request** | [**ConnectConnectorRequest**](ConnectConnectorRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**ConnectionLink**](ConnectionLink.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json


## connector_status

> <ConnectionStatus> connector_status(tenant_id, project_id, toolkit, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ConnectorsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
toolkit = 'toolkit_example' # String | Toolkit slug to scope the request to.
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.connector_status(tenant_id, project_id, toolkit, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ConnectorsApi->connector_status: #{e}"
end
```

#### Using the connector_status_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<ConnectionStatus>, Integer, Hash)> connector_status_with_http_info(tenant_id, project_id, toolkit, opts)

```ruby
begin

  data, status_code, headers = api_instance.connector_status_with_http_info(tenant_id, project_id, toolkit, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <ConnectionStatus>
rescue BeaterClient::ApiError => e
  puts "Error when calling ConnectorsApi->connector_status_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **toolkit** | **String** | Toolkit slug to scope the request to. |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**ConnectionStatus**](ConnectionStatus.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json


## get_connector_skills

> <ConnectorSkillsResponse> get_connector_skills(tenant_id, project_id, toolkit, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ConnectorsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
toolkit = 'toolkit_example' # String | Toolkit slug to scope the request to.
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.get_connector_skills(tenant_id, project_id, toolkit, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ConnectorsApi->get_connector_skills: #{e}"
end
```

#### Using the get_connector_skills_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<ConnectorSkillsResponse>, Integer, Hash)> get_connector_skills_with_http_info(tenant_id, project_id, toolkit, opts)

```ruby
begin

  data, status_code, headers = api_instance.get_connector_skills_with_http_info(tenant_id, project_id, toolkit, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <ConnectorSkillsResponse>
rescue BeaterClient::ApiError => e
  puts "Error when calling ConnectorsApi->get_connector_skills_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **toolkit** | **String** | Toolkit slug to scope the request to. |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**ConnectorSkillsResponse**](ConnectorSkillsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json


## invoke_connector_tool

> <ToolExecution> invoke_connector_tool(tenant_id, project_id, invoke_connector_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ConnectorsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
invoke_connector_request = BeaterClient::InvokeConnectorRequest.new({tool: 'tool_example'}) # InvokeConnectorRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.invoke_connector_tool(tenant_id, project_id, invoke_connector_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ConnectorsApi->invoke_connector_tool: #{e}"
end
```

#### Using the invoke_connector_tool_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<ToolExecution>, Integer, Hash)> invoke_connector_tool_with_http_info(tenant_id, project_id, invoke_connector_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.invoke_connector_tool_with_http_info(tenant_id, project_id, invoke_connector_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <ToolExecution>
rescue BeaterClient::ApiError => e
  puts "Error when calling ConnectorsApi->invoke_connector_tool_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **invoke_connector_request** | [**InvokeConnectorRequest**](InvokeConnectorRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**ToolExecution**](ToolExecution.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json


## list_connector_tools

> <Array<ConnectorTool>> list_connector_tools(tenant_id, project_id, toolkit, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ConnectorsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
toolkit = 'toolkit_example' # String | Toolkit slug to list tools for.
opts = {
  limit: 56, # Integer | Maximum number of tools to return (page size).
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.list_connector_tools(tenant_id, project_id, toolkit, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ConnectorsApi->list_connector_tools: #{e}"
end
```

#### Using the list_connector_tools_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<Array<ConnectorTool>>, Integer, Hash)> list_connector_tools_with_http_info(tenant_id, project_id, toolkit, opts)

```ruby
begin

  data, status_code, headers = api_instance.list_connector_tools_with_http_info(tenant_id, project_id, toolkit, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <Array<ConnectorTool>>
rescue BeaterClient::ApiError => e
  puts "Error when calling ConnectorsApi->list_connector_tools_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **toolkit** | **String** | Toolkit slug to list tools for. |  |
| **limit** | **Integer** | Maximum number of tools to return (page size). | [optional] |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**Array&lt;ConnectorTool&gt;**](ConnectorTool.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json


## list_connectors

> <Array<Toolkit>> list_connectors(tenant_id, project_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ConnectorsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
opts = {
  limit: 56, # Integer | Maximum number of apps to return (page size).
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.list_connectors(tenant_id, project_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ConnectorsApi->list_connectors: #{e}"
end
```

#### Using the list_connectors_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<Array<Toolkit>>, Integer, Hash)> list_connectors_with_http_info(tenant_id, project_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.list_connectors_with_http_info(tenant_id, project_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <Array<Toolkit>>
rescue BeaterClient::ApiError => e
  puts "Error when calling ConnectorsApi->list_connectors_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **limit** | **Integer** | Maximum number of apps to return (page size). | [optional] |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**Array&lt;Toolkit&gt;**](Toolkit.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json
