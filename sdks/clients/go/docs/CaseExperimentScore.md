# CaseExperimentScore

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**BaselineCached** | Pointer to **NullableBool** |  | [optional]
**BaselineCost** | Pointer to [**NullableMoney**](Money.md) |  | [optional]
**BaselineEvidence** | **interface{}** |  |
**BaselineJudgeCallId** | Pointer to **string** |  | [optional]
**BaselineOutput** | **interface{}** |  |
**BaselineScore** | **float64** |  |
**BaselineTrace** | Pointer to **interface{}** |  | [optional]
**CandidateCached** | Pointer to **NullableBool** |  | [optional]
**CandidateCost** | Pointer to [**NullableMoney**](Money.md) |  | [optional]
**CandidateEvidence** | **interface{}** |  |
**CandidateJudgeCallId** | Pointer to **string** |  | [optional]
**CandidateOutput** | **interface{}** |  |
**CandidateScore** | **float64** |  |
**CandidateTrace** | Pointer to **interface{}** |  | [optional]
**CaseId** | **string** |  |
**Delta** | **float64** |  |
**Reference** | Pointer to **interface{}** |  | [optional]

## Methods

### NewCaseExperimentScore

`func NewCaseExperimentScore(baselineEvidence interface{}, baselineOutput interface{}, baselineScore float64, candidateEvidence interface{}, candidateOutput interface{}, candidateScore float64, caseId string, delta float64, ) *CaseExperimentScore`

NewCaseExperimentScore instantiates a new CaseExperimentScore object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewCaseExperimentScoreWithDefaults

`func NewCaseExperimentScoreWithDefaults() *CaseExperimentScore`

NewCaseExperimentScoreWithDefaults instantiates a new CaseExperimentScore object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetBaselineCached

`func (o *CaseExperimentScore) GetBaselineCached() bool`

GetBaselineCached returns the BaselineCached field if non-nil, zero value otherwise.

### GetBaselineCachedOk

`func (o *CaseExperimentScore) GetBaselineCachedOk() (*bool, bool)`

GetBaselineCachedOk returns a tuple with the BaselineCached field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBaselineCached

`func (o *CaseExperimentScore) SetBaselineCached(v bool)`

SetBaselineCached sets BaselineCached field to given value.

### HasBaselineCached

`func (o *CaseExperimentScore) HasBaselineCached() bool`

HasBaselineCached returns a boolean if a field has been set.

### SetBaselineCachedNil

`func (o *CaseExperimentScore) SetBaselineCachedNil(b bool)`

 SetBaselineCachedNil sets the value for BaselineCached to be an explicit nil

### UnsetBaselineCached
`func (o *CaseExperimentScore) UnsetBaselineCached()`

UnsetBaselineCached ensures that no value is present for BaselineCached, not even an explicit nil
### GetBaselineCost

`func (o *CaseExperimentScore) GetBaselineCost() Money`

GetBaselineCost returns the BaselineCost field if non-nil, zero value otherwise.

### GetBaselineCostOk

`func (o *CaseExperimentScore) GetBaselineCostOk() (*Money, bool)`

GetBaselineCostOk returns a tuple with the BaselineCost field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBaselineCost

`func (o *CaseExperimentScore) SetBaselineCost(v Money)`

SetBaselineCost sets BaselineCost field to given value.

### HasBaselineCost

`func (o *CaseExperimentScore) HasBaselineCost() bool`

HasBaselineCost returns a boolean if a field has been set.

### SetBaselineCostNil

`func (o *CaseExperimentScore) SetBaselineCostNil(b bool)`

 SetBaselineCostNil sets the value for BaselineCost to be an explicit nil

### UnsetBaselineCost
`func (o *CaseExperimentScore) UnsetBaselineCost()`

UnsetBaselineCost ensures that no value is present for BaselineCost, not even an explicit nil
### GetBaselineEvidence

`func (o *CaseExperimentScore) GetBaselineEvidence() interface{}`

GetBaselineEvidence returns the BaselineEvidence field if non-nil, zero value otherwise.

### GetBaselineEvidenceOk

`func (o *CaseExperimentScore) GetBaselineEvidenceOk() (*interface{}, bool)`

