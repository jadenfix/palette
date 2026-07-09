# ReliabilityBin

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Accuracy** | Pointer to **NullableFloat64** |  | [optional]
**BinIndex** | **int32** |  |
**CalibrationGap** | Pointer to **NullableFloat64** |  | [optional]
**LowerBound** | **float64** |  |
**MeanConfidence** | Pointer to **NullableFloat64** |  | [optional]
**SampleCount** | **int32** |  |
**UpperBound** | **float64** |  |

## Methods

### NewReliabilityBin

`func NewReliabilityBin(binIndex int32, lowerBound float64, sampleCount int32, upperBound float64, ) *ReliabilityBin`

NewReliabilityBin instantiates a new ReliabilityBin object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewReliabilityBinWithDefaults

`func NewReliabilityBinWithDefaults() *ReliabilityBin`

NewReliabilityBinWithDefaults instantiates a new ReliabilityBin object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAccuracy

`func (o *ReliabilityBin) GetAccuracy() float64`

GetAccuracy returns the Accuracy field if non-nil, zero value otherwise.

### GetAccuracyOk

`func (o *ReliabilityBin) GetAccuracyOk() (*float64, bool)`

GetAccuracyOk returns a tuple with the Accuracy field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAccuracy

`func (o *ReliabilityBin) SetAccuracy(v float64)`

SetAccuracy sets Accuracy field to given value.

### HasAccuracy

`func (o *ReliabilityBin) HasAccuracy() bool`

HasAccuracy returns a boolean if a field has been set.

### SetAccuracyNil

`func (o *ReliabilityBin) SetAccuracyNil(b bool)`

 SetAccuracyNil sets the value for Accuracy to be an explicit nil

### UnsetAccuracy
`func (o *ReliabilityBin) UnsetAccuracy()`

UnsetAccuracy ensures that no value is present for Accuracy, not even an explicit nil
### GetBinIndex

`func (o *ReliabilityBin) GetBinIndex() int32`

GetBinIndex returns the BinIndex field if non-nil, zero value otherwise.

### GetBinIndexOk

`func (o *ReliabilityBin) GetBinIndexOk() (*int32, bool)`

GetBinIndexOk returns a tuple with the BinIndex field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBinIndex

`func (o *ReliabilityBin) SetBinIndex(v int32)`

SetBinIndex sets BinIndex field to given value.


### GetCalibrationGap

`func (o *ReliabilityBin) GetCalibrationGap() float64`

GetCalibrationGap returns the CalibrationGap field if non-nil, zero value otherwise.

### GetCalibrationGapOk

`func (o *ReliabilityBin) GetCalibrationGapOk() (*float64, bool)`

GetCalibrationGapOk returns a tuple with the CalibrationGap field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCalibrationGap

`func (o *ReliabilityBin) SetCalibrationGap(v float64)`

SetCalibrationGap sets CalibrationGap field to given value.

### HasCalibrationGap

`func (o *ReliabilityBin) HasCalibrationGap() bool`

HasCalibrationGap returns a boolean if a field has been set.

### SetCalibrationGapNil

`func (o *ReliabilityBin) SetCalibrationGapNil(b bool)`

 SetCalibrationGapNil sets the value for CalibrationGap to be an explicit nil

### UnsetCalibrationGap
`func (o *ReliabilityBin) UnsetCalibrationGap()`

UnsetCalibrationGap ensures that no value is present for CalibrationGap, not even an explicit nil
### GetLowerBound

`func (o *ReliabilityBin) GetLowerBound() float64`

GetLowerBound returns the LowerBound field if non-nil, zero value otherwise.

### GetLowerBoundOk

`func (o *ReliabilityBin) GetLowerBoundOk() (*float64, bool)`

GetLowerBoundOk returns a tuple with the LowerBound field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLowerBound

`func (o *ReliabilityBin) SetLowerBound(v float64)`

SetLowerBound sets LowerBound field to given value.


### GetMeanConfidence

`func (o *ReliabilityBin) GetMeanConfidence() float64`

GetMeanConfidence returns the MeanConfidence field if non-nil, zero value otherwise.

### GetMeanConfidenceOk

`func (o *ReliabilityBin) GetMeanConfidenceOk() (*float64, bool)`

GetMeanConfidenceOk returns a tuple with the MeanConfidence field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMeanConfidence

`func (o *ReliabilityBin) SetMeanConfidence(v float64)`

SetMeanConfidence sets MeanConfidence field to given value.

### HasMeanConfidence

`func (o *ReliabilityBin) HasMeanConfidence() bool`

HasMeanConfidence returns a boolean if a field has been set.

### SetMeanConfidenceNil

`func (o *ReliabilityBin) SetMeanConfidenceNil(b bool)`

 SetMeanConfidenceNil sets the value for MeanConfidence to be an explicit nil

### UnsetMeanConfidence
`func (o *ReliabilityBin) UnsetMeanConfidence()`

UnsetMeanConfidence ensures that no value is present for MeanConfidence, not even an explicit nil
### GetSampleCount

`func (o *ReliabilityBin) GetSampleCount() int32`

GetSampleCount returns the SampleCount field if non-nil, zero value otherwise.

### GetSampleCountOk

`func (o *ReliabilityBin) GetSampleCountOk() (*int32, bool)`

GetSampleCountOk returns a tuple with the SampleCount field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSampleCount

`func (o *ReliabilityBin) SetSampleCount(v int32)`

SetSampleCount sets SampleCount field to given value.


### GetUpperBound

`func (o *ReliabilityBin) GetUpperBound() float64`

GetUpperBound returns the UpperBound field if non-nil, zero value otherwise.

### GetUpperBoundOk

`func (o *ReliabilityBin) GetUpperBoundOk() (*float64, bool)`

GetUpperBoundOk returns a tuple with the UpperBound field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetUpperBound

`func (o *ReliabilityBin) SetUpperBound(v float64)`

SetUpperBound sets UpperBound field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
