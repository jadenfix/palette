# beater_client.IngestApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**drain_trace_ingested**](IngestApi.md#drain_trace_ingested) | **POST** /v1/ingest/{tenant_id}/{project_id}/trace-ingested/drain | 
[**drain_trace_writes**](IngestApi.md#drain_trace_writes) | **POST** /v1/ingest/{tenant_id}/{project_id}/trace-writes/drain | 
[**get_ingest_queue_status**](IngestApi.md#get_ingest_queue_status) | **GET** /v1/ingest/{tenant_id}/{project_id}/queue | 
[**import_source**](IngestApi.md#import_source) | **POST** /v1/import/{tenant_id}/{project_id}/{environment_id} | 
[**ingest_native**](IngestApi.md#ingest_native) | **POST** /v1/traces/native | 
[**ingest_otlp**](IngestApi.md#ingest_otlp) | **POST** /v1/otlp/{tenant_id}/{project_id}/{environment_id}/v1/traces | 
[**reconcile_trace**](IngestApi.md#reconcile_trace) | **POST** /v1/ingest/{tenant_id}/{project_id}/traces/{trace_id}/reconcile | 
[**replay_dead_letter**](IngestApi.md#replay_dead_letter) | **POST** /v1/ingest/{tenant_id}/{project_id}/dead-letters/{message_id}/replay | 


# **drain_trace_ingested**
> TraceIngestedDrainReport drain_trace_ingested(tenant_id, project_id, limit=limit, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.trace_ingested_drain_report import TraceIngestedDrainReport
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
    api_instance = beater_client.IngestApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    limit = 56 # int |  (optional)
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.drain_trace_ingested(tenant_id, project_id, limit=limit, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of IngestApi->drain_trace_ingested:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling IngestApi->drain_trace_ingested: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id | 
 **project_id** | **str**| project_id | 
 **limit** | **int**|  | [optional] 
 **authorization** | **str**| Bearer API token for strict auth | [optional] 
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional] 
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional] 
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional] 

### Return type

[**TraceIngestedDrainReport**](TraceIngestedDrainReport.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Drain pending trace-ingested events |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |
**422** | Drained with dead-letters |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **drain_trace_writes**
> TraceWriteDrainReport drain_trace_writes(tenant_id, project_id, limit=limit, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.trace_write_drain_report import TraceWriteDrainReport
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
    api_instance = beater_client.IngestApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    limit = 56 # int |  (optional)
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.drain_trace_writes(tenant_id, project_id, limit=limit, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of IngestApi->drain_trace_writes:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling IngestApi->drain_trace_writes: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id | 
 **project_id** | **str**| project_id | 
 **limit** | **int**|  | [optional] 
 **authorization** | **str**| Bearer API token for strict auth | [optional] 
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional] 
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional] 
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional] 

### Return type

[**TraceWriteDrainReport**](TraceWriteDrainReport.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Drain pending trace writes |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |
**422** | Drained with dead-letters |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_ingest_queue_status**
> IngestQueueStatus get_ingest_queue_status(tenant_id, project_id, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.ingest_queue_status import IngestQueueStatus
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
    api_instance = beater_client.IngestApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.get_ingest_queue_status(tenant_id, project_id, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of IngestApi->get_ingest_queue_status:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling IngestApi->get_ingest_queue_status: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id | 
 **project_id** | **str**| project_id | 
 **authorization** | **str**| Bearer API token for strict auth | [optional] 
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional] 
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional] 
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional] 

### Return type

[**IngestQueueStatus**](IngestQueueStatus.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Get ingest queue status |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **import_source**
> IngestOutcome import_source(tenant_id, project_id, environment_id, import_source_http_request, durability=durability, authorization=authorization, x_beater_api_key=x_beater_api_key)



### Example


```python
import beater_client
from beater_client.models.import_source_http_request import ImportSourceHttpRequest
from beater_client.models.ingest_outcome import IngestOutcome
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
    api_instance = beater_client.IngestApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    environment_id = 'environment_id_example' # str | environment_id
    import_source_http_request = beater_client.ImportSourceHttpRequest() # ImportSourceHttpRequest | 
    durability = 'durability_example' # str |  (optional)
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)

    try:
        api_response = api_instance.import_source(tenant_id, project_id, environment_id, import_source_http_request, durability=durability, authorization=authorization, x_beater_api_key=x_beater_api_key)
        print("The response of IngestApi->import_source:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling IngestApi->import_source: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id | 
 **project_id** | **str**| project_id | 
 **environment_id** | **str**| environment_id | 
 **import_source_http_request** | [**ImportSourceHttpRequest**](ImportSourceHttpRequest.md)|  | 
 **durability** | **str**|  | [optional] 
 **authorization** | **str**| Bearer API token for strict auth | [optional] 
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional] 

### Return type

[**IngestOutcome**](IngestOutcome.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Normalize an imported source document into canonical spans |  -  |
**400** | Invalid request, scope, or unknown source |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |
**413** | Payload or attribute cardinality too large |  -  |
**429** | Per-project quota exceeded or backpressure |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **ingest_native**
> IngestOutcome ingest_native(native_ingest_request, durability=durability, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.ingest_outcome import IngestOutcome
from beater_client.models.native_ingest_request import NativeIngestRequest
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
    api_instance = beater_client.IngestApi(api_client)
    native_ingest_request = beater_client.NativeIngestRequest() # NativeIngestRequest | 
    durability = 'durability_example' # str |  (optional)
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.ingest_native(native_ingest_request, durability=durability, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of IngestApi->ingest_native:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling IngestApi->ingest_native: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **native_ingest_request** | [**NativeIngestRequest**](NativeIngestRequest.md)|  | 
 **durability** | **str**|  | [optional] 
 **authorization** | **str**| Bearer API token for strict auth | [optional] 
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional] 
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional] 
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional] 

### Return type

[**IngestOutcome**](IngestOutcome.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Ingest native canonical spans |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |
**413** | Payload or attribute cardinality too large |  -  |
**429** | Per-project quota exceeded or backpressure |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **ingest_otlp**
> OtlpIngestOutcome ingest_otlp(tenant_id, project_id, environment_id, durability=durability, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.otlp_ingest_outcome import OtlpIngestOutcome
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
    api_instance = beater_client.IngestApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    environment_id = 'environment_id_example' # str | environment_id
    durability = 'durability_example' # str |  (optional)
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.ingest_otlp(tenant_id, project_id, environment_id, durability=durability, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of IngestApi->ingest_otlp:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling IngestApi->ingest_otlp: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id | 
 **project_id** | **str**| project_id | 
 **environment_id** | **str**| environment_id | 
 **durability** | **str**|  | [optional] 
 **authorization** | **str**| Bearer API token for strict auth | [optional] 
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional] 
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional] 
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional] 

### Return type

[**OtlpIngestOutcome**](OtlpIngestOutcome.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Ingest OTLP/HTTP protobuf traces |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |
**413** | Payload or attribute cardinality too large |  -  |
**429** | Per-project quota exceeded or backpressure |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **reconcile_trace**
> TraceIngestedReconcileReport reconcile_trace(tenant_id, project_id, trace_id, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.trace_ingested_reconcile_report import TraceIngestedReconcileReport
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
    api_instance = beater_client.IngestApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    trace_id = 'trace_id_example' # str | trace_id
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.reconcile_trace(tenant_id, project_id, trace_id, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of IngestApi->reconcile_trace:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling IngestApi->reconcile_trace: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id | 
 **project_id** | **str**| project_id | 
 **trace_id** | **str**| trace_id | 
 **authorization** | **str**| Bearer API token for strict auth | [optional] 
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional] 
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional] 
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional] 

### Return type

[**TraceIngestedReconcileReport**](TraceIngestedReconcileReport.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Reconcile a trace-ingested record |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |
**404** | Resource not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **replay_dead_letter**
> DeadLetterReplayReport replay_dead_letter(tenant_id, project_id, message_id, reset_attempts=reset_attempts, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.dead_letter_replay_report import DeadLetterReplayReport
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
    api_instance = beater_client.IngestApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    message_id = 'message_id_example' # str | message_id
    reset_attempts = True # bool |  (optional)
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.replay_dead_letter(tenant_id, project_id, message_id, reset_attempts=reset_attempts, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of IngestApi->replay_dead_letter:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling IngestApi->replay_dead_letter: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id | 
 **project_id** | **str**| project_id | 
 **message_id** | **str**| message_id | 
 **reset_attempts** | **bool**|  | [optional] 
 **authorization** | **str**| Bearer API token for strict auth | [optional] 
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional] 
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional] 
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional] 

### Return type

[**DeadLetterReplayReport**](DeadLetterReplayReport.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Replay a dead-letter message |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |
**404** | Resource not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

