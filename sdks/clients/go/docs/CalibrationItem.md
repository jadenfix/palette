# CalibrationItem

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Agreed** | **bool** |  |
**DatasetCaseId** | **string** |  |
**Evidence** | **interface{}** |  |
**HumanLabel** | [**CalibrationLabel**](CalibrationLabel.md) |  |
**JudgeLabel** | [**CalibrationLabel**](CalibrationLabel.md) |  |
**JudgeResultLabel** | Pointer to **NullableString** |  | [optional]
**JudgeScore** | **float64** |  |

## Methods

### NewCalibrationItem

`func NewCalibrationItem(agreed bool, datasetCaseId string, evidence interface{}, humanLabel CalibrationLabel, judgeLabel CalibrationLabel, judgeScore float64, ) *CalibrationItem`

NewCalibrationItem instantiates a new CalibrationItem object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewCalibrationItemWithDefaults

`func NewCalibrationItemWithDefaults() *CalibrationItem`

NewCalibrationItemWithDefaults instantiates a new CalibrationItem object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAgreed

`func (o *CalibrationItem) GetAgreed() bool`

GetAgreed returns the Agreed field if non-nil, zero value otherwise.

### GetAgreedOk

`func (o *CalibrationItem) GetAgreedOk() (*bool, bool)`

GetAgreedOk returns a tuple with the Agreed field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAgreed

`func (o *CalibrationItem) SetAgreed(v bool)`

SetAgreed sets Agreed field to given value.


### GetDatasetCaseId

`func (o *CalibrationItem) GetDatasetCaseId() string`

GetDatasetCaseId returns the DatasetCaseId field if non-nil, zero value otherwise.

### GetDatasetCaseIdOk

`func (o *CalibrationItem) GetDatasetCaseIdOk() (*string, bool)`

GetDatasetCaseIdOk returns a tuple with the DatasetCaseId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDatasetCaseId

`func (o *CalibrationItem) SetDatasetCaseId(v string)`

SetDatasetCaseId sets DatasetCaseId field to given value.


### GetEvidence

`func (o *CalibrationItem) GetEvidence() interface{}`

GetEvidence returns the Evidence field if non-nil, zero value otherwise.

### GetEvidenceOk

`func (o *CalibrationItem) GetEvidenceOk() (*interface{}, bool)`

GetEvidenceOk returns a tuple with the Evidence field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEvidence

`func (o *CalibrationItem) SetEvidence(v interface{})`

SetEvidence sets Evidence field to given value.


### SetEvidenceNil

`func (o *CalibrationItem) SetEvidenceNil(b bool)`

 SetEvidenceNil sets the value for Evidence to be an explicit nil

### UnsetEvidence
`func (o *CalibrationItem) UnsetEvidence()`

UnsetEvidence ensures that no value is present for Evidence, not even an explicit nil
### GetHumanLabel

`func (o *CalibrationItem) GetHumanLabel() CalibrationLabel`

GetHumanLabel returns the HumanLabel field if non-nil, zero value otherwise.

### GetHumanLabelOk

`func (o *CalibrationItem) GetHumanLabelOk() (*CalibrationLabel, bool)`

GetHumanLabelOk returns a tuple with the HumanLabel field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetHumanLabel

`func (o *CalibrationItem) SetHumanLabel(v CalibrationLabel)`

SetHumanLabel sets HumanLabel field to given value.


### GetJudgeLabel

`func (o *CalibrationItem) GetJudgeLabel() CalibrationLabel`

GetJudgeLabel returns the JudgeLabel field if non-nil, zero value otherwise.

### GetJudgeLabelOk

`func (o *CalibrationItem) GetJudgeLabelOk() (*CalibrationLabel, bool)`

GetJudgeLabelOk returns a tuple with the JudgeLabel field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetJudgeLabel

`func (o *CalibrationItem) SetJudgeLabel(v CalibrationLabel)`

SetJudgeLabel sets JudgeLabel field to given value.


### GetJudgeResultLabel

`func (o *CalibrationItem) GetJudgeResultLabel() string`

GetJudgeResultLabel returns the JudgeResultLabel field if non-nil, zero value otherwise.

### GetJudgeResultLabelOk

`func (o *CalibrationItem) GetJudgeResultLabelOk() (*string, bool)`

GetJudgeResultLabelOk returns a tuple with the JudgeResultLabel field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetJudgeResultLabel

`func (o *CalibrationItem) SetJudgeResultLabel(v string)`

SetJudgeResultLabel sets JudgeResultLabel field to given value.

### HasJudgeResultLabel

`func (o *CalibrationItem) HasJudgeResultLabel() bool`

HasJudgeResultLabel returns a boolean if a field has been set.

### SetJudgeResultLabelNil

`func (o *CalibrationItem) SetJudgeResultLabelNil(b bool)`

 SetJudgeResultLabelNil sets the value for JudgeResultLabel to be an explicit nil

### UnsetJudgeResultLabel
`func (o *CalibrationItem) UnsetJudgeResultLabel()`

UnsetJudgeResultLabel ensures that no value is present for JudgeResultLabel, not even an explicit nil
### GetJudgeScore

`func (o *CalibrationItem) GetJudgeScore() float64`

GetJudgeScore returns the JudgeScore field if non-nil, zero value otherwise.

### GetJudgeScoreOk

`func (o *CalibrationItem) GetJudgeScoreOk() (*float64, bool)`

GetJudgeScoreOk returns a tuple with the JudgeScore field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetJudgeScore

`func (o *CalibrationItem) SetJudgeScore(v float64)`

SetJudgeScore sets JudgeScore field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
