# RevokedApiKey

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Active** | **bool** |  |
**ApiKeyId** | **string** |  |
**RotatedAt** | **time.Time** |  |

## Methods

### NewRevokedApiKey

`func NewRevokedApiKey(active bool, apiKeyId string, rotatedAt time.Time, ) *RevokedApiKey`

NewRevokedApiKey instantiates a new RevokedApiKey object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewRevokedApiKeyWithDefaults

`func NewRevokedApiKeyWithDefaults() *RevokedApiKey`

NewRevokedApiKeyWithDefaults instantiates a new RevokedApiKey object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetActive

`func (o *RevokedApiKey) GetActive() bool`

GetActive returns the Active field if non-nil, zero value otherwise.

### GetActiveOk

`func (o *RevokedApiKey) GetActiveOk() (*bool, bool)`

GetActiveOk returns a tuple with the Active field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetActive

`func (o *RevokedApiKey) SetActive(v bool)`

SetActive sets Active field to given value.


### GetApiKeyId

`func (o *RevokedApiKey) GetApiKeyId() string`

GetApiKeyId returns the ApiKeyId field if non-nil, zero value otherwise.

### GetApiKeyIdOk

`func (o *RevokedApiKey) GetApiKeyIdOk() (*string, bool)`

GetApiKeyIdOk returns a tuple with the ApiKeyId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetApiKeyId

`func (o *RevokedApiKey) SetApiKeyId(v string)`

SetApiKeyId sets ApiKeyId field to given value.


### GetRotatedAt

`func (o *RevokedApiKey) GetRotatedAt() time.Time`

GetRotatedAt returns the RotatedAt field if non-nil, zero value otherwise.

### GetRotatedAtOk

`func (o *RevokedApiKey) GetRotatedAtOk() (*time.Time, bool)`

GetRotatedAtOk returns a tuple with the RotatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRotatedAt

`func (o *RevokedApiKey) SetRotatedAt(v time.Time)`

SetRotatedAt sets RotatedAt field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
