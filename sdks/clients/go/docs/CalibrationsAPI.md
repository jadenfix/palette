# \CalibrationsAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**CalibrationsRunCalibration**](CalibrationsAPI.md#CalibrationsRunCalibration) | **Post** /v1/calibrations/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id} |



## CalibrationsRunCalibration

> CalibrationReport CalibrationsRunCalibration(ctx, tenantId, projectId, datasetId, versionId).RunCalibrationHttpRequest(runCalibrationHttpRequest).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()



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
	datasetId := "datasetId_example" // string | dataset_id
	versionId := "versionId_example" // string | version_id
	runCalibrationHttpRequest := *openapiclient.NewRunCalibrationHttpRequest() // RunCalibrationHttpRequest |
	authorization := "authorization_example" // string | Bearer API token for strict auth (optional)
	xBeaterApiKey := "xBeaterApiKey_example" // string | API key alternative for strict auth (optional)
	xBeaterProjectId := "xBeaterProjectId_example" // string | Strict-auth project scope (optional)
	xBeaterEnvironmentId := "xBeaterEnvironmentId_example" // string | Strict-auth environment scope (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.CalibrationsAPI.CalibrationsRunCalibration(context.Background(), tenantId, projectId, datasetId, versionId).RunCalibrationHttpRequest(runCalibrationHttpRequest).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `CalibrationsAPI.CalibrationsRunCalibration``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `CalibrationsRunCalibration`: CalibrationReport
	fmt.Fprintf(os.Stdout, "Response from `CalibrationsAPI.CalibrationsRunCalibration`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**tenantId** | **string** | tenant_id |
**projectId** | **string** | project_id |
**datasetId** | **string** | dataset_id |
**versionId** | **string** | version_id |

### Other Parameters

Other parameters are passed through a pointer to a apiCalibrationsRunCalibrationRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------




 **runCalibrationHttpRequest** | [**RunCalibrationHttpRequest**](RunCalibrationHttpRequest.md) |  |
 **authorization** | **string** | Bearer API token for strict auth |
 **xBeaterApiKey** | **string** | API key alternative for strict auth |
 **xBeaterProjectId** | **string** | Strict-auth project scope |
 **xBeaterEnvironmentId** | **string** | Strict-auth environment scope |

### Return type

[**CalibrationReport**](CalibrationReport.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)
