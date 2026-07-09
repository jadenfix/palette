# beater_client.SearchApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**search_search_spans**](SearchApi.md#search_search_spans) | **GET** /v1/search/{tenant_id}/spans |


# **search_search_spans**
> SearchResponse search_search_spans(tenant_id, q=q, project_id=project_id, environment_id=environment_id, trace_id=trace_id, span_id=span_id, kind=kind, status=status, model=model, tool=tool, limit=limit, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.search_response import SearchResponse
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
    api_instance = beater_client.SearchApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    q = 'q_example' # str |  (optional)
    project_id = 'project_id_example' # str |  (optional)
    environment_id = 'environment_id_example' # str |  (optional)
    trace_id = 'trace_id_example' # str |  (optional)
    span_id = 'span_id_example' # str |  (optional)
    kind = 'kind_example' # str |  (optional)
    status = 'status_example' # str |  (optional)
    model = 'model_example' # str |  (optional)
    tool = 'tool_example' # str |  (optional)
    limit = 56 # int |  (optional)
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.search_search_spans(tenant_id, q=q, project_id=project_id, environment_id=environment_id, trace_id=trace_id, span_id=span_id, kind=kind, status=status, model=model, tool=tool, limit=limit, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of SearchApi->search_search_spans:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling SearchApi->search_search_spans: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id |
 **q** | **str**|  | [optional]
 **project_id** | **str**|  | [optional]
 **environment_id** | **str**|  | [optional]
 **trace_id** | **str**|  | [optional]
 **span_id** | **str**|  | [optional]
 **kind** | **str**|  | [optional]
 **status** | **str**|  | [optional]
 **model** | **str**|  | [optional]
 **tool** | **str**|  | [optional]
 **limit** | **int**|  | [optional]
 **authorization** | **str**| Bearer API token for strict auth | [optional]
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional]
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional]
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional]

### Return type

[**SearchResponse**](SearchResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Search spans |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
