# RunDeterministicEvalRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AgentReleaseId** | **string** |  |
**CodeHash** | Pointer to **NullableString** |  | [optional]
**EvaluatorId** | **string** |  |
**EvaluatorVersionId** | **string** |  |
**Kind** | [**EvaluatorKind**](EvaluatorKind.md) |  |
**PromptVersionId** | Pointer to **NullableString** |  | [optional]
**WasmHash** | Pointer to **NullableString** |  | [optional]

## Methods

### NewRunDeterministicEvalRequest

`func NewRunDeterministicEvalRequest(agentReleaseId string, evaluatorId string, evaluatorVersionId string, kind EvaluatorKind, ) *RunDeterministicEvalRequest`

NewRunDeterministicEvalRequest instantiates a new RunDeterministicEvalRequest object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewRunDeterministicEvalRequestWithDefaults

`func NewRunDeterministicEvalRequestWithDefaults() *RunDeterministicEvalRequest`

NewRunDeterministicEvalRequestWithDefaults instantiates a new RunDeterministicEvalRequest object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAgentReleaseId

`func (o *RunDeterministicEvalRequest) GetAgentReleaseId() string`

GetAgentReleaseId returns the AgentReleaseId field if non-nil, zero value otherwise.

### GetAgentReleaseIdOk

`func (o *RunDeterministicEvalRequest) GetAgentReleaseIdOk() (*string, bool)`

GetAgentReleaseIdOk returns a tuple with the AgentReleaseId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAgentReleaseId

`func (o *RunDeterministicEvalRequest) SetAgentReleaseId(v string)`

SetAgentReleaseId sets AgentReleaseId field to given value.


### GetCodeHash

`func (o *RunDeterministicEvalRequest) GetCodeHash() string`

GetCodeHash returns the CodeHash field if non-nil, zero value otherwise.

### GetCodeHashOk

`func (o *RunDeterministicEvalRequest) GetCodeHashOk() (*string, bool)`

GetCodeHashOk returns a tuple with the CodeHash field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCodeHash

`func (o *RunDeterministicEvalRequest) SetCodeHash(v string)`

SetCodeHash sets CodeHash field to given value.

### HasCodeHash

`func (o *RunDeterministicEvalRequest) HasCodeHash() bool`

HasCodeHash returns a boolean if a field has been set.

### SetCodeHashNil

`func (o *RunDeterministicEvalRequest) SetCodeHashNil(b bool)`

 SetCodeHashNil sets the value for CodeHash to be an explicit nil

### UnsetCodeHash
`func (o *RunDeterministicEvalRequest) UnsetCodeHash()`

UnsetCodeHash ensures that no value is present for CodeHash, not even an explicit nil
### GetEvaluatorId

`func (o *RunDeterministicEvalRequest) GetEvaluatorId() string`

GetEvaluatorId returns the EvaluatorId field if non-nil, zero value otherwise.

### GetEvaluatorIdOk

`func (o *RunDeterministicEvalRequest) GetEvaluatorIdOk() (*string, bool)`

GetEvaluatorIdOk returns a tuple with the EvaluatorId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEvaluatorId

`func (o *RunDeterministicEvalRequest) SetEvaluatorId(v string)`

SetEvaluatorId sets EvaluatorId field to given value.


### GetEvaluatorVersionId

`func (o *RunDeterministicEvalRequest) GetEvaluatorVersionId() string`

GetEvaluatorVersionId returns the EvaluatorVersionId field if non-nil, zero value otherwise.

### GetEvaluatorVersionIdOk

`func (o *RunDeterministicEvalRequest) GetEvaluatorVersionIdOk() (*string, bool)`

GetEvaluatorVersionIdOk returns a tuple with the EvaluatorVersionId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEvaluatorVersionId

`func (o *RunDeterministicEvalRequest) SetEvaluatorVersionId(v string)`

SetEvaluatorVersionId sets EvaluatorVersionId field to given value.


### GetKind

`func (o *RunDeterministicEvalRequest) GetKind() EvaluatorKind`

GetKind returns the Kind field if non-nil, zero value otherwise.

### GetKindOk

`func (o *RunDeterministicEvalRequest) GetKindOk() (*EvaluatorKind, bool)`

GetKindOk returns a tuple with the Kind field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetKind

`func (o *RunDeterministicEvalRequest) SetKind(v EvaluatorKind)`

SetKind sets Kind field to given value.


### GetPromptVersionId

`func (o *RunDeterministicEvalRequest) GetPromptVersionId() string`

GetPromptVersionId returns the PromptVersionId field if non-nil, zero value otherwise.

### GetPromptVersionIdOk

`func (o *RunDeterministicEvalRequest) GetPromptVersionIdOk() (*string, bool)`

GetPromptVersionIdOk returns a tuple with the PromptVersionId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPromptVersionId

`func (o *RunDeterministicEvalRequest) SetPromptVersionId(v string)`

SetPromptVersionId sets PromptVersionId field to given value.

### HasPromptVersionId

`func (o *RunDeterministicEvalRequest) HasPromptVersionId() bool`

HasPromptVersionId returns a boolean if a field has been set.

### SetPromptVersionIdNil

`func (o *RunDeterministicEvalRequest) SetPromptVersionIdNil(b bool)`

 SetPromptVersionIdNil sets the value for PromptVersionId to be an explicit nil

### UnsetPromptVersionId
`func (o *RunDeterministicEvalRequest) UnsetPromptVersionId()`

UnsetPromptVersionId ensures that no value is present for PromptVersionId, not even an explicit nil
### GetWasmHash

`func (o *RunDeterministicEvalRequest) GetWasmHash() string`

GetWasmHash returns the WasmHash field if non-nil, zero value otherwise.

### GetWasmHashOk

`func (o *RunDeterministicEvalRequest) GetWasmHashOk() (*string, bool)`

GetWasmHashOk returns a tuple with the WasmHash field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetWasmHash

`func (o *RunDeterministicEvalRequest) SetWasmHash(v string)`

SetWasmHash sets WasmHash field to given value.

### HasWasmHash

`func (o *RunDeterministicEvalRequest) HasWasmHash() bool`

HasWasmHash returns a boolean if a field has been set.

### SetWasmHashNil

`func (o *RunDeterministicEvalRequest) SetWasmHashNil(b bool)`

 SetWasmHashNil sets the value for WasmHash to be an explicit nil

### UnsetWasmHash
`func (o *RunDeterministicEvalRequest) UnsetWasmHash()`

UnsetWasmHash ensures that no value is present for WasmHash, not even an explicit nil

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
