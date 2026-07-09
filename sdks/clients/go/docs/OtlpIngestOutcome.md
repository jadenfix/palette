# OtlpIngestOutcome

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AcceptedRaw** | **int32** |  |
**AcceptedSpans** | **int32** |  |
**DownstreamQueued** | **bool** |  |
**DuplicateRaw** | **int32** |  |
**DuplicateSpans** | **int32** |  |

## Methods

### NewOtlpIngestOutcome

`func NewOtlpIngestOutcome(acceptedRaw int32, acceptedSpans int32, downstreamQueued bool, duplicateRaw int32, duplicateSpans int32, ) *OtlpIngestOutcome`

NewOtlpIngestOutcome instantiates a new OtlpIngestOutcome object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewOtlpIngestOutcomeWithDefaults

`func NewOtlpIngestOutcomeWithDefaults() *OtlpIngestOutcome`

NewOtlpIngestOutcomeWithDefaults instantiates a new OtlpIngestOutcome object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAcceptedRaw

`func (o *OtlpIngestOutcome) GetAcceptedRaw() int32`

GetAcceptedRaw returns the AcceptedRaw field if non-nil, zero value otherwise.

### GetAcceptedRawOk

`func (o *OtlpIngestOutcome) GetAcceptedRawOk() (*int32, bool)`

GetAcceptedRawOk returns a tuple with the AcceptedRaw field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAcceptedRaw

`func (o *OtlpIngestOutcome) SetAcceptedRaw(v int32)`

SetAcceptedRaw sets AcceptedRaw field to given value.


### GetAcceptedSpans

`func (o *OtlpIngestOutcome) GetAcceptedSpans() int32`

GetAcceptedSpans returns the AcceptedSpans field if non-nil, zero value otherwise.

### GetAcceptedSpansOk

`func (o *OtlpIngestOutcome) GetAcceptedSpansOk() (*int32, bool)`

GetAcceptedSpansOk returns a tuple with the AcceptedSpans field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAcceptedSpans

`func (o *OtlpIngestOutcome) SetAcceptedSpans(v int32)`

SetAcceptedSpans sets AcceptedSpans field to given value.


### GetDownstreamQueued

`func (o *OtlpIngestOutcome) GetDownstreamQueued() bool`

GetDownstreamQueued returns the DownstreamQueued field if non-nil, zero value otherwise.

### GetDownstreamQueuedOk

`func (o *OtlpIngestOutcome) GetDownstreamQueuedOk() (*bool, bool)`

GetDownstreamQueuedOk returns a tuple with the DownstreamQueued field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDownstreamQueued

`func (o *OtlpIngestOutcome) SetDownstreamQueued(v bool)`

SetDownstreamQueued sets DownstreamQueued field to given value.


### GetDuplicateRaw

`func (o *OtlpIngestOutcome) GetDuplicateRaw() int32`

GetDuplicateRaw returns the DuplicateRaw field if non-nil, zero value otherwise.

### GetDuplicateRawOk

`func (o *OtlpIngestOutcome) GetDuplicateRawOk() (*int32, bool)`

GetDuplicateRawOk returns a tuple with the DuplicateRaw field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDuplicateRaw

`func (o *OtlpIngestOutcome) SetDuplicateRaw(v int32)`

SetDuplicateRaw sets DuplicateRaw field to given value.


### GetDuplicateSpans

`func (o *OtlpIngestOutcome) GetDuplicateSpans() int32`

GetDuplicateSpans returns the DuplicateSpans field if non-nil, zero value otherwise.

### GetDuplicateSpansOk

`func (o *OtlpIngestOutcome) GetDuplicateSpansOk() (*int32, bool)`

GetDuplicateSpansOk returns a tuple with the DuplicateSpans field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDuplicateSpans

`func (o *OtlpIngestOutcome) SetDuplicateSpans(v int32)`

SetDuplicateSpans sets DuplicateSpans field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
