# InvokeConnectorRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Arguments** | Pointer to **map[string]interface{}** | Arguments object matching the tool&#39;s input schema. | [optional]
**Tool** | **string** | Tool slug to execute (e.g. &#x60;GITHUB_CREATE_AN_ISSUE&#x60;). |

## Methods

### NewInvokeConnectorRequest

`func NewInvokeConnectorRequest(tool string, ) *InvokeConnectorRequest`

NewInvokeConnectorRequest instantiates a new InvokeConnectorRequest object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewInvokeConnectorRequestWithDefaults

`func NewInvokeConnectorRequestWithDefaults() *InvokeConnectorRequest`

NewInvokeConnectorRequestWithDefaults instantiates a new InvokeConnectorRequest object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetArguments

`func (o *InvokeConnectorRequest) GetArguments() map[string]interface{}`

GetArguments returns the Arguments field if non-nil, zero value otherwise.

### GetArgumentsOk

`func (o *InvokeConnectorRequest) GetArgumentsOk() (*map[string]interface{}, bool)`

GetArgumentsOk returns a tuple with the Arguments field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetArguments

`func (o *InvokeConnectorRequest) SetArguments(v map[string]interface{})`

SetArguments sets Arguments field to given value.

### HasArguments

`func (o *InvokeConnectorRequest) HasArguments() bool`

HasArguments returns a boolean if a field has been set.

### GetTool

`func (o *InvokeConnectorRequest) GetTool() string`

GetTool returns the Tool field if non-nil, zero value otherwise.

### GetToolOk

`func (o *InvokeConnectorRequest) GetToolOk() (*string, bool)`

GetToolOk returns a tuple with the Tool field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTool

`func (o *InvokeConnectorRequest) SetTool(v string)`

SetTool sets Tool field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
