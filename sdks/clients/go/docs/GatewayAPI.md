# \GatewayAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**GatewayChatCompletions**](GatewayAPI.md#GatewayChatCompletions) | **Post** /v1/gateway/{tenant_id}/{project_id}/{environment_id}/chat/completions | 



## GatewayChatCompletions

> GatewayOutcome GatewayChatCompletions(ctx, tenantId, projectId, environmentId).GatewayChatHttpRequest(gatewayChatHttpRequest).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()



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
	gatewayChatHttpRequest := *openapiclient.NewGatewayChatHttpRequest(*openapiclient.NewChatCompletionRequest([]openapiclient.ChatMessage{*openapiclient.NewChatMessage("Content_example", "Role_example")}, "Model_example")) // GatewayChatHttpRequest | 
	authorization := "authorization_example" // string | Bearer API token for strict auth (optional)
	xBeaterApiKey := "xBeaterApiKey_example" // string | API key alternative for strict auth (optional)
	xBeaterProjectId := "xBeaterProjectId_example" // string | Strict-auth project scope (optional)
	xBeaterEnvironmentId := "xBeaterEnvironmentId_example" // string | Strict-auth environment scope (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.GatewayAPI.GatewayChatCompletions(context.Background(), tenantId, projectId, environmentId).GatewayChatHttpRequest(gatewayChatHttpRequest).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `GatewayAPI.GatewayChatCompletions``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GatewayChatCompletions`: GatewayOutcome
	fmt.Fprintf(os.Stdout, "Response from `GatewayAPI.GatewayChatCompletions`: %v\n", resp)
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

Other parameters are passed through a pointer to a apiGatewayChatCompletionsRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------



 **gatewayChatHttpRequest** | [**GatewayChatHttpRequest**](GatewayChatHttpRequest.md) |  | 
 **authorization** | **string** | Bearer API token for strict auth | 
 **xBeaterApiKey** | **string** | API key alternative for strict auth | 
 **xBeaterProjectId** | **string** | Strict-auth project scope | 
 **xBeaterEnvironmentId** | **string** | Strict-auth environment scope | 

### Return type

[**GatewayOutcome**](GatewayOutcome.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)

