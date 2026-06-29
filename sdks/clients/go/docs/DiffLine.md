# DiffLine

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Kind** | [**DiffLineKind**](DiffLineKind.md) |  | 
**NewLine** | Pointer to **NullableInt32** |  | [optional] 
**OldLine** | Pointer to **NullableInt32** |  | [optional] 
**Text** | **string** |  | 

## Methods

### NewDiffLine

`func NewDiffLine(kind DiffLineKind, text string, ) *DiffLine`

NewDiffLine instantiates a new DiffLine object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewDiffLineWithDefaults

`func NewDiffLineWithDefaults() *DiffLine`

NewDiffLineWithDefaults instantiates a new DiffLine object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetKind

`func (o *DiffLine) GetKind() DiffLineKind`

GetKind returns the Kind field if non-nil, zero value otherwise.

### GetKindOk

`func (o *DiffLine) GetKindOk() (*DiffLineKind, bool)`

GetKindOk returns a tuple with the Kind field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetKind

`func (o *DiffLine) SetKind(v DiffLineKind)`

SetKind sets Kind field to given value.


### GetNewLine

`func (o *DiffLine) GetNewLine() int32`

GetNewLine returns the NewLine field if non-nil, zero value otherwise.

### GetNewLineOk

`func (o *DiffLine) GetNewLineOk() (*int32, bool)`

GetNewLineOk returns a tuple with the NewLine field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetNewLine

`func (o *DiffLine) SetNewLine(v int32)`

SetNewLine sets NewLine field to given value.

### HasNewLine

`func (o *DiffLine) HasNewLine() bool`

HasNewLine returns a boolean if a field has been set.

### SetNewLineNil

`func (o *DiffLine) SetNewLineNil(b bool)`

 SetNewLineNil sets the value for NewLine to be an explicit nil

### UnsetNewLine
`func (o *DiffLine) UnsetNewLine()`

UnsetNewLine ensures that no value is present for NewLine, not even an explicit nil
### GetOldLine

`func (o *DiffLine) GetOldLine() int32`

GetOldLine returns the OldLine field if non-nil, zero value otherwise.

### GetOldLineOk

`func (o *DiffLine) GetOldLineOk() (*int32, bool)`

GetOldLineOk returns a tuple with the OldLine field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetOldLine

`func (o *DiffLine) SetOldLine(v int32)`

SetOldLine sets OldLine field to given value.

### HasOldLine

`func (o *DiffLine) HasOldLine() bool`

HasOldLine returns a boolean if a field has been set.

### SetOldLineNil

`func (o *DiffLine) SetOldLineNil(b bool)`

 SetOldLineNil sets the value for OldLine to be an explicit nil

### UnsetOldLine
`func (o *DiffLine) UnsetOldLine()`

UnsetOldLine ensures that no value is present for OldLine, not even an explicit nil
### GetText

`func (o *DiffLine) GetText() string`

GetText returns the Text field if non-nil, zero value otherwise.

### GetTextOk

`func (o *DiffLine) GetTextOk() (*string, bool)`

GetTextOk returns a tuple with the Text field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetText

`func (o *DiffLine) SetText(v string)`

SetText sets Text field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


