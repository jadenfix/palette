# DeadLetterReplayReport

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Ack** | [**PublishAck**](PublishAck.md) |  |
**MessageId** | **string** |  |
**ProjectId** | **string** |  |
**ResetAttempts** | **bool** |  |
**TenantId** | **string** |  |

## Methods

### NewDeadLetterReplayReport

`func NewDeadLetterReplayReport(ack PublishAck, messageId string, projectId string, resetAttempts bool, tenantId string, ) *DeadLetterReplayReport`

NewDeadLetterReplayReport instantiates a new DeadLetterReplayReport object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewDeadLetterReplayReportWithDefaults

`func NewDeadLetterReplayReportWithDefaults() *DeadLetterReplayReport`

NewDeadLetterReplayReportWithDefaults instantiates a new DeadLetterReplayReport object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAck

`func (o *DeadLetterReplayReport) GetAck() PublishAck`

GetAck returns the Ack field if non-nil, zero value otherwise.

### GetAckOk

`func (o *DeadLetterReplayReport) GetAckOk() (*PublishAck, bool)`

GetAckOk returns a tuple with the Ack field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAck

`func (o *DeadLetterReplayReport) SetAck(v PublishAck)`

SetAck sets Ack field to given value.


### GetMessageId

`func (o *DeadLetterReplayReport) GetMessageId() string`

GetMessageId returns the MessageId field if non-nil, zero value otherwise.

### GetMessageIdOk

`func (o *DeadLetterReplayReport) GetMessageIdOk() (*string, bool)`

GetMessageIdOk returns a tuple with the MessageId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMessageId

`func (o *DeadLetterReplayReport) SetMessageId(v string)`

SetMessageId sets MessageId field to given value.


### GetProjectId

`func (o *DeadLetterReplayReport) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *DeadLetterReplayReport) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *DeadLetterReplayReport) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetResetAttempts

`func (o *DeadLetterReplayReport) GetResetAttempts() bool`

GetResetAttempts returns the ResetAttempts field if non-nil, zero value otherwise.

### GetResetAttemptsOk

`func (o *DeadLetterReplayReport) GetResetAttemptsOk() (*bool, bool)`

GetResetAttemptsOk returns a tuple with the ResetAttempts field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetResetAttempts

`func (o *DeadLetterReplayReport) SetResetAttempts(v bool)`

SetResetAttempts sets ResetAttempts field to given value.


### GetTenantId

`func (o *DeadLetterReplayReport) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *DeadLetterReplayReport) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *DeadLetterReplayReport) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
