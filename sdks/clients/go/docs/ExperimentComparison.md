# ExperimentComparison

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AdjustedAlpha** | **float64** |  |
**BaselineMean** | **float64** |  |
**CandidateMean** | **float64** |  |
**CiHigh** | **float64** |  |
**CiLow** | **float64** |  |
**Decision** | [**GateDecision**](GateDecision.md) |  |
**Delta** | **float64** |  |
**Mde** | Pointer to **NullableFloat64** | Minimum detectable effect at the current sample size, in the metric&#39;s own units, at the gate&#39;s (adjusted) alpha and the standard power of 0.8 (§10.3 #5). Populated only when &#x60;decision&#x60; is &#x60;Inconclusive&#x60; — the comparison lacked the power to resolve the regression bound, and regressions smaller than this are invisible at this N. &#x60;None&#x60; on a conclusive decision (or when the paired differences have zero spread, so no effect-scale is defined). This replaces a bare \&quot;underpowered\&quot; flag with the actionable \&quot;how small an effect could we even have seen\&quot; number. | [optional]
**PValue** | **float64** | Real two-sided p-value from &#x60;test&#x60;. The previous normal-approximation path reported no p-value at all. |
**RequiredN** | Pointer to **NullableInt32** | Number of paired observations that would be required to detect the *observed* effect at the gate&#39;s (adjusted) alpha and power 0.8 (§10.3 #5). Populated only when &#x60;decision&#x60; is &#x60;Inconclusive&#x60; and the observed effect is non-degenerate (non-zero delta over non-zero difference spread). &#x60;None&#x60; otherwise. This answers \&quot;how many more cases would have made this conclusive?\&quot;. | [optional]
**SampleSize** | **int32** |  |
**Test** | [**StatisticalTest**](StatisticalTest.md) |  |

## Methods

### NewExperimentComparison

`func NewExperimentComparison(adjustedAlpha float64, baselineMean float64, candidateMean float64, ciHigh float64, ciLow float64, decision GateDecision, delta float64, pValue float64, sampleSize int32, test StatisticalTest, ) *ExperimentComparison`

NewExperimentComparison instantiates a new ExperimentComparison object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewExperimentComparisonWithDefaults

`func NewExperimentComparisonWithDefaults() *ExperimentComparison`

NewExperimentComparisonWithDefaults instantiates a new ExperimentComparison object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAdjustedAlpha

`func (o *ExperimentComparison) GetAdjustedAlpha() float64`

GetAdjustedAlpha returns the AdjustedAlpha field if non-nil, zero value otherwise.

### GetAdjustedAlphaOk

`func (o *ExperimentComparison) GetAdjustedAlphaOk() (*float64, bool)`

GetAdjustedAlphaOk returns a tuple with the AdjustedAlpha field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAdjustedAlpha

`func (o *ExperimentComparison) SetAdjustedAlpha(v float64)`

SetAdjustedAlpha sets AdjustedAlpha field to given value.


### GetBaselineMean

`func (o *ExperimentComparison) GetBaselineMean() float64`

GetBaselineMean returns the BaselineMean field if non-nil, zero value otherwise.

### GetBaselineMeanOk

`func (o *ExperimentComparison) GetBaselineMeanOk() (*float64, bool)`

GetBaselineMeanOk returns a tuple with the BaselineMean field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBaselineMean

`func (o *ExperimentComparison) SetBaselineMean(v float64)`

SetBaselineMean sets BaselineMean field to given value.


### GetCandidateMean

`func (o *ExperimentComparison) GetCandidateMean() float64`

GetCandidateMean returns the CandidateMean field if non-nil, zero value otherwise.

### GetCandidateMeanOk

`func (o *ExperimentComparison) GetCandidateMeanOk() (*float64, bool)`

GetCandidateMeanOk returns a tuple with the CandidateMean field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCandidateMean

`func (o *ExperimentComparison) SetCandidateMean(v float64)`

SetCandidateMean sets CandidateMean field to given value.


### GetCiHigh

`func (o *ExperimentComparison) GetCiHigh() float64`

GetCiHigh returns the CiHigh field if non-nil, zero value otherwise.

### GetCiHighOk

`func (o *ExperimentComparison) GetCiHighOk() (*float64, bool)`

GetCiHighOk returns a tuple with the CiHigh field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCiHigh

`func (o *ExperimentComparison) SetCiHigh(v float64)`

SetCiHigh sets CiHigh field to given value.


