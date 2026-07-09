# CaseOutputOverrideRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**CaseId** | **string** |  |
**Output** | **interface{}** |  |
**Trace** | Pointer to **interface{}** |  | [optional]

## Methods

### NewCaseOutputOverrideRequest

`func NewCaseOutputOverrideRequest(caseId string, output interface{}, ) *CaseOutputOverrideRequest`

NewCaseOutputOverrideRequest instantiates a new CaseOutputOverrideRequest object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewCaseOutputOverrideRequestWithDefaults

`func NewCaseOutputOverrideRequestWithDefaults() *CaseOutputOverrideRequest`

NewCaseOutputOverrideRequestWithDefaults instantiates a new CaseOutputOverrideRequest object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetCaseId

`func (o *CaseOutputOverrideRequest) GetCaseId() string`

GetCaseId returns the CaseId field if non-nil, zero value otherwise.

### GetCaseIdOk

`func (o *CaseOutputOverrideRequest) GetCaseIdOk() (*string, bool)`

GetCaseIdOk returns a tuple with the CaseId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCaseId

`func (o *CaseOutputOverrideRequest) SetCaseId(v string)`

SetCaseId sets CaseId field to given value.


### GetOutput

`func (o *CaseOutputOverrideRequest) GetOutput() interface{}`

GetOutput returns the Output field if non-nil, zero value otherwise.

### GetOutputOk

`func (o *CaseOutputOverrideRequest) GetOutputOk() (*interface{}, bool)`

GetOutputOk returns a tuple with the Output field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetOutput

`func (o *CaseOutputOverrideRequest) SetOutput(v interface{})`

SetOutput sets Output field to given value.


### SetOutputNil

`func (o *CaseOutputOverrideRequest) SetOutputNil(b bool)`

 SetOutputNil sets the value for Output to be an explicit nil

### UnsetOutput
`func (o *CaseOutputOverrideRequest) UnsetOutput()`

UnsetOutput ensures that no value is present for Output, not even an explicit nil
### GetTrace

`func (o *CaseOutputOverrideRequest) GetTrace() interface{}`

GetTrace returns the Trace field if non-nil, zero value otherwise.

### GetTraceOk

`func (o *CaseOutputOverrideRequest) GetTraceOk() (*interface{}, bool)`

GetTraceOk returns a tuple with the Trace field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTrace

`func (o *CaseOutputOverrideRequest) SetTrace(v interface{})`

SetTrace sets Trace field to given value.

### HasTrace

`func (o *CaseOutputOverrideRequest) HasTrace() bool`

HasTrace returns a boolean if a field has been set.

### SetTraceNil

`func (o *CaseOutputOverrideRequest) SetTraceNil(b bool)`

 SetTraceNil sets the value for Trace to be an explicit nil

### UnsetTrace
`func (o *CaseOutputOverrideRequest) UnsetTrace()`

UnsetTrace ensures that no value is present for Trace, not even an explicit nil

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
