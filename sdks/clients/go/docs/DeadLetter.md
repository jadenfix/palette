# DeadLetter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**FailedAt** | **time.Time** |  |
**Message** | [**BusMessage**](BusMessage.md) |  |
**Reason** | **string** |  |

## Methods

### NewDeadLetter

`func NewDeadLetter(failedAt time.Time, message BusMessage, reason string, ) *DeadLetter`

NewDeadLetter instantiates a new DeadLetter object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewDeadLetterWithDefaults

`func NewDeadLetterWithDefaults() *DeadLetter`

NewDeadLetterWithDefaults instantiates a new DeadLetter object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetFailedAt

`func (o *DeadLetter) GetFailedAt() time.Time`

GetFailedAt returns the FailedAt field if non-nil, zero value otherwise.

### GetFailedAtOk

`func (o *DeadLetter) GetFailedAtOk() (*time.Time, bool)`

GetFailedAtOk returns a tuple with the FailedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetFailedAt

`func (o *DeadLetter) SetFailedAt(v time.Time)`

SetFailedAt sets FailedAt field to given value.


### GetMessage

`func (o *DeadLetter) GetMessage() BusMessage`

GetMessage returns the Message field if non-nil, zero value otherwise.

### GetMessageOk

`func (o *DeadLetter) GetMessageOk() (*BusMessage, bool)`

GetMessageOk returns a tuple with the Message field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMessage

`func (o *DeadLetter) SetMessage(v BusMessage)`

SetMessage sets Message field to given value.


### GetReason

`func (o *DeadLetter) GetReason() string`

GetReason returns the Reason field if non-nil, zero value otherwise.

### GetReasonOk

`func (o *DeadLetter) GetReasonOk() (*string, bool)`

GetReasonOk returns a tuple with the Reason field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetReason

`func (o *DeadLetter) SetReason(v string)`

SetReason sets Reason field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
