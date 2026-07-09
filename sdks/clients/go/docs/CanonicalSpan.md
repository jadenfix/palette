# CanonicalSpan

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Attributes** | **map[string]interface{}** |  |
**Cost** | Pointer to [**NullableMoney**](Money.md) |  | [optional]
**EndTime** | Pointer to **NullableTime** |  | [optional]
**EnvironmentId** | **string** |  |
**InputRef** | Pointer to [**NullableArtifactRef**](ArtifactRef.md) |  | [optional]
**Kind** | **string** | Canonical agent span kind such as agent.run or llm.call |
**Model** | Pointer to [**NullableModelRef**](ModelRef.md) |  | [optional]
**Name** | **string** |  |
**NormalizerVersion** | **string** |  |
**OutputRef** | Pointer to [**NullableArtifactRef**](ArtifactRef.md) |  | [optional]
**ParentSpanId** | Pointer to **string** |  | [optional]
**ProjectId** | **string** |  |
**RawRef** | [**ArtifactRef**](ArtifactRef.md) |  |
**SchemaVersion** | **int32** |  |
**Seq** | **int64** |  |
**SpanId** | **string** |  |
**StartTime** | **time.Time** |  |
**Status** | [**SpanStatus**](SpanStatus.md) |  |
**TenantId** | **string** |  |
**Tokens** | Pointer to [**NullableTokenCounts**](TokenCounts.md) |  | [optional]
**TraceId** | **string** |  |
**UnmappedAttrs** | **interface{}** |  |

## Methods

### NewCanonicalSpan

`func NewCanonicalSpan(attributes map[string]interface{}, environmentId string, kind string, name string, normalizerVersion string, projectId string, rawRef ArtifactRef, schemaVersion int32, seq int64, spanId string, startTime time.Time, status SpanStatus, tenantId string, traceId string, unmappedAttrs interface{}, ) *CanonicalSpan`

NewCanonicalSpan instantiates a new CanonicalSpan object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewCanonicalSpanWithDefaults

`func NewCanonicalSpanWithDefaults() *CanonicalSpan`

NewCanonicalSpanWithDefaults instantiates a new CanonicalSpan object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAttributes

`func (o *CanonicalSpan) GetAttributes() map[string]interface{}`

GetAttributes returns the Attributes field if non-nil, zero value otherwise.

### GetAttributesOk

`func (o *CanonicalSpan) GetAttributesOk() (*map[string]interface{}, bool)`

GetAttributesOk returns a tuple with the Attributes field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAttributes

`func (o *CanonicalSpan) SetAttributes(v map[string]interface{})`

SetAttributes sets Attributes field to given value.


### GetCost

`func (o *CanonicalSpan) GetCost() Money`

GetCost returns the Cost field if non-nil, zero value otherwise.

### GetCostOk

`func (o *CanonicalSpan) GetCostOk() (*Money, bool)`

GetCostOk returns a tuple with the Cost field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCost

`func (o *CanonicalSpan) SetCost(v Money)`

SetCost sets Cost field to given value.

### HasCost

`func (o *CanonicalSpan) HasCost() bool`

HasCost returns a boolean if a field has been set.

### SetCostNil

`func (o *CanonicalSpan) SetCostNil(b bool)`

 SetCostNil sets the value for Cost to be an explicit nil

### UnsetCost
`func (o *CanonicalSpan) UnsetCost()`

UnsetCost ensures that no value is present for Cost, not even an explicit nil
### GetEndTime

`func (o *CanonicalSpan) GetEndTime() time.Time`

GetEndTime returns the EndTime field if non-nil, zero value otherwise.

### GetEndTimeOk

`func (o *CanonicalSpan) GetEndTimeOk() (*time.Time, bool)`

GetEndTimeOk returns a tuple with the EndTime field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEndTime

`func (o *CanonicalSpan) SetEndTime(v time.Time)`

SetEndTime sets EndTime field to given value.

### HasEndTime

`func (o *CanonicalSpan) HasEndTime() bool`

HasEndTime returns a boolean if a field has been set.

### SetEndTimeNil

`func (o *CanonicalSpan) SetEndTimeNil(b bool)`

 SetEndTimeNil sets the value for EndTime to be an explicit nil

### UnsetEndTime
`func (o *CanonicalSpan) UnsetEndTime()`

UnsetEndTime ensures that no value is present for EndTime, not even an explicit nil
### GetEnvironmentId

`func (o *CanonicalSpan) GetEnvironmentId() string`

GetEnvironmentId returns the EnvironmentId field if non-nil, zero value otherwise.

### GetEnvironmentIdOk

`func (o *CanonicalSpan) GetEnvironmentIdOk() (*string, bool)`

GetEnvironmentIdOk returns a tuple with the EnvironmentId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEnvironmentId

`func (o *CanonicalSpan) SetEnvironmentId(v string)`

SetEnvironmentId sets EnvironmentId field to given value.


### GetInputRef

`func (o *CanonicalSpan) GetInputRef() ArtifactRef`

