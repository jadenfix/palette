# \AlertsAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**AlertsEvaluateAlert**](AlertsAPI.md#AlertsEvaluateAlert) | **Post** /v1/alerts/{tenant_id}/{project_id}/traces/{trace_id}/webhook |



## AlertsEvaluateAlert

> AlertDecision AlertsEvaluateAlert(ctx, tenantId, projectId, traceId).EvaluateAlertRequest(evaluateAlertRequest).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()



### Example

```go
package main

import (
	"context"
	"fmt"
	"os"
    "time"
	openapiclient "github.com/GIT_USER_ID/GIT_REPO_ID/beaterclient"
)

func main() {
	tenantId := "tenantId_example" // string | tenant_id
	projectId := "projectId_example" // string | project_id
	traceId := "traceId_example" // string | trace_id
	evaluateAlertRequest := *openapiclient.NewEvaluateAlertRequest(*openapiclient.NewAlertInput("GroupKey_example", *openapiclient.NewAlertLinks("TraceUrl_example"), time.Now(), "ProjectId_example", float64(123), "TenantId_example", "Title_example", "TraceId_example"), *openapiclient.NewAlertPolicy(int64(123), "EndpointUrl_example", float64(123), []openapiclient.MaintenanceWindow{*openapiclient.NewMaintenanceWindow(time.Now(), time.Now())}, "PolicyId_example", openapiclient.AlertSeverity("info"), "SigningSecret_example")) // EvaluateAlertRequest |
	authorization := "authorization_example" // string | Bearer API token for strict auth (optional)
	xBeaterApiKey := "xBeaterApiKey_example" // string | API key alternative for strict auth (optional)
	xBeaterProjectId := "xBeaterProjectId_example" // string | Strict-auth project scope (optional)
	xBeaterEnvironmentId := "xBeaterEnvironmentId_example" // string | Strict-auth environment scope (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.AlertsAPI.AlertsEvaluateAlert(context.Background(), tenantId, projectId, traceId).EvaluateAlertRequest(evaluateAlertRequest).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `AlertsAPI.AlertsEvaluateAlert``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `AlertsEvaluateAlert`: AlertDecision
	fmt.Fprintf(os.Stdout, "Response from `AlertsAPI.AlertsEvaluateAlert`: %v\n", resp)
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

Other parameters are passed through a pointer to a apiAlertsEvaluateAlertRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------



 **evaluateAlertRequest** | [**EvaluateAlertRequest**](EvaluateAlertRequest.md) |  |
 **authorization** | **string** | Bearer API token for strict auth |
 **xBeaterApiKey** | **string** | API key alternative for strict auth |
 **xBeaterProjectId** | **string** | Strict-auth project scope |
 **xBeaterEnvironmentId** | **string** | Strict-auth environment scope |

### Return type

[**AlertDecision**](AlertDecision.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)
