# AlertsAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**AlertsAPI_alertsEvaluateAlert**](AlertsAPI.md#AlertsAPI_alertsEvaluateAlert) | **POST** /v1/alerts/{tenant_id}/{project_id}/traces/{trace_id}/webhook |


# **AlertsAPI_alertsEvaluateAlert**
```c
alert_decision_t* AlertsAPI_alertsEvaluateAlert(apiClient_t *apiClient, char *tenant_id, char *project_id, char *trace_id, evaluate_alert_request_t *evaluate_alert_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**trace_id** | **char \*** | trace_id |
**evaluate_alert_request** | **[evaluate_alert_request_t](evaluate_alert_request.md) \*** |  |
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[alert_decision_t](alert_decision.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
