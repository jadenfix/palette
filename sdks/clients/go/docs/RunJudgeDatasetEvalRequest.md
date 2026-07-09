# RunJudgeDatasetEvalRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AgentReleaseId** | **string** |  |
**CodeHash** | Pointer to **NullableString** |  | [optional]
**EvaluatorId** | **string** |  |
**EvaluatorVersionId** | **string** |  |
**Kind** | [**EvaluatorKind**](EvaluatorKind.md) |  |
**PromptVersionId** | Pointer to **NullableString** |  | [optional]
**ProviderSecretId** | **string** |  |

## Methods

### NewRunJudgeDatasetEvalRequest

`func NewRunJudgeDatasetEvalRequest(agentReleaseId string, evaluatorId string, evaluatorVersionId string, kind EvaluatorKind, providerSecretId string, ) *RunJudgeDatasetEvalRequest`

NewRunJudgeDatasetEvalRequest instantiates a new RunJudgeDatasetEvalRequest object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewRunJudgeDatasetEvalRequestWithDefaults

`func NewRunJudgeDatasetEvalRequestWithDefaults() *RunJudgeDatasetEvalRequest`

NewRunJudgeDatasetEvalRequestWithDefaults instantiates a new RunJudgeDatasetEvalRequest object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAgentReleaseId

`func (o *RunJudgeDatasetEvalRequest) GetAgentReleaseId() string`

GetAgentReleaseId returns the AgentReleaseId field if non-nil, zero value otherwise.

### GetAgentReleaseIdOk

`func (o *RunJudgeDatasetEvalRequest) GetAgentReleaseIdOk() (*string, bool)`

GetAgentReleaseIdOk returns a tuple with the AgentReleaseId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAgentReleaseId

`func (o *RunJudgeDatasetEvalRequest) SetAgentReleaseId(v string)`

SetAgentReleaseId sets AgentReleaseId field to given value.


### GetCodeHash

`func (o *RunJudgeDatasetEvalRequest) GetCodeHash() string`

GetCodeHash returns the CodeHash field if non-nil, zero value otherwise.

### GetCodeHashOk

`func (o *RunJudgeDatasetEvalRequest) GetCodeHashOk() (*string, bool)`

GetCodeHashOk returns a tuple with the CodeHash field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCodeHash

`func (o *RunJudgeDatasetEvalRequest) SetCodeHash(v string)`

SetCodeHash sets CodeHash field to given value.

### HasCodeHash

`func (o *RunJudgeDatasetEvalRequest) HasCodeHash() bool`

HasCodeHash returns a boolean if a field has been set.

### SetCodeHashNil

`func (o *RunJudgeDatasetEvalRequest) SetCodeHashNil(b bool)`

 SetCodeHashNil sets the value for CodeHash to be an explicit nil

### UnsetCodeHash
`func (o *RunJudgeDatasetEvalRequest) UnsetCodeHash()`

UnsetCodeHash ensures that no value is present for CodeHash, not even an explicit nil
### GetEvaluatorId

`func (o *RunJudgeDatasetEvalRequest) GetEvaluatorId() string`

GetEvaluatorId returns the EvaluatorId field if non-nil, zero value otherwise.

### GetEvaluatorIdOk

`func (o *RunJudgeDatasetEvalRequest) GetEvaluatorIdOk() (*string, bool)`

GetEvaluatorIdOk returns a tuple with the EvaluatorId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEvaluatorId

`func (o *RunJudgeDatasetEvalRequest) SetEvaluatorId(v string)`

SetEvaluatorId sets EvaluatorId field to given value.


### GetEvaluatorVersionId

`func (o *RunJudgeDatasetEvalRequest) GetEvaluatorVersionId() string`

GetEvaluatorVersionId returns the EvaluatorVersionId field if non-nil, zero value otherwise.

### GetEvaluatorVersionIdOk

`func (o *RunJudgeDatasetEvalRequest) GetEvaluatorVersionIdOk() (*string, bool)`

GetEvaluatorVersionIdOk returns a tuple with the EvaluatorVersionId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEvaluatorVersionId

`func (o *RunJudgeDatasetEvalRequest) SetEvaluatorVersionId(v string)`

SetEvaluatorVersionId sets EvaluatorVersionId field to given value.


### GetKind

`func (o *RunJudgeDatasetEvalRequest) GetKind() EvaluatorKind`

GetKind returns the Kind field if non-nil, zero value otherwise.

### GetKindOk

`func (o *RunJudgeDatasetEvalRequest) GetKindOk() (*EvaluatorKind, bool)`

GetKindOk returns a tuple with the Kind field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetKind

`func (o *RunJudgeDatasetEvalRequest) SetKind(v EvaluatorKind)`

SetKind sets Kind field to given value.


### GetPromptVersionId

`func (o *RunJudgeDatasetEvalRequest) GetPromptVersionId() string`

GetPromptVersionId returns the PromptVersionId field if non-nil, zero value otherwise.

### GetPromptVersionIdOk

`func (o *RunJudgeDatasetEvalRequest) GetPromptVersionIdOk() (*string, bool)`

GetPromptVersionIdOk returns a tuple with the PromptVersionId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPromptVersionId

`func (o *RunJudgeDatasetEvalRequest) SetPromptVersionId(v string)`

SetPromptVersionId sets PromptVersionId field to given value.

### HasPromptVersionId

`func (o *RunJudgeDatasetEvalRequest) HasPromptVersionId() bool`

HasPromptVersionId returns a boolean if a field has been set.

### SetPromptVersionIdNil

`func (o *RunJudgeDatasetEvalRequest) SetPromptVersionIdNil(b bool)`

 SetPromptVersionIdNil sets the value for PromptVersionId to be an explicit nil

### UnsetPromptVersionId
`func (o *RunJudgeDatasetEvalRequest) UnsetPromptVersionId()`

UnsetPromptVersionId ensures that no value is present for PromptVersionId, not even an explicit nil
### GetProviderSecretId

`func (o *RunJudgeDatasetEvalRequest) GetProviderSecretId() string`

GetProviderSecretId returns the ProviderSecretId field if non-nil, zero value otherwise.

### GetProviderSecretIdOk

`func (o *RunJudgeDatasetEvalRequest) GetProviderSecretIdOk() (*string, bool)`

GetProviderSecretIdOk returns a tuple with the ProviderSecretId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProviderSecretId

`func (o *RunJudgeDatasetEvalRequest) SetProviderSecretId(v string)`

SetProviderSecretId sets ProviderSecretId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
