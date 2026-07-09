# beater_client.JudgeApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**judge_evaluate_judge**](JudgeApi.md#judge_evaluate_judge) | **POST** /v1/judge/{tenant_id}/{project_id}/evaluate |
[**judge_list_judge_ledger**](JudgeApi.md#judge_list_judge_ledger) | **GET** /v1/judge/{tenant_id}/{project_id}/ledger |


# **judge_evaluate_judge**
> JudgeBrokerOutcome judge_evaluate_judge(tenant_id, project_id, run_judge_eval_http_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.judge_broker_outcome import JudgeBrokerOutcome
from beater_client.models.run_judge_eval_http_request import RunJudgeEvalHttpRequest
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
    api_instance = beater_client.JudgeApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    run_judge_eval_http_request = beater_client.RunJudgeEvalHttpRequest() # RunJudgeEvalHttpRequest |
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.judge_evaluate_judge(tenant_id, project_id, run_judge_eval_http_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of JudgeApi->judge_evaluate_judge:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling JudgeApi->judge_evaluate_judge: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id |
 **project_id** | **str**| project_id |
 **run_judge_eval_http_request** | [**RunJudgeEvalHttpRequest**](RunJudgeEvalHttpRequest.md)|  |
 **authorization** | **str**| Bearer API token for strict auth | [optional]
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional]
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional]
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional]

### Return type

[**JudgeBrokerOutcome**](JudgeBrokerOutcome.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Run an ad-hoc judge evaluation |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **judge_list_judge_ledger**
> List[JudgeAuditRecord] judge_list_judge_ledger(tenant_id, project_id, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.judge_audit_record import JudgeAuditRecord
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
    api_instance = beater_client.JudgeApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.judge_list_judge_ledger(tenant_id, project_id, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of JudgeApi->judge_list_judge_ledger:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling JudgeApi->judge_list_judge_ledger: %s\n" % e)
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

[**List[JudgeAuditRecord]**](JudgeAuditRecord.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | List judge ledger audit records |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
