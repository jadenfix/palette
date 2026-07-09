# beater_client.HealthApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**health_health**](HealthApi.md#health_health) | **GET** /health |


# **health_health**
> HealthResponse health_health()



### Example


```python
import beater_client
from beater_client.models.health_response import HealthResponse
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
    api_instance = beater_client.HealthApi(api_client)

    try:
        api_response = api_instance.health_health()
        print("The response of HealthApi->health_health:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling HealthApi->health_health: %s\n" % e)
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

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Runtime is accepting requests |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
