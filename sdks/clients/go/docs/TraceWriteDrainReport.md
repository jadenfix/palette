# TraceWriteDrainReport

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Consumed** | **int32** |  |
**DeadLettered** | **int32** |  |
**DownstreamPublished** | **int32** |  |
**DuplicateRaw** | **int32** |  |
**DuplicateSpans** | **int32** |  |
**FailedDownstreamPublishes** | **int32** |  |
**FailedWrites** | **int32** |  |
**InvalidMessages** | **int32** |  |
**Retried** | **int32** |  |
**TraceIds** | **[]string** |  |
**TraceRefs** | [**[]QueuedTraceWork**](QueuedTraceWork.md) |  |
**WrittenRaw** | **int32** |  |
**WrittenSpans** | **int32** |  |

## Methods

### NewTraceWriteDrainReport

`func NewTraceWriteDrainReport(consumed int32, deadLettered int32, downstreamPublished int32, duplicateRaw int32, duplicateSpans int32, failedDownstreamPublishes int32, failedWrites int32, invalidMessages int32, retried int32, traceIds []string, traceRefs []QueuedTraceWork, writtenRaw int32, writtenSpans int32, ) *TraceWriteDrainReport`

NewTraceWriteDrainReport instantiates a new TraceWriteDrainReport object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewTraceWriteDrainReportWithDefaults

`func NewTraceWriteDrainReportWithDefaults() *TraceWriteDrainReport`

NewTraceWriteDrainReportWithDefaults instantiates a new TraceWriteDrainReport object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetConsumed

`func (o *TraceWriteDrainReport) GetConsumed() int32`

GetConsumed returns the Consumed field if non-nil, zero value otherwise.

### GetConsumedOk

`func (o *TraceWriteDrainReport) GetConsumedOk() (*int32, bool)`

GetConsumedOk returns a tuple with the Consumed field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetConsumed

`func (o *TraceWriteDrainReport) SetConsumed(v int32)`

SetConsumed sets Consumed field to given value.


### GetDeadLettered

`func (o *TraceWriteDrainReport) GetDeadLettered() int32`

GetDeadLettered returns the DeadLettered field if non-nil, zero value otherwise.

### GetDeadLetteredOk

`func (o *TraceWriteDrainReport) GetDeadLetteredOk() (*int32, bool)`

GetDeadLetteredOk returns a tuple with the DeadLettered field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDeadLettered

`func (o *TraceWriteDrainReport) SetDeadLettered(v int32)`

SetDeadLettered sets DeadLettered field to given value.


### GetDownstreamPublished

`func (o *TraceWriteDrainReport) GetDownstreamPublished() int32`

GetDownstreamPublished returns the DownstreamPublished field if non-nil, zero value otherwise.

### GetDownstreamPublishedOk

`func (o *TraceWriteDrainReport) GetDownstreamPublishedOk() (*int32, bool)`

GetDownstreamPublishedOk returns a tuple with the DownstreamPublished field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDownstreamPublished

`func (o *TraceWriteDrainReport) SetDownstreamPublished(v int32)`

SetDownstreamPublished sets DownstreamPublished field to given value.


### GetDuplicateRaw

`func (o *TraceWriteDrainReport) GetDuplicateRaw() int32`

GetDuplicateRaw returns the DuplicateRaw field if non-nil, zero value otherwise.

### GetDuplicateRawOk

`func (o *TraceWriteDrainReport) GetDuplicateRawOk() (*int32, bool)`

GetDuplicateRawOk returns a tuple with the DuplicateRaw field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDuplicateRaw

`func (o *TraceWriteDrainReport) SetDuplicateRaw(v int32)`

SetDuplicateRaw sets DuplicateRaw field to given value.


### GetDuplicateSpans

`func (o *TraceWriteDrainReport) GetDuplicateSpans() int32`

GetDuplicateSpans returns the DuplicateSpans field if non-nil, zero value otherwise.

### GetDuplicateSpansOk

`func (o *TraceWriteDrainReport) GetDuplicateSpansOk() (*int32, bool)`

GetDuplicateSpansOk returns a tuple with the DuplicateSpans field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDuplicateSpans

`func (o *TraceWriteDrainReport) SetDuplicateSpans(v int32)`

SetDuplicateSpans sets DuplicateSpans field to given value.


### GetFailedDownstreamPublishes

