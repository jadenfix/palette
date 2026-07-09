# PublishAck

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Accepted** | **bool** |  |
**Duplicate** | **bool** |  |

## Methods

### NewPublishAck

`func NewPublishAck(accepted bool, duplicate bool, ) *PublishAck`

NewPublishAck instantiates a new PublishAck object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewPublishAckWithDefaults

`func NewPublishAckWithDefaults() *PublishAck`

NewPublishAckWithDefaults instantiates a new PublishAck object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAccepted

`func (o *PublishAck) GetAccepted() bool`

GetAccepted returns the Accepted field if non-nil, zero value otherwise.

### GetAcceptedOk

`func (o *PublishAck) GetAcceptedOk() (*bool, bool)`

GetAcceptedOk returns a tuple with the Accepted field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAccepted

`func (o *PublishAck) SetAccepted(v bool)`

SetAccepted sets Accepted field to given value.


### GetDuplicate

`func (o *PublishAck) GetDuplicate() bool`

GetDuplicate returns the Duplicate field if non-nil, zero value otherwise.

### GetDuplicateOk

`func (o *PublishAck) GetDuplicateOk() (*bool, bool)`

GetDuplicateOk returns a tuple with the Duplicate field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDuplicate

`func (o *PublishAck) SetDuplicate(v bool)`

SetDuplicate sets Duplicate field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
