# GatewayOutcome

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Attempts** | **int32** |  | 
**Cached** | **bool** |  | 
**Cost** | [**Money**](Money.md) |  | 
**Model** | [**ModelRef**](ModelRef.md) |  | 
**Provider** | **string** |  | 
**RequestHash** | **string** |  | 
**Response** | [**ChatCompletionResponse**](ChatCompletionResponse.md) |  | 
**Tokens** | [**TokenCounts**](TokenCounts.md) |  | 

## Methods

### NewGatewayOutcome

`func NewGatewayOutcome(attempts int32, cached bool, cost Money, model ModelRef, provider string, requestHash string, response ChatCompletionResponse, tokens TokenCounts, ) *GatewayOutcome`

NewGatewayOutcome instantiates a new GatewayOutcome object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewGatewayOutcomeWithDefaults

`func NewGatewayOutcomeWithDefaults() *GatewayOutcome`

NewGatewayOutcomeWithDefaults instantiates a new GatewayOutcome object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAttempts

`func (o *GatewayOutcome) GetAttempts() int32`

GetAttempts returns the Attempts field if non-nil, zero value otherwise.

### GetAttemptsOk

`func (o *GatewayOutcome) GetAttemptsOk() (*int32, bool)`

GetAttemptsOk returns a tuple with the Attempts field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAttempts

`func (o *GatewayOutcome) SetAttempts(v int32)`

SetAttempts sets Attempts field to given value.


### GetCached

`func (o *GatewayOutcome) GetCached() bool`

GetCached returns the Cached field if non-nil, zero value otherwise.

### GetCachedOk

`func (o *GatewayOutcome) GetCachedOk() (*bool, bool)`

GetCachedOk returns a tuple with the Cached field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCached

`func (o *GatewayOutcome) SetCached(v bool)`

SetCached sets Cached field to given value.


### GetCost

`func (o *GatewayOutcome) GetCost() Money`

GetCost returns the Cost field if non-nil, zero value otherwise.

### GetCostOk

`func (o *GatewayOutcome) GetCostOk() (*Money, bool)`

GetCostOk returns a tuple with the Cost field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCost

`func (o *GatewayOutcome) SetCost(v Money)`

SetCost sets Cost field to given value.


### GetModel

`func (o *GatewayOutcome) GetModel() ModelRef`

GetModel returns the Model field if non-nil, zero value otherwise.

### GetModelOk

`func (o *GatewayOutcome) GetModelOk() (*ModelRef, bool)`

GetModelOk returns a tuple with the Model field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetModel

`func (o *GatewayOutcome) SetModel(v ModelRef)`

SetModel sets Model field to given value.


### GetProvider

`func (o *GatewayOutcome) GetProvider() string`

GetProvider returns the Provider field if non-nil, zero value otherwise.

### GetProviderOk

`func (o *GatewayOutcome) GetProviderOk() (*string, bool)`

GetProviderOk returns a tuple with the Provider field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProvider

`func (o *GatewayOutcome) SetProvider(v string)`

SetProvider sets Provider field to given value.


### GetRequestHash

`func (o *GatewayOutcome) GetRequestHash() string`

GetRequestHash returns the RequestHash field if non-nil, zero value otherwise.

### GetRequestHashOk

`func (o *GatewayOutcome) GetRequestHashOk() (*string, bool)`

GetRequestHashOk returns a tuple with the RequestHash field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRequestHash

`func (o *GatewayOutcome) SetRequestHash(v string)`

SetRequestHash sets RequestHash field to given value.


### GetResponse

`func (o *GatewayOutcome) GetResponse() ChatCompletionResponse`

GetResponse returns the Response field if non-nil, zero value otherwise.

### GetResponseOk

`func (o *GatewayOutcome) GetResponseOk() (*ChatCompletionResponse, bool)`

GetResponseOk returns a tuple with the Response field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetResponse

`func (o *GatewayOutcome) SetResponse(v ChatCompletionResponse)`

SetResponse sets Response field to given value.


### GetTokens

`func (o *GatewayOutcome) GetTokens() TokenCounts`

GetTokens returns the Tokens field if non-nil, zero value otherwise.

### GetTokensOk

`func (o *GatewayOutcome) GetTokensOk() (*TokenCounts, bool)`

GetTokensOk returns a tuple with the Tokens field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTokens

`func (o *GatewayOutcome) SetTokens(v TokenCounts)`

SetTokens sets Tokens field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


