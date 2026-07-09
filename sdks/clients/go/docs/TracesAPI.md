# \TracesAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**TracesGetTrace**](TracesAPI.md#TracesGetTrace) | **Get** /v1/traces/{tenant_id}/{trace_id} |
[**TracesListTraces**](TracesAPI.md#TracesListTraces) | **Get** /v1/traces/{tenant_id} |



## TracesGetTrace

> TraceView TracesGetTrace(ctx, tenantId, traceId).Unmask(unmask).Reason(reason).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()



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
	traceId := "traceId_example" // string | trace_id
	unmask := true // bool |  (optional)
	reason := "reason_example" // string |  (optional)
	authorization := "authorization_example" // string | Bearer API token for strict auth (optional)
	xBeaterApiKey := "xBeaterApiKey_example" // string | API key alternative for strict auth (optional)
	xBeaterProjectId := "xBeaterProjectId_example" // string | Strict-auth project scope (optional)
	xBeaterEnvironmentId := "xBeaterEnvironmentId_example" // string | Strict-auth environment scope (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.TracesAPI.TracesGetTrace(context.Background(), tenantId, traceId).Unmask(unmask).Reason(reason).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `TracesAPI.TracesGetTrace``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `TracesGetTrace`: TraceView
	fmt.Fprintf(os.Stdout, "Response from `TracesAPI.TracesGetTrace`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**tenantId** | **string** | tenant_id |
**traceId** | **string** | trace_id |

### Other Parameters

Other parameters are passed through a pointer to a apiTracesGetTraceRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------


 **unmask** | **bool** |  |
 **reason** | **string** |  |
 **authorization** | **string** | Bearer API token for strict auth |
 **xBeaterApiKey** | **string** | API key alternative for strict auth |
 **xBeaterProjectId** | **string** | Strict-auth project scope |
 **xBeaterEnvironmentId** | **string** | Strict-auth environment scope |

### Return type

[**TraceView**](TraceView.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## TracesListTraces

> PageRunSummary TracesListTraces(ctx, tenantId).ProjectId(projectId).EnvironmentId(environmentId).TraceId(traceId).Kind(kind).Status(status).StartedAfter(startedAfter).StartedBefore(startedBefore).Model(model).Release(release).MinCostMicros(minCostMicros).MaxCostMicros(maxCostMicros).MinLatencyMs(minLatencyMs).MaxLatencyMs(maxLatencyMs).Limit(limit).Cursor(cursor).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()



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
	projectId := "projectId_example" // string |  (optional)
	environmentId := "environmentId_example" // string |  (optional)
	traceId := "traceId_example" // string |  (optional)
	kind := "kind_example" // string |  (optional)
	status := "status_example" // string |  (optional)
	startedAfter := "startedAfter_example" // string |  (optional)
	startedBefore := "startedBefore_example" // string |  (optional)
	model := "model_example" // string |  (optional)
	release := "release_example" // string |  (optional)
	minCostMicros := int64(789) // int64 |  (optional)
	maxCostMicros := int64(789) // int64 |  (optional)
	minLatencyMs := int64(789) // int64 |  (optional)
	maxLatencyMs := int64(789) // int64 |  (optional)
	limit := int32(56) // int32 |  (optional)
	cursor := "cursor_example" // string |  (optional)
	authorization := "authorization_example" // string | Bearer API token for strict auth (optional)
	xBeaterApiKey := "xBeaterApiKey_example" // string | API key alternative for strict auth (optional)
	xBeaterProjectId := "xBeaterProjectId_example" // string | Strict-auth project scope (optional)
	xBeaterEnvironmentId := "xBeaterEnvironmentId_example" // string | Strict-auth environment scope (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.TracesAPI.TracesListTraces(context.Background(), tenantId).ProjectId(projectId).EnvironmentId(environmentId).TraceId(traceId).Kind(kind).Status(status).StartedAfter(startedAfter).StartedBefore(startedBefore).Model(model).Release(release).MinCostMicros(minCostMicros).MaxCostMicros(maxCostMicros).MinLatencyMs(minLatencyMs).MaxLatencyMs(maxLatencyMs).Limit(limit).Cursor(cursor).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `TracesAPI.TracesListTraces``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `TracesListTraces`: PageRunSummary
	fmt.Fprintf(os.Stdout, "Response from `TracesAPI.TracesListTraces`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**tenantId** | **string** | tenant_id |

### Other Parameters

Other parameters are passed through a pointer to a apiTracesListTracesRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------

 **projectId** | **string** |  |
 **environmentId** | **string** |  |
 **traceId** | **string** |  |
 **kind** | **string** |  |
 **status** | **string** |  |
 **startedAfter** | **string** |  |
 **startedBefore** | **string** |  |
 **model** | **string** |  |
 **release** | **string** |  |
 **minCostMicros** | **int64** |  |
 **maxCostMicros** | **int64** |  |
 **minLatencyMs** | **int64** |  |
 **maxLatencyMs** | **int64** |  |
 **limit** | **int32** |  |
 **cursor** | **string** |  |
 **authorization** | **string** | Bearer API token for strict auth |
 **xBeaterApiKey** | **string** | API key alternative for strict auth |
 **xBeaterProjectId** | **string** | Strict-auth project scope |
 **xBeaterEnvironmentId** | **string** | Strict-auth environment scope |

### Return type

[**PageRunSummary**](PageRunSummary.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)
