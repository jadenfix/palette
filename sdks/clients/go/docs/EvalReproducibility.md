# EvalReproducibility

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AgentReleaseId** | **string** |  |
**CodeHash** | Pointer to **string** |  | [optional]
**DatasetCaseId** | **string** |  |
**DatasetVersionId** | **string** |  |
**EvaluatorVersionId** | **string** |  |
**InputArtifactHashes** | **[]string** |  |
**JudgeModelId** | Pointer to **NullableString** |  | [optional]
**JudgeParameters** | **interface{}** |  |
**JudgeProvider** | Pointer to **NullableString** |  | [optional]
**JudgeRubricVersion** | Pointer to **NullableString** |  | [optional]
**JudgeSeed** | Pointer to **NullableInt64** |  | [optional]
**NormalizerVersion** | **string** |  |
**PromptVersionId** | Pointer to **string** |  | [optional]
**TraceSchemaVersion** | **int32** |  |
**WasiAbiVersion** | Pointer to **NullableString** |  | [optional]
**WasmHash** | Pointer to **string** |  | [optional]

## Methods

### NewEvalReproducibility

`func NewEvalReproducibility(agentReleaseId string, datasetCaseId string, datasetVersionId string, evaluatorVersionId string, inputArtifactHashes []string, judgeParameters interface{}, normalizerVersion string, traceSchemaVersion int32, ) *EvalReproducibility`

NewEvalReproducibility instantiates a new EvalReproducibility object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewEvalReproducibilityWithDefaults

`func NewEvalReproducibilityWithDefaults() *EvalReproducibility`

NewEvalReproducibilityWithDefaults instantiates a new EvalReproducibility object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAgentReleaseId

`func (o *EvalReproducibility) GetAgentReleaseId() string`

GetAgentReleaseId returns the AgentReleaseId field if non-nil, zero value otherwise.

### GetAgentReleaseIdOk

`func (o *EvalReproducibility) GetAgentReleaseIdOk() (*string, bool)`

GetAgentReleaseIdOk returns a tuple with the AgentReleaseId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAgentReleaseId

`func (o *EvalReproducibility) SetAgentReleaseId(v string)`

SetAgentReleaseId sets AgentReleaseId field to given value.


### GetCodeHash

`func (o *EvalReproducibility) GetCodeHash() string`

GetCodeHash returns the CodeHash field if non-nil, zero value otherwise.

### GetCodeHashOk

`func (o *EvalReproducibility) GetCodeHashOk() (*string, bool)`

GetCodeHashOk returns a tuple with the CodeHash field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCodeHash

`func (o *EvalReproducibility) SetCodeHash(v string)`

SetCodeHash sets CodeHash field to given value.

### HasCodeHash

`func (o *EvalReproducibility) HasCodeHash() bool`

HasCodeHash returns a boolean if a field has been set.

### GetDatasetCaseId

`func (o *EvalReproducibility) GetDatasetCaseId() string`

GetDatasetCaseId returns the DatasetCaseId field if non-nil, zero value otherwise.

### GetDatasetCaseIdOk

`func (o *EvalReproducibility) GetDatasetCaseIdOk() (*string, bool)`

GetDatasetCaseIdOk returns a tuple with the DatasetCaseId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDatasetCaseId

`func (o *EvalReproducibility) SetDatasetCaseId(v string)`

SetDatasetCaseId sets DatasetCaseId field to given value.


### GetDatasetVersionId

`func (o *EvalReproducibility) GetDatasetVersionId() string`

GetDatasetVersionId returns the DatasetVersionId field if non-nil, zero value otherwise.

### GetDatasetVersionIdOk

`func (o *EvalReproducibility) GetDatasetVersionIdOk() (*string, bool)`

GetDatasetVersionIdOk returns a tuple with the DatasetVersionId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDatasetVersionId

`func (o *EvalReproducibility) SetDatasetVersionId(v string)`

SetDatasetVersionId sets DatasetVersionId field to given value.


### GetEvaluatorVersionId

`func (o *EvalReproducibility) GetEvaluatorVersionId() string`

GetEvaluatorVersionId returns the EvaluatorVersionId field if non-nil, zero value otherwise.

### GetEvaluatorVersionIdOk

`func (o *EvalReproducibility) GetEvaluatorVersionIdOk() (*string, bool)`

GetEvaluatorVersionIdOk returns a tuple with the EvaluatorVersionId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEvaluatorVersionId

`func (o *EvalReproducibility) SetEvaluatorVersionId(v string)`

SetEvaluatorVersionId sets EvaluatorVersionId field to given value.


### GetInputArtifactHashes

`func (o *EvalReproducibility) GetInputArtifactHashes() []string`

GetInputArtifactHashes returns the InputArtifactHashes field if non-nil, zero value otherwise.

