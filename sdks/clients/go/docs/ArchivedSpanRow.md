# ArchivedSpanRow

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AttributesJson** | **string** |  |
**CostAmountMicros** | Pointer to **NullableString** |  | [optional]
**CostCurrency** | Pointer to **NullableString** |  | [optional]
**EndTime** | Pointer to **NullableString** |  | [optional]
**EnvironmentId** | **string** |  |
**InputTokens** | Pointer to **NullableString** |  | [optional]
**InputUri** | Pointer to **NullableString** |  | [optional]
**Kind** | **string** |  |
**ModelName** | Pointer to **NullableString** |  | [optional]
**ModelProvider** | Pointer to **NullableString** |  | [optional]
**Name** | **string** |  |
**OutputTokens** | Pointer to **NullableString** |  | [optional]
**OutputUri** | Pointer to **NullableString** |  | [optional]
**ParentSpanId** | Pointer to **NullableString** |  | [optional]
**ProjectId** | **string** |  |
**RawUri** | **string** |  |
**ReasoningTokens** | Pointer to **NullableString** |  | [optional]
**Seq** | **int64** |  |
**SpanId** | **string** |  |
**StartTime** | **string** |  |
**Status** | **string** |  |
**TenantId** | **string** |  |
**TraceId** | **string** |  |
**UnmappedJson** | **string** |  |

## Methods

### NewArchivedSpanRow

`func NewArchivedSpanRow(attributesJson string, environmentId string, kind string, name string, projectId string, rawUri string, seq int64, spanId string, startTime string, status string, tenantId string, traceId string, unmappedJson string, ) *ArchivedSpanRow`

NewArchivedSpanRow instantiates a new ArchivedSpanRow object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewArchivedSpanRowWithDefaults

`func NewArchivedSpanRowWithDefaults() *ArchivedSpanRow`

NewArchivedSpanRowWithDefaults instantiates a new ArchivedSpanRow object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAttributesJson

`func (o *ArchivedSpanRow) GetAttributesJson() string`

GetAttributesJson returns the AttributesJson field if non-nil, zero value otherwise.

### GetAttributesJsonOk

`func (o *ArchivedSpanRow) GetAttributesJsonOk() (*string, bool)`

GetAttributesJsonOk returns a tuple with the AttributesJson field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAttributesJson

`func (o *ArchivedSpanRow) SetAttributesJson(v string)`

SetAttributesJson sets AttributesJson field to given value.


### GetCostAmountMicros

`func (o *ArchivedSpanRow) GetCostAmountMicros() string`

GetCostAmountMicros returns the CostAmountMicros field if non-nil, zero value otherwise.

### GetCostAmountMicrosOk

`func (o *ArchivedSpanRow) GetCostAmountMicrosOk() (*string, bool)`

GetCostAmountMicrosOk returns a tuple with the CostAmountMicros field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCostAmountMicros

`func (o *ArchivedSpanRow) SetCostAmountMicros(v string)`

SetCostAmountMicros sets CostAmountMicros field to given value.

### HasCostAmountMicros

`func (o *ArchivedSpanRow) HasCostAmountMicros() bool`

HasCostAmountMicros returns a boolean if a field has been set.

### SetCostAmountMicrosNil

`func (o *ArchivedSpanRow) SetCostAmountMicrosNil(b bool)`

 SetCostAmountMicrosNil sets the value for CostAmountMicros to be an explicit nil

### UnsetCostAmountMicros
`func (o *ArchivedSpanRow) UnsetCostAmountMicros()`

UnsetCostAmountMicros ensures that no value is present for CostAmountMicros, not even an explicit nil
### GetCostCurrency

`func (o *ArchivedSpanRow) GetCostCurrency() string`

GetCostCurrency returns the CostCurrency field if non-nil, zero value otherwise.

### GetCostCurrencyOk

`func (o *ArchivedSpanRow) GetCostCurrencyOk() (*string, bool)`

GetCostCurrencyOk returns a tuple with the CostCurrency field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCostCurrency

`func (o *ArchivedSpanRow) SetCostCurrency(v string)`

SetCostCurrency sets CostCurrency field to given value.

### HasCostCurrency

`func (o *ArchivedSpanRow) HasCostCurrency() bool`

HasCostCurrency returns a boolean if a field has been set.

### SetCostCurrencyNil

`func (o *ArchivedSpanRow) SetCostCurrencyNil(b bool)`

 SetCostCurrencyNil sets the value for CostCurrency to be an explicit nil

### UnsetCostCurrency
`func (o *ArchivedSpanRow) UnsetCostCurrency()`

