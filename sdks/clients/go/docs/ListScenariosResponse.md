# ListScenariosResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**NextCursor** | Pointer to **NullableString** |  | [optional] 
**Scenarios** | [**[]Scenario**](Scenario.md) |  | 

## Methods

### NewListScenariosResponse

`func NewListScenariosResponse(scenarios []Scenario, ) *ListScenariosResponse`

NewListScenariosResponse instantiates a new ListScenariosResponse object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewListScenariosResponseWithDefaults

`func NewListScenariosResponseWithDefaults() *ListScenariosResponse`

NewListScenariosResponseWithDefaults instantiates a new ListScenariosResponse object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetNextCursor

`func (o *ListScenariosResponse) GetNextCursor() string`

GetNextCursor returns the NextCursor field if non-nil, zero value otherwise.

### GetNextCursorOk

`func (o *ListScenariosResponse) GetNextCursorOk() (*string, bool)`

GetNextCursorOk returns a tuple with the NextCursor field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetNextCursor

`func (o *ListScenariosResponse) SetNextCursor(v string)`

SetNextCursor sets NextCursor field to given value.

### HasNextCursor

`func (o *ListScenariosResponse) HasNextCursor() bool`

HasNextCursor returns a boolean if a field has been set.

### SetNextCursorNil

`func (o *ListScenariosResponse) SetNextCursorNil(b bool)`

 SetNextCursorNil sets the value for NextCursor to be an explicit nil

### UnsetNextCursor
`func (o *ListScenariosResponse) UnsetNextCursor()`

UnsetNextCursor ensures that no value is present for NextCursor, not even an explicit nil
### GetScenarios

`func (o *ListScenariosResponse) GetScenarios() []Scenario`

GetScenarios returns the Scenarios field if non-nil, zero value otherwise.

### GetScenariosOk

`func (o *ListScenariosResponse) GetScenariosOk() (*[]Scenario, bool)`

GetScenariosOk returns a tuple with the Scenarios field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetScenarios

`func (o *ListScenariosResponse) SetScenarios(v []Scenario)`

SetScenarios sets Scenarios field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


