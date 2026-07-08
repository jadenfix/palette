# \IngestAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**DrainTraceIngested**](IngestAPI.md#DrainTraceIngested) | **Post** /v1/ingest/{tenant_id}/{project_id}/trace-ingested/drain |
[**DrainTraceWrites**](IngestAPI.md#DrainTraceWrites) | **Post** /v1/ingest/{tenant_id}/{project_id}/trace-writes/drain |
[**GetIngestQueueStatus**](IngestAPI.md#GetIngestQueueStatus) | **Get** /v1/ingest/{tenant_id}/{project_id}/queue |
[**ImportSource**](IngestAPI.md#ImportSource) | **Post** /v1/import/{tenant_id}/{project_id}/{environment_id} |
[**IngestNative**](IngestAPI.md#IngestNative) | **Post** /v1/traces/native |
[**IngestOtlp**](IngestAPI.md#IngestOtlp) | **Post** /v1/otlp/{tenant_id}/{project_id}/{environment_id}/v1/traces |
[**IngestOtlpJsonCollector**](IngestAPI.md#IngestOtlpJsonCollector) | **Post** /v1/traces |
[**ReconcileTrace**](IngestAPI.md#ReconcileTrace) | **Post** /v1/ingest/{tenant_id}/{project_id}/traces/{trace_id}/reconcile |
[**ReplayDeadLetter**](IngestAPI.md#ReplayDeadLetter) | **Post** /v1/ingest/{tenant_id}/{project_id}/dead-letters/{message_id}/replay |



## DrainTraceIngested

> TraceIngestedDrainReport DrainTraceIngested(ctx, tenantId, projectId).Limit(limit).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()



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
	limit := int32(56) // int32 |  (optional)
	authorization := "authorization_example" // string | Bearer API token for strict auth (optional)
	xBeaterApiKey := "xBeaterApiKey_example" // string | API key alternative for strict auth (optional)
	xBeaterProjectId := "xBeaterProjectId_example" // string | Strict-auth project scope (optional)
	xBeaterEnvironmentId := "xBeaterEnvironmentId_example" // string | Strict-auth environment scope (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.IngestAPI.DrainTraceIngested(context.Background(), tenantId, projectId).Limit(limit).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `IngestAPI.DrainTraceIngested``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `DrainTraceIngested`: TraceIngestedDrainReport
	fmt.Fprintf(os.Stdout, "Response from `IngestAPI.DrainTraceIngested`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**tenantId** | **string** | tenant_id |
**projectId** | **string** | project_id |

### Other Parameters

Other parameters are passed through a pointer to a apiDrainTraceIngestedRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------


 **limit** | **int32** |  |
 **authorization** | **string** | Bearer API token for strict auth |
 **xBeaterApiKey** | **string** | API key alternative for strict auth |
 **xBeaterProjectId** | **string** | Strict-auth project scope |
 **xBeaterEnvironmentId** | **string** | Strict-auth environment scope |

### Return type

[**TraceIngestedDrainReport**](TraceIngestedDrainReport.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## DrainTraceWrites

> TraceWriteDrainReport DrainTraceWrites(ctx, tenantId, projectId).Limit(limit).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()



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
	limit := int32(56) // int32 |  (optional)
	authorization := "authorization_example" // string | Bearer API token for strict auth (optional)
	xBeaterApiKey := "xBeaterApiKey_example" // string | API key alternative for strict auth (optional)
	xBeaterProjectId := "xBeaterProjectId_example" // string | Strict-auth project scope (optional)
	xBeaterEnvironmentId := "xBeaterEnvironmentId_example" // string | Strict-auth environment scope (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.IngestAPI.DrainTraceWrites(context.Background(), tenantId, projectId).Limit(limit).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `IngestAPI.DrainTraceWrites``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `DrainTraceWrites`: TraceWriteDrainReport
	fmt.Fprintf(os.Stdout, "Response from `IngestAPI.DrainTraceWrites`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**tenantId** | **string** | tenant_id |
**projectId** | **string** | project_id |

### Other Parameters

Other parameters are passed through a pointer to a apiDrainTraceWritesRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------


 **limit** | **int32** |  |
 **authorization** | **string** | Bearer API token for strict auth |
 **xBeaterApiKey** | **string** | API key alternative for strict auth |
 **xBeaterProjectId** | **string** | Strict-auth project scope |
 **xBeaterEnvironmentId** | **string** | Strict-auth environment scope |

### Return type

[**TraceWriteDrainReport**](TraceWriteDrainReport.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## GetIngestQueueStatus

> IngestQueueStatus GetIngestQueueStatus(ctx, tenantId, projectId).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()



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
	authorization := "authorization_example" // string | Bearer API token for strict auth (optional)
	xBeaterApiKey := "xBeaterApiKey_example" // string | API key alternative for strict auth (optional)
	xBeaterProjectId := "xBeaterProjectId_example" // string | Strict-auth project scope (optional)
	xBeaterEnvironmentId := "xBeaterEnvironmentId_example" // string | Strict-auth environment scope (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.IngestAPI.GetIngestQueueStatus(context.Background(), tenantId, projectId).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `IngestAPI.GetIngestQueueStatus``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetIngestQueueStatus`: IngestQueueStatus
	fmt.Fprintf(os.Stdout, "Response from `IngestAPI.GetIngestQueueStatus`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**tenantId** | **string** | tenant_id |
**projectId** | **string** | project_id |

### Other Parameters

Other parameters are passed through a pointer to a apiGetIngestQueueStatusRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------


 **authorization** | **string** | Bearer API token for strict auth |
 **xBeaterApiKey** | **string** | API key alternative for strict auth |
 **xBeaterProjectId** | **string** | Strict-auth project scope |
 **xBeaterEnvironmentId** | **string** | Strict-auth environment scope |

### Return type

[**IngestQueueStatus**](IngestQueueStatus.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## ImportSource

> IngestOutcome ImportSource(ctx, tenantId, projectId, environmentId).ImportSourceHttpRequest(importSourceHttpRequest).Durability(durability).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).Execute()



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
	importSourceHttpRequest := *openapiclient.NewImportSourceHttpRequest("Source_example") // ImportSourceHttpRequest |
	durability := "durability_example" // string |  (optional)
	authorization := "authorization_example" // string | Bearer API token for strict auth (optional)
	xBeaterApiKey := "xBeaterApiKey_example" // string | API key alternative for strict auth (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.IngestAPI.ImportSource(context.Background(), tenantId, projectId, environmentId).ImportSourceHttpRequest(importSourceHttpRequest).Durability(durability).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `IngestAPI.ImportSource``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `ImportSource`: IngestOutcome
	fmt.Fprintf(os.Stdout, "Response from `IngestAPI.ImportSource`: %v\n", resp)
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

Other parameters are passed through a pointer to a apiImportSourceRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------



 **importSourceHttpRequest** | [**ImportSourceHttpRequest**](ImportSourceHttpRequest.md) |  |
 **durability** | **string** |  |
 **authorization** | **string** | Bearer API token for strict auth |
 **xBeaterApiKey** | **string** | API key alternative for strict auth |

### Return type

[**IngestOutcome**](IngestOutcome.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## IngestNative

> IngestOutcome IngestNative(ctx).NativeIngestRequest(nativeIngestRequest).Durability(durability).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()



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
	nativeIngestRequest := *openapiclient.NewNativeIngestRequest(map[string]interface{}{"key": interface{}(123)}, "Kind_example", "Name_example", openapiclient.RedactionClass("public"), *openapiclient.NewTenantScope("EnvironmentId_example", "ProjectId_example", "TenantId_example"), int64(123), "SpanId_example", openapiclient.SpanStatus("ok"), "TraceId_example") // NativeIngestRequest |
	durability := "durability_example" // string |  (optional)
	authorization := "authorization_example" // string | Bearer API token for strict auth (optional)
	xBeaterApiKey := "xBeaterApiKey_example" // string | API key alternative for strict auth (optional)
	xBeaterProjectId := "xBeaterProjectId_example" // string | Strict-auth project scope (optional)
	xBeaterEnvironmentId := "xBeaterEnvironmentId_example" // string | Strict-auth environment scope (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.IngestAPI.IngestNative(context.Background()).NativeIngestRequest(nativeIngestRequest).Durability(durability).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `IngestAPI.IngestNative``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `IngestNative`: IngestOutcome
	fmt.Fprintf(os.Stdout, "Response from `IngestAPI.IngestNative`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiIngestNativeRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **nativeIngestRequest** | [**NativeIngestRequest**](NativeIngestRequest.md) |  |
 **durability** | **string** |  |
 **authorization** | **string** | Bearer API token for strict auth |
 **xBeaterApiKey** | **string** | API key alternative for strict auth |
 **xBeaterProjectId** | **string** | Strict-auth project scope |
 **xBeaterEnvironmentId** | **string** | Strict-auth environment scope |

### Return type

[**IngestOutcome**](IngestOutcome.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## IngestOtlp

> OtlpIngestOutcome IngestOtlp(ctx, tenantId, projectId, environmentId).Durability(durability).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()



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
	durability := "durability_example" // string |  (optional)
	authorization := "authorization_example" // string | Bearer API token for strict auth (optional)
	xBeaterApiKey := "xBeaterApiKey_example" // string | API key alternative for strict auth (optional)
	xBeaterProjectId := "xBeaterProjectId_example" // string | Strict-auth project scope (optional)
	xBeaterEnvironmentId := "xBeaterEnvironmentId_example" // string | Strict-auth environment scope (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.IngestAPI.IngestOtlp(context.Background(), tenantId, projectId, environmentId).Durability(durability).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `IngestAPI.IngestOtlp``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `IngestOtlp`: OtlpIngestOutcome
	fmt.Fprintf(os.Stdout, "Response from `IngestAPI.IngestOtlp`: %v\n", resp)
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

Other parameters are passed through a pointer to a apiIngestOtlpRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------



 **durability** | **string** |  |
 **authorization** | **string** | Bearer API token for strict auth |
 **xBeaterApiKey** | **string** | API key alternative for strict auth |
 **xBeaterProjectId** | **string** | Strict-auth project scope |
 **xBeaterEnvironmentId** | **string** | Strict-auth environment scope |

### Return type

[**OtlpIngestOutcome**](OtlpIngestOutcome.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## IngestOtlpJsonCollector

> OtlpIngestOutcome IngestOtlpJsonCollector(ctx).Durability(durability).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterTenantId(xBeaterTenantId).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()



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
	durability := "durability_example" // string |  (optional)
	authorization := "authorization_example" // string | Bearer API token for strict auth (optional)
	xBeaterApiKey := "xBeaterApiKey_example" // string | API key alternative for strict auth (optional)
	xBeaterTenantId := "xBeaterTenantId_example" // string | Tenant scope override for collector-style OTLP JSON (optional)
	xBeaterProjectId := "xBeaterProjectId_example" // string | Project scope override for collector-style OTLP JSON (optional)
	xBeaterEnvironmentId := "xBeaterEnvironmentId_example" // string | Environment scope override for collector-style OTLP JSON (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.IngestAPI.IngestOtlpJsonCollector(context.Background()).Durability(durability).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterTenantId(xBeaterTenantId).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `IngestAPI.IngestOtlpJsonCollector``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `IngestOtlpJsonCollector`: OtlpIngestOutcome
	fmt.Fprintf(os.Stdout, "Response from `IngestAPI.IngestOtlpJsonCollector`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiIngestOtlpJsonCollectorRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **durability** | **string** |  |
 **authorization** | **string** | Bearer API token for strict auth |
 **xBeaterApiKey** | **string** | API key alternative for strict auth |
 **xBeaterTenantId** | **string** | Tenant scope override for collector-style OTLP JSON |
 **xBeaterProjectId** | **string** | Project scope override for collector-style OTLP JSON |
 **xBeaterEnvironmentId** | **string** | Environment scope override for collector-style OTLP JSON |

### Return type

[**OtlpIngestOutcome**](OtlpIngestOutcome.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## ReconcileTrace

> TraceIngestedReconcileReport ReconcileTrace(ctx, tenantId, projectId, traceId).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()



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
	traceId := "traceId_example" // string | trace_id
	authorization := "authorization_example" // string | Bearer API token for strict auth (optional)
	xBeaterApiKey := "xBeaterApiKey_example" // string | API key alternative for strict auth (optional)
	xBeaterProjectId := "xBeaterProjectId_example" // string | Strict-auth project scope (optional)
	xBeaterEnvironmentId := "xBeaterEnvironmentId_example" // string | Strict-auth environment scope (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.IngestAPI.ReconcileTrace(context.Background(), tenantId, projectId, traceId).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `IngestAPI.ReconcileTrace``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `ReconcileTrace`: TraceIngestedReconcileReport
	fmt.Fprintf(os.Stdout, "Response from `IngestAPI.ReconcileTrace`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**tenantId** | **string** | tenant_id |
**projectId** | **string** | project_id |
**traceId** | **string** | trace_id |

### Other Parameters

Other parameters are passed through a pointer to a apiReconcileTraceRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------



 **authorization** | **string** | Bearer API token for strict auth |
 **xBeaterApiKey** | **string** | API key alternative for strict auth |
 **xBeaterProjectId** | **string** | Strict-auth project scope |
 **xBeaterEnvironmentId** | **string** | Strict-auth environment scope |

### Return type

[**TraceIngestedReconcileReport**](TraceIngestedReconcileReport.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## ReplayDeadLetter

> DeadLetterReplayReport ReplayDeadLetter(ctx, tenantId, projectId, messageId).ResetAttempts(resetAttempts).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()



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
	messageId := "messageId_example" // string | message_id
	resetAttempts := true // bool |  (optional)
	authorization := "authorization_example" // string | Bearer API token for strict auth (optional)
	xBeaterApiKey := "xBeaterApiKey_example" // string | API key alternative for strict auth (optional)
	xBeaterProjectId := "xBeaterProjectId_example" // string | Strict-auth project scope (optional)
	xBeaterEnvironmentId := "xBeaterEnvironmentId_example" // string | Strict-auth environment scope (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.IngestAPI.ReplayDeadLetter(context.Background(), tenantId, projectId, messageId).ResetAttempts(resetAttempts).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `IngestAPI.ReplayDeadLetter``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `ReplayDeadLetter`: DeadLetterReplayReport
	fmt.Fprintf(os.Stdout, "Response from `IngestAPI.ReplayDeadLetter`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**tenantId** | **string** | tenant_id |
**projectId** | **string** | project_id |
**messageId** | **string** | message_id |

### Other Parameters

Other parameters are passed through a pointer to a apiReplayDeadLetterRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------



 **resetAttempts** | **bool** |  |
 **authorization** | **string** | Bearer API token for strict auth |
 **xBeaterApiKey** | **string** | API key alternative for strict auth |
 **xBeaterProjectId** | **string** | Strict-auth project scope |
 **xBeaterEnvironmentId** | **string** | Strict-auth environment scope |

### Return type

[**DeadLetterReplayReport**](DeadLetterReplayReport.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)