UnsetCostCurrency ensures that no value is present for CostCurrency, not even an explicit nil
### GetEndTime

`func (o *ArchivedSpanRow) GetEndTime() string`

GetEndTime returns the EndTime field if non-nil, zero value otherwise.

### GetEndTimeOk

`func (o *ArchivedSpanRow) GetEndTimeOk() (*string, bool)`

GetEndTimeOk returns a tuple with the EndTime field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEndTime

`func (o *ArchivedSpanRow) SetEndTime(v string)`

SetEndTime sets EndTime field to given value.

### HasEndTime

`func (o *ArchivedSpanRow) HasEndTime() bool`

HasEndTime returns a boolean if a field has been set.

### SetEndTimeNil

`func (o *ArchivedSpanRow) SetEndTimeNil(b bool)`

 SetEndTimeNil sets the value for EndTime to be an explicit nil

### UnsetEndTime
`func (o *ArchivedSpanRow) UnsetEndTime()`

UnsetEndTime ensures that no value is present for EndTime, not even an explicit nil
### GetEnvironmentId

`func (o *ArchivedSpanRow) GetEnvironmentId() string`

GetEnvironmentId returns the EnvironmentId field if non-nil, zero value otherwise.

### GetEnvironmentIdOk

`func (o *ArchivedSpanRow) GetEnvironmentIdOk() (*string, bool)`

GetEnvironmentIdOk returns a tuple with the EnvironmentId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEnvironmentId

`func (o *ArchivedSpanRow) SetEnvironmentId(v string)`

SetEnvironmentId sets EnvironmentId field to given value.


### GetInputTokens

`func (o *ArchivedSpanRow) GetInputTokens() string`

GetInputTokens returns the InputTokens field if non-nil, zero value otherwise.

### GetInputTokensOk

`func (o *ArchivedSpanRow) GetInputTokensOk() (*string, bool)`

GetInputTokensOk returns a tuple with the InputTokens field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetInputTokens

`func (o *ArchivedSpanRow) SetInputTokens(v string)`

SetInputTokens sets InputTokens field to given value.

### HasInputTokens

`func (o *ArchivedSpanRow) HasInputTokens() bool`

HasInputTokens returns a boolean if a field has been set.

### SetInputTokensNil

`func (o *ArchivedSpanRow) SetInputTokensNil(b bool)`

 SetInputTokensNil sets the value for InputTokens to be an explicit nil

### UnsetInputTokens
`func (o *ArchivedSpanRow) UnsetInputTokens()`

UnsetInputTokens ensures that no value is present for InputTokens, not even an explicit nil
### GetInputUri

`func (o *ArchivedSpanRow) GetInputUri() string`

GetInputUri returns the InputUri field if non-nil, zero value otherwise.

### GetInputUriOk

`func (o *ArchivedSpanRow) GetInputUriOk() (*string, bool)`

GetInputUriOk returns a tuple with the InputUri field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetInputUri

`func (o *ArchivedSpanRow) SetInputUri(v string)`

SetInputUri sets InputUri field to given value.

### HasInputUri

`func (o *ArchivedSpanRow) HasInputUri() bool`

HasInputUri returns a boolean if a field has been set.

### SetInputUriNil

`func (o *ArchivedSpanRow) SetInputUriNil(b bool)`

 SetInputUriNil sets the value for InputUri to be an explicit nil

### UnsetInputUri
`func (o *ArchivedSpanRow) UnsetInputUri()`

UnsetInputUri ensures that no value is present for InputUri, not even an explicit nil
### GetKind

`func (o *ArchivedSpanRow) GetKind() string`

GetKind returns the Kind field if non-nil, zero value otherwise.

### GetKindOk

`func (o *ArchivedSpanRow) GetKindOk() (*string, bool)`

GetKindOk returns a tuple with the Kind field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetKind

`func (o *ArchivedSpanRow) SetKind(v string)`

SetKind sets Kind field to given value.


### GetModelName

`func (o *ArchivedSpanRow) GetModelName() string`

GetModelName returns the ModelName field if non-nil, zero value otherwise.

### GetModelNameOk

`func (o *ArchivedSpanRow) GetModelNameOk() (*string, bool)`

GetModelNameOk returns a tuple with the ModelName field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetModelName

`func (o *ArchivedSpanRow) SetModelName(v string)`

SetModelName sets ModelName field to given value.

### HasModelName

`func (o *ArchivedSpanRow) HasModelName() bool`

HasModelName returns a boolean if a field has been set.

### SetModelNameNil

`func (o *ArchivedSpanRow) SetModelNameNil(b bool)`

 SetModelNameNil sets the value for ModelName to be an explicit nil

