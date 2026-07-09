# RevokedProviderSecret

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Active** | **bool** |  |
**ProviderSecretId** | **string** |  |
**RotatedAt** | **time.Time** |  |

## Methods

### NewRevokedProviderSecret

`func NewRevokedProviderSecret(active bool, providerSecretId string, rotatedAt time.Time, ) *RevokedProviderSecret`

NewRevokedProviderSecret instantiates a new RevokedProviderSecret object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewRevokedProviderSecretWithDefaults

`func NewRevokedProviderSecretWithDefaults() *RevokedProviderSecret`

NewRevokedProviderSecretWithDefaults instantiates a new RevokedProviderSecret object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetActive

`func (o *RevokedProviderSecret) GetActive() bool`

GetActive returns the Active field if non-nil, zero value otherwise.

### GetActiveOk

`func (o *RevokedProviderSecret) GetActiveOk() (*bool, bool)`

GetActiveOk returns a tuple with the Active field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetActive

`func (o *RevokedProviderSecret) SetActive(v bool)`

SetActive sets Active field to given value.


### GetProviderSecretId

`func (o *RevokedProviderSecret) GetProviderSecretId() string`

GetProviderSecretId returns the ProviderSecretId field if non-nil, zero value otherwise.

### GetProviderSecretIdOk

`func (o *RevokedProviderSecret) GetProviderSecretIdOk() (*string, bool)`

GetProviderSecretIdOk returns a tuple with the ProviderSecretId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProviderSecretId

`func (o *RevokedProviderSecret) SetProviderSecretId(v string)`

SetProviderSecretId sets ProviderSecretId field to given value.


### GetRotatedAt

`func (o *RevokedProviderSecret) GetRotatedAt() time.Time`

GetRotatedAt returns the RotatedAt field if non-nil, zero value otherwise.

### GetRotatedAtOk

`func (o *RevokedProviderSecret) GetRotatedAtOk() (*time.Time, bool)`

GetRotatedAtOk returns a tuple with the RotatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRotatedAt

`func (o *RevokedProviderSecret) SetRotatedAt(v time.Time)`

SetRotatedAt sets RotatedAt field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
