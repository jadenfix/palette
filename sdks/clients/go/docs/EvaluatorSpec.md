# EvaluatorSpec

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Id** | **string** |  |
**Kind** | [**EvaluatorKind**](EvaluatorKind.md) |  |
**Lane** | [**EvaluatorLane**](EvaluatorLane.md) |  |

## Methods

### NewEvaluatorSpec

`func NewEvaluatorSpec(id string, kind EvaluatorKind, lane EvaluatorLane, ) *EvaluatorSpec`

NewEvaluatorSpec instantiates a new EvaluatorSpec object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewEvaluatorSpecWithDefaults

`func NewEvaluatorSpecWithDefaults() *EvaluatorSpec`

NewEvaluatorSpecWithDefaults instantiates a new EvaluatorSpec object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetId

`func (o *EvaluatorSpec) GetId() string`

GetId returns the Id field if non-nil, zero value otherwise.

### GetIdOk

`func (o *EvaluatorSpec) GetIdOk() (*string, bool)`

GetIdOk returns a tuple with the Id field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetId

`func (o *EvaluatorSpec) SetId(v string)`

SetId sets Id field to given value.


### GetKind

`func (o *EvaluatorSpec) GetKind() EvaluatorKind`

GetKind returns the Kind field if non-nil, zero value otherwise.

### GetKindOk

`func (o *EvaluatorSpec) GetKindOk() (*EvaluatorKind, bool)`

GetKindOk returns a tuple with the Kind field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetKind

`func (o *EvaluatorSpec) SetKind(v EvaluatorKind)`

SetKind sets Kind field to given value.


### GetLane

`func (o *EvaluatorSpec) GetLane() EvaluatorLane`

GetLane returns the Lane field if non-nil, zero value otherwise.

### GetLaneOk

`func (o *EvaluatorSpec) GetLaneOk() (*EvaluatorLane, bool)`

GetLaneOk returns a tuple with the Lane field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLane

`func (o *EvaluatorSpec) SetLane(v EvaluatorLane)`

SetLane sets Lane field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