### UnsetModelName
`func (o *ArchivedSpanRow) UnsetModelName()`

UnsetModelName ensures that no value is present for ModelName, not even an explicit nil
### GetModelProvider

`func (o *ArchivedSpanRow) GetModelProvider() string`

GetModelProvider returns the ModelProvider field if non-nil, zero value otherwise.

### GetModelProviderOk

`func (o *ArchivedSpanRow) GetModelProviderOk() (*string, bool)`

GetModelProviderOk returns a tuple with the ModelProvider field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetModelProvider

`func (o *ArchivedSpanRow) SetModelProvider(v string)`

SetModelProvider sets ModelProvider field to given value.

### HasModelProvider

`func (o *ArchivedSpanRow) HasModelProvider() bool`

HasModelProvider returns a boolean if a field has been set.

### SetModelProviderNil

`func (o *ArchivedSpanRow) SetModelProviderNil(b bool)`

 SetModelProviderNil sets the value for ModelProvider to be an explicit nil

### UnsetModelProvider
`func (o *ArchivedSpanRow) UnsetModelProvider()`

UnsetModelProvider ensures that no value is present for ModelProvider, not even an explicit nil
### GetName

`func (o *ArchivedSpanRow) GetName() string`

GetName returns the Name field if non-nil, zero value otherwise.

### GetNameOk

`func (o *ArchivedSpanRow) GetNameOk() (*string, bool)`

GetNameOk returns a tuple with the Name field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetName

`func (o *ArchivedSpanRow) SetName(v string)`

SetName sets Name field to given value.


### GetOutputTokens

`func (o *ArchivedSpanRow) GetOutputTokens() string`

GetOutputTokens returns the OutputTokens field if non-nil, zero value otherwise.

### GetOutputTokensOk

`func (o *ArchivedSpanRow) GetOutputTokensOk() (*string, bool)`

GetOutputTokensOk returns a tuple with the OutputTokens field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetOutputTokens

`func (o *ArchivedSpanRow) SetOutputTokens(v string)`

SetOutputTokens sets OutputTokens field to given value.

### HasOutputTokens

`func (o *ArchivedSpanRow) HasOutputTokens() bool`

HasOutputTokens returns a boolean if a field has been set.

### SetOutputTokensNil

`func (o *ArchivedSpanRow) SetOutputTokensNil(b bool)`

 SetOutputTokensNil sets the value for OutputTokens to be an explicit nil

### UnsetOutputTokens
`func (o *ArchivedSpanRow) UnsetOutputTokens()`

UnsetOutputTokens ensures that no value is present for OutputTokens, not even an explicit nil
### GetOutputUri

`func (o *ArchivedSpanRow) GetOutputUri() string`

GetOutputUri returns the OutputUri field if non-nil, zero value otherwise.

### GetOutputUriOk

`func (o *ArchivedSpanRow) GetOutputUriOk() (*string, bool)`

GetOutputUriOk returns a tuple with the OutputUri field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetOutputUri

`func (o *ArchivedSpanRow) SetOutputUri(v string)`

SetOutputUri sets OutputUri field to given value.

### HasOutputUri

`func (o *ArchivedSpanRow) HasOutputUri() bool`

HasOutputUri returns a boolean if a field has been set.

### SetOutputUriNil

`func (o *ArchivedSpanRow) SetOutputUriNil(b bool)`

 SetOutputUriNil sets the value for OutputUri to be an explicit nil

### UnsetOutputUri
`func (o *ArchivedSpanRow) UnsetOutputUri()`

UnsetOutputUri ensures that no value is present for OutputUri, not even an explicit nil
### GetParentSpanId

`func (o *ArchivedSpanRow) GetParentSpanId() string`

GetParentSpanId returns the ParentSpanId field if non-nil, zero value otherwise.

### GetParentSpanIdOk

`func (o *ArchivedSpanRow) GetParentSpanIdOk() (*string, bool)`

GetParentSpanIdOk returns a tuple with the ParentSpanId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetParentSpanId

`func (o *ArchivedSpanRow) SetParentSpanId(v string)`

SetParentSpanId sets ParentSpanId field to given value.

### HasParentSpanId

`func (o *ArchivedSpanRow) HasParentSpanId() bool`

HasParentSpanId returns a boolean if a field has been set.

### SetParentSpanIdNil

`func (o *ArchivedSpanRow) SetParentSpanIdNil(b bool)`

 SetParentSpanIdNil sets the value for ParentSpanId to be an explicit nil

### UnsetParentSpanId
`func (o *ArchivedSpanRow) UnsetParentSpanId()`

UnsetParentSpanId ensures that no value is present for ParentSpanId, not even an explicit nil
### GetProjectId

