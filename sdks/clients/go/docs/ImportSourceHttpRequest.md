# ImportSourceHttpRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Payload** | Pointer to **interface{}** |  | [optional]
**Source** | **string** | Registered importer key, e.g. &#x60;temporal_history&#x60; or &#x60;native&#x60;. |

## Methods

### NewImportSourceHttpRequest

`func NewImportSourceHttpRequest(source string, ) *ImportSourceHttpRequest`

NewImportSourceHttpRequest instantiates a new ImportSourceHttpRequest object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewImportSourceHttpRequestWithDefaults

`func NewImportSourceHttpRequestWithDefaults() *ImportSourceHttpRequest`

NewImportSourceHttpRequestWithDefaults instantiates a new ImportSourceHttpRequest object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetPayload

`func (o *ImportSourceHttpRequest) GetPayload() interface{}`

GetPayload returns the Payload field if non-nil, zero value otherwise.

### GetPayloadOk

`func (o *ImportSourceHttpRequest) GetPayloadOk() (*interface{}, bool)`

GetPayloadOk returns a tuple with the Payload field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPayload

`func (o *ImportSourceHttpRequest) SetPayload(v interface{})`

SetPayload sets Payload field to given value.

### HasPayload

`func (o *ImportSourceHttpRequest) HasPayload() bool`

HasPayload returns a boolean if a field has been set.

### SetPayloadNil

`func (o *ImportSourceHttpRequest) SetPayloadNil(b bool)`

 SetPayloadNil sets the value for Payload to be an explicit nil

### UnsetPayload
`func (o *ImportSourceHttpRequest) UnsetPayload()`

UnsetPayload ensures that no value is present for Payload, not even an explicit nil
### GetSource

`func (o *ImportSourceHttpRequest) GetSource() string`

GetSource returns the Source field if non-nil, zero value otherwise.

### GetSourceOk

`func (o *ImportSourceHttpRequest) GetSourceOk() (*string, bool)`

GetSourceOk returns a tuple with the Source field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSource

`func (o *ImportSourceHttpRequest) SetSource(v string)`

SetSource sets Source field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
