# ChatCompletionUsage

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**CompletionTokens** | **int64** |  | 
**PromptTokens** | **int64** |  | 
**TotalTokens** | **int64** |  | 

## Methods

### NewChatCompletionUsage

`func NewChatCompletionUsage(completionTokens int64, promptTokens int64, totalTokens int64, ) *ChatCompletionUsage`

NewChatCompletionUsage instantiates a new ChatCompletionUsage object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewChatCompletionUsageWithDefaults

`func NewChatCompletionUsageWithDefaults() *ChatCompletionUsage`

NewChatCompletionUsageWithDefaults instantiates a new ChatCompletionUsage object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetCompletionTokens

`func (o *ChatCompletionUsage) GetCompletionTokens() int64`

GetCompletionTokens returns the CompletionTokens field if non-nil, zero value otherwise.

### GetCompletionTokensOk

`func (o *ChatCompletionUsage) GetCompletionTokensOk() (*int64, bool)`

GetCompletionTokensOk returns a tuple with the CompletionTokens field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCompletionTokens

`func (o *ChatCompletionUsage) SetCompletionTokens(v int64)`

SetCompletionTokens sets CompletionTokens field to given value.


### GetPromptTokens

`func (o *ChatCompletionUsage) GetPromptTokens() int64`

GetPromptTokens returns the PromptTokens field if non-nil, zero value otherwise.

### GetPromptTokensOk

`func (o *ChatCompletionUsage) GetPromptTokensOk() (*int64, bool)`

GetPromptTokensOk returns a tuple with the PromptTokens field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPromptTokens

`func (o *ChatCompletionUsage) SetPromptTokens(v int64)`

SetPromptTokens sets PromptTokens field to given value.


### GetTotalTokens

`func (o *ChatCompletionUsage) GetTotalTokens() int64`

GetTotalTokens returns the TotalTokens field if non-nil, zero value otherwise.

### GetTotalTokensOk

`func (o *ChatCompletionUsage) GetTotalTokensOk() (*int64, bool)`

GetTotalTokensOk returns a tuple with the TotalTokens field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTotalTokens

`func (o *ChatCompletionUsage) SetTotalTokens(v int64)`

SetTotalTokens sets TotalTokens field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


