# ConnectorsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------------- | ------------- | ------------- |
| [**connectConnector**](ConnectorsApi.md#connectConnector) | **POST** /v1/connectors/{tenant_id}/{project_id}/connect |  |
| [**connectorStatus**](ConnectorsApi.md#connectorStatus) | **GET** /v1/connectors/{tenant_id}/{project_id}/status |  |
| [**getConnectorSkills**](ConnectorsApi.md#getConnectorSkills) | **GET** /v1/connectors/{tenant_id}/{project_id}/skills |  |
| [**invokeConnectorTool**](ConnectorsApi.md#invokeConnectorTool) | **POST** /v1/connectors/{tenant_id}/{project_id}/invoke |  |
| [**listConnectorTools**](ConnectorsApi.md#listConnectorTools) | **GET** /v1/connectors/{tenant_id}/{project_id}/tools |  |
| [**listConnectors**](ConnectorsApi.md#listConnectors) | **GET** /v1/connectors/{tenant_id}/{project_id} |  |


<a id="connectConnector"></a>
# **connectConnector**
> ConnectionLink connectConnector(tenantId, projectId, connectConnectorRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = ConnectorsApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val connectConnectorRequest : ConnectConnectorRequest =  // ConnectConnectorRequest |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : ConnectionLink = apiInstance.connectConnector(tenantId, projectId, connectConnectorRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling ConnectorsApi#connectConnector")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling ConnectorsApi#connectConnector")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **connectConnectorRequest** | [**ConnectConnectorRequest**](ConnectConnectorRequest.md)|  | |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**ConnectionLink**](ConnectionLink.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

<a id="connectorStatus"></a>
# **connectorStatus**
> ConnectionStatus connectorStatus(tenantId, projectId, toolkit, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = ConnectorsApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val toolkit : kotlin.String = toolkit_example // kotlin.String | Toolkit slug to scope the request to.
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : ConnectionStatus = apiInstance.connectorStatus(tenantId, projectId, toolkit, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling ConnectorsApi#connectorStatus")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling ConnectorsApi#connectorStatus")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **toolkit** | **kotlin.String**| Toolkit slug to scope the request to. | |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**ConnectionStatus**](ConnectionStatus.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

<a id="getConnectorSkills"></a>
# **getConnectorSkills**
> ConnectorSkillsResponse getConnectorSkills(tenantId, projectId, toolkit, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = ConnectorsApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val toolkit : kotlin.String = toolkit_example // kotlin.String | Toolkit slug to scope the request to.
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : ConnectorSkillsResponse = apiInstance.getConnectorSkills(tenantId, projectId, toolkit, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling ConnectorsApi#getConnectorSkills")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling ConnectorsApi#getConnectorSkills")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **toolkit** | **kotlin.String**| Toolkit slug to scope the request to. | |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**ConnectorSkillsResponse**](ConnectorSkillsResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

<a id="invokeConnectorTool"></a>
# **invokeConnectorTool**
> ToolExecution invokeConnectorTool(tenantId, projectId, invokeConnectorRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = ConnectorsApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val invokeConnectorRequest : InvokeConnectorRequest =  // InvokeConnectorRequest |
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : ToolExecution = apiInstance.invokeConnectorTool(tenantId, projectId, invokeConnectorRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling ConnectorsApi#invokeConnectorTool")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling ConnectorsApi#invokeConnectorTool")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **invokeConnectorRequest** | [**InvokeConnectorRequest**](InvokeConnectorRequest.md)|  | |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**ToolExecution**](ToolExecution.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

<a id="listConnectorTools"></a>
# **listConnectorTools**
> kotlin.collections.List&lt;ConnectorTool&gt; listConnectorTools(tenantId, projectId, toolkit, limit, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = ConnectorsApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val toolkit : kotlin.String = toolkit_example // kotlin.String | Toolkit slug to list tools for.
val limit : kotlin.Int = 56 // kotlin.Int | Maximum number of tools to return (page size).
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : kotlin.collections.List<ConnectorTool> = apiInstance.listConnectorTools(tenantId, projectId, toolkit, limit, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling ConnectorsApi#listConnectorTools")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling ConnectorsApi#listConnectorTools")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **toolkit** | **kotlin.String**| Toolkit slug to list tools for. | |
| **limit** | **kotlin.Int**| Maximum number of tools to return (page size). | [optional] |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**kotlin.collections.List&lt;ConnectorTool&gt;**](ConnectorTool.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

<a id="listConnectors"></a>
# **listConnectors**
> kotlin.collections.List&lt;Toolkit&gt; listConnectors(tenantId, projectId, limit, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = ConnectorsApi()
val tenantId : kotlin.String = tenantId_example // kotlin.String | tenant_id
val projectId : kotlin.String = projectId_example // kotlin.String | project_id
val limit : kotlin.Int = 56 // kotlin.Int | Maximum number of apps to return (page size).
val authorization : kotlin.String = authorization_example // kotlin.String | Bearer API token for strict auth
val xBeaterApiKey : kotlin.String = xBeaterApiKey_example // kotlin.String | API key alternative for strict auth
val xBeaterProjectId : kotlin.String = xBeaterProjectId_example // kotlin.String | Strict-auth project scope
val xBeaterEnvironmentId : kotlin.String = xBeaterEnvironmentId_example // kotlin.String | Strict-auth environment scope
try {
    val result : kotlin.collections.List<Toolkit> = apiInstance.listConnectors(tenantId, projectId, limit, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)
    println(result)
} catch (e: ClientException) {
    println("4xx response calling ConnectorsApi#listConnectors")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling ConnectorsApi#listConnectors")
    e.printStackTrace()
}
```

### Parameters
| **tenantId** | **kotlin.String**| tenant_id | |
| **projectId** | **kotlin.String**| project_id | |
| **limit** | **kotlin.Int**| Maximum number of apps to return (page size). | [optional] |
| **authorization** | **kotlin.String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **kotlin.String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **kotlin.String**| Strict-auth project scope | [optional] |
| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **xBeaterEnvironmentId** | **kotlin.String**| Strict-auth environment scope | [optional] |

### Return type

[**kotlin.collections.List&lt;Toolkit&gt;**](Toolkit.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json
