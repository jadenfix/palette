# ChatCompletionResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Choices** | [**[]ChatCompletionChoice**](ChatCompletionChoice.md) |  | 
**Created** | **int64** |  | 
**Id** | **string** |  | 
**Model** | **string** |  | 
**Object** | **string** |  | 
**Usage** | [**ChatCompletionUsage**](ChatCompletionUsage.md) |  | 

## Methods

### NewChatCompletionResponse

`func NewChatCompletionResponse(choices []ChatCompletionChoice, created int64, id string, model string, object string, usage ChatCompletionUsage, ) *ChatCompletionResponse`

NewChatCompletionResponse instantiates a new ChatCompletionResponse object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewChatCompletionResponseWithDefaults

`func NewChatCompletionResponseWithDefaults() *ChatCompletionResponse`

NewChatCompletionResponseWithDefaults instantiates a new ChatCompletionResponse object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetChoices

`func (o *ChatCompletionResponse) GetChoices() []ChatCompletionChoice`

GetChoices returns the Choices field if non-nil, zero value otherwise.

### GetChoicesOk

`func (o *ChatCompletionResponse) GetChoicesOk() (*[]ChatCompletionChoice, bool)`

GetChoicesOk returns a tuple with the Choices field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetChoices

`func (o *ChatCompletionResponse) SetChoices(v []ChatCompletionChoice)`

SetChoices sets Choices field to given value.


### GetCreated

`func (o *ChatCompletionResponse) GetCreated() int64`

GetCreated returns the Created field if non-nil, zero value otherwise.

### GetCreatedOk

`func (o *ChatCompletionResponse) GetCreatedOk() (*int64, bool)`

GetCreatedOk returns a tuple with the Created field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreated

`func (o *ChatCompletionResponse) SetCreated(v int64)`

SetCreated sets Created field to given value.


### GetId

`func (o *ChatCompletionResponse) GetId() string`

GetId returns the Id field if non-nil, zero value otherwise.

### GetIdOk

`func (o *ChatCompletionResponse) GetIdOk() (*string, bool)`

GetIdOk returns a tuple with the Id field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetId

`func (o *ChatCompletionResponse) SetId(v string)`

SetId sets Id field to given value.


### GetModel

`func (o *ChatCompletionResponse) GetModel() string`

GetModel returns the Model field if non-nil, zero value otherwise.

### GetModelOk

`func (o *ChatCompletionResponse) GetModelOk() (*string, bool)`

GetModelOk returns a tuple with the Model field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetModel

`func (o *ChatCompletionResponse) SetModel(v string)`

SetModel sets Model field to given value.


### GetObject

`func (o *ChatCompletionResponse) GetObject() string`

GetObject returns the Object field if non-nil, zero value otherwise.

### GetObjectOk

`func (o *ChatCompletionResponse) GetObjectOk() (*string, bool)`

GetObjectOk returns a tuple with the Object field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetObject

`func (o *ChatCompletionResponse) SetObject(v string)`

SetObject sets Object field to given value.


### GetUsage

`func (o *ChatCompletionResponse) GetUsage() ChatCompletionUsage`

GetUsage returns the Usage field if non-nil, zero value otherwise.

### GetUsageOk

`func (o *ChatCompletionResponse) GetUsageOk() (*ChatCompletionUsage, bool)`

GetUsageOk returns a tuple with the Usage field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetUsage

`func (o *ChatCompletionResponse) SetUsage(v ChatCompletionUsage)`

SetUsage sets Usage field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


