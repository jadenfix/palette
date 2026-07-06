# BeaterClient::AlertsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------ | ------------ | ----------- |
| [**evaluate_alert**](AlertsApi.md#evaluate_alert) | **POST** /v1/alerts/{tenant_id}/{project_id}/traces/{trace_id}/webhook |  |


## evaluate_alert

> <AlertDecision> evaluate_alert(tenant_id, project_id, trace_id, evaluate_alert_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::AlertsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
trace_id = 'trace_id_example' # String | trace_id
evaluate_alert_request = BeaterClient::EvaluateAlertRequest.new({input: BeaterClient::AlertInput.new({group_key: 'group_key_example', links: BeaterClient::AlertLinks.new({trace_url: 'trace_url_example'}), now: Time.now, project_id: 'project_id_example', score: 3.56, tenant_id: 'tenant_id_example', title: 'title_example', trace_id: 'trace_id_example'}), policy: BeaterClient::AlertPolicy.new({dedupe_window_seconds: 3.56, endpoint_url: 'endpoint_url_example', fire_when_score_at_or_below: 3.56, maintenance_windows: [BeaterClient::MaintenanceWindow.new({ends_at: Time.now, starts_at: Time.now})], policy_id: 'policy_id_example', severity: BeaterClient::AlertSeverity::INFO, signing_secret: 'signing_secret_example'})}) # EvaluateAlertRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.evaluate_alert(tenant_id, project_id, trace_id, evaluate_alert_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling AlertsApi->evaluate_alert: #{e}"
end
```

#### Using the evaluate_alert_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<AlertDecision>, Integer, Hash)> evaluate_alert_with_http_info(tenant_id, project_id, trace_id, evaluate_alert_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.evaluate_alert_with_http_info(tenant_id, project_id, trace_id, evaluate_alert_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <AlertDecision>
rescue BeaterClient::ApiError => e
  puts "Error when calling AlertsApi->evaluate_alert_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **trace_id** | **String** | trace_id |  |
| **evaluate_alert_request** | [**EvaluateAlertRequest**](EvaluateAlertRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**AlertDecision**](AlertDecision.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json
