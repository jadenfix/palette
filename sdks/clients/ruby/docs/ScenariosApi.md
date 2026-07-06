# BeaterClient::ScenariosApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------ | ------------ | ----------- |
| [**create_scenario**](ScenariosApi.md#create_scenario) | **POST** /v1/scenarios/{tenant_id}/{project_id} |  |
| [**get_scenario**](ScenariosApi.md#get_scenario) | **GET** /v1/scenarios/{tenant_id}/{project_id}/{scenario_id} |  |
| [**list_scenarios**](ScenariosApi.md#list_scenarios) | **GET** /v1/scenarios/{tenant_id}/{project_id} |  |
| [**mine_scenarios**](ScenariosApi.md#mine_scenarios) | **POST** /v1/scenarios/{tenant_id}/{project_id}/mine |  |


## create_scenario

> <Scenario> create_scenario(tenant_id, project_id, create_scenario_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ScenariosApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
create_scenario_request = BeaterClient::CreateScenarioRequest.new({source_trace_ids: ['source_trace_ids_example'], title: 'title_example'}) # CreateScenarioRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.create_scenario(tenant_id, project_id, create_scenario_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ScenariosApi->create_scenario: #{e}"
end
```

#### Using the create_scenario_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<Scenario>, Integer, Hash)> create_scenario_with_http_info(tenant_id, project_id, create_scenario_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.create_scenario_with_http_info(tenant_id, project_id, create_scenario_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <Scenario>
rescue BeaterClient::ApiError => e
  puts "Error when calling ScenariosApi->create_scenario_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **create_scenario_request** | [**CreateScenarioRequest**](CreateScenarioRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**Scenario**](Scenario.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json


## get_scenario

> <Scenario> get_scenario(tenant_id, project_id, scenario_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ScenariosApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
scenario_id = 'scenario_id_example' # String | scenario_id
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.get_scenario(tenant_id, project_id, scenario_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ScenariosApi->get_scenario: #{e}"
end
```

#### Using the get_scenario_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<Scenario>, Integer, Hash)> get_scenario_with_http_info(tenant_id, project_id, scenario_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.get_scenario_with_http_info(tenant_id, project_id, scenario_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <Scenario>
rescue BeaterClient::ApiError => e
  puts "Error when calling ScenariosApi->get_scenario_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **scenario_id** | **String** | scenario_id |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**Scenario**](Scenario.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json


## list_scenarios

> <ListScenariosResponse> list_scenarios(tenant_id, project_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ScenariosApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
opts = {
  limit: 56, # Integer |
  cursor: 'cursor_example', # String |
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.list_scenarios(tenant_id, project_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ScenariosApi->list_scenarios: #{e}"
end
```

#### Using the list_scenarios_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<ListScenariosResponse>, Integer, Hash)> list_scenarios_with_http_info(tenant_id, project_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.list_scenarios_with_http_info(tenant_id, project_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <ListScenariosResponse>
rescue BeaterClient::ApiError => e
  puts "Error when calling ScenariosApi->list_scenarios_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **limit** | **Integer** |  | [optional] |
| **cursor** | **String** |  | [optional] |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**ListScenariosResponse**](ListScenariosResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json


## mine_scenarios

> <MineScenariosResponse> mine_scenarios(tenant_id, project_id, mine_scenarios_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::ScenariosApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
mine_scenarios_request = BeaterClient::MineScenariosRequest.new({trace_ids: ['trace_ids_example']}) # MineScenariosRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.mine_scenarios(tenant_id, project_id, mine_scenarios_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling ScenariosApi->mine_scenarios: #{e}"
end
```

#### Using the mine_scenarios_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<MineScenariosResponse>, Integer, Hash)> mine_scenarios_with_http_info(tenant_id, project_id, mine_scenarios_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.mine_scenarios_with_http_info(tenant_id, project_id, mine_scenarios_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <MineScenariosResponse>
rescue BeaterClient::ApiError => e
  puts "Error when calling ScenariosApi->mine_scenarios_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **mine_scenarios_request** | [**MineScenariosRequest**](MineScenariosRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**MineScenariosResponse**](MineScenariosResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json
