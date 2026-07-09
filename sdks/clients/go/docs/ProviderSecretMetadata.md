# ProviderSecretMetadata

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Active** | **bool** |  |
**CreatedAt** | **time.Time** |  |
**DisplayName** | **string** |  |
**ProjectId** | **string** |  |
**Provider** | **string** |  |
**ProviderSecretId** | **string** |  |
**RotatedAt** | Pointer to **NullableTime** |  | [optional]
**TenantId** | **string** |  |

## Methods

### NewProviderSecretMetadata

`func NewProviderSecretMetadata(active bool, createdAt time.Time, displayName string, projectId string, provider string, providerSecretId string, tenantId string, ) *ProviderSecretMetadata`

NewProviderSecretMetadata instantiates a new ProviderSecretMetadata object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewProviderSecretMetadataWithDefaults

`func NewProviderSecretMetadataWithDefaults() *ProviderSecretMetadata`

NewProviderSecretMetadataWithDefaults instantiates a new ProviderSecretMetadata object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetActive

`func (o *ProviderSecretMetadata) GetActive() bool`

GetActive returns the Active field if non-nil, zero value otherwise.

### GetActiveOk

`func (o *ProviderSecretMetadata) GetActiveOk() (*bool, bool)`

GetActiveOk returns a tuple with the Active field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetActive

`func (o *ProviderSecretMetadata) SetActive(v bool)`

SetActive sets Active field to given value.


### GetCreatedAt

`func (o *ProviderSecretMetadata) GetCreatedAt() time.Time`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *ProviderSecretMetadata) GetCreatedAtOk() (*time.Time, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *ProviderSecretMetadata) SetCreatedAt(v time.Time)`

SetCreatedAt sets CreatedAt field to given value.


### GetDisplayName

`func (o *ProviderSecretMetadata) GetDisplayName() string`

GetDisplayName returns the DisplayName field if non-nil, zero value otherwise.

### GetDisplayNameOk

`func (o *ProviderSecretMetadata) GetDisplayNameOk() (*string, bool)`

GetDisplayNameOk returns a tuple with the DisplayName field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDisplayName

`func (o *ProviderSecretMetadata) SetDisplayName(v string)`

SetDisplayName sets DisplayName field to given value.


### GetProjectId

`func (o *ProviderSecretMetadata) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *ProviderSecretMetadata) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *ProviderSecretMetadata) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetProvider

`func (o *ProviderSecretMetadata) GetProvider() string`

GetProvider returns the Provider field if non-nil, zero value otherwise.

### GetProviderOk

`func (o *ProviderSecretMetadata) GetProviderOk() (*string, bool)`

GetProviderOk returns a tuple with the Provider field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProvider

`func (o *ProviderSecretMetadata) SetProvider(v string)`

SetProvider sets Provider field to given value.


### GetProviderSecretId

`func (o *ProviderSecretMetadata) GetProviderSecretId() string`

GetProviderSecretId returns the ProviderSecretId field if non-nil, zero value otherwise.

### GetProviderSecretIdOk

`func (o *ProviderSecretMetadata) GetProviderSecretIdOk() (*string, bool)`

GetProviderSecretIdOk returns a tuple with the ProviderSecretId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProviderSecretId

`func (o *ProviderSecretMetadata) SetProviderSecretId(v string)`

SetProviderSecretId sets ProviderSecretId field to given value.


### GetRotatedAt

`func (o *ProviderSecretMetadata) GetRotatedAt() time.Time`

GetRotatedAt returns the RotatedAt field if non-nil, zero value otherwise.

### GetRotatedAtOk

`func (o *ProviderSecretMetadata) GetRotatedAtOk() (*time.Time, bool)`

GetRotatedAtOk returns a tuple with the RotatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRotatedAt

`func (o *ProviderSecretMetadata) SetRotatedAt(v time.Time)`

SetRotatedAt sets RotatedAt field to given value.

### HasRotatedAt

`func (o *ProviderSecretMetadata) HasRotatedAt() bool`

HasRotatedAt returns a boolean if a field has been set.

### SetRotatedAtNil

`func (o *ProviderSecretMetadata) SetRotatedAtNil(b bool)`

 SetRotatedAtNil sets the value for RotatedAt to be an explicit nil

### UnsetRotatedAt
`func (o *ProviderSecretMetadata) UnsetRotatedAt()`

UnsetRotatedAt ensures that no value is present for RotatedAt, not even an explicit nil
### GetTenantId

`func (o *ProviderSecretMetadata) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *ProviderSecretMetadata) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *ProviderSecretMetadata) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
