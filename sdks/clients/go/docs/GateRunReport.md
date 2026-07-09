# GateRunReport

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**BaselineReleaseId** | **string** |  |
**CandidateReleaseId** | **string** |  |
**Comparison** | [**ExperimentComparison**](ExperimentComparison.md) |  |
**CreatedAt** | **time.Time** |  |
**DatasetId** | **string** |  |
**EvaluatorVersionId** | **string** |  |
**ExperimentCreatedAt** | **time.Time** |  |
**ExperimentDecision** | [**GateDecision**](GateDecision.md) |  |
**ExperimentGatePolicy** | [**GatePolicy**](GatePolicy.md) |  |
**ExperimentRunId** | **string** |  |
**GateDatasetId** | Pointer to **string** |  | [optional]
**GateEvaluatorVersionId** | Pointer to **string** |  | [optional]
**GateId** | **string** |  |
**GateName** | **string** |  |
**GateRunId** | **string** |  |
**InconclusivePolicy** | [**InconclusivePolicy**](InconclusivePolicy.md) |  |
**Passed** | **bool** |  |
**ProjectId** | **string** |  |
**Reason** | **string** |  |
**TenantId** | **string** |  |

## Methods

### NewGateRunReport

`func NewGateRunReport(baselineReleaseId string, candidateReleaseId string, comparison ExperimentComparison, createdAt time.Time, datasetId string, evaluatorVersionId string, experimentCreatedAt time.Time, experimentDecision GateDecision, experimentGatePolicy GatePolicy, experimentRunId string, gateId string, gateName string, gateRunId string, inconclusivePolicy InconclusivePolicy, passed bool, projectId string, reason string, tenantId string, ) *GateRunReport`

NewGateRunReport instantiates a new GateRunReport object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewGateRunReportWithDefaults

`func NewGateRunReportWithDefaults() *GateRunReport`

NewGateRunReportWithDefaults instantiates a new GateRunReport object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetBaselineReleaseId

`func (o *GateRunReport) GetBaselineReleaseId() string`

GetBaselineReleaseId returns the BaselineReleaseId field if non-nil, zero value otherwise.

### GetBaselineReleaseIdOk

`func (o *GateRunReport) GetBaselineReleaseIdOk() (*string, bool)`

GetBaselineReleaseIdOk returns a tuple with the BaselineReleaseId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBaselineReleaseId

`func (o *GateRunReport) SetBaselineReleaseId(v string)`

SetBaselineReleaseId sets BaselineReleaseId field to given value.


### GetCandidateReleaseId

`func (o *GateRunReport) GetCandidateReleaseId() string`

GetCandidateReleaseId returns the CandidateReleaseId field if non-nil, zero value otherwise.

### GetCandidateReleaseIdOk

`func (o *GateRunReport) GetCandidateReleaseIdOk() (*string, bool)`

GetCandidateReleaseIdOk returns a tuple with the CandidateReleaseId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCandidateReleaseId

`func (o *GateRunReport) SetCandidateReleaseId(v string)`

SetCandidateReleaseId sets CandidateReleaseId field to given value.


### GetComparison

`func (o *GateRunReport) GetComparison() ExperimentComparison`

GetComparison returns the Comparison field if non-nil, zero value otherwise.

### GetComparisonOk

`func (o *GateRunReport) GetComparisonOk() (*ExperimentComparison, bool)`

GetComparisonOk returns a tuple with the Comparison field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetComparison

`func (o *GateRunReport) SetComparison(v ExperimentComparison)`

SetComparison sets Comparison field to given value.


### GetCreatedAt

