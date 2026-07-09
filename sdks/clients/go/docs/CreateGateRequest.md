# CreateGateRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**DatasetId** | Pointer to **NullableString** |  | [optional]
**EvaluatorVersionId** | Pointer to **NullableString** |  | [optional]
**GateId** | **string** |  |
**InconclusivePolicy** | Pointer to [**NullableInconclusivePolicy**](InconclusivePolicy.md) |  | [optional]
**Name** | **string** |  |

## Methods

### NewCreateGateRequest

`func NewCreateGateRequest(gateId string, name string, ) *CreateGateRequest`

NewCreateGateRequest instantiates a new CreateGateRequest object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewCreateGateRequestWithDefaults

`func NewCreateGateRequestWithDefaults() *CreateGateRequest`

NewCreateGateRequestWithDefaults instantiates a new CreateGateRequest object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetDatasetId

`func (o *CreateGateRequest) GetDatasetId() string`

GetDatasetId returns the DatasetId field if non-nil, zero value otherwise.

### GetDatasetIdOk

`func (o *CreateGateRequest) GetDatasetIdOk() (*string, bool)`

GetDatasetIdOk returns a tuple with the DatasetId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDatasetId

`func (o *CreateGateRequest) SetDatasetId(v string)`

SetDatasetId sets DatasetId field to given value.

### HasDatasetId

`func (o *CreateGateRequest) HasDatasetId() bool`

HasDatasetId returns a boolean if a field has been set.

### SetDatasetIdNil

`func (o *CreateGateRequest) SetDatasetIdNil(b bool)`

 SetDatasetIdNil sets the value for DatasetId to be an explicit nil

### UnsetDatasetId
`func (o *CreateGateRequest) UnsetDatasetId()`

UnsetDatasetId ensures that no value is present for DatasetId, not even an explicit nil
### GetEvaluatorVersionId

`func (o *CreateGateRequest) GetEvaluatorVersionId() string`

GetEvaluatorVersionId returns the EvaluatorVersionId field if non-nil, zero value otherwise.

### GetEvaluatorVersionIdOk

`func (o *CreateGateRequest) GetEvaluatorVersionIdOk() (*string, bool)`

GetEvaluatorVersionIdOk returns a tuple with the EvaluatorVersionId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEvaluatorVersionId

`func (o *CreateGateRequest) SetEvaluatorVersionId(v string)`

SetEvaluatorVersionId sets EvaluatorVersionId field to given value.

### HasEvaluatorVersionId

`func (o *CreateGateRequest) HasEvaluatorVersionId() bool`

HasEvaluatorVersionId returns a boolean if a field has been set.

### SetEvaluatorVersionIdNil

`func (o *CreateGateRequest) SetEvaluatorVersionIdNil(b bool)`

 SetEvaluatorVersionIdNil sets the value for EvaluatorVersionId to be an explicit nil

### UnsetEvaluatorVersionId
`func (o *CreateGateRequest) UnsetEvaluatorVersionId()`

UnsetEvaluatorVersionId ensures that no value is present for EvaluatorVersionId, not even an explicit nil
### GetGateId

`func (o *CreateGateRequest) GetGateId() string`

GetGateId returns the GateId field if non-nil, zero value otherwise.

### GetGateIdOk

`func (o *CreateGateRequest) GetGateIdOk() (*string, bool)`

GetGateIdOk returns a tuple with the GateId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetGateId

`func (o *CreateGateRequest) SetGateId(v string)`

SetGateId sets GateId field to given value.


### GetInconclusivePolicy

`func (o *CreateGateRequest) GetInconclusivePolicy() InconclusivePolicy`

GetInconclusivePolicy returns the InconclusivePolicy field if non-nil, zero value otherwise.

### GetInconclusivePolicyOk

`func (o *CreateGateRequest) GetInconclusivePolicyOk() (*InconclusivePolicy, bool)`

GetInconclusivePolicyOk returns a tuple with the InconclusivePolicy field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetInconclusivePolicy

`func (o *CreateGateRequest) SetInconclusivePolicy(v InconclusivePolicy)`

SetInconclusivePolicy sets InconclusivePolicy field to given value.

### HasInconclusivePolicy

`func (o *CreateGateRequest) HasInconclusivePolicy() bool`

HasInconclusivePolicy returns a boolean if a field has been set.

### SetInconclusivePolicyNil

`func (o *CreateGateRequest) SetInconclusivePolicyNil(b bool)`

 SetInconclusivePolicyNil sets the value for InconclusivePolicy to be an explicit nil

### UnsetInconclusivePolicy
`func (o *CreateGateRequest) UnsetInconclusivePolicy()`

UnsetInconclusivePolicy ensures that no value is present for InconclusivePolicy, not even an explicit nil
### GetName

`func (o *CreateGateRequest) GetName() string`

GetName returns the Name field if non-nil, zero value otherwise.

### GetNameOk

`func (o *CreateGateRequest) GetNameOk() (*string, bool)`

GetNameOk returns a tuple with the Name field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetName

`func (o *CreateGateRequest) SetName(v string)`

SetName sets Name field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