### GetCiLow

`func (o *ExperimentComparison) GetCiLow() float64`

GetCiLow returns the CiLow field if non-nil, zero value otherwise.

### GetCiLowOk

`func (o *ExperimentComparison) GetCiLowOk() (*float64, bool)`

GetCiLowOk returns a tuple with the CiLow field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCiLow

`func (o *ExperimentComparison) SetCiLow(v float64)`

SetCiLow sets CiLow field to given value.


### GetDecision

`func (o *ExperimentComparison) GetDecision() GateDecision`

GetDecision returns the Decision field if non-nil, zero value otherwise.

### GetDecisionOk

`func (o *ExperimentComparison) GetDecisionOk() (*GateDecision, bool)`

GetDecisionOk returns a tuple with the Decision field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDecision

`func (o *ExperimentComparison) SetDecision(v GateDecision)`

SetDecision sets Decision field to given value.


### GetDelta

`func (o *ExperimentComparison) GetDelta() float64`

GetDelta returns the Delta field if non-nil, zero value otherwise.

### GetDeltaOk

`func (o *ExperimentComparison) GetDeltaOk() (*float64, bool)`

GetDeltaOk returns a tuple with the Delta field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDelta

`func (o *ExperimentComparison) SetDelta(v float64)`

SetDelta sets Delta field to given value.


### GetMde

`func (o *ExperimentComparison) GetMde() float64`

GetMde returns the Mde field if non-nil, zero value otherwise.

### GetMdeOk

`func (o *ExperimentComparison) GetMdeOk() (*float64, bool)`

GetMdeOk returns a tuple with the Mde field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMde

`func (o *ExperimentComparison) SetMde(v float64)`

SetMde sets Mde field to given value.

### HasMde

`func (o *ExperimentComparison) HasMde() bool`

HasMde returns a boolean if a field has been set.

### SetMdeNil

`func (o *ExperimentComparison) SetMdeNil(b bool)`

 SetMdeNil sets the value for Mde to be an explicit nil

### UnsetMde
`func (o *ExperimentComparison) UnsetMde()`

UnsetMde ensures that no value is present for Mde, not even an explicit nil
### GetPValue

`func (o *ExperimentComparison) GetPValue() float64`

GetPValue returns the PValue field if non-nil, zero value otherwise.

### GetPValueOk

`func (o *ExperimentComparison) GetPValueOk() (*float64, bool)`

GetPValueOk returns a tuple with the PValue field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPValue

`func (o *ExperimentComparison) SetPValue(v float64)`

SetPValue sets PValue field to given value.


### GetRequiredN

`func (o *ExperimentComparison) GetRequiredN() int32`

GetRequiredN returns the RequiredN field if non-nil, zero value otherwise.

### GetRequiredNOk

`func (o *ExperimentComparison) GetRequiredNOk() (*int32, bool)`

GetRequiredNOk returns a tuple with the RequiredN field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRequiredN

`func (o *ExperimentComparison) SetRequiredN(v int32)`

SetRequiredN sets RequiredN field to given value.

### HasRequiredN

`func (o *ExperimentComparison) HasRequiredN() bool`

HasRequiredN returns a boolean if a field has been set.

### SetRequiredNNil

`func (o *ExperimentComparison) SetRequiredNNil(b bool)`

 SetRequiredNNil sets the value for RequiredN to be an explicit nil

### UnsetRequiredN
`func (o *ExperimentComparison) UnsetRequiredN()`

UnsetRequiredN ensures that no value is present for RequiredN, not even an explicit nil
### GetSampleSize

`func (o *ExperimentComparison) GetSampleSize() int32`

GetSampleSize returns the SampleSize field if non-nil, zero value otherwise.

### GetSampleSizeOk

`func (o *ExperimentComparison) GetSampleSizeOk() (*int32, bool)`

GetSampleSizeOk returns a tuple with the SampleSize field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSampleSize

`func (o *ExperimentComparison) SetSampleSize(v int32)`

SetSampleSize sets SampleSize field to given value.


### GetTest

`func (o *ExperimentComparison) GetTest() StatisticalTest`

GetTest returns the Test field if non-nil, zero value otherwise.

### GetTestOk

`func (o *ExperimentComparison) GetTestOk() (*StatisticalTest, bool)`

GetTestOk returns a tuple with the Test field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTest

`func (o *ExperimentComparison) SetTest(v StatisticalTest)`

SetTest sets Test field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
