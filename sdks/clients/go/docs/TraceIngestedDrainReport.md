# TraceIngestedDrainReport

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Completed** | **int32** |  |
**Consumed** | **int32** |  |
**DeadLettered** | **int32** |  |
**FailedWork** | **int32** |  |
**InvalidMessages** | **int32** |  |
**Retried** | **int32** |  |
**TraceRefs** | [**[]QueuedTraceWork**](QueuedTraceWork.md) |  |

## Methods

### NewTraceIngestedDrainReport

`func NewTraceIngestedDrainReport(completed int32, consumed int32, deadLettered int32, failedWork int32, invalidMessages int32, retried int32, traceRefs []QueuedTraceWork, ) *TraceIngestedDrainReport`

NewTraceIngestedDrainReport instantiates a new TraceIngestedDrainReport object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewTraceIngestedDrainReportWithDefaults

`func NewTraceIngestedDrainReportWithDefaults() *TraceIngestedDrainReport`

NewTraceIngestedDrainReportWithDefaults instantiates a new TraceIngestedDrainReport object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetCompleted

`func (o *TraceIngestedDrainReport) GetCompleted() int32`

GetCompleted returns the Completed field if non-nil, zero value otherwise.

### GetCompletedOk

`func (o *TraceIngestedDrainReport) GetCompletedOk() (*int32, bool)`

GetCompletedOk returns a tuple with the Completed field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCompleted

`func (o *TraceIngestedDrainReport) SetCompleted(v int32)`

SetCompleted sets Completed field to given value.


### GetConsumed

`func (o *TraceIngestedDrainReport) GetConsumed() int32`

GetConsumed returns the Consumed field if non-nil, zero value otherwise.

### GetConsumedOk

`func (o *TraceIngestedDrainReport) GetConsumedOk() (*int32, bool)`

GetConsumedOk returns a tuple with the Consumed field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetConsumed

`func (o *TraceIngestedDrainReport) SetConsumed(v int32)`

SetConsumed sets Consumed field to given value.


### GetDeadLettered

`func (o *TraceIngestedDrainReport) GetDeadLettered() int32`

GetDeadLettered returns the DeadLettered field if non-nil, zero value otherwise.

### GetDeadLetteredOk

`func (o *TraceIngestedDrainReport) GetDeadLetteredOk() (*int32, bool)`

GetDeadLetteredOk returns a tuple with the DeadLettered field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDeadLettered

`func (o *TraceIngestedDrainReport) SetDeadLettered(v int32)`

SetDeadLettered sets DeadLettered field to given value.


### GetFailedWork

`func (o *TraceIngestedDrainReport) GetFailedWork() int32`

GetFailedWork returns the FailedWork field if non-nil, zero value otherwise.

### GetFailedWorkOk

`func (o *TraceIngestedDrainReport) GetFailedWorkOk() (*int32, bool)`

GetFailedWorkOk returns a tuple with the FailedWork field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetFailedWork

`func (o *TraceIngestedDrainReport) SetFailedWork(v int32)`

SetFailedWork sets FailedWork field to given value.


### GetInvalidMessages

`func (o *TraceIngestedDrainReport) GetInvalidMessages() int32`

GetInvalidMessages returns the InvalidMessages field if non-nil, zero value otherwise.

### GetInvalidMessagesOk

`func (o *TraceIngestedDrainReport) GetInvalidMessagesOk() (*int32, bool)`

GetInvalidMessagesOk returns a tuple with the InvalidMessages field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetInvalidMessages

`func (o *TraceIngestedDrainReport) SetInvalidMessages(v int32)`

SetInvalidMessages sets InvalidMessages field to given value.


### GetRetried

`func (o *TraceIngestedDrainReport) GetRetried() int32`

GetRetried returns the Retried field if non-nil, zero value otherwise.

### GetRetriedOk

`func (o *TraceIngestedDrainReport) GetRetriedOk() (*int32, bool)`

GetRetriedOk returns a tuple with the Retried field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRetried

`func (o *TraceIngestedDrainReport) SetRetried(v int32)`

SetRetried sets Retried field to given value.


### GetTraceRefs

`func (o *TraceIngestedDrainReport) GetTraceRefs() []QueuedTraceWork`

GetTraceRefs returns the TraceRefs field if non-nil, zero value otherwise.

### GetTraceRefsOk

`func (o *TraceIngestedDrainReport) GetTraceRefsOk() (*[]QueuedTraceWork, bool)`

GetTraceRefsOk returns a tuple with the TraceRefs field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTraceRefs

`func (o *TraceIngestedDrainReport) SetTraceRefs(v []QueuedTraceWork)`

SetTraceRefs sets TraceRefs field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
