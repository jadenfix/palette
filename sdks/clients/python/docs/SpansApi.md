# beater_client.SpansApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**spans_get_span**](SpansApi.md#spans_get_span) | **GET** /v1/spans/{tenant_id}/{trace_id}/{span_id} |
[**spans_get_span_io**](SpansApi.md#spans_get_span_io) | **GET** /v1/spans/{tenant_id}/{trace_id}/{span_id}/io |


# **spans_get_span**
> CanonicalSpan spans_get_span(tenant_id, trace_id, span_id, unmask=unmask, reason=reason, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.canonical_span import CanonicalSpan
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
    api_instance = beater_client.SpansApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    trace_id = 'trace_id_example' # str | trace_id
    span_id = 'span_id_example' # str | span_id
    unmask = True # bool |  (optional)
    reason = 'reason_example' # str |  (optional)
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.spans_get_span(tenant_id, trace_id, span_id, unmask=unmask, reason=reason, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of SpansApi->spans_get_span:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling SpansApi->spans_get_span: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id |
 **trace_id** | **str**| trace_id |
 **span_id** | **str**| span_id |
 **unmask** | **bool**|  | [optional]
 **reason** | **str**|  | [optional]
 **authorization** | **str**| Bearer API token for strict auth | [optional]
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional]
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional]
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional]

### Return type

[**CanonicalSpan**](CanonicalSpan.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Get a canonical span |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |
**404** | Resource not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **spans_get_span_io**
> SpanIoResponse spans_get_span_io(tenant_id, trace_id, span_id, unmask=unmask, reason=reason, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.span_io_response import SpanIoResponse
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
    api_instance = beater_client.SpansApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    trace_id = 'trace_id_example' # str | trace_id
    span_id = 'span_id_example' # str | span_id
    unmask = True # bool |  (optional)
    reason = 'reason_example' # str |  (optional)
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.spans_get_span_io(tenant_id, trace_id, span_id, unmask=unmask, reason=reason, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of SpansApi->spans_get_span_io:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling SpansApi->spans_get_span_io: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id |
 **trace_id** | **str**| trace_id |
 **span_id** | **str**| span_id |
 **unmask** | **bool**|  | [optional]
 **reason** | **str**|  | [optional]
 **authorization** | **str**| Bearer API token for strict auth | [optional]
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional]
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional]
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional]

### Return type

[**SpanIoResponse**](SpanIoResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Get span input/output metadata |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |
**404** | Resource not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
