# SamplingDecision

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Reason** | [**SamplingReason**](SamplingReason.md) |  |
**Selected** | **bool** |  |
**StableScorePerMille** | **int32** |  |

## Methods

### NewSamplingDecision

`func NewSamplingDecision(reason SamplingReason, selected bool, stableScorePerMille int32, ) *SamplingDecision`

NewSamplingDecision instantiates a new SamplingDecision object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewSamplingDecisionWithDefaults

`func NewSamplingDecisionWithDefaults() *SamplingDecision`

NewSamplingDecisionWithDefaults instantiates a new SamplingDecision object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetReason

`func (o *SamplingDecision) GetReason() SamplingReason`

GetReason returns the Reason field if non-nil, zero value otherwise.

### GetReasonOk

`func (o *SamplingDecision) GetReasonOk() (*SamplingReason, bool)`

GetReasonOk returns a tuple with the Reason field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetReason

`func (o *SamplingDecision) SetReason(v SamplingReason)`

SetReason sets Reason field to given value.


### GetSelected

`func (o *SamplingDecision) GetSelected() bool`

GetSelected returns the Selected field if non-nil, zero value otherwise.

### GetSelectedOk

`func (o *SamplingDecision) GetSelectedOk() (*bool, bool)`

GetSelectedOk returns a tuple with the Selected field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSelected

`func (o *SamplingDecision) SetSelected(v bool)`

SetSelected sets Selected field to given value.


### GetStableScorePerMille

`func (o *SamplingDecision) GetStableScorePerMille() int32`

GetStableScorePerMille returns the StableScorePerMille field if non-nil, zero value otherwise.

### GetStableScorePerMilleOk

`func (o *SamplingDecision) GetStableScorePerMilleOk() (*int32, bool)`

GetStableScorePerMilleOk returns a tuple with the StableScorePerMille field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetStableScorePerMille

`func (o *SamplingDecision) SetStableScorePerMille(v int32)`

SetStableScorePerMille sets StableScorePerMille field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
