# RunJudgeExperimentRequest

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
**ProviderSecretId** | **string** |  |

## Methods

### NewRunJudgeExperimentRequest

`func NewRunJudgeExperimentRequest(baselineOutputs []CaseOutputOverrideRequest, baselineReleaseId string, candidateOutputs []CaseOutputOverrideRequest, candidateReleaseId string, evaluatorId string, evaluatorVersionId string, kind EvaluatorKind, providerSecretId string, ) *RunJudgeExperimentRequest`

NewRunJudgeExperimentRequest instantiates a new RunJudgeExperimentRequest object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewRunJudgeExperimentRequestWithDefaults

`func NewRunJudgeExperimentRequestWithDefaults() *RunJudgeExperimentRequest`

NewRunJudgeExperimentRequestWithDefaults instantiates a new RunJudgeExperimentRequest object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetBaselineOutputs

`func (o *RunJudgeExperimentRequest) GetBaselineOutputs() []CaseOutputOverrideRequest`

GetBaselineOutputs returns the BaselineOutputs field if non-nil, zero value otherwise.

### GetBaselineOutputsOk

`func (o *RunJudgeExperimentRequest) GetBaselineOutputsOk() (*[]CaseOutputOverrideRequest, bool)`

GetBaselineOutputsOk returns a tuple with the BaselineOutputs field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBaselineOutputs

`func (o *RunJudgeExperimentRequest) SetBaselineOutputs(v []CaseOutputOverrideRequest)`

SetBaselineOutputs sets BaselineOutputs field to given value.


### GetBaselineReleaseId

`func (o *RunJudgeExperimentRequest) GetBaselineReleaseId() string`

GetBaselineReleaseId returns the BaselineReleaseId field if non-nil, zero value otherwise.

### GetBaselineReleaseIdOk

`func (o *RunJudgeExperimentRequest) GetBaselineReleaseIdOk() (*string, bool)`

GetBaselineReleaseIdOk returns a tuple with the BaselineReleaseId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBaselineReleaseId

`func (o *RunJudgeExperimentRequest) SetBaselineReleaseId(v string)`

SetBaselineReleaseId sets BaselineReleaseId field to given value.


### GetCandidateOutputs

`func (o *RunJudgeExperimentRequest) GetCandidateOutputs() []CaseOutputOverrideRequest`

GetCandidateOutputs returns the CandidateOutputs field if non-nil, zero value otherwise.

### GetCandidateOutputsOk

`func (o *RunJudgeExperimentRequest) GetCandidateOutputsOk() (*[]CaseOutputOverrideRequest, bool)`

GetCandidateOutputsOk returns a tuple with the CandidateOutputs field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCandidateOutputs

`func (o *RunJudgeExperimentRequest) SetCandidateOutputs(v []CaseOutputOverrideRequest)`

SetCandidateOutputs sets CandidateOutputs field to given value.


### GetCandidateReleaseId

`func (o *RunJudgeExperimentRequest) GetCandidateReleaseId() string`

GetCandidateReleaseId returns the CandidateReleaseId field if non-nil, zero value otherwise.

### GetCandidateReleaseIdOk

`func (o *RunJudgeExperimentRequest) GetCandidateReleaseIdOk() (*string, bool)`

GetCandidateReleaseIdOk returns a tuple with the CandidateReleaseId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCandidateReleaseId

`func (o *RunJudgeExperimentRequest) SetCandidateReleaseId(v string)`

SetCandidateReleaseId sets CandidateReleaseId field to given value.


### GetEvaluatorId

`func (o *RunJudgeExperimentRequest) GetEvaluatorId() string`

GetEvaluatorId returns the EvaluatorId field if non-nil, zero value otherwise.

### GetEvaluatorIdOk

`func (o *RunJudgeExperimentRequest) GetEvaluatorIdOk() (*string, bool)`

GetEvaluatorIdOk returns a tuple with the EvaluatorId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEvaluatorId

`func (o *RunJudgeExperimentRequest) SetEvaluatorId(v string)`

SetEvaluatorId sets EvaluatorId field to given value.


### GetEvaluatorVersionId

`func (o *RunJudgeExperimentRequest) GetEvaluatorVersionId() string`

GetEvaluatorVersionId returns the EvaluatorVersionId field if non-nil, zero value otherwise.

### GetEvaluatorVersionIdOk

`func (o *RunJudgeExperimentRequest) GetEvaluatorVersionIdOk() (*string, bool)`

GetEvaluatorVersionIdOk returns a tuple with the EvaluatorVersionId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEvaluatorVersionId

`func (o *RunJudgeExperimentRequest) SetEvaluatorVersionId(v string)`

SetEvaluatorVersionId sets EvaluatorVersionId field to given value.


### GetGatePolicy

`func (o *RunJudgeExperimentRequest) GetGatePolicy() GatePolicy`

GetGatePolicy returns the GatePolicy field if non-nil, zero value otherwise.

### GetGatePolicyOk

`func (o *RunJudgeExperimentRequest) GetGatePolicyOk() (*GatePolicy, bool)`

GetGatePolicyOk returns a tuple with the GatePolicy field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetGatePolicy

`func (o *RunJudgeExperimentRequest) SetGatePolicy(v GatePolicy)`

SetGatePolicy sets GatePolicy field to given value.

### HasGatePolicy

`func (o *RunJudgeExperimentRequest) HasGatePolicy() bool`

HasGatePolicy returns a boolean if a field has been set.

### SetGatePolicyNil

`func (o *RunJudgeExperimentRequest) SetGatePolicyNil(b bool)`

 SetGatePolicyNil sets the value for GatePolicy to be an explicit nil

### UnsetGatePolicy
`func (o *RunJudgeExperimentRequest) UnsetGatePolicy()`

UnsetGatePolicy ensures that no value is present for GatePolicy, not even an explicit nil
### GetKind

`func (o *RunJudgeExperimentRequest) GetKind() EvaluatorKind`

GetKind returns the Kind field if non-nil, zero value otherwise.

### GetKindOk

`func (o *RunJudgeExperimentRequest) GetKindOk() (*EvaluatorKind, bool)`

GetKindOk returns a tuple with the Kind field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetKind

`func (o *RunJudgeExperimentRequest) SetKind(v EvaluatorKind)`

SetKind sets Kind field to given value.


### GetProviderSecretId

`func (o *RunJudgeExperimentRequest) GetProviderSecretId() string`

GetProviderSecretId returns the ProviderSecretId field if non-nil, zero value otherwise.

### GetProviderSecretIdOk

`func (o *RunJudgeExperimentRequest) GetProviderSecretIdOk() (*string, bool)`

GetProviderSecretIdOk returns a tuple with the ProviderSecretId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProviderSecretId

`func (o *RunJudgeExperimentRequest) SetProviderSecretId(v string)`

SetProviderSecretId sets ProviderSecretId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
