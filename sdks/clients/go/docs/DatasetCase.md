# DatasetCase

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**CaseId** | **string** |  |
**CreatedAt** | **time.Time** |  |
**DatasetId** | **string** |  |
**Input** | **interface{}** |  |
**InputArtifactHashes** | **[]string** |  |
**NormalizerVersion** | **string** |  |
**Output** | **interface{}** |  |
**ProjectId** | **string** |  |
**Reference** | Pointer to **interface{}** |  | [optional]
**SourceEnvironmentId** | **string** |  |
**SourceSpanId** | **string** |  |
**SourceTraceId** | **string** |  |
**TenantId** | **string** |  |
**Trace** | **interface{}** |  |
**TraceSchemaVersion** | **int32** |  |

## Methods

### NewDatasetCase

`func NewDatasetCase(caseId string, createdAt time.Time, datasetId string, input interface{}, inputArtifactHashes []string, normalizerVersion string, output interface{}, projectId string, sourceEnvironmentId string, sourceSpanId string, sourceTraceId string, tenantId string, trace interface{}, traceSchemaVersion int32, ) *DatasetCase`

NewDatasetCase instantiates a new DatasetCase object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewDatasetCaseWithDefaults

`func NewDatasetCaseWithDefaults() *DatasetCase`

NewDatasetCaseWithDefaults instantiates a new DatasetCase object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetCaseId

`func (o *DatasetCase) GetCaseId() string`

GetCaseId returns the CaseId field if non-nil, zero value otherwise.

### GetCaseIdOk

`func (o *DatasetCase) GetCaseIdOk() (*string, bool)`

GetCaseIdOk returns a tuple with the CaseId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCaseId

`func (o *DatasetCase) SetCaseId(v string)`

SetCaseId sets CaseId field to given value.


### GetCreatedAt

`func (o *DatasetCase) GetCreatedAt() time.Time`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *DatasetCase) GetCreatedAtOk() (*time.Time, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *DatasetCase) SetCreatedAt(v time.Time)`

SetCreatedAt sets CreatedAt field to given value.


### GetDatasetId

`func (o *DatasetCase) GetDatasetId() string`

GetDatasetId returns the DatasetId field if non-nil, zero value otherwise.

### GetDatasetIdOk

`func (o *DatasetCase) GetDatasetIdOk() (*string, bool)`

GetDatasetIdOk returns a tuple with the DatasetId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDatasetId

`func (o *DatasetCase) SetDatasetId(v string)`

SetDatasetId sets DatasetId field to given value.


### GetInput

`func (o *DatasetCase) GetInput() interface{}`

GetInput returns the Input field if non-nil, zero value otherwise.

### GetInputOk

`func (o *DatasetCase) GetInputOk() (*interface{}, bool)`

GetInputOk returns a tuple with the Input field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetInput

`func (o *DatasetCase) SetInput(v interface{})`

SetInput sets Input field to given value.


### SetInputNil

`func (o *DatasetCase) SetInputNil(b bool)`

 SetInputNil sets the value for Input to be an explicit nil

### UnsetInput
`func (o *DatasetCase) UnsetInput()`

UnsetInput ensures that no value is present for Input, not even an explicit nil
### GetInputArtifactHashes

`func (o *DatasetCase) GetInputArtifactHashes() []string`

GetInputArtifactHashes returns the InputArtifactHashes field if non-nil, zero value otherwise.

### GetInputArtifactHashesOk

`func (o *DatasetCase) GetInputArtifactHashesOk() (*[]string, bool)`

GetInputArtifactHashesOk returns a tuple with the InputArtifactHashes field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetInputArtifactHashes

`func (o *DatasetCase) SetInputArtifactHashes(v []string)`

SetInputArtifactHashes sets InputArtifactHashes field to given value.


### GetNormalizerVersion

`func (o *DatasetCase) GetNormalizerVersion() string`

GetNormalizerVersion returns the NormalizerVersion field if non-nil, zero value otherwise.

### GetNormalizerVersionOk

`func (o *DatasetCase) GetNormalizerVersionOk() (*string, bool)`

GetNormalizerVersionOk returns a tuple with the NormalizerVersion field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetNormalizerVersion

`func (o *DatasetCase) SetNormalizerVersion(v string)`

SetNormalizerVersion sets NormalizerVersion field to given value.


### GetOutput

`func (o *DatasetCase) GetOutput() interface{}`

GetOutput returns the Output field if non-nil, zero value otherwise.

### GetOutputOk

`func (o *DatasetCase) GetOutputOk() (*interface{}, bool)`

GetOutputOk returns a tuple with the Output field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetOutput

`func (o *DatasetCase) SetOutput(v interface{})`

SetOutput sets Output field to given value.


### SetOutputNil

`func (o *DatasetCase) SetOutputNil(b bool)`

 SetOutputNil sets the value for Output to be an explicit nil

### UnsetOutput
`func (o *DatasetCase) UnsetOutput()`

UnsetOutput ensures that no value is present for Output, not even an explicit nil
### GetProjectId

`func (o *DatasetCase) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *DatasetCase) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *DatasetCase) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetReference

