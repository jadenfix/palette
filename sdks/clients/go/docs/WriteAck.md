# WriteAck

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AcceptedRaw** | **int32** |  |
**AcceptedSpans** | **int32** |  |
**DuplicateRaw** | **int32** |  |
**DuplicateSpans** | **int32** |  |

## Methods

### NewWriteAck

`func NewWriteAck(acceptedRaw int32, acceptedSpans int32, duplicateRaw int32, duplicateSpans int32, ) *WriteAck`

NewWriteAck instantiates a new WriteAck object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewWriteAckWithDefaults

`func NewWriteAckWithDefaults() *WriteAck`

NewWriteAckWithDefaults instantiates a new WriteAck object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAcceptedRaw

`func (o *WriteAck) GetAcceptedRaw() int32`

GetAcceptedRaw returns the AcceptedRaw field if non-nil, zero value otherwise.

### GetAcceptedRawOk

`func (o *WriteAck) GetAcceptedRawOk() (*int32, bool)`

GetAcceptedRawOk returns a tuple with the AcceptedRaw field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAcceptedRaw

`func (o *WriteAck) SetAcceptedRaw(v int32)`

SetAcceptedRaw sets AcceptedRaw field to given value.


### GetAcceptedSpans

`func (o *WriteAck) GetAcceptedSpans() int32`

GetAcceptedSpans returns the AcceptedSpans field if non-nil, zero value otherwise.

### GetAcceptedSpansOk

`func (o *WriteAck) GetAcceptedSpansOk() (*int32, bool)`

GetAcceptedSpansOk returns a tuple with the AcceptedSpans field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAcceptedSpans

`func (o *WriteAck) SetAcceptedSpans(v int32)`

SetAcceptedSpans sets AcceptedSpans field to given value.


### GetDuplicateRaw

`func (o *WriteAck) GetDuplicateRaw() int32`

GetDuplicateRaw returns the DuplicateRaw field if non-nil, zero value otherwise.

### GetDuplicateRawOk

`func (o *WriteAck) GetDuplicateRawOk() (*int32, bool)`

GetDuplicateRawOk returns a tuple with the DuplicateRaw field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDuplicateRaw

`func (o *WriteAck) SetDuplicateRaw(v int32)`

SetDuplicateRaw sets DuplicateRaw field to given value.


### GetDuplicateSpans

`func (o *WriteAck) GetDuplicateSpans() int32`

GetDuplicateSpans returns the DuplicateSpans field if non-nil, zero value otherwise.

### GetDuplicateSpansOk

`func (o *WriteAck) GetDuplicateSpansOk() (*int32, bool)`

GetDuplicateSpansOk returns a tuple with the DuplicateSpans field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDuplicateSpans

`func (o *WriteAck) SetDuplicateSpans(v int32)`

SetDuplicateSpans sets DuplicateSpans field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
