# ScoreResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Evidence** | **interface{}** |  |
**Label** | Pointer to **NullableString** |  | [optional]
**Score** | **float64** |  |

## Methods

### NewScoreResult

`func NewScoreResult(evidence interface{}, score float64, ) *ScoreResult`

NewScoreResult instantiates a new ScoreResult object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewScoreResultWithDefaults

`func NewScoreResultWithDefaults() *ScoreResult`

NewScoreResultWithDefaults instantiates a new ScoreResult object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetEvidence

`func (o *ScoreResult) GetEvidence() interface{}`

GetEvidence returns the Evidence field if non-nil, zero value otherwise.

### GetEvidenceOk

`func (o *ScoreResult) GetEvidenceOk() (*interface{}, bool)`

GetEvidenceOk returns a tuple with the Evidence field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEvidence

`func (o *ScoreResult) SetEvidence(v interface{})`

SetEvidence sets Evidence field to given value.


### SetEvidenceNil

`func (o *ScoreResult) SetEvidenceNil(b bool)`

 SetEvidenceNil sets the value for Evidence to be an explicit nil

### UnsetEvidence
`func (o *ScoreResult) UnsetEvidence()`

UnsetEvidence ensures that no value is present for Evidence, not even an explicit nil
### GetLabel

`func (o *ScoreResult) GetLabel() string`

GetLabel returns the Label field if non-nil, zero value otherwise.

### GetLabelOk

`func (o *ScoreResult) GetLabelOk() (*string, bool)`

GetLabelOk returns a tuple with the Label field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLabel

`func (o *ScoreResult) SetLabel(v string)`

SetLabel sets Label field to given value.

### HasLabel

`func (o *ScoreResult) HasLabel() bool`

HasLabel returns a boolean if a field has been set.

### SetLabelNil

`func (o *ScoreResult) SetLabelNil(b bool)`

 SetLabelNil sets the value for Label to be an explicit nil

### UnsetLabel
`func (o *ScoreResult) UnsetLabel()`

UnsetLabel ensures that no value is present for Label, not even an explicit nil
### GetScore

`func (o *ScoreResult) GetScore() float64`

GetScore returns the Score field if non-nil, zero value otherwise.

### GetScoreOk

`func (o *ScoreResult) GetScoreOk() (*float64, bool)`

GetScoreOk returns a tuple with the Score field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetScore

`func (o *ScoreResult) SetScore(v float64)`

SetScore sets Score field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
