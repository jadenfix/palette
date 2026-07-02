# ScenarioCluster

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**DominantFailureMode** | [**FailureMode**](FailureMode.md) | The most common failure mode across members. | 
**ExemplarTraceId** | **string** |  | 
**MemberTraceIds** | **[]string** | All member trace ids, sorted ascending. | 
**Signature** | [**Signature**](Signature.md) | The signature of the cluster&#39;s exemplar. | 
**Size** | **int32** | Number of member traces. | 

## Methods

### NewScenarioCluster

`func NewScenarioCluster(dominantFailureMode FailureMode, exemplarTraceId string, memberTraceIds []string, signature Signature, size int32, ) *ScenarioCluster`

NewScenarioCluster instantiates a new ScenarioCluster object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewScenarioClusterWithDefaults

`func NewScenarioClusterWithDefaults() *ScenarioCluster`

NewScenarioClusterWithDefaults instantiates a new ScenarioCluster object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetDominantFailureMode

`func (o *ScenarioCluster) GetDominantFailureMode() FailureMode`

GetDominantFailureMode returns the DominantFailureMode field if non-nil, zero value otherwise.

### GetDominantFailureModeOk

`func (o *ScenarioCluster) GetDominantFailureModeOk() (*FailureMode, bool)`

GetDominantFailureModeOk returns a tuple with the DominantFailureMode field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDominantFailureMode

`func (o *ScenarioCluster) SetDominantFailureMode(v FailureMode)`

SetDominantFailureMode sets DominantFailureMode field to given value.


### GetExemplarTraceId

`func (o *ScenarioCluster) GetExemplarTraceId() string`

GetExemplarTraceId returns the ExemplarTraceId field if non-nil, zero value otherwise.

### GetExemplarTraceIdOk

`func (o *ScenarioCluster) GetExemplarTraceIdOk() (*string, bool)`

GetExemplarTraceIdOk returns a tuple with the ExemplarTraceId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetExemplarTraceId

`func (o *ScenarioCluster) SetExemplarTraceId(v string)`

SetExemplarTraceId sets ExemplarTraceId field to given value.


### GetMemberTraceIds

`func (o *ScenarioCluster) GetMemberTraceIds() []string`

GetMemberTraceIds returns the MemberTraceIds field if non-nil, zero value otherwise.

### GetMemberTraceIdsOk

`func (o *ScenarioCluster) GetMemberTraceIdsOk() (*[]string, bool)`

GetMemberTraceIdsOk returns a tuple with the MemberTraceIds field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMemberTraceIds

`func (o *ScenarioCluster) SetMemberTraceIds(v []string)`

SetMemberTraceIds sets MemberTraceIds field to given value.


### GetSignature

`func (o *ScenarioCluster) GetSignature() Signature`

GetSignature returns the Signature field if non-nil, zero value otherwise.

### GetSignatureOk

`func (o *ScenarioCluster) GetSignatureOk() (*Signature, bool)`

GetSignatureOk returns a tuple with the Signature field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSignature

`func (o *ScenarioCluster) SetSignature(v Signature)`

SetSignature sets Signature field to given value.


### GetSize

`func (o *ScenarioCluster) GetSize() int32`

GetSize returns the Size field if non-nil, zero value otherwise.

### GetSizeOk

`func (o *ScenarioCluster) GetSizeOk() (*int32, bool)`

GetSizeOk returns a tuple with the Size field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSize

`func (o *ScenarioCluster) SetSize(v int32)`

SetSize sets Size field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


