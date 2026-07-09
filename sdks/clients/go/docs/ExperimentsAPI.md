# \ExperimentsAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**ExperimentsRunDeterministicExperiment**](ExperimentsAPI.md#ExperimentsRunDeterministicExperiment) | **Post** /v1/experiments/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id}/deterministic |
[**ExperimentsRunJudgeExperiment**](ExperimentsAPI.md#ExperimentsRunJudgeExperiment) | **Post** /v1/experiments/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id}/judge |



## ExperimentsRunDeterministicExperiment

> ExperimentRunReport ExperimentsRunDeterministicExperiment(ctx, tenantId, projectId, datasetId, versionId).RunExperimentRequest(runExperimentRequest).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()



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
	runExperimentRequest := *openapiclient.NewRunExperimentRequest([]openapiclient.CaseOutputOverrideRequest{*openapiclient.NewCaseOutputOverrideRequest("CaseId_example", interface{}(123))}, "BaselineReleaseId_example", []openapiclient.CaseOutputOverrideRequest{*openapiclient.NewCaseOutputOverrideRequest("CaseId_example", interface{}(123))}, "CandidateReleaseId_example", "EvaluatorId_example", "EvaluatorVersionId_example", openapiclient.EvaluatorKind{EvaluatorKindOneOf: openapiclient.NewEvaluatorKindOneOf("Type_example")}) // RunExperimentRequest |
	authorization := "authorization_example" // string | Bearer API token for strict auth (optional)
	xBeaterApiKey := "xBeaterApiKey_example" // string | API key alternative for strict auth (optional)
	xBeaterProjectId := "xBeaterProjectId_example" // string | Strict-auth project scope (optional)
	xBeaterEnvironmentId := "xBeaterEnvironmentId_example" // string | Strict-auth environment scope (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.ExperimentsAPI.ExperimentsRunDeterministicExperiment(context.Background(), tenantId, projectId, datasetId, versionId).RunExperimentRequest(runExperimentRequest).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `ExperimentsAPI.ExperimentsRunDeterministicExperiment``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `ExperimentsRunDeterministicExperiment`: ExperimentRunReport
	fmt.Fprintf(os.Stdout, "Response from `ExperimentsAPI.ExperimentsRunDeterministicExperiment`: %v\n", resp)
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

Other parameters are passed through a pointer to a apiExperimentsRunDeterministicExperimentRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------




 **runExperimentRequest** | [**RunExperimentRequest**](RunExperimentRequest.md) |  |
 **authorization** | **string** | Bearer API token for strict auth |
 **xBeaterApiKey** | **string** | API key alternative for strict auth |
 **xBeaterProjectId** | **string** | Strict-auth project scope |
 **xBeaterEnvironmentId** | **string** | Strict-auth environment scope |

### Return type

[**ExperimentRunReport**](ExperimentRunReport.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## ExperimentsRunJudgeExperiment

> ExperimentRunReport ExperimentsRunJudgeExperiment(ctx, tenantId, projectId, datasetId, versionId).RunJudgeExperimentRequest(runJudgeExperimentRequest).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()



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
	runJudgeExperimentRequest := *openapiclient.NewRunJudgeExperimentRequest([]openapiclient.CaseOutputOverrideRequest{*openapiclient.NewCaseOutputOverrideRequest("CaseId_example", interface{}(123))}, "BaselineReleaseId_example", []openapiclient.CaseOutputOverrideRequest{*openapiclient.NewCaseOutputOverrideRequest("CaseId_example", interface{}(123))}, "CandidateReleaseId_example", "EvaluatorId_example", "EvaluatorVersionId_example", openapiclient.EvaluatorKind{EvaluatorKindOneOf: openapiclient.NewEvaluatorKindOneOf("Type_example")}, "ProviderSecretId_example") // RunJudgeExperimentRequest |
	authorization := "authorization_example" // string | Bearer API token for strict auth (optional)
	xBeaterApiKey := "xBeaterApiKey_example" // string | API key alternative for strict auth (optional)
	xBeaterProjectId := "xBeaterProjectId_example" // string | Strict-auth project scope (optional)
	xBeaterEnvironmentId := "xBeaterEnvironmentId_example" // string | Strict-auth environment scope (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.ExperimentsAPI.ExperimentsRunJudgeExperiment(context.Background(), tenantId, projectId, datasetId, versionId).RunJudgeExperimentRequest(runJudgeExperimentRequest).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `ExperimentsAPI.ExperimentsRunJudgeExperiment``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `ExperimentsRunJudgeExperiment`: ExperimentRunReport
	fmt.Fprintf(os.Stdout, "Response from `ExperimentsAPI.ExperimentsRunJudgeExperiment`: %v\n", resp)
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

Other parameters are passed through a pointer to a apiExperimentsRunJudgeExperimentRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------




 **runJudgeExperimentRequest** | [**RunJudgeExperimentRequest**](RunJudgeExperimentRequest.md) |  |
 **authorization** | **string** | Bearer API token for strict auth |
 **xBeaterApiKey** | **string** | API key alternative for strict auth |
 **xBeaterProjectId** | **string** | Strict-auth project scope |
 **xBeaterEnvironmentId** | **string** | Strict-auth environment scope |

### Return type

[**ExperimentRunReport**](ExperimentRunReport.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)
