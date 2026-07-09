# EvaluatorKind

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Type** | **string** |  |
**Pattern** | **string** |  |
**Abs** | **float64** |  |
**Rel** | **float64** |  |
**MaxMicros** | **int64** |  |
**MaxMs** | **int64** |  |
**Model** | **string** |  |
**Rubric** | **string** |  |
**DomContains** | Pointer to **string** |  | [optional]
**UrlContains** | Pointer to **string** |  | [optional]
**MaxSteps** | **int64** |  |
**MinRatio** | **float64** |  |

## Methods

### NewEvaluatorKind

`func NewEvaluatorKind(type_ string, pattern string, abs float64, rel float64, maxMicros int64, maxMs int64, model string, rubric string, maxSteps int64, minRatio float64, ) *EvaluatorKind`

NewEvaluatorKind instantiates a new EvaluatorKind object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewEvaluatorKindWithDefaults

`func NewEvaluatorKindWithDefaults() *EvaluatorKind`

NewEvaluatorKindWithDefaults instantiates a new EvaluatorKind object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetType

`func (o *EvaluatorKind) GetType() string`

GetType returns the Type field if non-nil, zero value otherwise.

### GetTypeOk

`func (o *EvaluatorKind) GetTypeOk() (*string, bool)`

GetTypeOk returns a tuple with the Type field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetType

`func (o *EvaluatorKind) SetType(v string)`

SetType sets Type field to given value.


### GetPattern

`func (o *EvaluatorKind) GetPattern() string`

GetPattern returns the Pattern field if non-nil, zero value otherwise.

### GetPatternOk

`func (o *EvaluatorKind) GetPatternOk() (*string, bool)`

GetPatternOk returns a tuple with the Pattern field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPattern

`func (o *EvaluatorKind) SetPattern(v string)`

SetPattern sets Pattern field to given value.


### GetAbs

`func (o *EvaluatorKind) GetAbs() float64`

GetAbs returns the Abs field if non-nil, zero value otherwise.

### GetAbsOk

`func (o *EvaluatorKind) GetAbsOk() (*float64, bool)`

GetAbsOk returns a tuple with the Abs field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAbs

`func (o *EvaluatorKind) SetAbs(v float64)`

SetAbs sets Abs field to given value.


### GetRel

`func (o *EvaluatorKind) GetRel() float64`

GetRel returns the Rel field if non-nil, zero value otherwise.

### GetRelOk

`func (o *EvaluatorKind) GetRelOk() (*float64, bool)`

GetRelOk returns a tuple with the Rel field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRel

`func (o *EvaluatorKind) SetRel(v float64)`

SetRel sets Rel field to given value.


### GetMaxMicros

`func (o *EvaluatorKind) GetMaxMicros() int64`

GetMaxMicros returns the MaxMicros field if non-nil, zero value otherwise.

### GetMaxMicrosOk

`func (o *EvaluatorKind) GetMaxMicrosOk() (*int64, bool)`

GetMaxMicrosOk returns a tuple with the MaxMicros field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMaxMicros

`func (o *EvaluatorKind) SetMaxMicros(v int64)`

SetMaxMicros sets MaxMicros field to given value.


### GetMaxMs

`func (o *EvaluatorKind) GetMaxMs() int64`

GetMaxMs returns the MaxMs field if non-nil, zero value otherwise.

### GetMaxMsOk

`func (o *EvaluatorKind) GetMaxMsOk() (*int64, bool)`

GetMaxMsOk returns a tuple with the MaxMs field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMaxMs

`func (o *EvaluatorKind) SetMaxMs(v int64)`

SetMaxMs sets MaxMs field to given value.


### GetModel

`func (o *EvaluatorKind) GetModel() string`

GetModel returns the Model field if non-nil, zero value otherwise.

### GetModelOk

`func (o *EvaluatorKind) GetModelOk() (*string, bool)`

GetModelOk returns a tuple with the Model field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetModel

`func (o *EvaluatorKind) SetModel(v string)`

SetModel sets Model field to given value.


### GetRubric

`func (o *EvaluatorKind) GetRubric() string`

GetRubric returns the Rubric field if non-nil, zero value otherwise.

### GetRubricOk

`func (o *EvaluatorKind) GetRubricOk() (*string, bool)`

GetRubricOk returns a tuple with the Rubric field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRubric

`func (o *EvaluatorKind) SetRubric(v string)`

SetRubric sets Rubric field to given value.


### GetDomContains

`func (o *EvaluatorKind) GetDomContains() string`

GetDomContains returns the DomContains field if non-nil, zero value otherwise.

### GetDomContainsOk

`func (o *EvaluatorKind) GetDomContainsOk() (*string, bool)`

GetDomContainsOk returns a tuple with the DomContains field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDomContains

`func (o *EvaluatorKind) SetDomContains(v string)`

SetDomContains sets DomContains field to given value.

### HasDomContains

`func (o *EvaluatorKind) HasDomContains() bool`

HasDomContains returns a boolean if a field has been set.

### GetUrlContains

`func (o *EvaluatorKind) GetUrlContains() string`

GetUrlContains returns the UrlContains field if non-nil, zero value otherwise.

### GetUrlContainsOk

`func (o *EvaluatorKind) GetUrlContainsOk() (*string, bool)`

GetUrlContainsOk returns a tuple with the UrlContains field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetUrlContains

`func (o *EvaluatorKind) SetUrlContains(v string)`

SetUrlContains sets UrlContains field to given value.

### HasUrlContains

`func (o *EvaluatorKind) HasUrlContains() bool`

HasUrlContains returns a boolean if a field has been set.

### GetMaxSteps

`func (o *EvaluatorKind) GetMaxSteps() int64`

GetMaxSteps returns the MaxSteps field if non-nil, zero value otherwise.

### GetMaxStepsOk

`func (o *EvaluatorKind) GetMaxStepsOk() (*int64, bool)`

GetMaxStepsOk returns a tuple with the MaxSteps field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMaxSteps

`func (o *EvaluatorKind) SetMaxSteps(v int64)`

SetMaxSteps sets MaxSteps field to given value.


### GetMinRatio

`func (o *EvaluatorKind) GetMinRatio() float64`

GetMinRatio returns the MinRatio field if non-nil, zero value otherwise.

### GetMinRatioOk

`func (o *EvaluatorKind) GetMinRatioOk() (*float64, bool)`

GetMinRatioOk returns a tuple with the MinRatio field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMinRatio

`func (o *EvaluatorKind) SetMinRatio(v float64)`

SetMinRatio sets MinRatio field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
