# ModelRouting

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Kind** | **string** |  | 
**ProviderSecretId** | **string** |  | 
**DefaultModel** | [**ModelRef**](ModelRef.md) |  | 

## Methods

### NewModelRouting

`func NewModelRouting(kind string, providerSecretId string, defaultModel ModelRef, ) *ModelRouting`

NewModelRouting instantiates a new ModelRouting object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewModelRoutingWithDefaults

`func NewModelRoutingWithDefaults() *ModelRouting`

NewModelRoutingWithDefaults instantiates a new ModelRouting object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetKind

`func (o *ModelRouting) GetKind() string`

GetKind returns the Kind field if non-nil, zero value otherwise.

### GetKindOk

`func (o *ModelRouting) GetKindOk() (*string, bool)`

GetKindOk returns a tuple with the Kind field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetKind

`func (o *ModelRouting) SetKind(v string)`

SetKind sets Kind field to given value.


### GetProviderSecretId

`func (o *ModelRouting) GetProviderSecretId() string`

GetProviderSecretId returns the ProviderSecretId field if non-nil, zero value otherwise.

### GetProviderSecretIdOk

`func (o *ModelRouting) GetProviderSecretIdOk() (*string, bool)`

GetProviderSecretIdOk returns a tuple with the ProviderSecretId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProviderSecretId

`func (o *ModelRouting) SetProviderSecretId(v string)`

SetProviderSecretId sets ProviderSecretId field to given value.


### GetDefaultModel

`func (o *ModelRouting) GetDefaultModel() ModelRef`

GetDefaultModel returns the DefaultModel field if non-nil, zero value otherwise.

### GetDefaultModelOk

`func (o *ModelRouting) GetDefaultModelOk() (*ModelRef, bool)`

GetDefaultModelOk returns a tuple with the DefaultModel field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDefaultModel

`func (o *ModelRouting) SetDefaultModel(v ModelRef)`

SetDefaultModel sets DefaultModel field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


