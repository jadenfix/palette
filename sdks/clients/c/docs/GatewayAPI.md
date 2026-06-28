# GatewayAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**GatewayAPI_gatewayChatCompletions**](GatewayAPI.md#GatewayAPI_gatewayChatCompletions) | **POST** /v1/gateway/{tenant_id}/{project_id}/{environment_id}/chat/completions | 


# **GatewayAPI_gatewayChatCompletions**
```c
gateway_outcome_t* GatewayAPI_gatewayChatCompletions(apiClient_t *apiClient, char *tenant_id, char *project_id, char *environment_id, gateway_chat_http_request_t *gateway_chat_http_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id | 
**project_id** | **char \*** | project_id | 
**environment_id** | **char \*** | environment_id | 
**gateway_chat_http_request** | **[gateway_chat_http_request_t](gateway_chat_http_request.md) \*** |  | 
**authorization** | **char \*** | Bearer API token for strict auth | [optional] 
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional] 
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional] 
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional] 

### Return type

[gateway_outcome_t](gateway_outcome.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

