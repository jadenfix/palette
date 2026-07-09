# \ApiKeysAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**ApiKeysCreateApiKey**](ApiKeysAPI.md#ApiKeysCreateApiKey) | **Post** /v1/api-keys/{tenant_id}/{project_id}/{environment_id} |
[**ApiKeysRevokeApiKey**](ApiKeysAPI.md#ApiKeysRevokeApiKey) | **Post** /v1/api-keys/{tenant_id}/{project_id}/{environment_id}/{api_key_id}/revoke |



## ApiKeysCreateApiKey

> ApiKeyCreatedResponse ApiKeysCreateApiKey(ctx, tenantId, projectId, environmentId).CreateApiKeyHttpRequest(createApiKeyHttpRequest).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()



### Example

```go
package main

import (
	"context"
	"fmt"
	"os"
	openapiclient "github.com/GIT_USER_ID/GIT_REPO_ID/beaterclient"
)

func main() {
	tenantId := "tenantId_example" // string | tenant_id
	projectId := "projectId_example" // string | project_id
	environmentId := "environmentId_example" // string | environment_id
	createApiKeyHttpRequest := *openapiclient.NewCreateApiKeyHttpRequest([]openapiclient.ApiScope{openapiclient.ApiScope("trace:write")}) // CreateApiKeyHttpRequest |
	authorization := "authorization_example" // string | Bearer API token for strict auth (optional)
	xBeaterApiKey := "xBeaterApiKey_example" // string | API key alternative for strict auth (optional)
	xBeaterProjectId := "xBeaterProjectId_example" // string | Strict-auth project scope (optional)
	xBeaterEnvironmentId := "xBeaterEnvironmentId_example" // string | Strict-auth environment scope (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.ApiKeysAPI.ApiKeysCreateApiKey(context.Background(), tenantId, projectId, environmentId).CreateApiKeyHttpRequest(createApiKeyHttpRequest).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `ApiKeysAPI.ApiKeysCreateApiKey``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `ApiKeysCreateApiKey`: ApiKeyCreatedResponse
	fmt.Fprintf(os.Stdout, "Response from `ApiKeysAPI.ApiKeysCreateApiKey`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**tenantId** | **string** | tenant_id |
**projectId** | **string** | project_id |
**environmentId** | **string** | environment_id |

### Other Parameters

Other parameters are passed through a pointer to a apiApiKeysCreateApiKeyRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------



 **createApiKeyHttpRequest** | [**CreateApiKeyHttpRequest**](CreateApiKeyHttpRequest.md) |  |
 **authorization** | **string** | Bearer API token for strict auth |
 **xBeaterApiKey** | **string** | API key alternative for strict auth |
 **xBeaterProjectId** | **string** | Strict-auth project scope |
 **xBeaterEnvironmentId** | **string** | Strict-auth environment scope |

### Return type

[**ApiKeyCreatedResponse**](ApiKeyCreatedResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## ApiKeysRevokeApiKey

> RevokedApiKey ApiKeysRevokeApiKey(ctx, tenantId, projectId, environmentId, apiKeyId).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()



### Example

```go
package main

import (
	"context"
	"fmt"
	"os"
	openapiclient "github.com/GIT_USER_ID/GIT_REPO_ID/beaterclient"
)

func main() {
	tenantId := "tenantId_example" // string | tenant_id
	projectId := "projectId_example" // string | project_id
	environmentId := "environmentId_example" // string | environment_id
	apiKeyId := "apiKeyId_example" // string | api_key_id
	authorization := "authorization_example" // string | Bearer API token for strict auth (optional)
	xBeaterApiKey := "xBeaterApiKey_example" // string | API key alternative for strict auth (optional)
	xBeaterProjectId := "xBeaterProjectId_example" // string | Strict-auth project scope (optional)
	xBeaterEnvironmentId := "xBeaterEnvironmentId_example" // string | Strict-auth environment scope (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.ApiKeysAPI.ApiKeysRevokeApiKey(context.Background(), tenantId, projectId, environmentId, apiKeyId).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `ApiKeysAPI.ApiKeysRevokeApiKey``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `ApiKeysRevokeApiKey`: RevokedApiKey
	fmt.Fprintf(os.Stdout, "Response from `ApiKeysAPI.ApiKeysRevokeApiKey`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**tenantId** | **string** | tenant_id |
**projectId** | **string** | project_id |
**environmentId** | **string** | environment_id |
**apiKeyId** | **string** | api_key_id |

### Other Parameters

Other parameters are passed through a pointer to a apiApiKeysRevokeApiKeyRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------




 **authorization** | **string** | Bearer API token for strict auth |
 **xBeaterApiKey** | **string** | API key alternative for strict auth |
 **xBeaterProjectId** | **string** | Strict-auth project scope |
 **xBeaterEnvironmentId** | **string** | Strict-auth environment scope |

### Return type

[**RevokedApiKey**](RevokedApiKey.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)
