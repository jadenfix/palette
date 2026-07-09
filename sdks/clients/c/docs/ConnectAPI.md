# ConnectAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**ConnectAPI_connectGetPaletteConnectStatus**](ConnectAPI.md#ConnectAPI_connectGetPaletteConnectStatus) | **GET** /v1/connect/status/{tenant_id}/{project_id} |


# **ConnectAPI_connectGetPaletteConnectStatus**
```c
palette_connect_status_response_t* ConnectAPI_connectGetPaletteConnectStatus(apiClient_t *apiClient, char *tenant_id, char *project_id, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[palette_connect_status_response_t](palette_connect_status_response.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
