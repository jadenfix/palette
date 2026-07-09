# ExperimentRunReport

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**BaselineReleaseId** | **string** |  |
**CandidateReleaseId** | **string** |  |
**CaseScores** | [**[]CaseExperimentScore**](CaseExperimentScore.md) |  |
**Comparison** | [**ExperimentComparison**](ExperimentComparison.md) |  |
**CreatedAt** | **time.Time** |  |
**DatasetId** | **string** |  |
**DatasetVersionId** | **string** |  |
**Decision** | [**GateDecision**](GateDecision.md) |  |
**EvaluatorVersionId** | **string** |  |
**ExperimentRunId** | **string** |  |
**GatePolicy** | Pointer to [**GatePolicy**](GatePolicy.md) |  | [optional]
**ProjectId** | **string** |  |
**TenantId** | **string** |  |

## Methods

### NewExperimentRunReport

`func NewExperimentRunReport(baselineReleaseId string, candidateReleaseId string, caseScores []CaseExperimentScore, comparison ExperimentComparison, createdAt time.Time, datasetId string, datasetVersionId string, decision GateDecision, evaluatorVersionId string, experimentRunId string, projectId string, tenantId string, ) *ExperimentRunReport`

NewExperimentRunReport instantiates a new ExperimentRunReport object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewExperimentRunReportWithDefaults

`func NewExperimentRunReportWithDefaults() *ExperimentRunReport`

NewExperimentRunReportWithDefaults instantiates a new ExperimentRunReport object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetBaselineReleaseId

`func (o *ExperimentRunReport) GetBaselineReleaseId() string`

GetBaselineReleaseId returns the BaselineReleaseId field if non-nil, zero value otherwise.

### GetBaselineReleaseIdOk

`func (o *ExperimentRunReport) GetBaselineReleaseIdOk() (*string, bool)`

GetBaselineReleaseIdOk returns a tuple with the BaselineReleaseId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBaselineReleaseId

`func (o *ExperimentRunReport) SetBaselineReleaseId(v string)`

SetBaselineReleaseId sets BaselineReleaseId field to given value.


### GetCandidateReleaseId

`func (o *ExperimentRunReport) GetCandidateReleaseId() string`

GetCandidateReleaseId returns the CandidateReleaseId field if non-nil, zero value otherwise.

### GetCandidateReleaseIdOk

`func (o *ExperimentRunReport) GetCandidateReleaseIdOk() (*string, bool)`

GetCandidateReleaseIdOk returns a tuple with the CandidateReleaseId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCandidateReleaseId

`func (o *ExperimentRunReport) SetCandidateReleaseId(v string)`

SetCandidateReleaseId sets CandidateReleaseId field to given value.


### GetCaseScores

`func (o *ExperimentRunReport) GetCaseScores() []CaseExperimentScore`

GetCaseScores returns the CaseScores field if non-nil, zero value otherwise.

### GetCaseScoresOk

`func (o *ExperimentRunReport) GetCaseScoresOk() (*[]CaseExperimentScore, bool)`

GetCaseScoresOk returns a tuple with the CaseScores field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCaseScores

`func (o *ExperimentRunReport) SetCaseScores(v []CaseExperimentScore)`

SetCaseScores sets CaseScores field to given value.


### GetComparison

`func (o *ExperimentRunReport) GetComparison() ExperimentComparison`

GetComparison returns the Comparison field if non-nil, zero value otherwise.

### GetComparisonOk

`func (o *ExperimentRunReport) GetComparisonOk() (*ExperimentComparison, bool)`

GetComparisonOk returns a tuple with the Comparison field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetComparison

`func (o *ExperimentRunReport) SetComparison(v ExperimentComparison)`

SetComparison sets Comparison field to given value.


### GetCreatedAt

`func (o *ExperimentRunReport) GetCreatedAt() time.Time`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *ExperimentRunReport) GetCreatedAtOk() (*time.Time, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *ExperimentRunReport) SetCreatedAt(v time.Time)`

SetCreatedAt sets CreatedAt field to given value.


### GetDatasetId

`func (o *ExperimentRunReport) GetDatasetId() string`

GetDatasetId returns the DatasetId field if non-nil, zero value otherwise.

### GetDatasetIdOk

`func (o *ExperimentRunReport) GetDatasetIdOk() (*string, bool)`

GetDatasetIdOk returns a tuple with the DatasetId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDatasetId

`func (o *ExperimentRunReport) SetDatasetId(v string)`

SetDatasetId sets DatasetId field to given value.


### GetDatasetVersionId

`func (o *ExperimentRunReport) GetDatasetVersionId() string`

GetDatasetVersionId returns the DatasetVersionId field if non-nil, zero value otherwise.

### GetDatasetVersionIdOk

`func (o *ExperimentRunReport) GetDatasetVersionIdOk() (*string, bool)`

GetDatasetVersionIdOk returns a tuple with the DatasetVersionId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDatasetVersionId

`func (o *ExperimentRunReport) SetDatasetVersionId(v string)`

SetDatasetVersionId sets DatasetVersionId field to given value.


### GetDecision

`func (o *ExperimentRunReport) GetDecision() GateDecision`

GetDecision returns the Decision field if non-nil, zero value otherwise.

### GetDecisionOk

`func (o *ExperimentRunReport) GetDecisionOk() (*GateDecision, bool)`

GetDecisionOk returns a tuple with the Decision field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDecision

`func (o *ExperimentRunReport) SetDecision(v GateDecision)`

SetDecision sets Decision field to given value.


### GetEvaluatorVersionId

`func (o *ExperimentRunReport) GetEvaluatorVersionId() string`

GetEvaluatorVersionId returns the EvaluatorVersionId field if non-nil, zero value otherwise.

### GetEvaluatorVersionIdOk

`func (o *ExperimentRunReport) GetEvaluatorVersionIdOk() (*string, bool)`

GetEvaluatorVersionIdOk returns a tuple with the EvaluatorVersionId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEvaluatorVersionId

`func (o *ExperimentRunReport) SetEvaluatorVersionId(v string)`

SetEvaluatorVersionId sets EvaluatorVersionId field to given value.


### GetExperimentRunId

`func (o *ExperimentRunReport) GetExperimentRunId() string`

GetExperimentRunId returns the ExperimentRunId field if non-nil, zero value otherwise.

### GetExperimentRunIdOk

`func (o *ExperimentRunReport) GetExperimentRunIdOk() (*string, bool)`

GetExperimentRunIdOk returns a tuple with the ExperimentRunId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetExperimentRunId

`func (o *ExperimentRunReport) SetExperimentRunId(v string)`

SetExperimentRunId sets ExperimentRunId field to given value.


### GetGatePolicy

`func (o *ExperimentRunReport) GetGatePolicy() GatePolicy`

GetGatePolicy returns the GatePolicy field if non-nil, zero value otherwise.

### GetGatePolicyOk

`func (o *ExperimentRunReport) GetGatePolicyOk() (*GatePolicy, bool)`

GetGatePolicyOk returns a tuple with the GatePolicy field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetGatePolicy

`func (o *ExperimentRunReport) SetGatePolicy(v GatePolicy)`

SetGatePolicy sets GatePolicy field to given value.

### HasGatePolicy

`func (o *ExperimentRunReport) HasGatePolicy() bool`

HasGatePolicy returns a boolean if a field has been set.

### GetProjectId

`func (o *ExperimentRunReport) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *ExperimentRunReport) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *ExperimentRunReport) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetTenantId

`func (o *ExperimentRunReport) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *ExperimentRunReport) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *ExperimentRunReport) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
