# beater_client.GatewayApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**gateway_chat_completions**](GatewayApi.md#gateway_chat_completions) | **POST** /v1/gateway/{tenant_id}/{project_id}/{environment_id}/chat/completions | 


# **gateway_chat_completions**
> GatewayOutcome gateway_chat_completions(tenant_id, project_id, environment_id, gateway_chat_http_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.gateway_chat_http_request import GatewayChatHttpRequest
from beater_client.models.gateway_outcome import GatewayOutcome
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
    api_instance = beater_client.GatewayApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    environment_id = 'environment_id_example' # str | environment_id
    gateway_chat_http_request = beater_client.GatewayChatHttpRequest() # GatewayChatHttpRequest | 
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.gateway_chat_completions(tenant_id, project_id, environment_id, gateway_chat_http_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of GatewayApi->gateway_chat_completions:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling GatewayApi->gateway_chat_completions: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id | 
 **project_id** | **str**| project_id | 
 **environment_id** | **str**| environment_id | 
 **gateway_chat_http_request** | [**GatewayChatHttpRequest**](GatewayChatHttpRequest.md)|  | 
 **authorization** | **str**| Bearer API token for strict auth | [optional] 
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional] 
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional] 
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional] 

### Return type

[**GatewayOutcome**](GatewayOutcome.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Proxied OpenAI-compatible chat completion |  -  |
**400** | Invalid request, scope, or no key + no managed default |  -  |
**401** | Missing or invalid credentials |  -  |
**402** | Per-tenant budget exceeded |  -  |
**403** | Credentials lack the required scope |  -  |
**404** | Provider secret not found or inactive |  -  |
**502** | All providers/keys failed |  -  |
**504** | Gateway request timed out |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

