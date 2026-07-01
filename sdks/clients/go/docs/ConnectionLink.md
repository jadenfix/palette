# ConnectionLink

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**ConnectedAccountId** | **string** | Composio connection id (&#x60;ca_…&#x60;) created for this handshake. | 
**ExpiresAt** | Pointer to **NullableString** | When the link expires (RFC 3339), if provided. | [optional] 
**RedirectUrl** | **string** | URL the end user opens once to authorize the app. | 

## Methods

### NewConnectionLink

`func NewConnectionLink(connectedAccountId string, redirectUrl string, ) *ConnectionLink`

NewConnectionLink instantiates a new ConnectionLink object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewConnectionLinkWithDefaults

`func NewConnectionLinkWithDefaults() *ConnectionLink`

NewConnectionLinkWithDefaults instantiates a new ConnectionLink object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetConnectedAccountId

`func (o *ConnectionLink) GetConnectedAccountId() string`

GetConnectedAccountId returns the ConnectedAccountId field if non-nil, zero value otherwise.

### GetConnectedAccountIdOk

`func (o *ConnectionLink) GetConnectedAccountIdOk() (*string, bool)`

GetConnectedAccountIdOk returns a tuple with the ConnectedAccountId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetConnectedAccountId

`func (o *ConnectionLink) SetConnectedAccountId(v string)`

SetConnectedAccountId sets ConnectedAccountId field to given value.


### GetExpiresAt

`func (o *ConnectionLink) GetExpiresAt() string`

GetExpiresAt returns the ExpiresAt field if non-nil, zero value otherwise.

### GetExpiresAtOk

`func (o *ConnectionLink) GetExpiresAtOk() (*string, bool)`

GetExpiresAtOk returns a tuple with the ExpiresAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetExpiresAt

`func (o *ConnectionLink) SetExpiresAt(v string)`

SetExpiresAt sets ExpiresAt field to given value.

### HasExpiresAt

`func (o *ConnectionLink) HasExpiresAt() bool`

HasExpiresAt returns a boolean if a field has been set.

### SetExpiresAtNil

`func (o *ConnectionLink) SetExpiresAtNil(b bool)`

 SetExpiresAtNil sets the value for ExpiresAt to be an explicit nil

### UnsetExpiresAt
`func (o *ConnectionLink) UnsetExpiresAt()`

UnsetExpiresAt ensures that no value is present for ExpiresAt, not even an explicit nil
### GetRedirectUrl

`func (o *ConnectionLink) GetRedirectUrl() string`

GetRedirectUrl returns the RedirectUrl field if non-nil, zero value otherwise.

### GetRedirectUrlOk

`func (o *ConnectionLink) GetRedirectUrlOk() (*string, bool)`

GetRedirectUrlOk returns a tuple with the RedirectUrl field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRedirectUrl

`func (o *ConnectionLink) SetRedirectUrl(v string)`

SetRedirectUrl sets RedirectUrl field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


