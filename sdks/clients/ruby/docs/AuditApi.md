# BeaterClient::AuditApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------ | ------------ | ----------- |
| [**list_audit_events**](AuditApi.md#list_audit_events) | **GET** /v1/audit/{tenant_id}/{project_id} |  |


## list_audit_events

> <Array<AuditEvent>> list_audit_events(tenant_id, project_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::AuditApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.list_audit_events(tenant_id, project_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling AuditApi->list_audit_events: #{e}"
end
```

#### Using the list_audit_events_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<Array<AuditEvent>>, Integer, Hash)> list_audit_events_with_http_info(tenant_id, project_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.list_audit_events_with_http_info(tenant_id, project_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <Array<AuditEvent>>
rescue BeaterClient::ApiError => e
  puts "Error when calling AuditApi->list_audit_events_with_http_info: #{e}"
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

[**Array&lt;AuditEvent&gt;**](AuditEvent.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json
