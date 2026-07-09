# NativeIngestRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Attributes** | **map[string]interface{}** |  |
**AuthContext** | Pointer to [**NullableAuthContext**](AuthContext.md) |  | [optional]
**Cost** | Pointer to [**NullableMoney**](Money.md) |  | [optional]
**EndTime** | Pointer to **NullableTime** |  | [optional]
**IdempotencyKey** | Pointer to **string** |  | [optional]
**Input** | Pointer to **interface{}** |  | [optional]
**Kind** | **string** | Canonical agent span kind such as agent.run or llm.call |
**Model** | Pointer to [**NullableModelRef**](ModelRef.md) |  | [optional]
**Name** | **string** |  |
**Output** | Pointer to **interface{}** |  | [optional]
**ParentSpanId** | Pointer to **string** |  | [optional]
**RedactionClass** | [**RedactionClass**](RedactionClass.md) |  |
**Scope** | [**TenantScope**](TenantScope.md) |  |
**Seq** | **int64** |  |
**SpanId** | **string** |  |
**StartTime** | Pointer to **NullableTime** |  | [optional]
**Status** | [**SpanStatus**](SpanStatus.md) |  |
**Tokens** | Pointer to [**NullableTokenCounts**](TokenCounts.md) |  | [optional]
**TraceId** | **string** |  |

## Methods

### NewNativeIngestRequest

`func NewNativeIngestRequest(attributes map[string]interface{}, kind string, name string, redactionClass RedactionClass, scope TenantScope, seq int64, spanId string, status SpanStatus, traceId string, ) *NativeIngestRequest`

NewNativeIngestRequest instantiates a new NativeIngestRequest object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewNativeIngestRequestWithDefaults

`func NewNativeIngestRequestWithDefaults() *NativeIngestRequest`

NewNativeIngestRequestWithDefaults instantiates a new NativeIngestRequest object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAttributes

`func (o *NativeIngestRequest) GetAttributes() map[string]interface{}`

GetAttributes returns the Attributes field if non-nil, zero value otherwise.

### GetAttributesOk

`func (o *NativeIngestRequest) GetAttributesOk() (*map[string]interface{}, bool)`

GetAttributesOk returns a tuple with the Attributes field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAttributes

`func (o *NativeIngestRequest) SetAttributes(v map[string]interface{})`

SetAttributes sets Attributes field to given value.


### GetAuthContext

`func (o *NativeIngestRequest) GetAuthContext() AuthContext`

GetAuthContext returns the AuthContext field if non-nil, zero value otherwise.

### GetAuthContextOk

`func (o *NativeIngestRequest) GetAuthContextOk() (*AuthContext, bool)`

GetAuthContextOk returns a tuple with the AuthContext field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAuthContext

`func (o *NativeIngestRequest) SetAuthContext(v AuthContext)`

SetAuthContext sets AuthContext field to given value.

### HasAuthContext

`func (o *NativeIngestRequest) HasAuthContext() bool`

HasAuthContext returns a boolean if a field has been set.

### SetAuthContextNil

`func (o *NativeIngestRequest) SetAuthContextNil(b bool)`

 SetAuthContextNil sets the value for AuthContext to be an explicit nil

### UnsetAuthContext
`func (o *NativeIngestRequest) UnsetAuthContext()`

UnsetAuthContext ensures that no value is present for AuthContext, not even an explicit nil
### GetCost

`func (o *NativeIngestRequest) GetCost() Money`

GetCost returns the Cost field if non-nil, zero value otherwise.

### GetCostOk

`func (o *NativeIngestRequest) GetCostOk() (*Money, bool)`

GetCostOk returns a tuple with the Cost field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCost

`func (o *NativeIngestRequest) SetCost(v Money)`

SetCost sets Cost field to given value.

### HasCost

`func (o *NativeIngestRequest) HasCost() bool`

HasCost returns a boolean if a field has been set.

### SetCostNil

`func (o *NativeIngestRequest) SetCostNil(b bool)`

 SetCostNil sets the value for Cost to be an explicit nil

### UnsetCost
`func (o *NativeIngestRequest) UnsetCost()`

UnsetCost ensures that no value is present for Cost, not even an explicit nil
### GetEndTime

`func (o *NativeIngestRequest) GetEndTime() time.Time`

GetEndTime returns the EndTime field if non-nil, zero value otherwise.

### GetEndTimeOk

`func (o *NativeIngestRequest) GetEndTimeOk() (*time.Time, bool)`

GetEndTimeOk returns a tuple with the EndTime field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEndTime

`func (o *NativeIngestRequest) SetEndTime(v time.Time)`

SetEndTime sets EndTime field to given value.

### HasEndTime

`func (o *NativeIngestRequest) HasEndTime() bool`

HasEndTime returns a boolean if a field has been set.

### SetEndTimeNil

