# beater_client.EvalsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**evals_run_deterministic_eval**](EvalsApi.md#evals_run_deterministic_eval) | **POST** /v1/datasets/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id}/evals/deterministic |
[**evals_run_judge_eval**](EvalsApi.md#evals_run_judge_eval) | **POST** /v1/datasets/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id}/evals/judge |


# **evals_run_deterministic_eval**
> DatasetEvalReport evals_run_deterministic_eval(tenant_id, project_id, dataset_id, version_id, run_deterministic_eval_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.dataset_eval_report import DatasetEvalReport
from beater_client.models.run_deterministic_eval_request import RunDeterministicEvalRequest
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
    api_instance = beater_client.EvalsApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    dataset_id = 'dataset_id_example' # str | dataset_id
    version_id = 'version_id_example' # str | version_id
    run_deterministic_eval_request = beater_client.RunDeterministicEvalRequest() # RunDeterministicEvalRequest |
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.evals_run_deterministic_eval(tenant_id, project_id, dataset_id, version_id, run_deterministic_eval_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of EvalsApi->evals_run_deterministic_eval:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling EvalsApi->evals_run_deterministic_eval: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id |
 **project_id** | **str**| project_id |
 **dataset_id** | **str**| dataset_id |
 **version_id** | **str**| version_id |
 **run_deterministic_eval_request** | [**RunDeterministicEvalRequest**](RunDeterministicEvalRequest.md)|  |
 **authorization** | **str**| Bearer API token for strict auth | [optional]
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional]
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional]
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional]

### Return type

[**DatasetEvalReport**](DatasetEvalReport.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Run a deterministic dataset evaluation |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |
**404** | Resource not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **evals_run_judge_eval**
> DatasetEvalReport evals_run_judge_eval(tenant_id, project_id, dataset_id, version_id, run_judge_dataset_eval_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.dataset_eval_report import DatasetEvalReport
from beater_client.models.run_judge_dataset_eval_request import RunJudgeDatasetEvalRequest
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
    api_instance = beater_client.EvalsApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    dataset_id = 'dataset_id_example' # str | dataset_id
    version_id = 'version_id_example' # str | version_id
    run_judge_dataset_eval_request = beater_client.RunJudgeDatasetEvalRequest() # RunJudgeDatasetEvalRequest |
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.evals_run_judge_eval(tenant_id, project_id, dataset_id, version_id, run_judge_dataset_eval_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of EvalsApi->evals_run_judge_eval:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling EvalsApi->evals_run_judge_eval: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id |
 **project_id** | **str**| project_id |
 **dataset_id** | **str**| dataset_id |
 **version_id** | **str**| version_id |
 **run_judge_dataset_eval_request** | [**RunJudgeDatasetEvalRequest**](RunJudgeDatasetEvalRequest.md)|  |
 **authorization** | **str**| Bearer API token for strict auth | [optional]
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional]
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional]
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional]

### Return type

[**DatasetEvalReport**](DatasetEvalReport.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Run a judge dataset evaluation |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |
**404** | Resource not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
