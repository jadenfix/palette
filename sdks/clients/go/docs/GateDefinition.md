# GateDefinition

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**CreatedAt** | **time.Time** |  |
**DatasetId** | Pointer to **string** |  | [optional]
**EvaluatorVersionId** | Pointer to **string** |  | [optional]
**GateId** | **string** |  |
**InconclusivePolicy** | Pointer to [**InconclusivePolicy**](InconclusivePolicy.md) |  | [optional]
**Name** | **string** |  |
**ProjectId** | **string** |  |
**TenantId** | **string** |  |

## Methods

### NewGateDefinition

`func NewGateDefinition(createdAt time.Time, gateId string, name string, projectId string, tenantId string, ) *GateDefinition`

NewGateDefinition instantiates a new GateDefinition object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewGateDefinitionWithDefaults

`func NewGateDefinitionWithDefaults() *GateDefinition`

NewGateDefinitionWithDefaults instantiates a new GateDefinition object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetCreatedAt

`func (o *GateDefinition) GetCreatedAt() time.Time`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *GateDefinition) GetCreatedAtOk() (*time.Time, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *GateDefinition) SetCreatedAt(v time.Time)`

SetCreatedAt sets CreatedAt field to given value.


### GetDatasetId

`func (o *GateDefinition) GetDatasetId() string`

GetDatasetId returns the DatasetId field if non-nil, zero value otherwise.

### GetDatasetIdOk

`func (o *GateDefinition) GetDatasetIdOk() (*string, bool)`

GetDatasetIdOk returns a tuple with the DatasetId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDatasetId

`func (o *GateDefinition) SetDatasetId(v string)`

SetDatasetId sets DatasetId field to given value.

### HasDatasetId

`func (o *GateDefinition) HasDatasetId() bool`

HasDatasetId returns a boolean if a field has been set.

### GetEvaluatorVersionId

`func (o *GateDefinition) GetEvaluatorVersionId() string`

GetEvaluatorVersionId returns the EvaluatorVersionId field if non-nil, zero value otherwise.

### GetEvaluatorVersionIdOk

`func (o *GateDefinition) GetEvaluatorVersionIdOk() (*string, bool)`

GetEvaluatorVersionIdOk returns a tuple with the EvaluatorVersionId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEvaluatorVersionId

`func (o *GateDefinition) SetEvaluatorVersionId(v string)`

SetEvaluatorVersionId sets EvaluatorVersionId field to given value.

### HasEvaluatorVersionId

`func (o *GateDefinition) HasEvaluatorVersionId() bool`

HasEvaluatorVersionId returns a boolean if a field has been set.

### GetGateId

`func (o *GateDefinition) GetGateId() string`

GetGateId returns the GateId field if non-nil, zero value otherwise.

### GetGateIdOk

`func (o *GateDefinition) GetGateIdOk() (*string, bool)`

GetGateIdOk returns a tuple with the GateId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetGateId

`func (o *GateDefinition) SetGateId(v string)`

SetGateId sets GateId field to given value.


### GetInconclusivePolicy

`func (o *GateDefinition) GetInconclusivePolicy() InconclusivePolicy`

GetInconclusivePolicy returns the InconclusivePolicy field if non-nil, zero value otherwise.

### GetInconclusivePolicyOk

`func (o *GateDefinition) GetInconclusivePolicyOk() (*InconclusivePolicy, bool)`

GetInconclusivePolicyOk returns a tuple with the InconclusivePolicy field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetInconclusivePolicy

`func (o *GateDefinition) SetInconclusivePolicy(v InconclusivePolicy)`

SetInconclusivePolicy sets InconclusivePolicy field to given value.

### HasInconclusivePolicy

`func (o *GateDefinition) HasInconclusivePolicy() bool`

HasInconclusivePolicy returns a boolean if a field has been set.

### GetName

`func (o *GateDefinition) GetName() string`

GetName returns the Name field if non-nil, zero value otherwise.

### GetNameOk

`func (o *GateDefinition) GetNameOk() (*string, bool)`

GetNameOk returns a tuple with the Name field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetName

`func (o *GateDefinition) SetName(v string)`

SetName sets Name field to given value.


### GetProjectId

`func (o *GateDefinition) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *GateDefinition) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *GateDefinition) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetTenantId

`func (o *GateDefinition) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *GateDefinition) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *GateDefinition) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
