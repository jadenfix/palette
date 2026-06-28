# ChatCompletionChoice

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**FinishReason** | Pointer to **NullableString** |  | [optional] 
**Index** | **int32** |  | 
**Message** | [**ChatMessage**](ChatMessage.md) |  | 

## Methods

### NewChatCompletionChoice

`func NewChatCompletionChoice(index int32, message ChatMessage, ) *ChatCompletionChoice`

NewChatCompletionChoice instantiates a new ChatCompletionChoice object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewChatCompletionChoiceWithDefaults

`func NewChatCompletionChoiceWithDefaults() *ChatCompletionChoice`

NewChatCompletionChoiceWithDefaults instantiates a new ChatCompletionChoice object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetFinishReason

`func (o *ChatCompletionChoice) GetFinishReason() string`

GetFinishReason returns the FinishReason field if non-nil, zero value otherwise.

### GetFinishReasonOk

`func (o *ChatCompletionChoice) GetFinishReasonOk() (*string, bool)`

GetFinishReasonOk returns a tuple with the FinishReason field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetFinishReason

`func (o *ChatCompletionChoice) SetFinishReason(v string)`

SetFinishReason sets FinishReason field to given value.

### HasFinishReason

`func (o *ChatCompletionChoice) HasFinishReason() bool`

HasFinishReason returns a boolean if a field has been set.

### SetFinishReasonNil

`func (o *ChatCompletionChoice) SetFinishReasonNil(b bool)`

 SetFinishReasonNil sets the value for FinishReason to be an explicit nil

### UnsetFinishReason
`func (o *ChatCompletionChoice) UnsetFinishReason()`

UnsetFinishReason ensures that no value is present for FinishReason, not even an explicit nil
### GetIndex

`func (o *ChatCompletionChoice) GetIndex() int32`

GetIndex returns the Index field if non-nil, zero value otherwise.

### GetIndexOk

`func (o *ChatCompletionChoice) GetIndexOk() (*int32, bool)`

GetIndexOk returns a tuple with the Index field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetIndex

`func (o *ChatCompletionChoice) SetIndex(v int32)`

SetIndex sets Index field to given value.


### GetMessage

`func (o *ChatCompletionChoice) GetMessage() ChatMessage`

GetMessage returns the Message field if non-nil, zero value otherwise.

### GetMessageOk

`func (o *ChatCompletionChoice) GetMessageOk() (*ChatMessage, bool)`

GetMessageOk returns a tuple with the Message field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMessage

`func (o *ChatCompletionChoice) SetMessage(v ChatMessage)`

SetMessage sets Message field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


