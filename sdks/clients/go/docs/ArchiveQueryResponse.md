# ArchiveQueryResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Rows** | [**[]ArchivedSpanRow**](ArchivedSpanRow.md) |  |

## Methods

### NewArchiveQueryResponse

`func NewArchiveQueryResponse(rows []ArchivedSpanRow, ) *ArchiveQueryResponse`

NewArchiveQueryResponse instantiates a new ArchiveQueryResponse object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewArchiveQueryResponseWithDefaults

`func NewArchiveQueryResponseWithDefaults() *ArchiveQueryResponse`

NewArchiveQueryResponseWithDefaults instantiates a new ArchiveQueryResponse object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetRows

`func (o *ArchiveQueryResponse) GetRows() []ArchivedSpanRow`

GetRows returns the Rows field if non-nil, zero value otherwise.

### GetRowsOk

`func (o *ArchiveQueryResponse) GetRowsOk() (*[]ArchivedSpanRow, bool)`

GetRowsOk returns a tuple with the Rows field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRows

`func (o *ArchiveQueryResponse) SetRows(v []ArchivedSpanRow)`

SetRows sets Rows field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