GetBaselineEvidenceOk returns a tuple with the BaselineEvidence field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBaselineEvidence

`func (o *CaseExperimentScore) SetBaselineEvidence(v interface{})`

SetBaselineEvidence sets BaselineEvidence field to given value.


### SetBaselineEvidenceNil

`func (o *CaseExperimentScore) SetBaselineEvidenceNil(b bool)`

 SetBaselineEvidenceNil sets the value for BaselineEvidence to be an explicit nil

### UnsetBaselineEvidence
`func (o *CaseExperimentScore) UnsetBaselineEvidence()`

UnsetBaselineEvidence ensures that no value is present for BaselineEvidence, not even an explicit nil
### GetBaselineJudgeCallId

`func (o *CaseExperimentScore) GetBaselineJudgeCallId() string`

GetBaselineJudgeCallId returns the BaselineJudgeCallId field if non-nil, zero value otherwise.

### GetBaselineJudgeCallIdOk

`func (o *CaseExperimentScore) GetBaselineJudgeCallIdOk() (*string, bool)`

GetBaselineJudgeCallIdOk returns a tuple with the BaselineJudgeCallId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBaselineJudgeCallId

`func (o *CaseExperimentScore) SetBaselineJudgeCallId(v string)`

SetBaselineJudgeCallId sets BaselineJudgeCallId field to given value.

### HasBaselineJudgeCallId

`func (o *CaseExperimentScore) HasBaselineJudgeCallId() bool`

HasBaselineJudgeCallId returns a boolean if a field has been set.

### GetBaselineOutput

`func (o *CaseExperimentScore) GetBaselineOutput() interface{}`

GetBaselineOutput returns the BaselineOutput field if non-nil, zero value otherwise.

### GetBaselineOutputOk

`func (o *CaseExperimentScore) GetBaselineOutputOk() (*interface{}, bool)`

GetBaselineOutputOk returns a tuple with the BaselineOutput field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBaselineOutput

`func (o *CaseExperimentScore) SetBaselineOutput(v interface{})`

SetBaselineOutput sets BaselineOutput field to given value.


### SetBaselineOutputNil

`func (o *CaseExperimentScore) SetBaselineOutputNil(b bool)`

 SetBaselineOutputNil sets the value for BaselineOutput to be an explicit nil

### UnsetBaselineOutput
`func (o *CaseExperimentScore) UnsetBaselineOutput()`

UnsetBaselineOutput ensures that no value is present for BaselineOutput, not even an explicit nil
### GetBaselineScore

`func (o *CaseExperimentScore) GetBaselineScore() float64`

GetBaselineScore returns the BaselineScore field if non-nil, zero value otherwise.

### GetBaselineScoreOk

`func (o *CaseExperimentScore) GetBaselineScoreOk() (*float64, bool)`

GetBaselineScoreOk returns a tuple with the BaselineScore field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBaselineScore

`func (o *CaseExperimentScore) SetBaselineScore(v float64)`

SetBaselineScore sets BaselineScore field to given value.


### GetBaselineTrace

`func (o *CaseExperimentScore) GetBaselineTrace() interface{}`

GetBaselineTrace returns the BaselineTrace field if non-nil, zero value otherwise.

### GetBaselineTraceOk

`func (o *CaseExperimentScore) GetBaselineTraceOk() (*interface{}, bool)`

GetBaselineTraceOk returns a tuple with the BaselineTrace field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBaselineTrace

`func (o *CaseExperimentScore) SetBaselineTrace(v interface{})`

SetBaselineTrace sets BaselineTrace field to given value.

### HasBaselineTrace

`func (o *CaseExperimentScore) HasBaselineTrace() bool`

HasBaselineTrace returns a boolean if a field has been set.

### SetBaselineTraceNil

`func (o *CaseExperimentScore) SetBaselineTraceNil(b bool)`

 SetBaselineTraceNil sets the value for BaselineTrace to be an explicit nil

### UnsetBaselineTrace
`func (o *CaseExperimentScore) UnsetBaselineTrace()`

UnsetBaselineTrace ensures that no value is present for BaselineTrace, not even an explicit nil
### GetCandidateCached