`func (o *ArchivedSpanRow) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *ArchivedSpanRow) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *ArchivedSpanRow) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetRawUri

`func (o *ArchivedSpanRow) GetRawUri() string`

GetRawUri returns the RawUri field if non-nil, zero value otherwise.

### GetRawUriOk

`func (o *ArchivedSpanRow) GetRawUriOk() (*string, bool)`

GetRawUriOk returns a tuple with the RawUri field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRawUri

`func (o *ArchivedSpanRow) SetRawUri(v string)`

SetRawUri sets RawUri field to given value.


### GetReasoningTokens

`func (o *ArchivedSpanRow) GetReasoningTokens() string`

GetReasoningTokens returns the ReasoningTokens field if non-nil, zero value otherwise.

### GetReasoningTokensOk

`func (o *ArchivedSpanRow) GetReasoningTokensOk() (*string, bool)`

GetReasoningTokensOk returns a tuple with the ReasoningTokens field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetReasoningTokens

`func (o *ArchivedSpanRow) SetReasoningTokens(v string)`

SetReasoningTokens sets ReasoningTokens field to given value.

### HasReasoningTokens

`func (o *ArchivedSpanRow) HasReasoningTokens() bool`

HasReasoningTokens returns a boolean if a field has been set.

### SetReasoningTokensNil

`func (o *ArchivedSpanRow) SetReasoningTokensNil(b bool)`

 SetReasoningTokensNil sets the value for ReasoningTokens to be an explicit nil

### UnsetReasoningTokens
`func (o *ArchivedSpanRow) UnsetReasoningTokens()`

UnsetReasoningTokens ensures that no value is present for ReasoningTokens, not even an explicit nil
### GetSeq

`func (o *ArchivedSpanRow) GetSeq() int64`

GetSeq returns the Seq field if non-nil, zero value otherwise.

### GetSeqOk

`func (o *ArchivedSpanRow) GetSeqOk() (*int64, bool)`

GetSeqOk returns a tuple with the Seq field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSeq

`func (o *ArchivedSpanRow) SetSeq(v int64)`

SetSeq sets Seq field to given value.


### GetSpanId

`func (o *ArchivedSpanRow) GetSpanId() string`

GetSpanId returns the SpanId field if non-nil, zero value otherwise.

### GetSpanIdOk

`func (o *ArchivedSpanRow) GetSpanIdOk() (*string, bool)`

GetSpanIdOk returns a tuple with the SpanId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSpanId

`func (o *ArchivedSpanRow) SetSpanId(v string)`

SetSpanId sets SpanId field to given value.


### GetStartTime

`func (o *ArchivedSpanRow) GetStartTime() string`

GetStartTime returns the StartTime field if non-nil, zero value otherwise.

### GetStartTimeOk

`func (o *ArchivedSpanRow) GetStartTimeOk() (*string, bool)`

GetStartTimeOk returns a tuple with the StartTime field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetStartTime

`func (o *ArchivedSpanRow) SetStartTime(v string)`

SetStartTime sets StartTime field to given value.


### GetStatus

`func (o *ArchivedSpanRow) GetStatus() string`

GetStatus returns the Status field if non-nil, zero value otherwise.

### GetStatusOk

`func (o *ArchivedSpanRow) GetStatusOk() (*string, bool)`

GetStatusOk returns a tuple with the Status field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetStatus

`func (o *ArchivedSpanRow) SetStatus(v string)`

SetStatus sets Status field to given value.


### GetTenantId

`func (o *ArchivedSpanRow) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *ArchivedSpanRow) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *ArchivedSpanRow) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.


### GetTraceId

`func (o *ArchivedSpanRow) GetTraceId() string`

GetTraceId returns the TraceId field if non-nil, zero value otherwise.

### GetTraceIdOk

`func (o *ArchivedSpanRow) GetTraceIdOk() (*string, bool)`

GetTraceIdOk returns a tuple with the TraceId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTraceId

`func (o *ArchivedSpanRow) SetTraceId(v string)`

SetTraceId sets TraceId field to given value.


### GetUnmappedJson

`func (o *ArchivedSpanRow) GetUnmappedJson() string`

GetUnmappedJson returns the UnmappedJson field if non-nil, zero value otherwise.

### GetUnmappedJsonOk

`func (o *ArchivedSpanRow) GetUnmappedJsonOk() (*string, bool)`

GetUnmappedJsonOk returns a tuple with the UnmappedJson field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetUnmappedJson

`func (o *ArchivedSpanRow) SetUnmappedJson(v string)`

SetUnmappedJson sets UnmappedJson field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
