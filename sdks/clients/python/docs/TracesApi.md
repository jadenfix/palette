# beater_client.TracesApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**traces_get_trace**](TracesApi.md#traces_get_trace) | **GET** /v1/traces/{tenant_id}/{trace_id} |
[**traces_list_traces**](TracesApi.md#traces_list_traces) | **GET** /v1/traces/{tenant_id} |


# **traces_get_trace**
> TraceView traces_get_trace(tenant_id, trace_id, unmask=unmask, reason=reason, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.trace_view import TraceView
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
    api_instance = beater_client.TracesApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    trace_id = 'trace_id_example' # str | trace_id
    unmask = True # bool |  (optional)
    reason = 'reason_example' # str |  (optional)
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.traces_get_trace(tenant_id, trace_id, unmask=unmask, reason=reason, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of TracesApi->traces_get_trace:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling TracesApi->traces_get_trace: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id |
 **trace_id** | **str**| trace_id |
 **unmask** | **bool**|  | [optional]
 **reason** | **str**|  | [optional]
 **authorization** | **str**| Bearer API token for strict auth | [optional]
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional]
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional]
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional]

### Return type

[**TraceView**](TraceView.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Get a canonical trace |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |
**404** | Resource not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **traces_list_traces**
> PageRunSummary traces_list_traces(tenant_id, project_id=project_id, environment_id=environment_id, trace_id=trace_id, kind=kind, status=status, started_after=started_after, started_before=started_before, model=model, release=release, min_cost_micros=min_cost_micros, max_cost_micros=max_cost_micros, min_latency_ms=min_latency_ms, max_latency_ms=max_latency_ms, limit=limit, cursor=cursor, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.page_run_summary import PageRunSummary
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
    api_instance = beater_client.TracesApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str |  (optional)
    environment_id = 'environment_id_example' # str |  (optional)
    trace_id = 'trace_id_example' # str |  (optional)
    kind = 'kind_example' # str |  (optional)
    status = 'status_example' # str |  (optional)
    started_after = 'started_after_example' # str |  (optional)
    started_before = 'started_before_example' # str |  (optional)
    model = 'model_example' # str |  (optional)
    release = 'release_example' # str |  (optional)
    min_cost_micros = 56 # int |  (optional)
    max_cost_micros = 56 # int |  (optional)
    min_latency_ms = 56 # int |  (optional)
    max_latency_ms = 56 # int |  (optional)
    limit = 56 # int |  (optional)
    cursor = 'cursor_example' # str |  (optional)
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.traces_list_traces(tenant_id, project_id=project_id, environment_id=environment_id, trace_id=trace_id, kind=kind, status=status, started_after=started_after, started_before=started_before, model=model, release=release, min_cost_micros=min_cost_micros, max_cost_micros=max_cost_micros, min_latency_ms=min_latency_ms, max_latency_ms=max_latency_ms, limit=limit, cursor=cursor, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of TracesApi->traces_list_traces:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling TracesApi->traces_list_traces: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id |
 **project_id** | **str**|  | [optional]
 **environment_id** | **str**|  | [optional]
 **trace_id** | **str**|  | [optional]
 **kind** | **str**|  | [optional]
 **status** | **str**|  | [optional]
 **started_after** | **str**|  | [optional]
 **started_before** | **str**|  | [optional]
 **model** | **str**|  | [optional]
 **release** | **str**|  | [optional]
 **min_cost_micros** | **int**|  | [optional]
 **max_cost_micros** | **int**|  | [optional]
 **min_latency_ms** | **int**|  | [optional]
 **max_latency_ms** | **int**|  | [optional]
 **limit** | **int**|  | [optional]
 **cursor** | **str**|  | [optional]
 **authorization** | **str**| Bearer API token for strict auth | [optional]
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional]
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional]
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional]

### Return type

[**PageRunSummary**](PageRunSummary.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | List trace run summaries |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
