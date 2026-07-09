# EvaluateAlertRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Input** | [**AlertInput**](AlertInput.md) |  |
**Policy** | [**AlertPolicy**](AlertPolicy.md) |  |

## Methods

### NewEvaluateAlertRequest

`func NewEvaluateAlertRequest(input AlertInput, policy AlertPolicy, ) *EvaluateAlertRequest`

NewEvaluateAlertRequest instantiates a new EvaluateAlertRequest object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewEvaluateAlertRequestWithDefaults

`func NewEvaluateAlertRequestWithDefaults() *EvaluateAlertRequest`

NewEvaluateAlertRequestWithDefaults instantiates a new EvaluateAlertRequest object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetInput

`func (o *EvaluateAlertRequest) GetInput() AlertInput`

GetInput returns the Input field if non-nil, zero value otherwise.

### GetInputOk

`func (o *EvaluateAlertRequest) GetInputOk() (*AlertInput, bool)`

GetInputOk returns a tuple with the Input field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetInput

`func (o *EvaluateAlertRequest) SetInput(v AlertInput)`

SetInput sets Input field to given value.


### GetPolicy

`func (o *EvaluateAlertRequest) GetPolicy() AlertPolicy`

GetPolicy returns the Policy field if non-nil, zero value otherwise.

### GetPolicyOk

`func (o *EvaluateAlertRequest) GetPolicyOk() (*AlertPolicy, bool)`

GetPolicyOk returns a tuple with the Policy field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPolicy

`func (o *EvaluateAlertRequest) SetPolicy(v AlertPolicy)`

SetPolicy sets Policy field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