`func (o *NativeIngestRequest) SetEndTimeNil(b bool)`

 SetEndTimeNil sets the value for EndTime to be an explicit nil

### UnsetEndTime
`func (o *NativeIngestRequest) UnsetEndTime()`

UnsetEndTime ensures that no value is present for EndTime, not even an explicit nil
### GetIdempotencyKey

`func (o *NativeIngestRequest) GetIdempotencyKey() string`

GetIdempotencyKey returns the IdempotencyKey field if non-nil, zero value otherwise.

### GetIdempotencyKeyOk

`func (o *NativeIngestRequest) GetIdempotencyKeyOk() (*string, bool)`

GetIdempotencyKeyOk returns a tuple with the IdempotencyKey field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetIdempotencyKey

`func (o *NativeIngestRequest) SetIdempotencyKey(v string)`

SetIdempotencyKey sets IdempotencyKey field to given value.

### HasIdempotencyKey

`func (o *NativeIngestRequest) HasIdempotencyKey() bool`

HasIdempotencyKey returns a boolean if a field has been set.

### GetInput

`func (o *NativeIngestRequest) GetInput() interface{}`

GetInput returns the Input field if non-nil, zero value otherwise.

### GetInputOk

`func (o *NativeIngestRequest) GetInputOk() (*interface{}, bool)`

GetInputOk returns a tuple with the Input field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetInput

`func (o *NativeIngestRequest) SetInput(v interface{})`

SetInput sets Input field to given value.

### HasInput

`func (o *NativeIngestRequest) HasInput() bool`

HasInput returns a boolean if a field has been set.

### SetInputNil

`func (o *NativeIngestRequest) SetInputNil(b bool)`

 SetInputNil sets the value for Input to be an explicit nil

### UnsetInput
`func (o *NativeIngestRequest) UnsetInput()`

UnsetInput ensures that no value is present for Input, not even an explicit nil
### GetKind

`func (o *NativeIngestRequest) GetKind() string`

GetKind returns the Kind field if non-nil, zero value otherwise.

### GetKindOk

`func (o *NativeIngestRequest) GetKindOk() (*string, bool)`

GetKindOk returns a tuple with the Kind field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetKind

`func (o *NativeIngestRequest) SetKind(v string)`

SetKind sets Kind field to given value.


### GetModel

`func (o *NativeIngestRequest) GetModel() ModelRef`

GetModel returns the Model field if non-nil, zero value otherwise.

### GetModelOk

`func (o *NativeIngestRequest) GetModelOk() (*ModelRef, bool)`

GetModelOk returns a tuple with the Model field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetModel

`func (o *NativeIngestRequest) SetModel(v ModelRef)`

SetModel sets Model field to given value.

### HasModel

`func (o *NativeIngestRequest) HasModel() bool`

HasModel returns a boolean if a field has been set.

### SetModelNil

`func (o *NativeIngestRequest) SetModelNil(b bool)`

 SetModelNil sets the value for Model to be an explicit nil

### UnsetModel
`func (o *NativeIngestRequest) UnsetModel()`

UnsetModel ensures that no value is present for Model, not even an explicit nil
### GetName

`func (o *NativeIngestRequest) GetName() string`

GetName returns the Name field if non-nil, zero value otherwise.

### GetNameOk

`func (o *NativeIngestRequest) GetNameOk() (*string, bool)`

GetNameOk returns a tuple with the Name field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetName

`func (o *NativeIngestRequest) SetName(v string)`

SetName sets Name field to given value.


### GetOutput

`func (o *NativeIngestRequest) GetOutput() interface{}`

GetOutput returns the Output field if non-nil, zero value otherwise.

### GetOutputOk

`func (o *NativeIngestRequest) GetOutputOk() (*interface{}, bool)`

GetOutputOk returns a tuple with the Output field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetOutput

`func (o *NativeIngestRequest) SetOutput(v interface{})`

SetOutput sets Output field to given value.

### HasOutput

`func (o *NativeIngestRequest) HasOutput() bool`

HasOutput returns a boolean if a field has been set.

### SetOutputNil

`func (o *NativeIngestRequest) SetOutputNil(b bool)`

 SetOutputNil sets the value for Output to be an explicit nil

### UnsetOutput
`func (o *NativeIngestRequest) UnsetOutput()`

UnsetOutput ensures that no value is present for Output, not even an explicit nil
### GetParentSpanId

`func (o *NativeIngestRequest) GetParentSpanId() string`

GetParentSpanId returns the ParentSpanId field if non-nil, zero value otherwise.

### GetParentSpanIdOk

`func (o *NativeIngestRequest) GetParentSpanIdOk() (*string, bool)`

GetParentSpanIdOk returns a tuple with the ParentSpanId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetParentSpanId

`func (o *NativeIngestRequest) SetParentSpanId(v string)`

