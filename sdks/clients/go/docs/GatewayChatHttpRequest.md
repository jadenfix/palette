# GatewayChatHttpRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**BudgetMicros** | Pointer to **NullableInt64** | Per-tenant budget ceiling in micros; defaults when omitted. | [optional] 
**Completion** | [**ChatCompletionRequest**](ChatCompletionRequest.md) | The OpenAI-compatible chat completion request. | 
**Routing** | Pointer to [**[]ModelRouting**](ModelRouting.md) | Ordered failover list of credential policies (BYOK or managed). | [optional] 

## Methods

### NewGatewayChatHttpRequest

`func NewGatewayChatHttpRequest(completion ChatCompletionRequest, ) *GatewayChatHttpRequest`

NewGatewayChatHttpRequest instantiates a new GatewayChatHttpRequest object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewGatewayChatHttpRequestWithDefaults

`func NewGatewayChatHttpRequestWithDefaults() *GatewayChatHttpRequest`

NewGatewayChatHttpRequestWithDefaults instantiates a new GatewayChatHttpRequest object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetBudgetMicros

`func (o *GatewayChatHttpRequest) GetBudgetMicros() int64`

GetBudgetMicros returns the BudgetMicros field if non-nil, zero value otherwise.

### GetBudgetMicrosOk

`func (o *GatewayChatHttpRequest) GetBudgetMicrosOk() (*int64, bool)`

GetBudgetMicrosOk returns a tuple with the BudgetMicros field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBudgetMicros

`func (o *GatewayChatHttpRequest) SetBudgetMicros(v int64)`

SetBudgetMicros sets BudgetMicros field to given value.

### HasBudgetMicros

`func (o *GatewayChatHttpRequest) HasBudgetMicros() bool`

HasBudgetMicros returns a boolean if a field has been set.

### SetBudgetMicrosNil

`func (o *GatewayChatHttpRequest) SetBudgetMicrosNil(b bool)`

 SetBudgetMicrosNil sets the value for BudgetMicros to be an explicit nil

### UnsetBudgetMicros
`func (o *GatewayChatHttpRequest) UnsetBudgetMicros()`

UnsetBudgetMicros ensures that no value is present for BudgetMicros, not even an explicit nil
### GetCompletion

`func (o *GatewayChatHttpRequest) GetCompletion() ChatCompletionRequest`

GetCompletion returns the Completion field if non-nil, zero value otherwise.

### GetCompletionOk

`func (o *GatewayChatHttpRequest) GetCompletionOk() (*ChatCompletionRequest, bool)`

GetCompletionOk returns a tuple with the Completion field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCompletion

`func (o *GatewayChatHttpRequest) SetCompletion(v ChatCompletionRequest)`

SetCompletion sets Completion field to given value.


### GetRouting

`func (o *GatewayChatHttpRequest) GetRouting() []ModelRouting`

GetRouting returns the Routing field if non-nil, zero value otherwise.

### GetRoutingOk

`func (o *GatewayChatHttpRequest) GetRoutingOk() (*[]ModelRouting, bool)`

GetRoutingOk returns a tuple with the Routing field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRouting

`func (o *GatewayChatHttpRequest) SetRouting(v []ModelRouting)`

SetRouting sets Routing field to given value.

### HasRouting

`func (o *GatewayChatHttpRequest) HasRouting() bool`

HasRouting returns a boolean if a field has been set.


[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


