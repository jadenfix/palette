# ChatCompletionRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**MaxTokens** | Pointer to **NullableInt64** |  | [optional] 
**Messages** | [**[]ChatMessage**](ChatMessage.md) |  | 
**Model** | **string** |  | 
**ReasoningEffort** | Pointer to **NullableString** | OpenAI-style reasoning effort hint (&#x60;low&#x60; | &#x60;medium&#x60; | &#x60;high&#x60;), forwarded to providers that understand it. | [optional] 
**Temperature** | Pointer to **NullableFloat64** |  | [optional] 
**TopP** | Pointer to **NullableFloat64** |  | [optional] 

## Methods

### NewChatCompletionRequest

`func NewChatCompletionRequest(messages []ChatMessage, model string, ) *ChatCompletionRequest`

NewChatCompletionRequest instantiates a new ChatCompletionRequest object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewChatCompletionRequestWithDefaults

`func NewChatCompletionRequestWithDefaults() *ChatCompletionRequest`

NewChatCompletionRequestWithDefaults instantiates a new ChatCompletionRequest object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetMaxTokens

`func (o *ChatCompletionRequest) GetMaxTokens() int64`

GetMaxTokens returns the MaxTokens field if non-nil, zero value otherwise.

### GetMaxTokensOk

`func (o *ChatCompletionRequest) GetMaxTokensOk() (*int64, bool)`

GetMaxTokensOk returns a tuple with the MaxTokens field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMaxTokens

`func (o *ChatCompletionRequest) SetMaxTokens(v int64)`

SetMaxTokens sets MaxTokens field to given value.

### HasMaxTokens

`func (o *ChatCompletionRequest) HasMaxTokens() bool`

HasMaxTokens returns a boolean if a field has been set.

### SetMaxTokensNil

`func (o *ChatCompletionRequest) SetMaxTokensNil(b bool)`

 SetMaxTokensNil sets the value for MaxTokens to be an explicit nil

### UnsetMaxTokens
`func (o *ChatCompletionRequest) UnsetMaxTokens()`

UnsetMaxTokens ensures that no value is present for MaxTokens, not even an explicit nil
### GetMessages

`func (o *ChatCompletionRequest) GetMessages() []ChatMessage`

GetMessages returns the Messages field if non-nil, zero value otherwise.

### GetMessagesOk

`func (o *ChatCompletionRequest) GetMessagesOk() (*[]ChatMessage, bool)`

GetMessagesOk returns a tuple with the Messages field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMessages

`func (o *ChatCompletionRequest) SetMessages(v []ChatMessage)`

SetMessages sets Messages field to given value.


### GetModel

`func (o *ChatCompletionRequest) GetModel() string`

GetModel returns the Model field if non-nil, zero value otherwise.

### GetModelOk

`func (o *ChatCompletionRequest) GetModelOk() (*string, bool)`

GetModelOk returns a tuple with the Model field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetModel

`func (o *ChatCompletionRequest) SetModel(v string)`

SetModel sets Model field to given value.


### GetReasoningEffort

`func (o *ChatCompletionRequest) GetReasoningEffort() string`

GetReasoningEffort returns the ReasoningEffort field if non-nil, zero value otherwise.

### GetReasoningEffortOk

`func (o *ChatCompletionRequest) GetReasoningEffortOk() (*string, bool)`

GetReasoningEffortOk returns a tuple with the ReasoningEffort field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetReasoningEffort

`func (o *ChatCompletionRequest) SetReasoningEffort(v string)`

SetReasoningEffort sets ReasoningEffort field to given value.

### HasReasoningEffort

`func (o *ChatCompletionRequest) HasReasoningEffort() bool`

HasReasoningEffort returns a boolean if a field has been set.

### SetReasoningEffortNil

`func (o *ChatCompletionRequest) SetReasoningEffortNil(b bool)`

 SetReasoningEffortNil sets the value for ReasoningEffort to be an explicit nil

### UnsetReasoningEffort
`func (o *ChatCompletionRequest) UnsetReasoningEffort()`

UnsetReasoningEffort ensures that no value is present for ReasoningEffort, not even an explicit nil
### GetTemperature

`func (o *ChatCompletionRequest) GetTemperature() float64`

GetTemperature returns the Temperature field if non-nil, zero value otherwise.

### GetTemperatureOk

`func (o *ChatCompletionRequest) GetTemperatureOk() (*float64, bool)`

GetTemperatureOk returns a tuple with the Temperature field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTemperature

`func (o *ChatCompletionRequest) SetTemperature(v float64)`

SetTemperature sets Temperature field to given value.

### HasTemperature

`func (o *ChatCompletionRequest) HasTemperature() bool`

HasTemperature returns a boolean if a field has been set.

### SetTemperatureNil

`func (o *ChatCompletionRequest) SetTemperatureNil(b bool)`

 SetTemperatureNil sets the value for Temperature to be an explicit nil

### UnsetTemperature
`func (o *ChatCompletionRequest) UnsetTemperature()`

UnsetTemperature ensures that no value is present for Temperature, not even an explicit nil
### GetTopP

`func (o *ChatCompletionRequest) GetTopP() float64`

GetTopP returns the TopP field if non-nil, zero value otherwise.

### GetTopPOk

`func (o *ChatCompletionRequest) GetTopPOk() (*float64, bool)`

GetTopPOk returns a tuple with the TopP field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTopP

`func (o *ChatCompletionRequest) SetTopP(v float64)`

SetTopP sets TopP field to given value.

### HasTopP

`func (o *ChatCompletionRequest) HasTopP() bool`

HasTopP returns a boolean if a field has been set.

### SetTopPNil

`func (o *ChatCompletionRequest) SetTopPNil(b bool)`

 SetTopPNil sets the value for TopP to be an explicit nil

### UnsetTopP
`func (o *ChatCompletionRequest) UnsetTopP()`

UnsetTopP ensures that no value is present for TopP, not even an explicit nil

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


