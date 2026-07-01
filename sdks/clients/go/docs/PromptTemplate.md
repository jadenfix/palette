# PromptTemplate

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Body** | **string** |  | 
**Tags** | **[]string** |  | 
**Variables** | [**[]PromptVariable**](PromptVariable.md) |  | 

## Methods

### NewPromptTemplate

`func NewPromptTemplate(body string, tags []string, variables []PromptVariable, ) *PromptTemplate`

NewPromptTemplate instantiates a new PromptTemplate object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewPromptTemplateWithDefaults

`func NewPromptTemplateWithDefaults() *PromptTemplate`

NewPromptTemplateWithDefaults instantiates a new PromptTemplate object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetBody

`func (o *PromptTemplate) GetBody() string`

GetBody returns the Body field if non-nil, zero value otherwise.

### GetBodyOk

`func (o *PromptTemplate) GetBodyOk() (*string, bool)`

GetBodyOk returns a tuple with the Body field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBody

`func (o *PromptTemplate) SetBody(v string)`

SetBody sets Body field to given value.


### GetTags

`func (o *PromptTemplate) GetTags() []string`

GetTags returns the Tags field if non-nil, zero value otherwise.

### GetTagsOk

`func (o *PromptTemplate) GetTagsOk() (*[]string, bool)`

GetTagsOk returns a tuple with the Tags field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTags

`func (o *PromptTemplate) SetTags(v []string)`

SetTags sets Tags field to given value.


### GetVariables

`func (o *PromptTemplate) GetVariables() []PromptVariable`

GetVariables returns the Variables field if non-nil, zero value otherwise.

### GetVariablesOk

`func (o *PromptTemplate) GetVariablesOk() (*[]PromptVariable, bool)`

GetVariablesOk returns a tuple with the Variables field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetVariables

`func (o *PromptTemplate) SetVariables(v []PromptVariable)`

SetVariables sets Variables field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