### GetInputArtifactHashesOk

`func (o *EvalReproducibility) GetInputArtifactHashesOk() (*[]string, bool)`

GetInputArtifactHashesOk returns a tuple with the InputArtifactHashes field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetInputArtifactHashes

`func (o *EvalReproducibility) SetInputArtifactHashes(v []string)`

SetInputArtifactHashes sets InputArtifactHashes field to given value.


### GetJudgeModelId

`func (o *EvalReproducibility) GetJudgeModelId() string`

GetJudgeModelId returns the JudgeModelId field if non-nil, zero value otherwise.

### GetJudgeModelIdOk

`func (o *EvalReproducibility) GetJudgeModelIdOk() (*string, bool)`

GetJudgeModelIdOk returns a tuple with the JudgeModelId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetJudgeModelId

`func (o *EvalReproducibility) SetJudgeModelId(v string)`

SetJudgeModelId sets JudgeModelId field to given value.

### HasJudgeModelId

`func (o *EvalReproducibility) HasJudgeModelId() bool`

HasJudgeModelId returns a boolean if a field has been set.

### SetJudgeModelIdNil

`func (o *EvalReproducibility) SetJudgeModelIdNil(b bool)`

 SetJudgeModelIdNil sets the value for JudgeModelId to be an explicit nil

### UnsetJudgeModelId
`func (o *EvalReproducibility) UnsetJudgeModelId()`

UnsetJudgeModelId ensures that no value is present for JudgeModelId, not even an explicit nil
### GetJudgeParameters

`func (o *EvalReproducibility) GetJudgeParameters() interface{}`

GetJudgeParameters returns the JudgeParameters field if non-nil, zero value otherwise.

### GetJudgeParametersOk

`func (o *EvalReproducibility) GetJudgeParametersOk() (*interface{}, bool)`

GetJudgeParametersOk returns a tuple with the JudgeParameters field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetJudgeParameters

`func (o *EvalReproducibility) SetJudgeParameters(v interface{})`

SetJudgeParameters sets JudgeParameters field to given value.


### SetJudgeParametersNil

`func (o *EvalReproducibility) SetJudgeParametersNil(b bool)`

 SetJudgeParametersNil sets the value for JudgeParameters to be an explicit nil

### UnsetJudgeParameters
`func (o *EvalReproducibility) UnsetJudgeParameters()`

UnsetJudgeParameters ensures that no value is present for JudgeParameters, not even an explicit nil
### GetJudgeProvider

`func (o *EvalReproducibility) GetJudgeProvider() string`

GetJudgeProvider returns the JudgeProvider field if non-nil, zero value otherwise.

### GetJudgeProviderOk

`func (o *EvalReproducibility) GetJudgeProviderOk() (*string, bool)`

GetJudgeProviderOk returns a tuple with the JudgeProvider field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetJudgeProvider

`func (o *EvalReproducibility) SetJudgeProvider(v string)`

SetJudgeProvider sets JudgeProvider field to given value.

### HasJudgeProvider

`func (o *EvalReproducibility) HasJudgeProvider() bool`

HasJudgeProvider returns a boolean if a field has been set.

### SetJudgeProviderNil

`func (o *EvalReproducibility) SetJudgeProviderNil(b bool)`

 SetJudgeProviderNil sets the value for JudgeProvider to be an explicit nil

### UnsetJudgeProvider
`func (o *EvalReproducibility) UnsetJudgeProvider()`

UnsetJudgeProvider ensures that no value is present for JudgeProvider, not even an explicit nil
### GetJudgeRubricVersion

`func (o *EvalReproducibility) GetJudgeRubricVersion() string`

GetJudgeRubricVersion returns the JudgeRubricVersion field if non-nil, zero value otherwise.

### GetJudgeRubricVersionOk

`func (o *EvalReproducibility) GetJudgeRubricVersionOk() (*string, bool)`

GetJudgeRubricVersionOk returns a tuple with the JudgeRubricVersion field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetJudgeRubricVersion

`func (o *EvalReproducibility) SetJudgeRubricVersion(v string)`

SetJudgeRubricVersion sets JudgeRubricVersion field to given value.

### HasJudgeRubricVersion

`func (o *EvalReproducibility) HasJudgeRubricVersion() bool`

HasJudgeRubricVersion returns a boolean if a field has been set.

### SetJudgeRubricVersionNil

`func (o *EvalReproducibility) SetJudgeRubricVersionNil(b bool)`

 SetJudgeRubricVersionNil sets the value for JudgeRubricVersion to be an explicit nil

### UnsetJudgeRubricVersion
`func (o *EvalReproducibility) UnsetJudgeRubricVersion()`

UnsetJudgeRubricVersion ensures that no value is present for JudgeRubricVersion, not even an explicit nil
### GetJudgeSeed

