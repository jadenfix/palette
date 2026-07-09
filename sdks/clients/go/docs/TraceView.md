# TraceView

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Spans** | [**[]CanonicalSpan**](CanonicalSpan.md) |  |
**TenantId** | **string** |  |
**TraceId** | **string** |  |

## Methods

### NewTraceView

`func NewTraceView(spans []CanonicalSpan, tenantId string, traceId string, ) *TraceView`

NewTraceView instantiates a new TraceView object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewTraceViewWithDefaults

`func NewTraceViewWithDefaults() *TraceView`

NewTraceViewWithDefaults instantiates a new TraceView object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetSpans

`func (o *TraceView) GetSpans() []CanonicalSpan`

GetSpans returns the Spans field if non-nil, zero value otherwise.

### GetSpansOk

`func (o *TraceView) GetSpansOk() (*[]CanonicalSpan, bool)`

GetSpansOk returns a tuple with the Spans field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSpans

`func (o *TraceView) SetSpans(v []CanonicalSpan)`

SetSpans sets Spans field to given value.


### GetTenantId

`func (o *TraceView) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *TraceView) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *TraceView) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.


### GetTraceId

`func (o *TraceView) GetTraceId() string`

GetTraceId returns the TraceId field if non-nil, zero value otherwise.

### GetTraceIdOk

`func (o *TraceView) GetTraceIdOk() (*string, bool)`

GetTraceIdOk returns a tuple with the TraceId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTraceId

`func (o *TraceView) SetTraceId(v string)`

SetTraceId sets TraceId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
