# RunExperimentRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**BaselineOutputs** | [**[]CaseOutputOverrideRequest**](CaseOutputOverrideRequest.md) |  |
**BaselineReleaseId** | **string** |  |
**CandidateOutputs** | [**[]CaseOutputOverrideRequest**](CaseOutputOverrideRequest.md) |  |
**CandidateReleaseId** | **string** |  |
**EvaluatorId** | **string** |  |
**EvaluatorVersionId** | **string** |  |
**GatePolicy** | Pointer to [**NullableGatePolicy**](GatePolicy.md) |  | [optional]
**Kind** | [**EvaluatorKind**](EvaluatorKind.md) |  |

## Methods

### NewRunExperimentRequest

`func NewRunExperimentRequest(baselineOutputs []CaseOutputOverrideRequest, baselineReleaseId string, candidateOutputs []CaseOutputOverrideRequest, candidateReleaseId string, evaluatorId string, evaluatorVersionId string, kind EvaluatorKind, ) *RunExperimentRequest`

NewRunExperimentRequest instantiates a new RunExperimentRequest object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewRunExperimentRequestWithDefaults

`func NewRunExperimentRequestWithDefaults() *RunExperimentRequest`

NewRunExperimentRequestWithDefaults instantiates a new RunExperimentRequest object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetBaselineOutputs

`func (o *RunExperimentRequest) GetBaselineOutputs() []CaseOutputOverrideRequest`

GetBaselineOutputs returns the BaselineOutputs field if non-nil, zero value otherwise.

### GetBaselineOutputsOk

`func (o *RunExperimentRequest) GetBaselineOutputsOk() (*[]CaseOutputOverrideRequest, bool)`

GetBaselineOutputsOk returns a tuple with the BaselineOutputs field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBaselineOutputs

`func (o *RunExperimentRequest) SetBaselineOutputs(v []CaseOutputOverrideRequest)`

SetBaselineOutputs sets BaselineOutputs field to given value.


### GetBaselineReleaseId

`func (o *RunExperimentRequest) GetBaselineReleaseId() string`

GetBaselineReleaseId returns the BaselineReleaseId field if non-nil, zero value otherwise.

### GetBaselineReleaseIdOk

`func (o *RunExperimentRequest) GetBaselineReleaseIdOk() (*string, bool)`

GetBaselineReleaseIdOk returns a tuple with the BaselineReleaseId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBaselineReleaseId

`func (o *RunExperimentRequest) SetBaselineReleaseId(v string)`

SetBaselineReleaseId sets BaselineReleaseId field to given value.


### GetCandidateOutputs

`func (o *RunExperimentRequest) GetCandidateOutputs() []CaseOutputOverrideRequest`

GetCandidateOutputs returns the CandidateOutputs field if non-nil, zero value otherwise.

### GetCandidateOutputsOk

`func (o *RunExperimentRequest) GetCandidateOutputsOk() (*[]CaseOutputOverrideRequest, bool)`

GetCandidateOutputsOk returns a tuple with the CandidateOutputs field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCandidateOutputs

`func (o *RunExperimentRequest) SetCandidateOutputs(v []CaseOutputOverrideRequest)`

SetCandidateOutputs sets CandidateOutputs field to given value.


### GetCandidateReleaseId

`func (o *RunExperimentRequest) GetCandidateReleaseId() string`

GetCandidateReleaseId returns the CandidateReleaseId field if non-nil, zero value otherwise.

### GetCandidateReleaseIdOk

`func (o *RunExperimentRequest) GetCandidateReleaseIdOk() (*string, bool)`

GetCandidateReleaseIdOk returns a tuple with the CandidateReleaseId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCandidateReleaseId

`func (o *RunExperimentRequest) SetCandidateReleaseId(v string)`

SetCandidateReleaseId sets CandidateReleaseId field to given value.


### GetEvaluatorId

`func (o *RunExperimentRequest) GetEvaluatorId() string`

GetEvaluatorId returns the EvaluatorId field if non-nil, zero value otherwise.

### GetEvaluatorIdOk

`func (o *RunExperimentRequest) GetEvaluatorIdOk() (*string, bool)`

GetEvaluatorIdOk returns a tuple with the EvaluatorId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEvaluatorId

`func (o *RunExperimentRequest) SetEvaluatorId(v string)`

SetEvaluatorId sets EvaluatorId field to given value.


### GetEvaluatorVersionId

`func (o *RunExperimentRequest) GetEvaluatorVersionId() string`

GetEvaluatorVersionId returns the EvaluatorVersionId field if non-nil, zero value otherwise.

### GetEvaluatorVersionIdOk

`func (o *RunExperimentRequest) GetEvaluatorVersionIdOk() (*string, bool)`

GetEvaluatorVersionIdOk returns a tuple with the EvaluatorVersionId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEvaluatorVersionId

`func (o *RunExperimentRequest) SetEvaluatorVersionId(v string)`

SetEvaluatorVersionId sets EvaluatorVersionId field to given value.


### GetGatePolicy

`func (o *RunExperimentRequest) GetGatePolicy() GatePolicy`

GetGatePolicy returns the GatePolicy field if non-nil, zero value otherwise.

### GetGatePolicyOk

`func (o *RunExperimentRequest) GetGatePolicyOk() (*GatePolicy, bool)`

GetGatePolicyOk returns a tuple with the GatePolicy field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetGatePolicy

`func (o *RunExperimentRequest) SetGatePolicy(v GatePolicy)`

SetGatePolicy sets GatePolicy field to given value.

### HasGatePolicy

`func (o *RunExperimentRequest) HasGatePolicy() bool`

HasGatePolicy returns a boolean if a field has been set.

### SetGatePolicyNil

`func (o *RunExperimentRequest) SetGatePolicyNil(b bool)`

 SetGatePolicyNil sets the value for GatePolicy to be an explicit nil

### UnsetGatePolicy
`func (o *RunExperimentRequest) UnsetGatePolicy()`

UnsetGatePolicy ensures that no value is present for GatePolicy, not even an explicit nil
### GetKind

`func (o *RunExperimentRequest) GetKind() EvaluatorKind`

GetKind returns the Kind field if non-nil, zero value otherwise.

### GetKindOk

`func (o *RunExperimentRequest) GetKindOk() (*EvaluatorKind, bool)`

GetKindOk returns a tuple with the Kind field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetKind

`func (o *RunExperimentRequest) SetKind(v EvaluatorKind)`

SetKind sets Kind field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