GetInputRef returns the InputRef field if non-nil, zero value otherwise.

### GetInputRefOk

`func (o *CanonicalSpan) GetInputRefOk() (*ArtifactRef, bool)`

GetInputRefOk returns a tuple with the InputRef field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetInputRef

`func (o *CanonicalSpan) SetInputRef(v ArtifactRef)`

SetInputRef sets InputRef field to given value.

### HasInputRef

`func (o *CanonicalSpan) HasInputRef() bool`

HasInputRef returns a boolean if a field has been set.

### SetInputRefNil

`func (o *CanonicalSpan) SetInputRefNil(b bool)`

 SetInputRefNil sets the value for InputRef to be an explicit nil

### UnsetInputRef
`func (o *CanonicalSpan) UnsetInputRef()`

UnsetInputRef ensures that no value is present for InputRef, not even an explicit nil
### GetKind

`func (o *CanonicalSpan) GetKind() string`

GetKind returns the Kind field if non-nil, zero value otherwise.

### GetKindOk

`func (o *CanonicalSpan) GetKindOk() (*string, bool)`

GetKindOk returns a tuple with the Kind field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetKind

`func (o *CanonicalSpan) SetKind(v string)`

SetKind sets Kind field to given value.


### GetModel

`func (o *CanonicalSpan) GetModel() ModelRef`

GetModel returns the Model field if non-nil, zero value otherwise.

### GetModelOk

`func (o *CanonicalSpan) GetModelOk() (*ModelRef, bool)`

GetModelOk returns a tuple with the Model field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetModel

`func (o *CanonicalSpan) SetModel(v ModelRef)`

SetModel sets Model field to given value.

### HasModel

`func (o *CanonicalSpan) HasModel() bool`

HasModel returns a boolean if a field has been set.

### SetModelNil

`func (o *CanonicalSpan) SetModelNil(b bool)`

 SetModelNil sets the value for Model to be an explicit nil

### UnsetModel
`func (o *CanonicalSpan) UnsetModel()`

UnsetModel ensures that no value is present for Model, not even an explicit nil
### GetName

`func (o *CanonicalSpan) GetName() string`

GetName returns the Name field if non-nil, zero value otherwise.

### GetNameOk

`func (o *CanonicalSpan) GetNameOk() (*string, bool)`

GetNameOk returns a tuple with the Name field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetName

`func (o *CanonicalSpan) SetName(v string)`

SetName sets Name field to given value.


### GetNormalizerVersion

`func (o *CanonicalSpan) GetNormalizerVersion() string`

GetNormalizerVersion returns the NormalizerVersion field if non-nil, zero value otherwise.

### GetNormalizerVersionOk

`func (o *CanonicalSpan) GetNormalizerVersionOk() (*string, bool)`

GetNormalizerVersionOk returns a tuple with the NormalizerVersion field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetNormalizerVersion

`func (o *CanonicalSpan) SetNormalizerVersion(v string)`

SetNormalizerVersion sets NormalizerVersion field to given value.


### GetOutputRef

`func (o *CanonicalSpan) GetOutputRef() ArtifactRef`

GetOutputRef returns the OutputRef field if non-nil, zero value otherwise.

### GetOutputRefOk

`func (o *CanonicalSpan) GetOutputRefOk() (*ArtifactRef, bool)`

GetOutputRefOk returns a tuple with the OutputRef field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetOutputRef

`func (o *CanonicalSpan) SetOutputRef(v ArtifactRef)`

SetOutputRef sets OutputRef field to given value.

### HasOutputRef

`func (o *CanonicalSpan) HasOutputRef() bool`

HasOutputRef returns a boolean if a field has been set.

### SetOutputRefNil

`func (o *CanonicalSpan) SetOutputRefNil(b bool)`

 SetOutputRefNil sets the value for OutputRef to be an explicit nil

### UnsetOutputRef
`func (o *CanonicalSpan) UnsetOutputRef()`

UnsetOutputRef ensures that no value is present for OutputRef, not even an explicit nil
### GetParentSpanId

`func (o *CanonicalSpan) GetParentSpanId() string`

GetParentSpanId returns the ParentSpanId field if non-nil, zero value otherwise.

### GetParentSpanIdOk

`func (o *CanonicalSpan) GetParentSpanIdOk() (*string, bool)`

GetParentSpanIdOk returns a tuple with the ParentSpanId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetParentSpanId

`func (o *CanonicalSpan) SetParentSpanId(v string)`

SetParentSpanId sets ParentSpanId field to given value.

### HasParentSpanId

`func (o *CanonicalSpan) HasParentSpanId() bool`

HasParentSpanId returns a boolean if a field has been set.

### GetProjectId

