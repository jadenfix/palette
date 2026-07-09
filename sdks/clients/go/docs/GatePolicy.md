# GatePolicy

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Alpha** | **float64** |  |
**ComparisonCount** | **int32** |  |
**MaxRegression** | **float64** |  |
**MinSampleSize** | **int32** |  |

## Methods

### NewGatePolicy

`func NewGatePolicy(alpha float64, comparisonCount int32, maxRegression float64, minSampleSize int32, ) *GatePolicy`

NewGatePolicy instantiates a new GatePolicy object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewGatePolicyWithDefaults

`func NewGatePolicyWithDefaults() *GatePolicy`

NewGatePolicyWithDefaults instantiates a new GatePolicy object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAlpha

`func (o *GatePolicy) GetAlpha() float64`

GetAlpha returns the Alpha field if non-nil, zero value otherwise.

### GetAlphaOk

`func (o *GatePolicy) GetAlphaOk() (*float64, bool)`

GetAlphaOk returns a tuple with the Alpha field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAlpha

`func (o *GatePolicy) SetAlpha(v float64)`

SetAlpha sets Alpha field to given value.


### GetComparisonCount

`func (o *GatePolicy) GetComparisonCount() int32`

GetComparisonCount returns the ComparisonCount field if non-nil, zero value otherwise.

### GetComparisonCountOk

`func (o *GatePolicy) GetComparisonCountOk() (*int32, bool)`

GetComparisonCountOk returns a tuple with the ComparisonCount field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetComparisonCount

`func (o *GatePolicy) SetComparisonCount(v int32)`

SetComparisonCount sets ComparisonCount field to given value.


### GetMaxRegression

`func (o *GatePolicy) GetMaxRegression() float64`

GetMaxRegression returns the MaxRegression field if non-nil, zero value otherwise.

### GetMaxRegressionOk

`func (o *GatePolicy) GetMaxRegressionOk() (*float64, bool)`

GetMaxRegressionOk returns a tuple with the MaxRegression field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMaxRegression

`func (o *GatePolicy) SetMaxRegression(v float64)`

SetMaxRegression sets MaxRegression field to given value.


### GetMinSampleSize

`func (o *GatePolicy) GetMinSampleSize() int32`

GetMinSampleSize returns the MinSampleSize field if non-nil, zero value otherwise.

### GetMinSampleSizeOk

`func (o *GatePolicy) GetMinSampleSizeOk() (*int32, bool)`

GetMinSampleSizeOk returns a tuple with the MinSampleSize field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMinSampleSize

`func (o *GatePolicy) SetMinSampleSize(v int32)`

SetMinSampleSize sets MinSampleSize field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
