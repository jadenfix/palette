# OnlineSamplingPolicy

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**HighCostMicrosThreshold** | Pointer to **NullableInt64** |  | [optional]
**KeepErrors** | **bool** |  |
**SampleRatePerMille** | **int32** |  |
**SlowMsThreshold** | Pointer to **NullableInt64** |  | [optional]

## Methods

### NewOnlineSamplingPolicy

`func NewOnlineSamplingPolicy(keepErrors bool, sampleRatePerMille int32, ) *OnlineSamplingPolicy`

NewOnlineSamplingPolicy instantiates a new OnlineSamplingPolicy object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewOnlineSamplingPolicyWithDefaults

`func NewOnlineSamplingPolicyWithDefaults() *OnlineSamplingPolicy`

NewOnlineSamplingPolicyWithDefaults instantiates a new OnlineSamplingPolicy object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetHighCostMicrosThreshold

`func (o *OnlineSamplingPolicy) GetHighCostMicrosThreshold() int64`

GetHighCostMicrosThreshold returns the HighCostMicrosThreshold field if non-nil, zero value otherwise.

### GetHighCostMicrosThresholdOk

`func (o *OnlineSamplingPolicy) GetHighCostMicrosThresholdOk() (*int64, bool)`

GetHighCostMicrosThresholdOk returns a tuple with the HighCostMicrosThreshold field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetHighCostMicrosThreshold

`func (o *OnlineSamplingPolicy) SetHighCostMicrosThreshold(v int64)`

SetHighCostMicrosThreshold sets HighCostMicrosThreshold field to given value.

### HasHighCostMicrosThreshold

`func (o *OnlineSamplingPolicy) HasHighCostMicrosThreshold() bool`

HasHighCostMicrosThreshold returns a boolean if a field has been set.

### SetHighCostMicrosThresholdNil

`func (o *OnlineSamplingPolicy) SetHighCostMicrosThresholdNil(b bool)`

 SetHighCostMicrosThresholdNil sets the value for HighCostMicrosThreshold to be an explicit nil

### UnsetHighCostMicrosThreshold
`func (o *OnlineSamplingPolicy) UnsetHighCostMicrosThreshold()`

UnsetHighCostMicrosThreshold ensures that no value is present for HighCostMicrosThreshold, not even an explicit nil
### GetKeepErrors

`func (o *OnlineSamplingPolicy) GetKeepErrors() bool`

GetKeepErrors returns the KeepErrors field if non-nil, zero value otherwise.

### GetKeepErrorsOk

`func (o *OnlineSamplingPolicy) GetKeepErrorsOk() (*bool, bool)`

GetKeepErrorsOk returns a tuple with the KeepErrors field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetKeepErrors

`func (o *OnlineSamplingPolicy) SetKeepErrors(v bool)`

SetKeepErrors sets KeepErrors field to given value.


### GetSampleRatePerMille

`func (o *OnlineSamplingPolicy) GetSampleRatePerMille() int32`

GetSampleRatePerMille returns the SampleRatePerMille field if non-nil, zero value otherwise.

### GetSampleRatePerMilleOk

`func (o *OnlineSamplingPolicy) GetSampleRatePerMilleOk() (*int32, bool)`

GetSampleRatePerMilleOk returns a tuple with the SampleRatePerMille field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSampleRatePerMille

`func (o *OnlineSamplingPolicy) SetSampleRatePerMille(v int32)`

SetSampleRatePerMille sets SampleRatePerMille field to given value.


### GetSlowMsThreshold

`func (o *OnlineSamplingPolicy) GetSlowMsThreshold() int64`

GetSlowMsThreshold returns the SlowMsThreshold field if non-nil, zero value otherwise.

### GetSlowMsThresholdOk

`func (o *OnlineSamplingPolicy) GetSlowMsThresholdOk() (*int64, bool)`

GetSlowMsThresholdOk returns a tuple with the SlowMsThreshold field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSlowMsThreshold

`func (o *OnlineSamplingPolicy) SetSlowMsThreshold(v int64)`

SetSlowMsThreshold sets SlowMsThreshold field to given value.

### HasSlowMsThreshold

`func (o *OnlineSamplingPolicy) HasSlowMsThreshold() bool`

HasSlowMsThreshold returns a boolean if a field has been set.

### SetSlowMsThresholdNil

`func (o *OnlineSamplingPolicy) SetSlowMsThresholdNil(b bool)`

 SetSlowMsThresholdNil sets the value for SlowMsThreshold to be an explicit nil

### UnsetSlowMsThreshold
`func (o *OnlineSamplingPolicy) UnsetSlowMsThreshold()`

UnsetSlowMsThreshold ensures that no value is present for SlowMsThreshold, not even an explicit nil

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
