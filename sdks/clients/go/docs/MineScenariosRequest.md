# MineScenariosRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**JaccardThreshold** | Pointer to **NullableFloat64** |  | [optional]
**TraceIds** | **[]string** |  |

## Methods

### NewMineScenariosRequest

`func NewMineScenariosRequest(traceIds []string, ) *MineScenariosRequest`

NewMineScenariosRequest instantiates a new MineScenariosRequest object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewMineScenariosRequestWithDefaults

`func NewMineScenariosRequestWithDefaults() *MineScenariosRequest`

NewMineScenariosRequestWithDefaults instantiates a new MineScenariosRequest object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetJaccardThreshold

`func (o *MineScenariosRequest) GetJaccardThreshold() float64`

GetJaccardThreshold returns the JaccardThreshold field if non-nil, zero value otherwise.

### GetJaccardThresholdOk

`func (o *MineScenariosRequest) GetJaccardThresholdOk() (*float64, bool)`

GetJaccardThresholdOk returns a tuple with the JaccardThreshold field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetJaccardThreshold

`func (o *MineScenariosRequest) SetJaccardThreshold(v float64)`

SetJaccardThreshold sets JaccardThreshold field to given value.

### HasJaccardThreshold

`func (o *MineScenariosRequest) HasJaccardThreshold() bool`

HasJaccardThreshold returns a boolean if a field has been set.

### SetJaccardThresholdNil

`func (o *MineScenariosRequest) SetJaccardThresholdNil(b bool)`

 SetJaccardThresholdNil sets the value for JaccardThreshold to be an explicit nil

### UnsetJaccardThreshold
`func (o *MineScenariosRequest) UnsetJaccardThreshold()`

UnsetJaccardThreshold ensures that no value is present for JaccardThreshold, not even an explicit nil
### GetTraceIds

`func (o *MineScenariosRequest) GetTraceIds() []string`

GetTraceIds returns the TraceIds field if non-nil, zero value otherwise.

### GetTraceIdsOk

`func (o *MineScenariosRequest) GetTraceIdsOk() (*[]string, bool)`

GetTraceIdsOk returns a tuple with the TraceIds field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTraceIds

`func (o *MineScenariosRequest) SetTraceIds(v []string)`

SetTraceIds sets TraceIds field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
