# \ProviderSecretsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**provider_secrets_period_create_provider_secret**](ProviderSecretsApi.md#provider_secrets_period_create_provider_secret) | **POST** /v1/provider-secrets/{tenant_id}/{project_id} |
[**provider_secrets_period_list_provider_secrets**](ProviderSecretsApi.md#provider_secrets_period_list_provider_secrets) | **GET** /v1/provider-secrets/{tenant_id}/{project_id} |
[**provider_secrets_period_revoke_provider_secret**](ProviderSecretsApi.md#provider_secrets_period_revoke_provider_secret) | **POST** /v1/provider-secrets/{tenant_id}/{project_id}/{provider_secret_id}/revoke |



## provider_secrets_period_create_provider_secret

> models::ProviderSecretMetadata provider_secrets_period_create_provider_secret(tenant_id, project_id, create_provider_secret_http_request, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**project_id** | **String** | project_id | [required] |
**create_provider_secret_http_request** | [**CreateProviderSecretHttpRequest**](CreateProviderSecretHttpRequest.md) |  | [required] |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::ProviderSecretMetadata**](ProviderSecretMetadata.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## provider_secrets_period_list_provider_secrets

> Vec<models::ProviderSecretMetadata> provider_secrets_period_list_provider_secrets(tenant_id, project_id, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**project_id** | **String** | project_id | [required] |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**Vec<models::ProviderSecretMetadata>**](ProviderSecretMetadata.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## provider_secrets_period_revoke_provider_secret

> models::RevokedProviderSecret provider_secrets_period_revoke_provider_secret(tenant_id, project_id, provider_secret_id, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**project_id** | **String** | project_id | [required] |
**provider_secret_id** | **String** | provider_secret_id | [required] |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::RevokedProviderSecret**](RevokedProviderSecret.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
