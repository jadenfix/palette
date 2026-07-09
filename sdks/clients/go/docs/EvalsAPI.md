# \EvalsAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**EvalsRunDeterministicEval**](EvalsAPI.md#EvalsRunDeterministicEval) | **Post** /v1/datasets/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id}/evals/deterministic |
[**EvalsRunJudgeEval**](EvalsAPI.md#EvalsRunJudgeEval) | **Post** /v1/datasets/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id}/evals/judge |



## EvalsRunDeterministicEval

> DatasetEvalReport EvalsRunDeterministicEval(ctx, tenantId, projectId, datasetId, versionId).RunDeterministicEvalRequest(runDeterministicEvalRequest).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()



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
	runDeterministicEvalRequest := *openapiclient.NewRunDeterministicEvalRequest("AgentReleaseId_example", "EvaluatorId_example", "EvaluatorVersionId_example", openapiclient.EvaluatorKind{EvaluatorKindOneOf: openapiclient.NewEvaluatorKindOneOf("Type_example")}) // RunDeterministicEvalRequest |
	authorization := "authorization_example" // string | Bearer API token for strict auth (optional)
	xBeaterApiKey := "xBeaterApiKey_example" // string | API key alternative for strict auth (optional)
	xBeaterProjectId := "xBeaterProjectId_example" // string | Strict-auth project scope (optional)
	xBeaterEnvironmentId := "xBeaterEnvironmentId_example" // string | Strict-auth environment scope (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.EvalsAPI.EvalsRunDeterministicEval(context.Background(), tenantId, projectId, datasetId, versionId).RunDeterministicEvalRequest(runDeterministicEvalRequest).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `EvalsAPI.EvalsRunDeterministicEval``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `EvalsRunDeterministicEval`: DatasetEvalReport
	fmt.Fprintf(os.Stdout, "Response from `EvalsAPI.EvalsRunDeterministicEval`: %v\n", resp)
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

Other parameters are passed through a pointer to a apiEvalsRunDeterministicEvalRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------




 **runDeterministicEvalRequest** | [**RunDeterministicEvalRequest**](RunDeterministicEvalRequest.md) |  |
 **authorization** | **string** | Bearer API token for strict auth |
 **xBeaterApiKey** | **string** | API key alternative for strict auth |
 **xBeaterProjectId** | **string** | Strict-auth project scope |
 **xBeaterEnvironmentId** | **string** | Strict-auth environment scope |

### Return type

[**DatasetEvalReport**](DatasetEvalReport.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## EvalsRunJudgeEval

> DatasetEvalReport EvalsRunJudgeEval(ctx, tenantId, projectId, datasetId, versionId).RunJudgeDatasetEvalRequest(runJudgeDatasetEvalRequest).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()



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
	runJudgeDatasetEvalRequest := *openapiclient.NewRunJudgeDatasetEvalRequest("AgentReleaseId_example", "EvaluatorId_example", "EvaluatorVersionId_example", openapiclient.EvaluatorKind{EvaluatorKindOneOf: openapiclient.NewEvaluatorKindOneOf("Type_example")}, "ProviderSecretId_example") // RunJudgeDatasetEvalRequest |
	authorization := "authorization_example" // string | Bearer API token for strict auth (optional)
	xBeaterApiKey := "xBeaterApiKey_example" // string | API key alternative for strict auth (optional)
	xBeaterProjectId := "xBeaterProjectId_example" // string | Strict-auth project scope (optional)
	xBeaterEnvironmentId := "xBeaterEnvironmentId_example" // string | Strict-auth environment scope (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.EvalsAPI.EvalsRunJudgeEval(context.Background(), tenantId, projectId, datasetId, versionId).RunJudgeDatasetEvalRequest(runJudgeDatasetEvalRequest).Authorization(authorization).XBeaterApiKey(xBeaterApiKey).XBeaterProjectId(xBeaterProjectId).XBeaterEnvironmentId(xBeaterEnvironmentId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `EvalsAPI.EvalsRunJudgeEval``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `EvalsRunJudgeEval`: DatasetEvalReport
	fmt.Fprintf(os.Stdout, "Response from `EvalsAPI.EvalsRunJudgeEval`: %v\n", resp)
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

Other parameters are passed through a pointer to a apiEvalsRunJudgeEvalRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------




 **runJudgeDatasetEvalRequest** | [**RunJudgeDatasetEvalRequest**](RunJudgeDatasetEvalRequest.md) |  |
 **authorization** | **string** | Bearer API token for strict auth |
 **xBeaterApiKey** | **string** | API key alternative for strict auth |
 **xBeaterProjectId** | **string** | Strict-auth project scope |
 **xBeaterEnvironmentId** | **string** | Strict-auth environment scope |

### Return type

[**DatasetEvalReport**](DatasetEvalReport.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)
