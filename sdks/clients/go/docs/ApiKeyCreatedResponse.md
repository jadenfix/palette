# ApiKeyCreatedResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Active** | **bool** |  |
**ApiKeyId** | **string** |  |
**CreatedAt** | **time.Time** |  |
**EnvironmentId** | **string** |  |
**ProjectId** | **string** |  |
**Scopes** | [**[]ApiScope**](ApiScope.md) |  |
**Secret** | **string** |  |
**TenantId** | **string** |  |

## Methods

### NewApiKeyCreatedResponse

`func NewApiKeyCreatedResponse(active bool, apiKeyId string, createdAt time.Time, environmentId string, projectId string, scopes []ApiScope, secret string, tenantId string, ) *ApiKeyCreatedResponse`

NewApiKeyCreatedResponse instantiates a new ApiKeyCreatedResponse object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewApiKeyCreatedResponseWithDefaults

`func NewApiKeyCreatedResponseWithDefaults() *ApiKeyCreatedResponse`

NewApiKeyCreatedResponseWithDefaults instantiates a new ApiKeyCreatedResponse object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetActive

`func (o *ApiKeyCreatedResponse) GetActive() bool`

GetActive returns the Active field if non-nil, zero value otherwise.

### GetActiveOk

`func (o *ApiKeyCreatedResponse) GetActiveOk() (*bool, bool)`

GetActiveOk returns a tuple with the Active field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetActive

`func (o *ApiKeyCreatedResponse) SetActive(v bool)`

SetActive sets Active field to given value.


### GetApiKeyId

`func (o *ApiKeyCreatedResponse) GetApiKeyId() string`

GetApiKeyId returns the ApiKeyId field if non-nil, zero value otherwise.

### GetApiKeyIdOk

`func (o *ApiKeyCreatedResponse) GetApiKeyIdOk() (*string, bool)`

GetApiKeyIdOk returns a tuple with the ApiKeyId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetApiKeyId

`func (o *ApiKeyCreatedResponse) SetApiKeyId(v string)`

SetApiKeyId sets ApiKeyId field to given value.


### GetCreatedAt

`func (o *ApiKeyCreatedResponse) GetCreatedAt() time.Time`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *ApiKeyCreatedResponse) GetCreatedAtOk() (*time.Time, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *ApiKeyCreatedResponse) SetCreatedAt(v time.Time)`

SetCreatedAt sets CreatedAt field to given value.


### GetEnvironmentId

`func (o *ApiKeyCreatedResponse) GetEnvironmentId() string`

GetEnvironmentId returns the EnvironmentId field if non-nil, zero value otherwise.

### GetEnvironmentIdOk

`func (o *ApiKeyCreatedResponse) GetEnvironmentIdOk() (*string, bool)`

GetEnvironmentIdOk returns a tuple with the EnvironmentId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEnvironmentId

`func (o *ApiKeyCreatedResponse) SetEnvironmentId(v string)`

SetEnvironmentId sets EnvironmentId field to given value.


### GetProjectId

`func (o *ApiKeyCreatedResponse) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *ApiKeyCreatedResponse) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *ApiKeyCreatedResponse) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetScopes

`func (o *ApiKeyCreatedResponse) GetScopes() []ApiScope`

GetScopes returns the Scopes field if non-nil, zero value otherwise.

### GetScopesOk

`func (o *ApiKeyCreatedResponse) GetScopesOk() (*[]ApiScope, bool)`

GetScopesOk returns a tuple with the Scopes field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetScopes

`func (o *ApiKeyCreatedResponse) SetScopes(v []ApiScope)`

SetScopes sets Scopes field to given value.


### GetSecret

`func (o *ApiKeyCreatedResponse) GetSecret() string`

GetSecret returns the Secret field if non-nil, zero value otherwise.

### GetSecretOk

`func (o *ApiKeyCreatedResponse) GetSecretOk() (*string, bool)`

GetSecretOk returns a tuple with the Secret field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSecret

`func (o *ApiKeyCreatedResponse) SetSecret(v string)`

SetSecret sets Secret field to given value.


### GetTenantId

`func (o *ApiKeyCreatedResponse) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *ApiKeyCreatedResponse) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *ApiKeyCreatedResponse) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