SetParentSpanId sets ParentSpanId field to given value.

### HasParentSpanId

`func (o *NativeIngestRequest) HasParentSpanId() bool`

HasParentSpanId returns a boolean if a field has been set.

### GetRedactionClass

`func (o *NativeIngestRequest) GetRedactionClass() RedactionClass`

GetRedactionClass returns the RedactionClass field if non-nil, zero value otherwise.

### GetRedactionClassOk

`func (o *NativeIngestRequest) GetRedactionClassOk() (*RedactionClass, bool)`

GetRedactionClassOk returns a tuple with the RedactionClass field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRedactionClass

`func (o *NativeIngestRequest) SetRedactionClass(v RedactionClass)`

SetRedactionClass sets RedactionClass field to given value.


### GetScope

`func (o *NativeIngestRequest) GetScope() TenantScope`

GetScope returns the Scope field if non-nil, zero value otherwise.

### GetScopeOk

`func (o *NativeIngestRequest) GetScopeOk() (*TenantScope, bool)`

GetScopeOk returns a tuple with the Scope field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetScope

`func (o *NativeIngestRequest) SetScope(v TenantScope)`

SetScope sets Scope field to given value.


### GetSeq

`func (o *NativeIngestRequest) GetSeq() int64`

GetSeq returns the Seq field if non-nil, zero value otherwise.

### GetSeqOk

`func (o *NativeIngestRequest) GetSeqOk() (*int64, bool)`

GetSeqOk returns a tuple with the Seq field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSeq

`func (o *NativeIngestRequest) SetSeq(v int64)`

SetSeq sets Seq field to given value.


### GetSpanId

`func (o *NativeIngestRequest) GetSpanId() string`

GetSpanId returns the SpanId field if non-nil, zero value otherwise.

### GetSpanIdOk

`func (o *NativeIngestRequest) GetSpanIdOk() (*string, bool)`

GetSpanIdOk returns a tuple with the SpanId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSpanId

`func (o *NativeIngestRequest) SetSpanId(v string)`

SetSpanId sets SpanId field to given value.


### GetStartTime

`func (o *NativeIngestRequest) GetStartTime() time.Time`

GetStartTime returns the StartTime field if non-nil, zero value otherwise.

### GetStartTimeOk

`func (o *NativeIngestRequest) GetStartTimeOk() (*time.Time, bool)`

GetStartTimeOk returns a tuple with the StartTime field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetStartTime

`func (o *NativeIngestRequest) SetStartTime(v time.Time)`

SetStartTime sets StartTime field to given value.

### HasStartTime

`func (o *NativeIngestRequest) HasStartTime() bool`

HasStartTime returns a boolean if a field has been set.

### SetStartTimeNil

`func (o *NativeIngestRequest) SetStartTimeNil(b bool)`

 SetStartTimeNil sets the value for StartTime to be an explicit nil

### UnsetStartTime
`func (o *NativeIngestRequest) UnsetStartTime()`

UnsetStartTime ensures that no value is present for StartTime, not even an explicit nil
### GetStatus

`func (o *NativeIngestRequest) GetStatus() SpanStatus`

GetStatus returns the Status field if non-nil, zero value otherwise.

### GetStatusOk

`func (o *NativeIngestRequest) GetStatusOk() (*SpanStatus, bool)`

GetStatusOk returns a tuple with the Status field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetStatus

`func (o *NativeIngestRequest) SetStatus(v SpanStatus)`

SetStatus sets Status field to given value.


### GetTokens

`func (o *NativeIngestRequest) GetTokens() TokenCounts`

GetTokens returns the Tokens field if non-nil, zero value otherwise.

### GetTokensOk

`func (o *NativeIngestRequest) GetTokensOk() (*TokenCounts, bool)`

GetTokensOk returns a tuple with the Tokens field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTokens

`func (o *NativeIngestRequest) SetTokens(v TokenCounts)`

SetTokens sets Tokens field to given value.

### HasTokens

`func (o *NativeIngestRequest) HasTokens() bool`

HasTokens returns a boolean if a field has been set.

### SetTokensNil

`func (o *NativeIngestRequest) SetTokensNil(b bool)`

 SetTokensNil sets the value for Tokens to be an explicit nil

### UnsetTokens
`func (o *NativeIngestRequest) UnsetTokens()`

UnsetTokens ensures that no value is present for Tokens, not even an explicit nil
### GetTraceId

`func (o *NativeIngestRequest) GetTraceId() string`

GetTraceId returns the TraceId field if non-nil, zero value otherwise.

### GetTraceIdOk

`func (o *NativeIngestRequest) GetTraceIdOk() (*string, bool)`

GetTraceIdOk returns a tuple with the TraceId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTraceId

`func (o *NativeIngestRequest) SetTraceId(v string)`

SetTraceId sets TraceId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
