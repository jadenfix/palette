# SpanIoValue

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Kind** | **string** |  |
**Value** | **interface{}** |  |
**ArtifactRef** | [**ArtifactRef**](ArtifactRef.md) |  |
**Reason** | **string** |  |

## Methods

### NewSpanIoValue

`func NewSpanIoValue(kind string, value interface{}, artifactRef ArtifactRef, reason string, ) *SpanIoValue`

NewSpanIoValue instantiates a new SpanIoValue object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewSpanIoValueWithDefaults

`func NewSpanIoValueWithDefaults() *SpanIoValue`

NewSpanIoValueWithDefaults instantiates a new SpanIoValue object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetKind

`func (o *SpanIoValue) GetKind() string`

GetKind returns the Kind field if non-nil, zero value otherwise.

### GetKindOk

`func (o *SpanIoValue) GetKindOk() (*string, bool)`

GetKindOk returns a tuple with the Kind field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetKind

`func (o *SpanIoValue) SetKind(v string)`

SetKind sets Kind field to given value.


### GetValue

`func (o *SpanIoValue) GetValue() interface{}`

GetValue returns the Value field if non-nil, zero value otherwise.

### GetValueOk

`func (o *SpanIoValue) GetValueOk() (*interface{}, bool)`

GetValueOk returns a tuple with the Value field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetValue

`func (o *SpanIoValue) SetValue(v interface{})`

SetValue sets Value field to given value.


### SetValueNil

`func (o *SpanIoValue) SetValueNil(b bool)`

 SetValueNil sets the value for Value to be an explicit nil

### UnsetValue
`func (o *SpanIoValue) UnsetValue()`

UnsetValue ensures that no value is present for Value, not even an explicit nil
### GetArtifactRef

`func (o *SpanIoValue) GetArtifactRef() ArtifactRef`

GetArtifactRef returns the ArtifactRef field if non-nil, zero value otherwise.

### GetArtifactRefOk

`func (o *SpanIoValue) GetArtifactRefOk() (*ArtifactRef, bool)`

GetArtifactRefOk returns a tuple with the ArtifactRef field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetArtifactRef

`func (o *SpanIoValue) SetArtifactRef(v ArtifactRef)`

SetArtifactRef sets ArtifactRef field to given value.


### GetReason

`func (o *SpanIoValue) GetReason() string`

GetReason returns the Reason field if non-nil, zero value otherwise.

### GetReasonOk

`func (o *SpanIoValue) GetReasonOk() (*string, bool)`

GetReasonOk returns a tuple with the Reason field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetReason

`func (o *SpanIoValue) SetReason(v string)`

SetReason sets Reason field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
