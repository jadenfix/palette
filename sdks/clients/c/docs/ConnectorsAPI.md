# ConnectorsAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**ConnectorsAPI_connectConnector**](ConnectorsAPI.md#ConnectorsAPI_connectConnector) | **POST** /v1/connectors/{tenant_id}/{project_id}/connect | 
[**ConnectorsAPI_connectorStatus**](ConnectorsAPI.md#ConnectorsAPI_connectorStatus) | **GET** /v1/connectors/{tenant_id}/{project_id}/status | 
[**ConnectorsAPI_getConnectorSkills**](ConnectorsAPI.md#ConnectorsAPI_getConnectorSkills) | **GET** /v1/connectors/{tenant_id}/{project_id}/skills | 
[**ConnectorsAPI_invokeConnectorTool**](ConnectorsAPI.md#ConnectorsAPI_invokeConnectorTool) | **POST** /v1/connectors/{tenant_id}/{project_id}/invoke | 
[**ConnectorsAPI_listConnectorTools**](ConnectorsAPI.md#ConnectorsAPI_listConnectorTools) | **GET** /v1/connectors/{tenant_id}/{project_id}/tools | 
[**ConnectorsAPI_listConnectors**](ConnectorsAPI.md#ConnectorsAPI_listConnectors) | **GET** /v1/connectors/{tenant_id}/{project_id} | 


# **ConnectorsAPI_connectConnector**
```c
connection_link_t* ConnectorsAPI_connectConnector(apiClient_t *apiClient, char *tenant_id, char *project_id, connect_connector_request_t *connect_connector_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id | 
**project_id** | **char \*** | project_id | 
**connect_connector_request** | **[connect_connector_request_t](connect_connector_request.md) \*** |  | 
**authorization** | **char \*** | Bearer API token for strict auth | [optional] 
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional] 
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional] 
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional] 

### Return type

[connection_link_t](connection_link.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **ConnectorsAPI_connectorStatus**
```c
connection_status_t* ConnectorsAPI_connectorStatus(apiClient_t *apiClient, char *tenant_id, char *project_id, char *toolkit, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id | 
**project_id** | **char \*** | project_id | 
**toolkit** | **char \*** | Toolkit slug to scope the request to. | 
**authorization** | **char \*** | Bearer API token for strict auth | [optional] 
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional] 
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional] 
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional] 

### Return type

[connection_status_t](connection_status.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **ConnectorsAPI_getConnectorSkills**
```c
connector_skills_response_t* ConnectorsAPI_getConnectorSkills(apiClient_t *apiClient, char *tenant_id, char *project_id, char *toolkit, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id | 
**project_id** | **char \*** | project_id | 
**toolkit** | **char \*** | Toolkit slug to scope the request to. | 
**authorization** | **char \*** | Bearer API token for strict auth | [optional] 
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional] 
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional] 
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional] 

### Return type

[connector_skills_response_t](connector_skills_response.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **ConnectorsAPI_invokeConnectorTool**
```c
tool_execution_t* ConnectorsAPI_invokeConnectorTool(apiClient_t *apiClient, char *tenant_id, char *project_id, invoke_connector_request_t *invoke_connector_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id | 
**project_id** | **char \*** | project_id | 
**invoke_connector_request** | **[invoke_connector_request_t](invoke_connector_request.md) \*** |  | 
**authorization** | **char \*** | Bearer API token for strict auth | [optional] 
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional] 
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional] 
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional] 

### Return type

[tool_execution_t](tool_execution.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **ConnectorsAPI_listConnectorTools**
```c
list_t* ConnectorsAPI_listConnectorTools(apiClient_t *apiClient, char *tenant_id, char *project_id, char *toolkit, int *limit, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id | 
**project_id** | **char \*** | project_id | 
**toolkit** | **char \*** | Toolkit slug to list tools for. | 
**limit** | **int \*** | Maximum number of tools to return (page size). | 
**authorization** | **char \*** | Bearer API token for strict auth | [optional] 
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional] 
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional] 
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional] 

### Return type

[list_t](connector_tool.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **ConnectorsAPI_listConnectors**
```c
list_t* ConnectorsAPI_listConnectors(apiClient_t *apiClient, char *tenant_id, char *project_id, int *limit, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id | 
**project_id** | **char \*** | project_id | 
**limit** | **int \*** | Maximum number of apps to return (page size). | 
**authorization** | **char \*** | Bearer API token for strict auth | [optional] 
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional] 
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional] 
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional] 

### Return type

[list_t](toolkit.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

