# AlertDecision

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Delivery** | Pointer to [**NullableWebhookDelivery**](WebhookDelivery.md) |  | [optional]
**Emitted** | **bool** |  |
**SuppressedReason** | Pointer to **NullableString** |  | [optional]

## Methods

### NewAlertDecision

`func NewAlertDecision(emitted bool, ) *AlertDecision`

NewAlertDecision instantiates a new AlertDecision object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewAlertDecisionWithDefaults

`func NewAlertDecisionWithDefaults() *AlertDecision`

NewAlertDecisionWithDefaults instantiates a new AlertDecision object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetDelivery

`func (o *AlertDecision) GetDelivery() WebhookDelivery`

GetDelivery returns the Delivery field if non-nil, zero value otherwise.

### GetDeliveryOk

`func (o *AlertDecision) GetDeliveryOk() (*WebhookDelivery, bool)`

GetDeliveryOk returns a tuple with the Delivery field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDelivery

`func (o *AlertDecision) SetDelivery(v WebhookDelivery)`

SetDelivery sets Delivery field to given value.

### HasDelivery

`func (o *AlertDecision) HasDelivery() bool`

HasDelivery returns a boolean if a field has been set.

### SetDeliveryNil

`func (o *AlertDecision) SetDeliveryNil(b bool)`

 SetDeliveryNil sets the value for Delivery to be an explicit nil

### UnsetDelivery
`func (o *AlertDecision) UnsetDelivery()`

UnsetDelivery ensures that no value is present for Delivery, not even an explicit nil
### GetEmitted

`func (o *AlertDecision) GetEmitted() bool`

GetEmitted returns the Emitted field if non-nil, zero value otherwise.

### GetEmittedOk

`func (o *AlertDecision) GetEmittedOk() (*bool, bool)`

GetEmittedOk returns a tuple with the Emitted field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEmitted

`func (o *AlertDecision) SetEmitted(v bool)`

SetEmitted sets Emitted field to given value.


### GetSuppressedReason

`func (o *AlertDecision) GetSuppressedReason() string`

GetSuppressedReason returns the SuppressedReason field if non-nil, zero value otherwise.

### GetSuppressedReasonOk

`func (o *AlertDecision) GetSuppressedReasonOk() (*string, bool)`

GetSuppressedReasonOk returns a tuple with the SuppressedReason field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSuppressedReason

`func (o *AlertDecision) SetSuppressedReason(v string)`

SetSuppressedReason sets SuppressedReason field to given value.

### HasSuppressedReason

`func (o *AlertDecision) HasSuppressedReason() bool`

HasSuppressedReason returns a boolean if a field has been set.

### SetSuppressedReasonNil

`func (o *AlertDecision) SetSuppressedReasonNil(b bool)`

 SetSuppressedReasonNil sets the value for SuppressedReason to be an explicit nil

### UnsetSuppressedReason
`func (o *AlertDecision) UnsetSuppressedReason()`

UnsetSuppressedReason ensures that no value is present for SuppressedReason, not even an explicit nil

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
