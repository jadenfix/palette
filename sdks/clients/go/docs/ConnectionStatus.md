# ConnectionStatus

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Connected** | **bool** | &#x60;true&#x60; only when an account exists and is &#x60;ACTIVE&#x60;. |
**ConnectedAccountId** | Pointer to **NullableString** | The connected-account id, when one exists. | [optional]
**Status** | **string** | Raw Composio status (&#x60;ACTIVE&#x60;, &#x60;INITIALIZING&#x60;, &#x60;FAILED&#x60;, …) or &#x60;not_connected&#x60; when no account exists yet. |
**Toolkit** | **string** | Toolkit slug this status is for. |

## Methods

### NewConnectionStatus

`func NewConnectionStatus(connected bool, status string, toolkit string, ) *ConnectionStatus`

NewConnectionStatus instantiates a new ConnectionStatus object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewConnectionStatusWithDefaults

`func NewConnectionStatusWithDefaults() *ConnectionStatus`

NewConnectionStatusWithDefaults instantiates a new ConnectionStatus object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetConnected

`func (o *ConnectionStatus) GetConnected() bool`

GetConnected returns the Connected field if non-nil, zero value otherwise.

### GetConnectedOk

`func (o *ConnectionStatus) GetConnectedOk() (*bool, bool)`

GetConnectedOk returns a tuple with the Connected field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetConnected

`func (o *ConnectionStatus) SetConnected(v bool)`

SetConnected sets Connected field to given value.


### GetConnectedAccountId

`func (o *ConnectionStatus) GetConnectedAccountId() string`

GetConnectedAccountId returns the ConnectedAccountId field if non-nil, zero value otherwise.

### GetConnectedAccountIdOk

`func (o *ConnectionStatus) GetConnectedAccountIdOk() (*string, bool)`

GetConnectedAccountIdOk returns a tuple with the ConnectedAccountId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetConnectedAccountId

`func (o *ConnectionStatus) SetConnectedAccountId(v string)`

SetConnectedAccountId sets ConnectedAccountId field to given value.

### HasConnectedAccountId

`func (o *ConnectionStatus) HasConnectedAccountId() bool`

HasConnectedAccountId returns a boolean if a field has been set.

### SetConnectedAccountIdNil

`func (o *ConnectionStatus) SetConnectedAccountIdNil(b bool)`

 SetConnectedAccountIdNil sets the value for ConnectedAccountId to be an explicit nil

### UnsetConnectedAccountId
`func (o *ConnectionStatus) UnsetConnectedAccountId()`

UnsetConnectedAccountId ensures that no value is present for ConnectedAccountId, not even an explicit nil
### GetStatus

`func (o *ConnectionStatus) GetStatus() string`

GetStatus returns the Status field if non-nil, zero value otherwise.

### GetStatusOk

`func (o *ConnectionStatus) GetStatusOk() (*string, bool)`

GetStatusOk returns a tuple with the Status field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetStatus

`func (o *ConnectionStatus) SetStatus(v string)`

SetStatus sets Status field to given value.


### GetToolkit

`func (o *ConnectionStatus) GetToolkit() string`

GetToolkit returns the Toolkit field if non-nil, zero value otherwise.

### GetToolkitOk

`func (o *ConnectionStatus) GetToolkitOk() (*string, bool)`

GetToolkitOk returns a tuple with the Toolkit field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetToolkit

`func (o *ConnectionStatus) SetToolkit(v string)`

SetToolkit sets Toolkit field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