`func (o *TraceWriteDrainReport) GetFailedDownstreamPublishes() int32`

GetFailedDownstreamPublishes returns the FailedDownstreamPublishes field if non-nil, zero value otherwise.

### GetFailedDownstreamPublishesOk

`func (o *TraceWriteDrainReport) GetFailedDownstreamPublishesOk() (*int32, bool)`

GetFailedDownstreamPublishesOk returns a tuple with the FailedDownstreamPublishes field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetFailedDownstreamPublishes

`func (o *TraceWriteDrainReport) SetFailedDownstreamPublishes(v int32)`

SetFailedDownstreamPublishes sets FailedDownstreamPublishes field to given value.


### GetFailedWrites

`func (o *TraceWriteDrainReport) GetFailedWrites() int32`

GetFailedWrites returns the FailedWrites field if non-nil, zero value otherwise.

### GetFailedWritesOk

`func (o *TraceWriteDrainReport) GetFailedWritesOk() (*int32, bool)`

GetFailedWritesOk returns a tuple with the FailedWrites field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetFailedWrites

`func (o *TraceWriteDrainReport) SetFailedWrites(v int32)`

SetFailedWrites sets FailedWrites field to given value.


### GetInvalidMessages

`func (o *TraceWriteDrainReport) GetInvalidMessages() int32`

GetInvalidMessages returns the InvalidMessages field if non-nil, zero value otherwise.

### GetInvalidMessagesOk

`func (o *TraceWriteDrainReport) GetInvalidMessagesOk() (*int32, bool)`

GetInvalidMessagesOk returns a tuple with the InvalidMessages field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetInvalidMessages

`func (o *TraceWriteDrainReport) SetInvalidMessages(v int32)`

SetInvalidMessages sets InvalidMessages field to given value.


### GetRetried

`func (o *TraceWriteDrainReport) GetRetried() int32`

GetRetried returns the Retried field if non-nil, zero value otherwise.

### GetRetriedOk

`func (o *TraceWriteDrainReport) GetRetriedOk() (*int32, bool)`

GetRetriedOk returns a tuple with the Retried field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRetried

`func (o *TraceWriteDrainReport) SetRetried(v int32)`

SetRetried sets Retried field to given value.


### GetTraceIds

`func (o *TraceWriteDrainReport) GetTraceIds() []string`

GetTraceIds returns the TraceIds field if non-nil, zero value otherwise.

### GetTraceIdsOk

`func (o *TraceWriteDrainReport) GetTraceIdsOk() (*[]string, bool)`

GetTraceIdsOk returns a tuple with the TraceIds field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTraceIds

`func (o *TraceWriteDrainReport) SetTraceIds(v []string)`

SetTraceIds sets TraceIds field to given value.


### GetTraceRefs

`func (o *TraceWriteDrainReport) GetTraceRefs() []QueuedTraceWork`

GetTraceRefs returns the TraceRefs field if non-nil, zero value otherwise.

### GetTraceRefsOk

`func (o *TraceWriteDrainReport) GetTraceRefsOk() (*[]QueuedTraceWork, bool)`

GetTraceRefsOk returns a tuple with the TraceRefs field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTraceRefs

`func (o *TraceWriteDrainReport) SetTraceRefs(v []QueuedTraceWork)`

SetTraceRefs sets TraceRefs field to given value.


### GetWrittenRaw

`func (o *TraceWriteDrainReport) GetWrittenRaw() int32`

GetWrittenRaw returns the WrittenRaw field if non-nil, zero value otherwise.

### GetWrittenRawOk

`func (o *TraceWriteDrainReport) GetWrittenRawOk() (*int32, bool)`

GetWrittenRawOk returns a tuple with the WrittenRaw field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetWrittenRaw

`func (o *TraceWriteDrainReport) SetWrittenRaw(v int32)`

SetWrittenRaw sets WrittenRaw field to given value.


### GetWrittenSpans

`func (o *TraceWriteDrainReport) GetWrittenSpans() int32`

GetWrittenSpans returns the WrittenSpans field if non-nil, zero value otherwise.

### GetWrittenSpansOk

`func (o *TraceWriteDrainReport) GetWrittenSpansOk() (*int32, bool)`

GetWrittenSpansOk returns a tuple with the WrittenSpans field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetWrittenSpans

`func (o *TraceWriteDrainReport) SetWrittenSpans(v int32)`

SetWrittenSpans sets WrittenSpans field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
