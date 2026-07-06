# HealthApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------------- | ------------- | ------------- |
| [**health**](HealthApi.md#health) | **GET** /health |  |


<a id="health"></a>
# **health**
> HealthResponse health()



### Example
```kotlin
// Import classes:
//import ai.beater.client.kotlin.infrastructure.*
//import ai.beater.client.kotlin.models.*

val apiInstance = HealthApi()
try {
    val result : HealthResponse = apiInstance.health()
    println(result)
} catch (e: ClientException) {
    println("4xx response calling HealthApi#health")
    e.printStackTrace()
} catch (e: ServerException) {
    println("5xx response calling HealthApi#health")
    e.printStackTrace()
}
```

### Parameters
This endpoint does not need any parameter.

### Return type

[**HealthResponse**](HealthResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json
