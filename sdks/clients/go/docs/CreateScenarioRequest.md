# CreateScenarioRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**ExemplarTraceId** | Pointer to **NullableString** |  | [optional]
**ExpectedOutcome** | Pointer to **NullableString** |  | [optional]
**FailureMode** | Pointer to [**NullableFailureMode**](FailureMode.md) |  | [optional]
**SourceTraceIds** | **[]string** |  |
**Title** | **string** |  |

## Methods

### NewCreateScenarioRequest

`func NewCreateScenarioRequest(sourceTraceIds []string, title string, ) *CreateScenarioRequest`

NewCreateScenarioRequest instantiates a new CreateScenarioRequest object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewCreateScenarioRequestWithDefaults

`func NewCreateScenarioRequestWithDefaults() *CreateScenarioRequest`

NewCreateScenarioRequestWithDefaults instantiates a new CreateScenarioRequest object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetExemplarTraceId

`func (o *CreateScenarioRequest) GetExemplarTraceId() string`

GetExemplarTraceId returns the ExemplarTraceId field if non-nil, zero value otherwise.

### GetExemplarTraceIdOk

`func (o *CreateScenarioRequest) GetExemplarTraceIdOk() (*string, bool)`

GetExemplarTraceIdOk returns a tuple with the ExemplarTraceId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetExemplarTraceId

`func (o *CreateScenarioRequest) SetExemplarTraceId(v string)`

SetExemplarTraceId sets ExemplarTraceId field to given value.

### HasExemplarTraceId

`func (o *CreateScenarioRequest) HasExemplarTraceId() bool`

HasExemplarTraceId returns a boolean if a field has been set.

### SetExemplarTraceIdNil

`func (o *CreateScenarioRequest) SetExemplarTraceIdNil(b bool)`

 SetExemplarTraceIdNil sets the value for ExemplarTraceId to be an explicit nil

### UnsetExemplarTraceId
`func (o *CreateScenarioRequest) UnsetExemplarTraceId()`

UnsetExemplarTraceId ensures that no value is present for ExemplarTraceId, not even an explicit nil
### GetExpectedOutcome

`func (o *CreateScenarioRequest) GetExpectedOutcome() string`

GetExpectedOutcome returns the ExpectedOutcome field if non-nil, zero value otherwise.

### GetExpectedOutcomeOk

`func (o *CreateScenarioRequest) GetExpectedOutcomeOk() (*string, bool)`

GetExpectedOutcomeOk returns a tuple with the ExpectedOutcome field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetExpectedOutcome

`func (o *CreateScenarioRequest) SetExpectedOutcome(v string)`

SetExpectedOutcome sets ExpectedOutcome field to given value.

### HasExpectedOutcome

`func (o *CreateScenarioRequest) HasExpectedOutcome() bool`

HasExpectedOutcome returns a boolean if a field has been set.

### SetExpectedOutcomeNil

`func (o *CreateScenarioRequest) SetExpectedOutcomeNil(b bool)`

 SetExpectedOutcomeNil sets the value for ExpectedOutcome to be an explicit nil

### UnsetExpectedOutcome
`func (o *CreateScenarioRequest) UnsetExpectedOutcome()`

UnsetExpectedOutcome ensures that no value is present for ExpectedOutcome, not even an explicit nil
### GetFailureMode

`func (o *CreateScenarioRequest) GetFailureMode() FailureMode`

GetFailureMode returns the FailureMode field if non-nil, zero value otherwise.

### GetFailureModeOk

`func (o *CreateScenarioRequest) GetFailureModeOk() (*FailureMode, bool)`

GetFailureModeOk returns a tuple with the FailureMode field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetFailureMode

`func (o *CreateScenarioRequest) SetFailureMode(v FailureMode)`

SetFailureMode sets FailureMode field to given value.

### HasFailureMode

`func (o *CreateScenarioRequest) HasFailureMode() bool`

HasFailureMode returns a boolean if a field has been set.

### SetFailureModeNil

`func (o *CreateScenarioRequest) SetFailureModeNil(b bool)`

 SetFailureModeNil sets the value for FailureMode to be an explicit nil

### UnsetFailureMode
`func (o *CreateScenarioRequest) UnsetFailureMode()`

UnsetFailureMode ensures that no value is present for FailureMode, not even an explicit nil
### GetSourceTraceIds

`func (o *CreateScenarioRequest) GetSourceTraceIds() []string`

GetSourceTraceIds returns the SourceTraceIds field if non-nil, zero value otherwise.

### GetSourceTraceIdsOk

`func (o *CreateScenarioRequest) GetSourceTraceIdsOk() (*[]string, bool)`

GetSourceTraceIdsOk returns a tuple with the SourceTraceIds field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSourceTraceIds

`func (o *CreateScenarioRequest) SetSourceTraceIds(v []string)`

SetSourceTraceIds sets SourceTraceIds field to given value.


### GetTitle

`func (o *CreateScenarioRequest) GetTitle() string`

GetTitle returns the Title field if non-nil, zero value otherwise.

### GetTitleOk

`func (o *CreateScenarioRequest) GetTitleOk() (*string, bool)`

GetTitleOk returns a tuple with the Title field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTitle

`func (o *CreateScenarioRequest) SetTitle(v string)`

SetTitle sets Title field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
