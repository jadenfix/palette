# SpanIoResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Input** | [**SpanIoValue**](SpanIoValue.md) |  |
**Output** | [**SpanIoValue**](SpanIoValue.md) |  |
**SpanId** | **string** |  |
**TenantId** | **string** |  |
**TraceId** | **string** |  |

## Methods

### NewSpanIoResponse

`func NewSpanIoResponse(input SpanIoValue, output SpanIoValue, spanId string, tenantId string, traceId string, ) *SpanIoResponse`

NewSpanIoResponse instantiates a new SpanIoResponse object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewSpanIoResponseWithDefaults

`func NewSpanIoResponseWithDefaults() *SpanIoResponse`

NewSpanIoResponseWithDefaults instantiates a new SpanIoResponse object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetInput

`func (o *SpanIoResponse) GetInput() SpanIoValue`

GetInput returns the Input field if non-nil, zero value otherwise.

### GetInputOk

`func (o *SpanIoResponse) GetInputOk() (*SpanIoValue, bool)`

GetInputOk returns a tuple with the Input field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetInput

`func (o *SpanIoResponse) SetInput(v SpanIoValue)`

SetInput sets Input field to given value.


### GetOutput

`func (o *SpanIoResponse) GetOutput() SpanIoValue`

GetOutput returns the Output field if non-nil, zero value otherwise.

### GetOutputOk

`func (o *SpanIoResponse) GetOutputOk() (*SpanIoValue, bool)`

GetOutputOk returns a tuple with the Output field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetOutput

`func (o *SpanIoResponse) SetOutput(v SpanIoValue)`

SetOutput sets Output field to given value.


### GetSpanId

`func (o *SpanIoResponse) GetSpanId() string`

GetSpanId returns the SpanId field if non-nil, zero value otherwise.

### GetSpanIdOk

`func (o *SpanIoResponse) GetSpanIdOk() (*string, bool)`

GetSpanIdOk returns a tuple with the SpanId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSpanId

`func (o *SpanIoResponse) SetSpanId(v string)`

SetSpanId sets SpanId field to given value.


### GetTenantId

`func (o *SpanIoResponse) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *SpanIoResponse) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *SpanIoResponse) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.


### GetTraceId

`func (o *SpanIoResponse) GetTraceId() string`

GetTraceId returns the TraceId field if non-nil, zero value otherwise.

### GetTraceIdOk

`func (o *SpanIoResponse) GetTraceIdOk() (*string, bool)`

GetTraceIdOk returns a tuple with the TraceId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTraceId

`func (o *SpanIoResponse) SetTraceId(v string)`

SetTraceId sets TraceId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
