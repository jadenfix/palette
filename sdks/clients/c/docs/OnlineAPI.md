# OnlineAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**OnlineAPI_onlineDecideOnlineSampling**](OnlineAPI.md#OnlineAPI_onlineDecideOnlineSampling) | **POST** /v1/online/{tenant_id}/{project_id}/traces/{trace_id}/sampling |


# **OnlineAPI_onlineDecideOnlineSampling**
```c
sampling_decision_t* OnlineAPI_onlineDecideOnlineSampling(apiClient_t *apiClient, char *tenant_id, char *project_id, char *trace_id, online_sampling_policy_t *online_sampling_policy, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**trace_id** | **char \*** | trace_id |
**online_sampling_policy** | **[online_sampling_policy_t](online_sampling_policy.md) \*** |  |
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[sampling_decision_t](sampling_decision.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
