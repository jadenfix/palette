# PromoteTraceCaseRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Reference** | Pointer to **interface{}** |  | [optional]
**SpanId** | Pointer to **NullableString** |  | [optional]
**TraceId** | **string** |  |

## Methods

### NewPromoteTraceCaseRequest

`func NewPromoteTraceCaseRequest(traceId string, ) *PromoteTraceCaseRequest`

NewPromoteTraceCaseRequest instantiates a new PromoteTraceCaseRequest object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewPromoteTraceCaseRequestWithDefaults

`func NewPromoteTraceCaseRequestWithDefaults() *PromoteTraceCaseRequest`

NewPromoteTraceCaseRequestWithDefaults instantiates a new PromoteTraceCaseRequest object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetReference

`func (o *PromoteTraceCaseRequest) GetReference() interface{}`

GetReference returns the Reference field if non-nil, zero value otherwise.

### GetReferenceOk

`func (o *PromoteTraceCaseRequest) GetReferenceOk() (*interface{}, bool)`

GetReferenceOk returns a tuple with the Reference field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetReference

`func (o *PromoteTraceCaseRequest) SetReference(v interface{})`

SetReference sets Reference field to given value.

### HasReference

`func (o *PromoteTraceCaseRequest) HasReference() bool`

HasReference returns a boolean if a field has been set.

### SetReferenceNil

`func (o *PromoteTraceCaseRequest) SetReferenceNil(b bool)`

 SetReferenceNil sets the value for Reference to be an explicit nil

### UnsetReference
`func (o *PromoteTraceCaseRequest) UnsetReference()`

UnsetReference ensures that no value is present for Reference, not even an explicit nil
### GetSpanId

`func (o *PromoteTraceCaseRequest) GetSpanId() string`

GetSpanId returns the SpanId field if non-nil, zero value otherwise.

### GetSpanIdOk

`func (o *PromoteTraceCaseRequest) GetSpanIdOk() (*string, bool)`

GetSpanIdOk returns a tuple with the SpanId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSpanId

`func (o *PromoteTraceCaseRequest) SetSpanId(v string)`

SetSpanId sets SpanId field to given value.

### HasSpanId

`func (o *PromoteTraceCaseRequest) HasSpanId() bool`

HasSpanId returns a boolean if a field has been set.

### SetSpanIdNil

`func (o *PromoteTraceCaseRequest) SetSpanIdNil(b bool)`

 SetSpanIdNil sets the value for SpanId to be an explicit nil

### UnsetSpanId
`func (o *PromoteTraceCaseRequest) UnsetSpanId()`

UnsetSpanId ensures that no value is present for SpanId, not even an explicit nil
### GetTraceId

`func (o *PromoteTraceCaseRequest) GetTraceId() string`

GetTraceId returns the TraceId field if non-nil, zero value otherwise.

### GetTraceIdOk

`func (o *PromoteTraceCaseRequest) GetTraceIdOk() (*string, bool)`

GetTraceIdOk returns a tuple with the TraceId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTraceId

`func (o *PromoteTraceCaseRequest) SetTraceId(v string)`

SetTraceId sets TraceId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