`func (o *CaseExperimentScore) GetCandidateCached() bool`

GetCandidateCached returns the CandidateCached field if non-nil, zero value otherwise.

### GetCandidateCachedOk

`func (o *CaseExperimentScore) GetCandidateCachedOk() (*bool, bool)`

GetCandidateCachedOk returns a tuple with the CandidateCached field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCandidateCached

`func (o *CaseExperimentScore) SetCandidateCached(v bool)`

SetCandidateCached sets CandidateCached field to given value.

### HasCandidateCached

`func (o *CaseExperimentScore) HasCandidateCached() bool`

HasCandidateCached returns a boolean if a field has been set.

### SetCandidateCachedNil

`func (o *CaseExperimentScore) SetCandidateCachedNil(b bool)`

 SetCandidateCachedNil sets the value for CandidateCached to be an explicit nil

### UnsetCandidateCached
`func (o *CaseExperimentScore) UnsetCandidateCached()`

UnsetCandidateCached ensures that no value is present for CandidateCached, not even an explicit nil
### GetCandidateCost

`func (o *CaseExperimentScore) GetCandidateCost() Money`

GetCandidateCost returns the CandidateCost field if non-nil, zero value otherwise.

### GetCandidateCostOk

`func (o *CaseExperimentScore) GetCandidateCostOk() (*Money, bool)`

GetCandidateCostOk returns a tuple with the CandidateCost field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCandidateCost

`func (o *CaseExperimentScore) SetCandidateCost(v Money)`

SetCandidateCost sets CandidateCost field to given value.

### HasCandidateCost

`func (o *CaseExperimentScore) HasCandidateCost() bool`

HasCandidateCost returns a boolean if a field has been set.

### SetCandidateCostNil

`func (o *CaseExperimentScore) SetCandidateCostNil(b bool)`

 SetCandidateCostNil sets the value for CandidateCost to be an explicit nil

### UnsetCandidateCost
`func (o *CaseExperimentScore) UnsetCandidateCost()`

UnsetCandidateCost ensures that no value is present for CandidateCost, not even an explicit nil
### GetCandidateEvidence

`func (o *CaseExperimentScore) GetCandidateEvidence() interface{}`

GetCandidateEvidence returns the CandidateEvidence field if non-nil, zero value otherwise.

### GetCandidateEvidenceOk

`func (o *CaseExperimentScore) GetCandidateEvidenceOk() (*interface{}, bool)`

GetCandidateEvidenceOk returns a tuple with the CandidateEvidence field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCandidateEvidence

`func (o *CaseExperimentScore) SetCandidateEvidence(v interface{})`

SetCandidateEvidence sets CandidateEvidence field to given value.


### SetCandidateEvidenceNil

`func (o *CaseExperimentScore) SetCandidateEvidenceNil(b bool)`

 SetCandidateEvidenceNil sets the value for CandidateEvidence to be an explicit nil

### UnsetCandidateEvidence
`func (o *CaseExperimentScore) UnsetCandidateEvidence()`

UnsetCandidateEvidence ensures that no value is present for CandidateEvidence, not even an explicit nil
### GetCandidateJudgeCallId

`func (o *CaseExperimentScore) GetCandidateJudgeCallId() string`

GetCandidateJudgeCallId returns the CandidateJudgeCallId field if non-nil, zero value otherwise.

### GetCandidateJudgeCallIdOk

`func (o *CaseExperimentScore) GetCandidateJudgeCallIdOk() (*string, bool)`

GetCandidateJudgeCallIdOk returns a tuple with the CandidateJudgeCallId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCandidateJudgeCallId

`func (o *CaseExperimentScore) SetCandidateJudgeCallId(v string)`

SetCandidateJudgeCallId sets CandidateJudgeCallId field to given value.

### HasCandidateJudgeCallId

`func (o *CaseExperimentScore) HasCandidateJudgeCallId() bool`

HasCandidateJudgeCallId returns a boolean if a field has been set.

### GetCandidateOutput

`func (o *CaseExperimentScore) GetCandidateOutput() interface{}`

GetCandidateOutput returns the CandidateOutput field if non-nil, zero value otherwise.

