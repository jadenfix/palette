# ToolExecution

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Data** | Pointer to **map[string]interface{}** | Tool output payload (shape is tool-specific). | [optional] 
**Error** | Pointer to **NullableString** | Error message when &#x60;successful&#x60; is false. | [optional] 
**LogId** | Pointer to **NullableString** | Composio execution log id, for tracing. | [optional] 
**Successful** | **bool** | Whether the tool reported success. | 

## Methods

### NewToolExecution

`func NewToolExecution(successful bool, ) *ToolExecution`

NewToolExecution instantiates a new ToolExecution object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewToolExecutionWithDefaults

`func NewToolExecutionWithDefaults() *ToolExecution`

NewToolExecutionWithDefaults instantiates a new ToolExecution object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetData

`func (o *ToolExecution) GetData() map[string]interface{}`

GetData returns the Data field if non-nil, zero value otherwise.

### GetDataOk

`func (o *ToolExecution) GetDataOk() (*map[string]interface{}, bool)`

GetDataOk returns a tuple with the Data field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetData

`func (o *ToolExecution) SetData(v map[string]interface{})`

SetData sets Data field to given value.

### HasData

`func (o *ToolExecution) HasData() bool`

HasData returns a boolean if a field has been set.

### GetError

`func (o *ToolExecution) GetError() string`

GetError returns the Error field if non-nil, zero value otherwise.

### GetErrorOk

`func (o *ToolExecution) GetErrorOk() (*string, bool)`

GetErrorOk returns a tuple with the Error field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetError

`func (o *ToolExecution) SetError(v string)`

SetError sets Error field to given value.

### HasError

`func (o *ToolExecution) HasError() bool`

HasError returns a boolean if a field has been set.

### SetErrorNil

`func (o *ToolExecution) SetErrorNil(b bool)`

 SetErrorNil sets the value for Error to be an explicit nil

### UnsetError
`func (o *ToolExecution) UnsetError()`

UnsetError ensures that no value is present for Error, not even an explicit nil
### GetLogId

`func (o *ToolExecution) GetLogId() string`

GetLogId returns the LogId field if non-nil, zero value otherwise.

### GetLogIdOk

`func (o *ToolExecution) GetLogIdOk() (*string, bool)`

GetLogIdOk returns a tuple with the LogId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLogId

`func (o *ToolExecution) SetLogId(v string)`

SetLogId sets LogId field to given value.

### HasLogId

`func (o *ToolExecution) HasLogId() bool`

HasLogId returns a boolean if a field has been set.

### SetLogIdNil

`func (o *ToolExecution) SetLogIdNil(b bool)`

 SetLogIdNil sets the value for LogId to be an explicit nil

### UnsetLogId
`func (o *ToolExecution) UnsetLogId()`

UnsetLogId ensures that no value is present for LogId, not even an explicit nil
### GetSuccessful

`func (o *ToolExecution) GetSuccessful() bool`

GetSuccessful returns the Successful field if non-nil, zero value otherwise.

### GetSuccessfulOk

`func (o *ToolExecution) GetSuccessfulOk() (*bool, bool)`

GetSuccessfulOk returns a tuple with the Successful field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSuccessful

`func (o *ToolExecution) SetSuccessful(v bool)`

SetSuccessful sets Successful field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


