# beater_client.GatesApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**gates_create_gate**](GatesApi.md#gates_create_gate) | **POST** /v1/gates/{tenant_id}/{project_id} |
[**gates_run_gate**](GatesApi.md#gates_run_gate) | **POST** /v1/gates/{tenant_id}/{project_id}/{gate_id}/run |


# **gates_create_gate**
> GateDefinition gates_create_gate(tenant_id, project_id, create_gate_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.create_gate_request import CreateGateRequest
from beater_client.models.gate_definition import GateDefinition
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
    api_instance = beater_client.GatesApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    create_gate_request = beater_client.CreateGateRequest() # CreateGateRequest |
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.gates_create_gate(tenant_id, project_id, create_gate_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of GatesApi->gates_create_gate:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling GatesApi->gates_create_gate: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id |
 **project_id** | **str**| project_id |
 **create_gate_request** | [**CreateGateRequest**](CreateGateRequest.md)|  |
 **authorization** | **str**| Bearer API token for strict auth | [optional]
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional]
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional]
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional]

### Return type

[**GateDefinition**](GateDefinition.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Create a release gate |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **gates_run_gate**
> GateRunReport gates_run_gate(tenant_id, project_id, gate_id, run_gate_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.gate_run_report import GateRunReport
from beater_client.models.run_gate_request import RunGateRequest
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
    api_instance = beater_client.GatesApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    gate_id = 'gate_id_example' # str | gate_id
    run_gate_request = beater_client.RunGateRequest() # RunGateRequest |
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.gates_run_gate(tenant_id, project_id, gate_id, run_gate_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of GatesApi->gates_run_gate:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling GatesApi->gates_run_gate: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id |
 **project_id** | **str**| project_id |
 **gate_id** | **str**| gate_id |
 **run_gate_request** | [**RunGateRequest**](RunGateRequest.md)|  |
 **authorization** | **str**| Bearer API token for strict auth | [optional]
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional]
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional]
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional]

### Return type

[**GateRunReport**](GateRunReport.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Run a gate against an experiment |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |
**404** | Resource not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
