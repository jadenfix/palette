# PromptVersionDiff

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**FromVersionId** | **string** |  | 
**Lines** | [**[]DiffLine**](DiffLine.md) |  | 
**ToVersionId** | **string** |  | 

## Methods

### NewPromptVersionDiff

`func NewPromptVersionDiff(fromVersionId string, lines []DiffLine, toVersionId string, ) *PromptVersionDiff`

NewPromptVersionDiff instantiates a new PromptVersionDiff object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewPromptVersionDiffWithDefaults

`func NewPromptVersionDiffWithDefaults() *PromptVersionDiff`

NewPromptVersionDiffWithDefaults instantiates a new PromptVersionDiff object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetFromVersionId

`func (o *PromptVersionDiff) GetFromVersionId() string`

GetFromVersionId returns the FromVersionId field if non-nil, zero value otherwise.

### GetFromVersionIdOk

`func (o *PromptVersionDiff) GetFromVersionIdOk() (*string, bool)`

GetFromVersionIdOk returns a tuple with the FromVersionId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetFromVersionId

`func (o *PromptVersionDiff) SetFromVersionId(v string)`

SetFromVersionId sets FromVersionId field to given value.


### GetLines

`func (o *PromptVersionDiff) GetLines() []DiffLine`

GetLines returns the Lines field if non-nil, zero value otherwise.

### GetLinesOk

`func (o *PromptVersionDiff) GetLinesOk() (*[]DiffLine, bool)`

GetLinesOk returns a tuple with the Lines field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLines

`func (o *PromptVersionDiff) SetLines(v []DiffLine)`

SetLines sets Lines field to given value.


### GetToVersionId

`func (o *PromptVersionDiff) GetToVersionId() string`

GetToVersionId returns the ToVersionId field if non-nil, zero value otherwise.

### GetToVersionIdOk

`func (o *PromptVersionDiff) GetToVersionIdOk() (*string, bool)`

GetToVersionIdOk returns a tuple with the ToVersionId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetToVersionId

`func (o *PromptVersionDiff) SetToVersionId(v string)`

SetToVersionId sets ToVersionId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