`func (o *GateRunReport) GetCreatedAt() time.Time`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *GateRunReport) GetCreatedAtOk() (*time.Time, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *GateRunReport) SetCreatedAt(v time.Time)`

SetCreatedAt sets CreatedAt field to given value.


### GetDatasetId

`func (o *GateRunReport) GetDatasetId() string`

GetDatasetId returns the DatasetId field if non-nil, zero value otherwise.

### GetDatasetIdOk

`func (o *GateRunReport) GetDatasetIdOk() (*string, bool)`

GetDatasetIdOk returns a tuple with the DatasetId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDatasetId

`func (o *GateRunReport) SetDatasetId(v string)`

SetDatasetId sets DatasetId field to given value.


### GetEvaluatorVersionId

`func (o *GateRunReport) GetEvaluatorVersionId() string`

GetEvaluatorVersionId returns the EvaluatorVersionId field if non-nil, zero value otherwise.

### GetEvaluatorVersionIdOk

`func (o *GateRunReport) GetEvaluatorVersionIdOk() (*string, bool)`

GetEvaluatorVersionIdOk returns a tuple with the EvaluatorVersionId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEvaluatorVersionId

`func (o *GateRunReport) SetEvaluatorVersionId(v string)`

SetEvaluatorVersionId sets EvaluatorVersionId field to given value.


### GetExperimentCreatedAt

`func (o *GateRunReport) GetExperimentCreatedAt() time.Time`

GetExperimentCreatedAt returns the ExperimentCreatedAt field if non-nil, zero value otherwise.

### GetExperimentCreatedAtOk

`func (o *GateRunReport) GetExperimentCreatedAtOk() (*time.Time, bool)`

GetExperimentCreatedAtOk returns a tuple with the ExperimentCreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetExperimentCreatedAt

`func (o *GateRunReport) SetExperimentCreatedAt(v time.Time)`

SetExperimentCreatedAt sets ExperimentCreatedAt field to given value.


### GetExperimentDecision

`func (o *GateRunReport) GetExperimentDecision() GateDecision`

GetExperimentDecision returns the ExperimentDecision field if non-nil, zero value otherwise.

### GetExperimentDecisionOk

`func (o *GateRunReport) GetExperimentDecisionOk() (*GateDecision, bool)`

GetExperimentDecisionOk returns a tuple with the ExperimentDecision field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetExperimentDecision

`func (o *GateRunReport) SetExperimentDecision(v GateDecision)`

SetExperimentDecision sets ExperimentDecision field to given value.


### GetExperimentGatePolicy

`func (o *GateRunReport) GetExperimentGatePolicy() GatePolicy`

GetExperimentGatePolicy returns the ExperimentGatePolicy field if non-nil, zero value otherwise.

### GetExperimentGatePolicyOk

`func (o *GateRunReport) GetExperimentGatePolicyOk() (*GatePolicy, bool)`

GetExperimentGatePolicyOk returns a tuple with the ExperimentGatePolicy field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetExperimentGatePolicy

`func (o *GateRunReport) SetExperimentGatePolicy(v GatePolicy)`

SetExperimentGatePolicy sets ExperimentGatePolicy field to given value.


### GetExperimentRunId

`func (o *GateRunReport) GetExperimentRunId() string`

GetExperimentRunId returns the ExperimentRunId field if non-nil, zero value otherwise.

### GetExperimentRunIdOk

`func (o *GateRunReport) GetExperimentRunIdOk() (*string, bool)`

GetExperimentRunIdOk returns a tuple with the ExperimentRunId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetExperimentRunId

`func (o *GateRunReport) SetExperimentRunId(v string)`

SetExperimentRunId sets ExperimentRunId field to given value.


### GetGateDatasetId

`func (o *GateRunReport) GetGateDatasetId() string`

GetGateDatasetId returns the GateDatasetId field if non-nil, zero value otherwise.

### GetGateDatasetIdOk

`func (o *GateRunReport) GetGateDatasetIdOk() (*string, bool)`

GetGateDatasetIdOk returns a tuple with the GateDatasetId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetGateDatasetId

`func (o *GateRunReport) SetGateDatasetId(v string)`

SetGateDatasetId sets GateDatasetId field to given value.

### HasGateDatasetId

`func (o *GateRunReport) HasGateDatasetId() bool`

HasGateDatasetId returns a boolean if a field has been set.

### GetGateEvaluatorVersionId

`func (o *GateRunReport) GetGateEvaluatorVersionId() string`

GetGateEvaluatorVersionId returns the GateEvaluatorVersionId field if non-nil, zero value otherwise.

### GetGateEvaluatorVersionIdOk

`func (o *GateRunReport) GetGateEvaluatorVersionIdOk() (*string, bool)`

GetGateEvaluatorVersionIdOk returns a tuple with the GateEvaluatorVersionId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetGateEvaluatorVersionId

`func (o *GateRunReport) SetGateEvaluatorVersionId(v string)`

SetGateEvaluatorVersionId sets GateEvaluatorVersionId field to given value.

### HasGateEvaluatorVersionId

`func (o *GateRunReport) HasGateEvaluatorVersionId() bool`

HasGateEvaluatorVersionId returns a boolean if a field has been set.

### GetGateId

`func (o *GateRunReport) GetGateId() string`

GetGateId returns the GateId field if non-nil, zero value otherwise.

### GetGateIdOk

`func (o *GateRunReport) GetGateIdOk() (*string, bool)`

GetGateIdOk returns a tuple with the GateId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetGateId

`func (o *GateRunReport) SetGateId(v string)`

SetGateId sets GateId field to given value.


### GetGateName

`func (o *GateRunReport) GetGateName() string`

GetGateName returns the GateName field if non-nil, zero value otherwise.

### GetGateNameOk

`func (o *GateRunReport) GetGateNameOk() (*string, bool)`

GetGateNameOk returns a tuple with the GateName field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetGateName

`func (o *GateRunReport) SetGateName(v string)`

SetGateName sets GateName field to given value.


### GetGateRunId

`func (o *GateRunReport) GetGateRunId() string`

GetGateRunId returns the GateRunId field if non-nil, zero value otherwise.

### GetGateRunIdOk

`func (o *GateRunReport) GetGateRunIdOk() (*string, bool)`

GetGateRunIdOk returns a tuple with the GateRunId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetGateRunId

`func (o *GateRunReport) SetGateRunId(v string)`

SetGateRunId sets GateRunId field to given value.


### GetInconclusivePolicy

`func (o *GateRunReport) GetInconclusivePolicy() InconclusivePolicy`

GetInconclusivePolicy returns the InconclusivePolicy field if non-nil, zero value otherwise.

### GetInconclusivePolicyOk

`func (o *GateRunReport) GetInconclusivePolicyOk() (*InconclusivePolicy, bool)`

GetInconclusivePolicyOk returns a tuple with the InconclusivePolicy field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetInconclusivePolicy

`func (o *GateRunReport) SetInconclusivePolicy(v InconclusivePolicy)`

SetInconclusivePolicy sets InconclusivePolicy field to given value.


### GetPassed

`func (o *GateRunReport) GetPassed() bool`

GetPassed returns the Passed field if non-nil, zero value otherwise.

### GetPassedOk

`func (o *GateRunReport) GetPassedOk() (*bool, bool)`

GetPassedOk returns a tuple with the Passed field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPassed

`func (o *GateRunReport) SetPassed(v bool)`

SetPassed sets Passed field to given value.


### GetProjectId

`func (o *GateRunReport) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *GateRunReport) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *GateRunReport) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetReason

`func (o *GateRunReport) GetReason() string`

GetReason returns the Reason field if non-nil, zero value otherwise.

### GetReasonOk

`func (o *GateRunReport) GetReasonOk() (*string, bool)`

GetReasonOk returns a tuple with the Reason field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetReason

`func (o *GateRunReport) SetReason(v string)`

SetReason sets Reason field to given value.


### GetTenantId

`func (o *GateRunReport) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *GateRunReport) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *GateRunReport) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