`func (o *DatasetCase) GetReference() interface{}`

GetReference returns the Reference field if non-nil, zero value otherwise.

### GetReferenceOk

`func (o *DatasetCase) GetReferenceOk() (*interface{}, bool)`

GetReferenceOk returns a tuple with the Reference field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetReference

`func (o *DatasetCase) SetReference(v interface{})`

SetReference sets Reference field to given value.

### HasReference

`func (o *DatasetCase) HasReference() bool`

HasReference returns a boolean if a field has been set.

### SetReferenceNil

`func (o *DatasetCase) SetReferenceNil(b bool)`

 SetReferenceNil sets the value for Reference to be an explicit nil

### UnsetReference
`func (o *DatasetCase) UnsetReference()`

UnsetReference ensures that no value is present for Reference, not even an explicit nil
### GetSourceEnvironmentId

`func (o *DatasetCase) GetSourceEnvironmentId() string`

GetSourceEnvironmentId returns the SourceEnvironmentId field if non-nil, zero value otherwise.

### GetSourceEnvironmentIdOk

`func (o *DatasetCase) GetSourceEnvironmentIdOk() (*string, bool)`

GetSourceEnvironmentIdOk returns a tuple with the SourceEnvironmentId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSourceEnvironmentId

`func (o *DatasetCase) SetSourceEnvironmentId(v string)`

SetSourceEnvironmentId sets SourceEnvironmentId field to given value.


### GetSourceSpanId

`func (o *DatasetCase) GetSourceSpanId() string`

GetSourceSpanId returns the SourceSpanId field if non-nil, zero value otherwise.

### GetSourceSpanIdOk

`func (o *DatasetCase) GetSourceSpanIdOk() (*string, bool)`

GetSourceSpanIdOk returns a tuple with the SourceSpanId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSourceSpanId

`func (o *DatasetCase) SetSourceSpanId(v string)`

SetSourceSpanId sets SourceSpanId field to given value.


### GetSourceTraceId

`func (o *DatasetCase) GetSourceTraceId() string`

GetSourceTraceId returns the SourceTraceId field if non-nil, zero value otherwise.

### GetSourceTraceIdOk

`func (o *DatasetCase) GetSourceTraceIdOk() (*string, bool)`

GetSourceTraceIdOk returns a tuple with the SourceTraceId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSourceTraceId

`func (o *DatasetCase) SetSourceTraceId(v string)`

SetSourceTraceId sets SourceTraceId field to given value.


### GetTenantId

`func (o *DatasetCase) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *DatasetCase) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *DatasetCase) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.


### GetTrace

`func (o *DatasetCase) GetTrace() interface{}`

GetTrace returns the Trace field if non-nil, zero value otherwise.

### GetTraceOk

`func (o *DatasetCase) GetTraceOk() (*interface{}, bool)`

GetTraceOk returns a tuple with the Trace field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTrace

`func (o *DatasetCase) SetTrace(v interface{})`

SetTrace sets Trace field to given value.


### SetTraceNil

`func (o *DatasetCase) SetTraceNil(b bool)`

 SetTraceNil sets the value for Trace to be an explicit nil

### UnsetTrace
`func (o *DatasetCase) UnsetTrace()`

UnsetTrace ensures that no value is present for Trace, not even an explicit nil
### GetTraceSchemaVersion

`func (o *DatasetCase) GetTraceSchemaVersion() int32`

GetTraceSchemaVersion returns the TraceSchemaVersion field if non-nil, zero value otherwise.

### GetTraceSchemaVersionOk

`func (o *DatasetCase) GetTraceSchemaVersionOk() (*int32, bool)`

GetTraceSchemaVersionOk returns a tuple with the TraceSchemaVersion field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTraceSchemaVersion

`func (o *DatasetCase) SetTraceSchemaVersion(v int32)`

SetTraceSchemaVersion sets TraceSchemaVersion field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
