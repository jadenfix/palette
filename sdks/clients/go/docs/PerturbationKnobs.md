# PerturbationKnobs

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AuthFailure** | **bool** | Force an auth failure on a dependency. |
**ContradictorySource** | **bool** | Inject a contradictory context source. |
**PromptInjection** | **bool** | Attempt a prompt-injection payload. |
**StaleSource** | **bool** | Serve a stale version of a context source. |
**Timeout** | **bool** | Force a timeout on a dependency. |
**ToolSchemaMismatch** | **bool** | Present a tool whose schema mismatches expectations. |

## Methods

### NewPerturbationKnobs

`func NewPerturbationKnobs(authFailure bool, contradictorySource bool, promptInjection bool, staleSource bool, timeout bool, toolSchemaMismatch bool, ) *PerturbationKnobs`

NewPerturbationKnobs instantiates a new PerturbationKnobs object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewPerturbationKnobsWithDefaults

`func NewPerturbationKnobsWithDefaults() *PerturbationKnobs`

NewPerturbationKnobsWithDefaults instantiates a new PerturbationKnobs object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAuthFailure

`func (o *PerturbationKnobs) GetAuthFailure() bool`

GetAuthFailure returns the AuthFailure field if non-nil, zero value otherwise.

### GetAuthFailureOk

`func (o *PerturbationKnobs) GetAuthFailureOk() (*bool, bool)`

GetAuthFailureOk returns a tuple with the AuthFailure field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAuthFailure

`func (o *PerturbationKnobs) SetAuthFailure(v bool)`

SetAuthFailure sets AuthFailure field to given value.


### GetContradictorySource

`func (o *PerturbationKnobs) GetContradictorySource() bool`

GetContradictorySource returns the ContradictorySource field if non-nil, zero value otherwise.

### GetContradictorySourceOk

`func (o *PerturbationKnobs) GetContradictorySourceOk() (*bool, bool)`

GetContradictorySourceOk returns a tuple with the ContradictorySource field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetContradictorySource

`func (o *PerturbationKnobs) SetContradictorySource(v bool)`

SetContradictorySource sets ContradictorySource field to given value.


### GetPromptInjection

`func (o *PerturbationKnobs) GetPromptInjection() bool`

GetPromptInjection returns the PromptInjection field if non-nil, zero value otherwise.

### GetPromptInjectionOk

`func (o *PerturbationKnobs) GetPromptInjectionOk() (*bool, bool)`

GetPromptInjectionOk returns a tuple with the PromptInjection field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPromptInjection

`func (o *PerturbationKnobs) SetPromptInjection(v bool)`

SetPromptInjection sets PromptInjection field to given value.


### GetStaleSource

`func (o *PerturbationKnobs) GetStaleSource() bool`

GetStaleSource returns the StaleSource field if non-nil, zero value otherwise.

### GetStaleSourceOk

`func (o *PerturbationKnobs) GetStaleSourceOk() (*bool, bool)`

GetStaleSourceOk returns a tuple with the StaleSource field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetStaleSource

`func (o *PerturbationKnobs) SetStaleSource(v bool)`

SetStaleSource sets StaleSource field to given value.


### GetTimeout

`func (o *PerturbationKnobs) GetTimeout() bool`

GetTimeout returns the Timeout field if non-nil, zero value otherwise.

### GetTimeoutOk

`func (o *PerturbationKnobs) GetTimeoutOk() (*bool, bool)`

GetTimeoutOk returns a tuple with the Timeout field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTimeout

`func (o *PerturbationKnobs) SetTimeout(v bool)`

SetTimeout sets Timeout field to given value.


### GetToolSchemaMismatch

`func (o *PerturbationKnobs) GetToolSchemaMismatch() bool`

GetToolSchemaMismatch returns the ToolSchemaMismatch field if non-nil, zero value otherwise.

### GetToolSchemaMismatchOk

`func (o *PerturbationKnobs) GetToolSchemaMismatchOk() (*bool, bool)`

GetToolSchemaMismatchOk returns a tuple with the ToolSchemaMismatch field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetToolSchemaMismatch

`func (o *PerturbationKnobs) SetToolSchemaMismatch(v bool)`

SetToolSchemaMismatch sets ToolSchemaMismatch field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
