# beater_client.AlertsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**alerts_evaluate_alert**](AlertsApi.md#alerts_evaluate_alert) | **POST** /v1/alerts/{tenant_id}/{project_id}/traces/{trace_id}/webhook |


# **alerts_evaluate_alert**
> AlertDecision alerts_evaluate_alert(tenant_id, project_id, trace_id, evaluate_alert_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.alert_decision import AlertDecision
from beater_client.models.evaluate_alert_request import EvaluateAlertRequest
from beater_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = beater_client.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with beater_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = beater_client.AlertsApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    trace_id = 'trace_id_example' # str | trace_id
    evaluate_alert_request = beater_client.EvaluateAlertRequest() # EvaluateAlertRequest |
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.alerts_evaluate_alert(tenant_id, project_id, trace_id, evaluate_alert_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of AlertsApi->alerts_evaluate_alert:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AlertsApi->alerts_evaluate_alert: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id |
 **project_id** | **str**| project_id |
 **trace_id** | **str**| trace_id |
 **evaluate_alert_request** | [**EvaluateAlertRequest**](EvaluateAlertRequest.md)|  |
 **authorization** | **str**| Bearer API token for strict auth | [optional]
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional]
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional]
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional]

### Return type

[**AlertDecision**](AlertDecision.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Evaluate an alert policy for a trace |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