### GetCandidateOutputOk

`func (o *CaseExperimentScore) GetCandidateOutputOk() (*interface{}, bool)`

GetCandidateOutputOk returns a tuple with the CandidateOutput field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCandidateOutput

`func (o *CaseExperimentScore) SetCandidateOutput(v interface{})`

SetCandidateOutput sets CandidateOutput field to given value.


### SetCandidateOutputNil

`func (o *CaseExperimentScore) SetCandidateOutputNil(b bool)`

 SetCandidateOutputNil sets the value for CandidateOutput to be an explicit nil

### UnsetCandidateOutput
`func (o *CaseExperimentScore) UnsetCandidateOutput()`

UnsetCandidateOutput ensures that no value is present for CandidateOutput, not even an explicit nil
### GetCandidateScore

`func (o *CaseExperimentScore) GetCandidateScore() float64`

GetCandidateScore returns the CandidateScore field if non-nil, zero value otherwise.

### GetCandidateScoreOk

`func (o *CaseExperimentScore) GetCandidateScoreOk() (*float64, bool)`

GetCandidateScoreOk returns a tuple with the CandidateScore field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCandidateScore

`func (o *CaseExperimentScore) SetCandidateScore(v float64)`

SetCandidateScore sets CandidateScore field to given value.


### GetCandidateTrace

`func (o *CaseExperimentScore) GetCandidateTrace() interface{}`

GetCandidateTrace returns the CandidateTrace field if non-nil, zero value otherwise.

### GetCandidateTraceOk

`func (o *CaseExperimentScore) GetCandidateTraceOk() (*interface{}, bool)`

GetCandidateTraceOk returns a tuple with the CandidateTrace field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCandidateTrace

`func (o *CaseExperimentScore) SetCandidateTrace(v interface{})`

SetCandidateTrace sets CandidateTrace field to given value.

### HasCandidateTrace

`func (o *CaseExperimentScore) HasCandidateTrace() bool`

HasCandidateTrace returns a boolean if a field has been set.

### SetCandidateTraceNil

`func (o *CaseExperimentScore) SetCandidateTraceNil(b bool)`

 SetCandidateTraceNil sets the value for CandidateTrace to be an explicit nil

### UnsetCandidateTrace
`func (o *CaseExperimentScore) UnsetCandidateTrace()`

UnsetCandidateTrace ensures that no value is present for CandidateTrace, not even an explicit nil
### GetCaseId

`func (o *CaseExperimentScore) GetCaseId() string`

GetCaseId returns the CaseId field if non-nil, zero value otherwise.

### GetCaseIdOk

`func (o *CaseExperimentScore) GetCaseIdOk() (*string, bool)`

GetCaseIdOk returns a tuple with the CaseId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCaseId

`func (o *CaseExperimentScore) SetCaseId(v string)`

SetCaseId sets CaseId field to given value.


### GetDelta

`func (o *CaseExperimentScore) GetDelta() float64`

GetDelta returns the Delta field if non-nil, zero value otherwise.

### GetDeltaOk

`func (o *CaseExperimentScore) GetDeltaOk() (*float64, bool)`

GetDeltaOk returns a tuple with the Delta field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDelta

`func (o *CaseExperimentScore) SetDelta(v float64)`

SetDelta sets Delta field to given value.


### GetReference

`func (o *CaseExperimentScore) GetReference() interface{}`

GetReference returns the Reference field if non-nil, zero value otherwise.

### GetReferenceOk

`func (o *CaseExperimentScore) GetReferenceOk() (*interface{}, bool)`

GetReferenceOk returns a tuple with the Reference field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetReference

`func (o *CaseExperimentScore) SetReference(v interface{})`

SetReference sets Reference field to given value.

### HasReference

`func (o *CaseExperimentScore) HasReference() bool`

HasReference returns a boolean if a field has been set.

### SetReferenceNil

`func (o *CaseExperimentScore) SetReferenceNil(b bool)`

 SetReferenceNil sets the value for Reference to be an explicit nil

### UnsetReference
`func (o *CaseExperimentScore) UnsetReference()`

UnsetReference ensures that no value is present for Reference, not even an explicit nil

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