`func (o *EvalReproducibility) GetJudgeSeed() int64`

GetJudgeSeed returns the JudgeSeed field if non-nil, zero value otherwise.

### GetJudgeSeedOk

`func (o *EvalReproducibility) GetJudgeSeedOk() (*int64, bool)`

GetJudgeSeedOk returns a tuple with the JudgeSeed field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetJudgeSeed

`func (o *EvalReproducibility) SetJudgeSeed(v int64)`

SetJudgeSeed sets JudgeSeed field to given value.

### HasJudgeSeed

`func (o *EvalReproducibility) HasJudgeSeed() bool`

HasJudgeSeed returns a boolean if a field has been set.

### SetJudgeSeedNil

`func (o *EvalReproducibility) SetJudgeSeedNil(b bool)`

 SetJudgeSeedNil sets the value for JudgeSeed to be an explicit nil

### UnsetJudgeSeed
`func (o *EvalReproducibility) UnsetJudgeSeed()`

UnsetJudgeSeed ensures that no value is present for JudgeSeed, not even an explicit nil
### GetNormalizerVersion

`func (o *EvalReproducibility) GetNormalizerVersion() string`

GetNormalizerVersion returns the NormalizerVersion field if non-nil, zero value otherwise.

### GetNormalizerVersionOk

`func (o *EvalReproducibility) GetNormalizerVersionOk() (*string, bool)`

GetNormalizerVersionOk returns a tuple with the NormalizerVersion field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetNormalizerVersion

`func (o *EvalReproducibility) SetNormalizerVersion(v string)`

SetNormalizerVersion sets NormalizerVersion field to given value.


### GetPromptVersionId

`func (o *EvalReproducibility) GetPromptVersionId() string`

GetPromptVersionId returns the PromptVersionId field if non-nil, zero value otherwise.

### GetPromptVersionIdOk

`func (o *EvalReproducibility) GetPromptVersionIdOk() (*string, bool)`

GetPromptVersionIdOk returns a tuple with the PromptVersionId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPromptVersionId

`func (o *EvalReproducibility) SetPromptVersionId(v string)`

SetPromptVersionId sets PromptVersionId field to given value.

### HasPromptVersionId

`func (o *EvalReproducibility) HasPromptVersionId() bool`

HasPromptVersionId returns a boolean if a field has been set.

### GetTraceSchemaVersion

`func (o *EvalReproducibility) GetTraceSchemaVersion() int32`

GetTraceSchemaVersion returns the TraceSchemaVersion field if non-nil, zero value otherwise.

### GetTraceSchemaVersionOk

`func (o *EvalReproducibility) GetTraceSchemaVersionOk() (*int32, bool)`

GetTraceSchemaVersionOk returns a tuple with the TraceSchemaVersion field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTraceSchemaVersion

`func (o *EvalReproducibility) SetTraceSchemaVersion(v int32)`

SetTraceSchemaVersion sets TraceSchemaVersion field to given value.


### GetWasiAbiVersion

`func (o *EvalReproducibility) GetWasiAbiVersion() string`

GetWasiAbiVersion returns the WasiAbiVersion field if non-nil, zero value otherwise.

### GetWasiAbiVersionOk

`func (o *EvalReproducibility) GetWasiAbiVersionOk() (*string, bool)`

GetWasiAbiVersionOk returns a tuple with the WasiAbiVersion field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetWasiAbiVersion

`func (o *EvalReproducibility) SetWasiAbiVersion(v string)`

SetWasiAbiVersion sets WasiAbiVersion field to given value.

### HasWasiAbiVersion

`func (o *EvalReproducibility) HasWasiAbiVersion() bool`

HasWasiAbiVersion returns a boolean if a field has been set.

### SetWasiAbiVersionNil

`func (o *EvalReproducibility) SetWasiAbiVersionNil(b bool)`

 SetWasiAbiVersionNil sets the value for WasiAbiVersion to be an explicit nil

### UnsetWasiAbiVersion
`func (o *EvalReproducibility) UnsetWasiAbiVersion()`

UnsetWasiAbiVersion ensures that no value is present for WasiAbiVersion, not even an explicit nil
### GetWasmHash

`func (o *EvalReproducibility) GetWasmHash() string`

GetWasmHash returns the WasmHash field if non-nil, zero value otherwise.

### GetWasmHashOk

`func (o *EvalReproducibility) GetWasmHashOk() (*string, bool)`

GetWasmHashOk returns a tuple with the WasmHash field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetWasmHash

`func (o *EvalReproducibility) SetWasmHash(v string)`

SetWasmHash sets WasmHash field to given value.

### HasWasmHash

`func (o *EvalReproducibility) HasWasmHash() bool`

HasWasmHash returns a boolean if a field has been set.


[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
