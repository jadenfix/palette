# \GatewayApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**gateway_chat_completions**](GatewayApi.md#gateway_chat_completions) | **POST** /v1/gateway/{tenant_id}/{project_id}/{environment_id}/chat/completions | 



## gateway_chat_completions

> models::GatewayOutcome gateway_chat_completions(tenant_id, project_id, environment_id, gateway_chat_http_request, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**project_id** | **String** | project_id | [required] |
**environment_id** | **String** | environment_id | [required] |
**gateway_chat_http_request** | [**GatewayChatHttpRequest**](GatewayChatHttpRequest.md) |  | [required] |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::GatewayOutcome**](GatewayOutcome.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

