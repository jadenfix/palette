# beater_client.PromptsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**add_prompt_version**](PromptsApi.md#add_prompt_version) | **POST** /v1/prompts/{tenant_id}/{project_id}/{prompt_id}/versions | 
[**create_prompt**](PromptsApi.md#create_prompt) | **POST** /v1/prompts/{tenant_id}/{project_id} | 
[**diff_prompt_versions**](PromptsApi.md#diff_prompt_versions) | **GET** /v1/prompts/{tenant_id}/{project_id}/{prompt_id}/diff | 
[**get_prompt**](PromptsApi.md#get_prompt) | **GET** /v1/prompts/{tenant_id}/{project_id}/{prompt_id} | 
[**list_prompt_versions**](PromptsApi.md#list_prompt_versions) | **GET** /v1/prompts/{tenant_id}/{project_id}/{prompt_id}/versions | 
[**list_prompts**](PromptsApi.md#list_prompts) | **GET** /v1/prompts/{tenant_id}/{project_id} | 


# **add_prompt_version**
> PromptVersion add_prompt_version(tenant_id, project_id, prompt_id, add_prompt_version_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.add_prompt_version_request import AddPromptVersionRequest
from beater_client.models.prompt_version import PromptVersion
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
    api_instance = beater_client.PromptsApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    prompt_id = 'prompt_id_example' # str | prompt_id
    add_prompt_version_request = beater_client.AddPromptVersionRequest() # AddPromptVersionRequest | 
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.add_prompt_version(tenant_id, project_id, prompt_id, add_prompt_version_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of PromptsApi->add_prompt_version:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling PromptsApi->add_prompt_version: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id | 
 **project_id** | **str**| project_id | 
 **prompt_id** | **str**| prompt_id | 
 **add_prompt_version_request** | [**AddPromptVersionRequest**](AddPromptVersionRequest.md)|  | 
 **authorization** | **str**| Bearer API token for strict auth | [optional] 
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional] 
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional] 
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional] 

### Return type

[**PromptVersion**](PromptVersion.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Append an immutable prompt version |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |
**404** | Resource not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_prompt**
> CreatedPrompt create_prompt(tenant_id, project_id, create_prompt_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.create_prompt_request import CreatePromptRequest
from beater_client.models.created_prompt import CreatedPrompt
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
    api_instance = beater_client.PromptsApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    create_prompt_request = beater_client.CreatePromptRequest() # CreatePromptRequest | 
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.create_prompt(tenant_id, project_id, create_prompt_request, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of PromptsApi->create_prompt:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling PromptsApi->create_prompt: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id | 
 **project_id** | **str**| project_id | 
 **create_prompt_request** | [**CreatePromptRequest**](CreatePromptRequest.md)|  | 
 **authorization** | **str**| Bearer API token for strict auth | [optional] 
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional] 
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional] 
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional] 

### Return type

[**CreatedPrompt**](CreatedPrompt.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Create a prompt and its initial version |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **diff_prompt_versions**
> PromptVersionDiff diff_prompt_versions(tenant_id, project_id, prompt_id, var_from, to, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.prompt_version_diff import PromptVersionDiff
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
    api_instance = beater_client.PromptsApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    prompt_id = 'prompt_id_example' # str | prompt_id
    var_from = 'var_from_example' # str | 
    to = 'to_example' # str | 
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.diff_prompt_versions(tenant_id, project_id, prompt_id, var_from, to, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of PromptsApi->diff_prompt_versions:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling PromptsApi->diff_prompt_versions: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id | 
 **project_id** | **str**| project_id | 
 **prompt_id** | **str**| prompt_id | 
 **var_from** | **str**|  | 
 **to** | **str**|  | 
 **authorization** | **str**| Bearer API token for strict auth | [optional] 
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional] 
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional] 
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional] 

### Return type

[**PromptVersionDiff**](PromptVersionDiff.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Line diff between two prompt versions |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |
**404** | Resource not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_prompt**
> Prompt get_prompt(tenant_id, project_id, prompt_id, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.prompt import Prompt
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
    api_instance = beater_client.PromptsApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    prompt_id = 'prompt_id_example' # str | prompt_id
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.get_prompt(tenant_id, project_id, prompt_id, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of PromptsApi->get_prompt:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling PromptsApi->get_prompt: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id | 
 **project_id** | **str**| project_id | 
 **prompt_id** | **str**| prompt_id | 
 **authorization** | **str**| Bearer API token for strict auth | [optional] 
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional] 
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional] 
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional] 

### Return type

[**Prompt**](Prompt.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Get a prompt&#39;s metadata |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |
**404** | Resource not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **list_prompt_versions**
> PromptVersionListResponse list_prompt_versions(tenant_id, project_id, prompt_id, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.prompt_version_list_response import PromptVersionListResponse
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
    api_instance = beater_client.PromptsApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    prompt_id = 'prompt_id_example' # str | prompt_id
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.list_prompt_versions(tenant_id, project_id, prompt_id, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of PromptsApi->list_prompt_versions:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling PromptsApi->list_prompt_versions: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| tenant_id | 
 **project_id** | **str**| project_id | 
 **prompt_id** | **str**| prompt_id | 
 **authorization** | **str**| Bearer API token for strict auth | [optional] 
 **x_beater_api_key** | **str**| API key alternative for strict auth | [optional] 
 **x_beater_project_id** | **str**| Strict-auth project scope | [optional] 
 **x_beater_environment_id** | **str**| Strict-auth environment scope | [optional] 

### Return type

[**PromptVersionListResponse**](PromptVersionListResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | List a prompt&#39;s versions oldest-first |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |
**404** | Resource not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **list_prompts**
> PromptListResponse list_prompts(tenant_id, project_id, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)



### Example


```python
import beater_client
from beater_client.models.prompt_list_response import PromptListResponse
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
    api_instance = beater_client.PromptsApi(api_client)
    tenant_id = 'tenant_id_example' # str | tenant_id
    project_id = 'project_id_example' # str | project_id
    authorization = 'authorization_example' # str | Bearer API token for strict auth (optional)
    x_beater_api_key = 'x_beater_api_key_example' # str | API key alternative for strict auth (optional)
    x_beater_project_id = 'x_beater_project_id_example' # str | Strict-auth project scope (optional)
    x_beater_environment_id = 'x_beater_environment_id_example' # str | Strict-auth environment scope (optional)

    try:
        api_response = api_instance.list_prompts(tenant_id, project_id, authorization=authorization, x_beater_api_key=x_beater_api_key, x_beater_project_id=x_beater_project_id, x_beater_environment_id=x_beater_environment_id)
        print("The response of PromptsApi->list_prompts:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling PromptsApi->list_prompts: %s\n" % e)
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

[**PromptListResponse**](PromptListResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | List prompts in a project |  -  |
**400** | Invalid request, scope, or filter |  -  |
**401** | Missing or invalid credentials |  -  |
**403** | Credentials lack the required scope |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

