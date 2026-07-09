# IngestOutcome

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Ack** | [**WriteAck**](WriteAck.md) |  |
**DownstreamQueued** | **bool** |  |

## Methods

### NewIngestOutcome

`func NewIngestOutcome(ack WriteAck, downstreamQueued bool, ) *IngestOutcome`

NewIngestOutcome instantiates a new IngestOutcome object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewIngestOutcomeWithDefaults

`func NewIngestOutcomeWithDefaults() *IngestOutcome`

NewIngestOutcomeWithDefaults instantiates a new IngestOutcome object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAck

`func (o *IngestOutcome) GetAck() WriteAck`

GetAck returns the Ack field if non-nil, zero value otherwise.

### GetAckOk

`func (o *IngestOutcome) GetAckOk() (*WriteAck, bool)`

GetAckOk returns a tuple with the Ack field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAck

`func (o *IngestOutcome) SetAck(v WriteAck)`

SetAck sets Ack field to given value.


### GetDownstreamQueued

`func (o *IngestOutcome) GetDownstreamQueued() bool`

GetDownstreamQueued returns the DownstreamQueued field if non-nil, zero value otherwise.

### GetDownstreamQueuedOk

`func (o *IngestOutcome) GetDownstreamQueuedOk() (*bool, bool)`

GetDownstreamQueuedOk returns a tuple with the DownstreamQueued field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDownstreamQueued

`func (o *IngestOutcome) SetDownstreamQueued(v bool)`

SetDownstreamQueued sets DownstreamQueued field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
