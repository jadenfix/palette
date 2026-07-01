# ConnectConnectorRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Toolkit** | **string** | Toolkit slug to connect (e.g. &#x60;github&#x60;, &#x60;gmail&#x60;, &#x60;slack&#x60;). | 

## Methods

### NewConnectConnectorRequest

`func NewConnectConnectorRequest(toolkit string, ) *ConnectConnectorRequest`

NewConnectConnectorRequest instantiates a new ConnectConnectorRequest object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewConnectConnectorRequestWithDefaults

`func NewConnectConnectorRequestWithDefaults() *ConnectConnectorRequest`

NewConnectConnectorRequestWithDefaults instantiates a new ConnectConnectorRequest object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetToolkit

`func (o *ConnectConnectorRequest) GetToolkit() string`

GetToolkit returns the Toolkit field if non-nil, zero value otherwise.

### GetToolkitOk

`func (o *ConnectConnectorRequest) GetToolkitOk() (*string, bool)`

GetToolkitOk returns a tuple with the Toolkit field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetToolkit

`func (o *ConnectConnectorRequest) SetToolkit(v string)`

SetToolkit sets Toolkit field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


