# ScenariosApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**createScenario**](ScenariosApi.md#createScenario) | **POST** /v1/scenarios/{tenant_id}/{project_id} |  |
| [**createScenarioWithHttpInfo**](ScenariosApi.md#createScenarioWithHttpInfo) | **POST** /v1/scenarios/{tenant_id}/{project_id} |  |
| [**getScenario**](ScenariosApi.md#getScenario) | **GET** /v1/scenarios/{tenant_id}/{project_id}/{scenario_id} |  |
| [**getScenarioWithHttpInfo**](ScenariosApi.md#getScenarioWithHttpInfo) | **GET** /v1/scenarios/{tenant_id}/{project_id}/{scenario_id} |  |
| [**listScenarios**](ScenariosApi.md#listScenarios) | **GET** /v1/scenarios/{tenant_id}/{project_id} |  |
| [**listScenariosWithHttpInfo**](ScenariosApi.md#listScenariosWithHttpInfo) | **GET** /v1/scenarios/{tenant_id}/{project_id} |  |
| [**mineScenarios**](ScenariosApi.md#mineScenarios) | **POST** /v1/scenarios/{tenant_id}/{project_id}/mine |  |
| [**mineScenariosWithHttpInfo**](ScenariosApi.md#mineScenariosWithHttpInfo) | **POST** /v1/scenarios/{tenant_id}/{project_id}/mine |  |



## createScenario

> Scenario createScenario(tenantId, projectId, createScenarioRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example

```java
// Import classes:
import ai.beater.client.ApiClient;
import ai.beater.client.ApiException;
import ai.beater.client.Configuration;
import ai.beater.client.models.*;
import ai.beater.client.api.ScenariosApi;

public class Example {
    public static void main(String[] args) {
        ApiClient defaultClient = Configuration.getDefaultApiClient();
        defaultClient.setBasePath("http://localhost");

        ScenariosApi apiInstance = new ScenariosApi(defaultClient);
        String tenantId = "tenantId_example"; // String | tenant_id
        String projectId = "projectId_example"; // String | project_id
        CreateScenarioRequest createScenarioRequest = new CreateScenarioRequest(); // CreateScenarioRequest | 
        String authorization = "authorization_example"; // String | Bearer API token for strict auth
        String xBeaterApiKey = "xBeaterApiKey_example"; // String | API key alternative for strict auth
        String xBeaterProjectId = "xBeaterProjectId_example"; // String | Strict-auth project scope
        String xBeaterEnvironmentId = "xBeaterEnvironmentId_example"; // String | Strict-auth environment scope
        try {
            Scenario result = apiInstance.createScenario(tenantId, projectId, createScenarioRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
            System.out.println(result);
        } catch (ApiException e) {
            System.err.println("Exception when calling ScenariosApi#createScenario");
            System.err.println("Status code: " + e.getCode());
            System.err.println("Reason: " + e.getResponseBody());
            System.err.println("Response headers: " + e.getResponseHeaders());
            e.printStackTrace();
        }
    }
}
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **tenantId** | **String**| tenant_id | |
| **projectId** | **String**| project_id | |
| **createScenarioRequest** | [**CreateScenarioRequest**](CreateScenarioRequest.md)|  | |
| **authorization** | **String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **String**| Strict-auth project scope | [optional] |
| **xBeaterEnvironmentId** | **String**| Strict-auth environment scope | [optional] |

### Return type

[**Scenario**](Scenario.md)


### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Create a scenario |  -  |
| **400** | Invalid request, scope, or filter |  -  |
| **401** | Missing or invalid credentials |  -  |
| **403** | Credentials lack the required scope |  -  |

## createScenarioWithHttpInfo

> ApiResponse<Scenario> createScenario createScenarioWithHttpInfo(tenantId, projectId, createScenarioRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example

```java
// Import classes:
import ai.beater.client.ApiClient;
import ai.beater.client.ApiException;
import ai.beater.client.ApiResponse;
import ai.beater.client.Configuration;
import ai.beater.client.models.*;
import ai.beater.client.api.ScenariosApi;

public class Example {
    public static void main(String[] args) {
        ApiClient defaultClient = Configuration.getDefaultApiClient();
        defaultClient.setBasePath("http://localhost");

        ScenariosApi apiInstance = new ScenariosApi(defaultClient);
        String tenantId = "tenantId_example"; // String | tenant_id
        String projectId = "projectId_example"; // String | project_id
        CreateScenarioRequest createScenarioRequest = new CreateScenarioRequest(); // CreateScenarioRequest | 
        String authorization = "authorization_example"; // String | Bearer API token for strict auth
        String xBeaterApiKey = "xBeaterApiKey_example"; // String | API key alternative for strict auth
        String xBeaterProjectId = "xBeaterProjectId_example"; // String | Strict-auth project scope
        String xBeaterEnvironmentId = "xBeaterEnvironmentId_example"; // String | Strict-auth environment scope
        try {
            ApiResponse<Scenario> response = apiInstance.createScenarioWithHttpInfo(tenantId, projectId, createScenarioRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
            System.out.println("Status code: " + response.getStatusCode());
            System.out.println("Response headers: " + response.getHeaders());
            System.out.println("Response body: " + response.getData());
        } catch (ApiException e) {
            System.err.println("Exception when calling ScenariosApi#createScenario");
            System.err.println("Status code: " + e.getCode());
            System.err.println("Response headers: " + e.getResponseHeaders());
            System.err.println("Reason: " + e.getResponseBody());
            e.printStackTrace();
        }
    }
}
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **tenantId** | **String**| tenant_id | |
| **projectId** | **String**| project_id | |
| **createScenarioRequest** | [**CreateScenarioRequest**](CreateScenarioRequest.md)|  | |
| **authorization** | **String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **String**| Strict-auth project scope | [optional] |
| **xBeaterEnvironmentId** | **String**| Strict-auth environment scope | [optional] |

### Return type

ApiResponse<[**Scenario**](Scenario.md)>


### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Create a scenario |  -  |
| **400** | Invalid request, scope, or filter |  -  |
| **401** | Missing or invalid credentials |  -  |
| **403** | Credentials lack the required scope |  -  |


## getScenario

> Scenario getScenario(tenantId, projectId, scenarioId, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example

```java
// Import classes:
import ai.beater.client.ApiClient;
import ai.beater.client.ApiException;
import ai.beater.client.Configuration;
import ai.beater.client.models.*;
import ai.beater.client.api.ScenariosApi;

public class Example {
    public static void main(String[] args) {
        ApiClient defaultClient = Configuration.getDefaultApiClient();
        defaultClient.setBasePath("http://localhost");

        ScenariosApi apiInstance = new ScenariosApi(defaultClient);
        String tenantId = "tenantId_example"; // String | tenant_id
        String projectId = "projectId_example"; // String | project_id
        String scenarioId = "scenarioId_example"; // String | scenario_id
        String authorization = "authorization_example"; // String | Bearer API token for strict auth
        String xBeaterApiKey = "xBeaterApiKey_example"; // String | API key alternative for strict auth
        String xBeaterProjectId = "xBeaterProjectId_example"; // String | Strict-auth project scope
        String xBeaterEnvironmentId = "xBeaterEnvironmentId_example"; // String | Strict-auth environment scope
        try {
            Scenario result = apiInstance.getScenario(tenantId, projectId, scenarioId, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
            System.out.println(result);
        } catch (ApiException e) {
            System.err.println("Exception when calling ScenariosApi#getScenario");
            System.err.println("Status code: " + e.getCode());
            System.err.println("Reason: " + e.getResponseBody());
            System.err.println("Response headers: " + e.getResponseHeaders());
            e.printStackTrace();
        }
    }
}
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **tenantId** | **String**| tenant_id | |
| **projectId** | **String**| project_id | |
| **scenarioId** | **String**| scenario_id | |
| **authorization** | **String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **String**| Strict-auth project scope | [optional] |
| **xBeaterEnvironmentId** | **String**| Strict-auth environment scope | [optional] |

### Return type

[**Scenario**](Scenario.md)


### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Get a scenario |  -  |
| **400** | Invalid request, scope, or filter |  -  |
| **401** | Missing or invalid credentials |  -  |
| **403** | Credentials lack the required scope |  -  |
| **404** | Resource not found |  -  |

## getScenarioWithHttpInfo

> ApiResponse<Scenario> getScenario getScenarioWithHttpInfo(tenantId, projectId, scenarioId, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example

```java
// Import classes:
import ai.beater.client.ApiClient;
import ai.beater.client.ApiException;
import ai.beater.client.ApiResponse;
import ai.beater.client.Configuration;
import ai.beater.client.models.*;
import ai.beater.client.api.ScenariosApi;

public class Example {
    public static void main(String[] args) {
        ApiClient defaultClient = Configuration.getDefaultApiClient();
        defaultClient.setBasePath("http://localhost");

        ScenariosApi apiInstance = new ScenariosApi(defaultClient);
        String tenantId = "tenantId_example"; // String | tenant_id
        String projectId = "projectId_example"; // String | project_id
        String scenarioId = "scenarioId_example"; // String | scenario_id
        String authorization = "authorization_example"; // String | Bearer API token for strict auth
        String xBeaterApiKey = "xBeaterApiKey_example"; // String | API key alternative for strict auth
        String xBeaterProjectId = "xBeaterProjectId_example"; // String | Strict-auth project scope
        String xBeaterEnvironmentId = "xBeaterEnvironmentId_example"; // String | Strict-auth environment scope
        try {
            ApiResponse<Scenario> response = apiInstance.getScenarioWithHttpInfo(tenantId, projectId, scenarioId, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
            System.out.println("Status code: " + response.getStatusCode());
            System.out.println("Response headers: " + response.getHeaders());
            System.out.println("Response body: " + response.getData());
        } catch (ApiException e) {
            System.err.println("Exception when calling ScenariosApi#getScenario");
            System.err.println("Status code: " + e.getCode());
            System.err.println("Response headers: " + e.getResponseHeaders());
            System.err.println("Reason: " + e.getResponseBody());
            e.printStackTrace();
        }
    }
}
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **tenantId** | **String**| tenant_id | |
| **projectId** | **String**| project_id | |
| **scenarioId** | **String**| scenario_id | |
| **authorization** | **String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **String**| Strict-auth project scope | [optional] |
| **xBeaterEnvironmentId** | **String**| Strict-auth environment scope | [optional] |

### Return type

ApiResponse<[**Scenario**](Scenario.md)>


### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Get a scenario |  -  |
| **400** | Invalid request, scope, or filter |  -  |
| **401** | Missing or invalid credentials |  -  |
| **403** | Credentials lack the required scope |  -  |
| **404** | Resource not found |  -  |


## listScenarios

> ListScenariosResponse listScenarios(tenantId, projectId, limit, cursor, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example

```java
// Import classes:
import ai.beater.client.ApiClient;
import ai.beater.client.ApiException;
import ai.beater.client.Configuration;
import ai.beater.client.models.*;
import ai.beater.client.api.ScenariosApi;

public class Example {
    public static void main(String[] args) {
        ApiClient defaultClient = Configuration.getDefaultApiClient();
        defaultClient.setBasePath("http://localhost");

        ScenariosApi apiInstance = new ScenariosApi(defaultClient);
        String tenantId = "tenantId_example"; // String | tenant_id
        String projectId = "projectId_example"; // String | project_id
        Integer limit = 56; // Integer | 
        String cursor = "cursor_example"; // String | 
        String authorization = "authorization_example"; // String | Bearer API token for strict auth
        String xBeaterApiKey = "xBeaterApiKey_example"; // String | API key alternative for strict auth
        String xBeaterProjectId = "xBeaterProjectId_example"; // String | Strict-auth project scope
        String xBeaterEnvironmentId = "xBeaterEnvironmentId_example"; // String | Strict-auth environment scope
        try {
            ListScenariosResponse result = apiInstance.listScenarios(tenantId, projectId, limit, cursor, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
            System.out.println(result);
        } catch (ApiException e) {
            System.err.println("Exception when calling ScenariosApi#listScenarios");
            System.err.println("Status code: " + e.getCode());
            System.err.println("Reason: " + e.getResponseBody());
            System.err.println("Response headers: " + e.getResponseHeaders());
            e.printStackTrace();
        }
    }
}
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **tenantId** | **String**| tenant_id | |
| **projectId** | **String**| project_id | |
| **limit** | **Integer**|  | [optional] |
| **cursor** | **String**|  | [optional] |
| **authorization** | **String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **String**| Strict-auth project scope | [optional] |
| **xBeaterEnvironmentId** | **String**| Strict-auth environment scope | [optional] |

### Return type

[**ListScenariosResponse**](ListScenariosResponse.md)


### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | List scenarios |  -  |
| **400** | Invalid request, scope, or filter |  -  |
| **401** | Missing or invalid credentials |  -  |
| **403** | Credentials lack the required scope |  -  |

## listScenariosWithHttpInfo

> ApiResponse<ListScenariosResponse> listScenarios listScenariosWithHttpInfo(tenantId, projectId, limit, cursor, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example

```java
// Import classes:
import ai.beater.client.ApiClient;
import ai.beater.client.ApiException;
import ai.beater.client.ApiResponse;
import ai.beater.client.Configuration;
import ai.beater.client.models.*;
import ai.beater.client.api.ScenariosApi;

public class Example {
    public static void main(String[] args) {
        ApiClient defaultClient = Configuration.getDefaultApiClient();
        defaultClient.setBasePath("http://localhost");

        ScenariosApi apiInstance = new ScenariosApi(defaultClient);
        String tenantId = "tenantId_example"; // String | tenant_id
        String projectId = "projectId_example"; // String | project_id
        Integer limit = 56; // Integer | 
        String cursor = "cursor_example"; // String | 
        String authorization = "authorization_example"; // String | Bearer API token for strict auth
        String xBeaterApiKey = "xBeaterApiKey_example"; // String | API key alternative for strict auth
        String xBeaterProjectId = "xBeaterProjectId_example"; // String | Strict-auth project scope
        String xBeaterEnvironmentId = "xBeaterEnvironmentId_example"; // String | Strict-auth environment scope
        try {
            ApiResponse<ListScenariosResponse> response = apiInstance.listScenariosWithHttpInfo(tenantId, projectId, limit, cursor, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
            System.out.println("Status code: " + response.getStatusCode());
            System.out.println("Response headers: " + response.getHeaders());
            System.out.println("Response body: " + response.getData());
        } catch (ApiException e) {
            System.err.println("Exception when calling ScenariosApi#listScenarios");
            System.err.println("Status code: " + e.getCode());
            System.err.println("Response headers: " + e.getResponseHeaders());
            System.err.println("Reason: " + e.getResponseBody());
            e.printStackTrace();
        }
    }
}
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **tenantId** | **String**| tenant_id | |
| **projectId** | **String**| project_id | |
| **limit** | **Integer**|  | [optional] |
| **cursor** | **String**|  | [optional] |
| **authorization** | **String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **String**| Strict-auth project scope | [optional] |
| **xBeaterEnvironmentId** | **String**| Strict-auth environment scope | [optional] |

### Return type

ApiResponse<[**ListScenariosResponse**](ListScenariosResponse.md)>


### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | List scenarios |  -  |
| **400** | Invalid request, scope, or filter |  -  |
| **401** | Missing or invalid credentials |  -  |
| **403** | Credentials lack the required scope |  -  |


## mineScenarios

> MineScenariosResponse mineScenarios(tenantId, projectId, mineScenariosRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example

```java
// Import classes:
import ai.beater.client.ApiClient;
import ai.beater.client.ApiException;
import ai.beater.client.Configuration;
import ai.beater.client.models.*;
import ai.beater.client.api.ScenariosApi;

public class Example {
    public static void main(String[] args) {
        ApiClient defaultClient = Configuration.getDefaultApiClient();
        defaultClient.setBasePath("http://localhost");

        ScenariosApi apiInstance = new ScenariosApi(defaultClient);
        String tenantId = "tenantId_example"; // String | tenant_id
        String projectId = "projectId_example"; // String | project_id
        MineScenariosRequest mineScenariosRequest = new MineScenariosRequest(); // MineScenariosRequest | 
        String authorization = "authorization_example"; // String | Bearer API token for strict auth
        String xBeaterApiKey = "xBeaterApiKey_example"; // String | API key alternative for strict auth
        String xBeaterProjectId = "xBeaterProjectId_example"; // String | Strict-auth project scope
        String xBeaterEnvironmentId = "xBeaterEnvironmentId_example"; // String | Strict-auth environment scope
        try {
            MineScenariosResponse result = apiInstance.mineScenarios(tenantId, projectId, mineScenariosRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
            System.out.println(result);
        } catch (ApiException e) {
            System.err.println("Exception when calling ScenariosApi#mineScenarios");
            System.err.println("Status code: " + e.getCode());
            System.err.println("Reason: " + e.getResponseBody());
            System.err.println("Response headers: " + e.getResponseHeaders());
            e.printStackTrace();
        }
    }
}
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **tenantId** | **String**| tenant_id | |
| **projectId** | **String**| project_id | |
| **mineScenariosRequest** | [**MineScenariosRequest**](MineScenariosRequest.md)|  | |
| **authorization** | **String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **String**| Strict-auth project scope | [optional] |
| **xBeaterEnvironmentId** | **String**| Strict-auth environment scope | [optional] |

### Return type

[**MineScenariosResponse**](MineScenariosResponse.md)


### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Mine scenario clusters from traces |  -  |
| **400** | Invalid request, scope, or filter |  -  |
| **401** | Missing or invalid credentials |  -  |
| **403** | Credentials lack the required scope |  -  |
| **404** | Resource not found |  -  |

## mineScenariosWithHttpInfo

> ApiResponse<MineScenariosResponse> mineScenarios mineScenariosWithHttpInfo(tenantId, projectId, mineScenariosRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId)



### Example

```java
// Import classes:
import ai.beater.client.ApiClient;
import ai.beater.client.ApiException;
import ai.beater.client.ApiResponse;
import ai.beater.client.Configuration;
import ai.beater.client.models.*;
import ai.beater.client.api.ScenariosApi;

public class Example {
    public static void main(String[] args) {
        ApiClient defaultClient = Configuration.getDefaultApiClient();
        defaultClient.setBasePath("http://localhost");

        ScenariosApi apiInstance = new ScenariosApi(defaultClient);
        String tenantId = "tenantId_example"; // String | tenant_id
        String projectId = "projectId_example"; // String | project_id
        MineScenariosRequest mineScenariosRequest = new MineScenariosRequest(); // MineScenariosRequest | 
        String authorization = "authorization_example"; // String | Bearer API token for strict auth
        String xBeaterApiKey = "xBeaterApiKey_example"; // String | API key alternative for strict auth
        String xBeaterProjectId = "xBeaterProjectId_example"; // String | Strict-auth project scope
        String xBeaterEnvironmentId = "xBeaterEnvironmentId_example"; // String | Strict-auth environment scope
        try {
            ApiResponse<MineScenariosResponse> response = apiInstance.mineScenariosWithHttpInfo(tenantId, projectId, mineScenariosRequest, authorization, xBeaterApiKey, xBeaterProjectId, xBeaterEnvironmentId);
            System.out.println("Status code: " + response.getStatusCode());
            System.out.println("Response headers: " + response.getHeaders());
            System.out.println("Response body: " + response.getData());
        } catch (ApiException e) {
            System.err.println("Exception when calling ScenariosApi#mineScenarios");
            System.err.println("Status code: " + e.getCode());
            System.err.println("Response headers: " + e.getResponseHeaders());
            System.err.println("Reason: " + e.getResponseBody());
            e.printStackTrace();
        }
    }
}
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **tenantId** | **String**| tenant_id | |
| **projectId** | **String**| project_id | |
| **mineScenariosRequest** | [**MineScenariosRequest**](MineScenariosRequest.md)|  | |
| **authorization** | **String**| Bearer API token for strict auth | [optional] |
| **xBeaterApiKey** | **String**| API key alternative for strict auth | [optional] |
| **xBeaterProjectId** | **String**| Strict-auth project scope | [optional] |
| **xBeaterEnvironmentId** | **String**| Strict-auth environment scope | [optional] |

### Return type

ApiResponse<[**MineScenariosResponse**](MineScenariosResponse.md)>


### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Mine scenario clusters from traces |  -  |
| **400** | Invalid request, scope, or filter |  -  |
| **401** | Missing or invalid credentials |  -  |
| **403** | Credentials lack the required scope |  -  |
| **404** | Resource not found |  -  |