`func (o *CanonicalSpan) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *CanonicalSpan) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *CanonicalSpan) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetRawRef

`func (o *CanonicalSpan) GetRawRef() ArtifactRef`

GetRawRef returns the RawRef field if non-nil, zero value otherwise.

### GetRawRefOk

`func (o *CanonicalSpan) GetRawRefOk() (*ArtifactRef, bool)`

GetRawRefOk returns a tuple with the RawRef field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRawRef

`func (o *CanonicalSpan) SetRawRef(v ArtifactRef)`

SetRawRef sets RawRef field to given value.


### GetSchemaVersion

`func (o *CanonicalSpan) GetSchemaVersion() int32`

GetSchemaVersion returns the SchemaVersion field if non-nil, zero value otherwise.

### GetSchemaVersionOk

`func (o *CanonicalSpan) GetSchemaVersionOk() (*int32, bool)`

GetSchemaVersionOk returns a tuple with the SchemaVersion field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSchemaVersion

`func (o *CanonicalSpan) SetSchemaVersion(v int32)`

SetSchemaVersion sets SchemaVersion field to given value.


### GetSeq

`func (o *CanonicalSpan) GetSeq() int64`

GetSeq returns the Seq field if non-nil, zero value otherwise.

### GetSeqOk

`func (o *CanonicalSpan) GetSeqOk() (*int64, bool)`

GetSeqOk returns a tuple with the Seq field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSeq

`func (o *CanonicalSpan) SetSeq(v int64)`

SetSeq sets Seq field to given value.


### GetSpanId

`func (o *CanonicalSpan) GetSpanId() string`

GetSpanId returns the SpanId field if non-nil, zero value otherwise.

### GetSpanIdOk

`func (o *CanonicalSpan) GetSpanIdOk() (*string, bool)`

GetSpanIdOk returns a tuple with the SpanId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSpanId

`func (o *CanonicalSpan) SetSpanId(v string)`

SetSpanId sets SpanId field to given value.


### GetStartTime

`func (o *CanonicalSpan) GetStartTime() time.Time`

GetStartTime returns the StartTime field if non-nil, zero value otherwise.

### GetStartTimeOk

`func (o *CanonicalSpan) GetStartTimeOk() (*time.Time, bool)`

GetStartTimeOk returns a tuple with the StartTime field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetStartTime

`func (o *CanonicalSpan) SetStartTime(v time.Time)`

SetStartTime sets StartTime field to given value.


### GetStatus

`func (o *CanonicalSpan) GetStatus() SpanStatus`

GetStatus returns the Status field if non-nil, zero value otherwise.

### GetStatusOk

`func (o *CanonicalSpan) GetStatusOk() (*SpanStatus, bool)`

GetStatusOk returns a tuple with the Status field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetStatus

`func (o *CanonicalSpan) SetStatus(v SpanStatus)`

SetStatus sets Status field to given value.


### GetTenantId

`func (o *CanonicalSpan) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *CanonicalSpan) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *CanonicalSpan) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.


### GetTokens

`func (o *CanonicalSpan) GetTokens() TokenCounts`

GetTokens returns the Tokens field if non-nil, zero value otherwise.

### GetTokensOk

`func (o *CanonicalSpan) GetTokensOk() (*TokenCounts, bool)`

GetTokensOk returns a tuple with the Tokens field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTokens

`func (o *CanonicalSpan) SetTokens(v TokenCounts)`

SetTokens sets Tokens field to given value.

### HasTokens

`func (o *CanonicalSpan) HasTokens() bool`

HasTokens returns a boolean if a field has been set.

### SetTokensNil

`func (o *CanonicalSpan) SetTokensNil(b bool)`

 SetTokensNil sets the value for Tokens to be an explicit nil

### UnsetTokens
`func (o *CanonicalSpan) UnsetTokens()`

UnsetTokens ensures that no value is present for Tokens, not even an explicit nil
### GetTraceId

`func (o *CanonicalSpan) GetTraceId() string`

GetTraceId returns the TraceId field if non-nil, zero value otherwise.

### GetTraceIdOk

`func (o *CanonicalSpan) GetTraceIdOk() (*string, bool)`

GetTraceIdOk returns a tuple with the TraceId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTraceId

`func (o *CanonicalSpan) SetTraceId(v string)`

SetTraceId sets TraceId field to given value.


### GetUnmappedAttrs

`func (o *CanonicalSpan) GetUnmappedAttrs() interface{}`

GetUnmappedAttrs returns the UnmappedAttrs field if non-nil, zero value otherwise.

### GetUnmappedAttrsOk

`func (o *CanonicalSpan) GetUnmappedAttrsOk() (*interface{}, bool)`

GetUnmappedAttrsOk returns a tuple with the UnmappedAttrs field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetUnmappedAttrs

`func (o *CanonicalSpan) SetUnmappedAttrs(v interface{})`

SetUnmappedAttrs sets UnmappedAttrs field to given value.


### SetUnmappedAttrsNil

`func (o *CanonicalSpan) SetUnmappedAttrsNil(b bool)`

 SetUnmappedAttrsNil sets the value for UnmappedAttrs to be an explicit nil

### UnsetUnmappedAttrs
`func (o *CanonicalSpan) UnsetUnmappedAttrs()`

UnsetUnmappedAttrs ensures that no value is present for UnmappedAttrs, not even an explicit nil

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
