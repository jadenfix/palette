# BusMessage

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Attempts** | **int32** |  |
**EnqueuedAt** | **time.Time** |  |
**IdempotencyKey** | **string** |  |
**Kind** | **string** |  |
**MaxAttempts** | **int32** |  |
**MessageId** | **string** |  |
**Payload** | **[]int32** |  |
**ProjectId** | **string** |  |
**TenantId** | **string** |  |

## Methods

### NewBusMessage

`func NewBusMessage(attempts int32, enqueuedAt time.Time, idempotencyKey string, kind string, maxAttempts int32, messageId string, payload []int32, projectId string, tenantId string, ) *BusMessage`

NewBusMessage instantiates a new BusMessage object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewBusMessageWithDefaults

`func NewBusMessageWithDefaults() *BusMessage`

NewBusMessageWithDefaults instantiates a new BusMessage object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAttempts

`func (o *BusMessage) GetAttempts() int32`

GetAttempts returns the Attempts field if non-nil, zero value otherwise.

### GetAttemptsOk

`func (o *BusMessage) GetAttemptsOk() (*int32, bool)`

GetAttemptsOk returns a tuple with the Attempts field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAttempts

`func (o *BusMessage) SetAttempts(v int32)`

SetAttempts sets Attempts field to given value.


### GetEnqueuedAt

`func (o *BusMessage) GetEnqueuedAt() time.Time`

GetEnqueuedAt returns the EnqueuedAt field if non-nil, zero value otherwise.

### GetEnqueuedAtOk

`func (o *BusMessage) GetEnqueuedAtOk() (*time.Time, bool)`

GetEnqueuedAtOk returns a tuple with the EnqueuedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEnqueuedAt

`func (o *BusMessage) SetEnqueuedAt(v time.Time)`

SetEnqueuedAt sets EnqueuedAt field to given value.


### GetIdempotencyKey

`func (o *BusMessage) GetIdempotencyKey() string`

GetIdempotencyKey returns the IdempotencyKey field if non-nil, zero value otherwise.

### GetIdempotencyKeyOk

`func (o *BusMessage) GetIdempotencyKeyOk() (*string, bool)`

GetIdempotencyKeyOk returns a tuple with the IdempotencyKey field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetIdempotencyKey

`func (o *BusMessage) SetIdempotencyKey(v string)`

SetIdempotencyKey sets IdempotencyKey field to given value.


### GetKind

`func (o *BusMessage) GetKind() string`

GetKind returns the Kind field if non-nil, zero value otherwise.

### GetKindOk

`func (o *BusMessage) GetKindOk() (*string, bool)`

GetKindOk returns a tuple with the Kind field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetKind

`func (o *BusMessage) SetKind(v string)`

SetKind sets Kind field to given value.


### GetMaxAttempts

`func (o *BusMessage) GetMaxAttempts() int32`

GetMaxAttempts returns the MaxAttempts field if non-nil, zero value otherwise.

### GetMaxAttemptsOk

`func (o *BusMessage) GetMaxAttemptsOk() (*int32, bool)`

GetMaxAttemptsOk returns a tuple with the MaxAttempts field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMaxAttempts

`func (o *BusMessage) SetMaxAttempts(v int32)`

SetMaxAttempts sets MaxAttempts field to given value.


### GetMessageId

`func (o *BusMessage) GetMessageId() string`

GetMessageId returns the MessageId field if non-nil, zero value otherwise.

### GetMessageIdOk

`func (o *BusMessage) GetMessageIdOk() (*string, bool)`

GetMessageIdOk returns a tuple with the MessageId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMessageId

`func (o *BusMessage) SetMessageId(v string)`

SetMessageId sets MessageId field to given value.


### GetPayload

`func (o *BusMessage) GetPayload() []int32`

GetPayload returns the Payload field if non-nil, zero value otherwise.

### GetPayloadOk

`func (o *BusMessage) GetPayloadOk() (*[]int32, bool)`

GetPayloadOk returns a tuple with the Payload field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPayload

`func (o *BusMessage) SetPayload(v []int32)`

SetPayload sets Payload field to given value.


### GetProjectId

`func (o *BusMessage) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *BusMessage) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *BusMessage) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetTenantId

`func (o *BusMessage) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *BusMessage) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *BusMessage) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
