# PageRunSummary

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Items** | [**[]PageRunSummaryItemsInner**](PageRunSummaryItemsInner.md) |  |
**NextCursor** | Pointer to **NullableString** |  | [optional]

## Methods

### NewPageRunSummary

`func NewPageRunSummary(items []PageRunSummaryItemsInner, ) *PageRunSummary`

NewPageRunSummary instantiates a new PageRunSummary object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewPageRunSummaryWithDefaults

`func NewPageRunSummaryWithDefaults() *PageRunSummary`

NewPageRunSummaryWithDefaults instantiates a new PageRunSummary object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetItems

`func (o *PageRunSummary) GetItems() []PageRunSummaryItemsInner`

GetItems returns the Items field if non-nil, zero value otherwise.

### GetItemsOk

`func (o *PageRunSummary) GetItemsOk() (*[]PageRunSummaryItemsInner, bool)`

GetItemsOk returns a tuple with the Items field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetItems

`func (o *PageRunSummary) SetItems(v []PageRunSummaryItemsInner)`

SetItems sets Items field to given value.


### GetNextCursor

`func (o *PageRunSummary) GetNextCursor() string`

GetNextCursor returns the NextCursor field if non-nil, zero value otherwise.

### GetNextCursorOk

`func (o *PageRunSummary) GetNextCursorOk() (*string, bool)`

GetNextCursorOk returns a tuple with the NextCursor field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetNextCursor

`func (o *PageRunSummary) SetNextCursor(v string)`

SetNextCursor sets NextCursor field to given value.

### HasNextCursor

`func (o *PageRunSummary) HasNextCursor() bool`

HasNextCursor returns a boolean if a field has been set.

### SetNextCursorNil

`func (o *PageRunSummary) SetNextCursorNil(b bool)`

 SetNextCursorNil sets the value for NextCursor to be an explicit nil

### UnsetNextCursor
`func (o *PageRunSummary) UnsetNextCursor()`

UnsetNextCursor ensures that no value is present for NextCursor, not even an explicit nil

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
