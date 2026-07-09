# RunSummary

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**DurationMs** | Pointer to **NullableInt64** |  | [optional]
**EndedAt** | Pointer to **NullableTime** |  | [optional]
**FirstSpanName** | **string** |  |
**Models** | [**[]ModelRef**](ModelRef.md) |  |
**ProjectId** | **string** |  |
**ReleaseIds** | **[]string** |  |
**SpanCount** | **int32** |  |
**StartedAt** | **time.Time** |  |
**Status** | [**SpanStatus**](SpanStatus.md) |  |
**TenantId** | **string** |  |
**TotalCost** | Pointer to [**NullableMoney**](Money.md) |  | [optional]
**TraceId** | **string** |  |

## Methods

### NewRunSummary

`func NewRunSummary(firstSpanName string, models []ModelRef, projectId string, releaseIds []string, spanCount int32, startedAt time.Time, status SpanStatus, tenantId string, traceId string, ) *RunSummary`

NewRunSummary instantiates a new RunSummary object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewRunSummaryWithDefaults

`func NewRunSummaryWithDefaults() *RunSummary`

NewRunSummaryWithDefaults instantiates a new RunSummary object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetDurationMs

`func (o *RunSummary) GetDurationMs() int64`

GetDurationMs returns the DurationMs field if non-nil, zero value otherwise.

### GetDurationMsOk

`func (o *RunSummary) GetDurationMsOk() (*int64, bool)`

GetDurationMsOk returns a tuple with the DurationMs field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDurationMs

`func (o *RunSummary) SetDurationMs(v int64)`

SetDurationMs sets DurationMs field to given value.

### HasDurationMs

`func (o *RunSummary) HasDurationMs() bool`

HasDurationMs returns a boolean if a field has been set.

### SetDurationMsNil

`func (o *RunSummary) SetDurationMsNil(b bool)`

 SetDurationMsNil sets the value for DurationMs to be an explicit nil

### UnsetDurationMs
`func (o *RunSummary) UnsetDurationMs()`

UnsetDurationMs ensures that no value is present for DurationMs, not even an explicit nil
### GetEndedAt

`func (o *RunSummary) GetEndedAt() time.Time`

GetEndedAt returns the EndedAt field if non-nil, zero value otherwise.

### GetEndedAtOk

`func (o *RunSummary) GetEndedAtOk() (*time.Time, bool)`

GetEndedAtOk returns a tuple with the EndedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEndedAt

`func (o *RunSummary) SetEndedAt(v time.Time)`

SetEndedAt sets EndedAt field to given value.

### HasEndedAt

`func (o *RunSummary) HasEndedAt() bool`

HasEndedAt returns a boolean if a field has been set.

### SetEndedAtNil

`func (o *RunSummary) SetEndedAtNil(b bool)`

 SetEndedAtNil sets the value for EndedAt to be an explicit nil

### UnsetEndedAt
`func (o *RunSummary) UnsetEndedAt()`

UnsetEndedAt ensures that no value is present for EndedAt, not even an explicit nil
### GetFirstSpanName

`func (o *RunSummary) GetFirstSpanName() string`

GetFirstSpanName returns the FirstSpanName field if non-nil, zero value otherwise.

### GetFirstSpanNameOk

`func (o *RunSummary) GetFirstSpanNameOk() (*string, bool)`

GetFirstSpanNameOk returns a tuple with the FirstSpanName field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetFirstSpanName

`func (o *RunSummary) SetFirstSpanName(v string)`

SetFirstSpanName sets FirstSpanName field to given value.


### GetModels

`func (o *RunSummary) GetModels() []ModelRef`

GetModels returns the Models field if non-nil, zero value otherwise.

### GetModelsOk

`func (o *RunSummary) GetModelsOk() (*[]ModelRef, bool)`

GetModelsOk returns a tuple with the Models field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetModels

`func (o *RunSummary) SetModels(v []ModelRef)`

SetModels sets Models field to given value.


### GetProjectId

`func (o *RunSummary) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *RunSummary) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *RunSummary) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetReleaseIds

`func (o *RunSummary) GetReleaseIds() []string`

GetReleaseIds returns the ReleaseIds field if non-nil, zero value otherwise.

### GetReleaseIdsOk

`func (o *RunSummary) GetReleaseIdsOk() (*[]string, bool)`

GetReleaseIdsOk returns a tuple with the ReleaseIds field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetReleaseIds

`func (o *RunSummary) SetReleaseIds(v []string)`

SetReleaseIds sets ReleaseIds field to given value.


### GetSpanCount

`func (o *RunSummary) GetSpanCount() int32`

GetSpanCount returns the SpanCount field if non-nil, zero value otherwise.

### GetSpanCountOk

`func (o *RunSummary) GetSpanCountOk() (*int32, bool)`

GetSpanCountOk returns a tuple with the SpanCount field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSpanCount

`func (o *RunSummary) SetSpanCount(v int32)`

SetSpanCount sets SpanCount field to given value.


### GetStartedAt

`func (o *RunSummary) GetStartedAt() time.Time`

GetStartedAt returns the StartedAt field if non-nil, zero value otherwise.

### GetStartedAtOk

`func (o *RunSummary) GetStartedAtOk() (*time.Time, bool)`

GetStartedAtOk returns a tuple with the StartedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetStartedAt

`func (o *RunSummary) SetStartedAt(v time.Time)`

SetStartedAt sets StartedAt field to given value.


### GetStatus

`func (o *RunSummary) GetStatus() SpanStatus`

GetStatus returns the Status field if non-nil, zero value otherwise.

### GetStatusOk

`func (o *RunSummary) GetStatusOk() (*SpanStatus, bool)`

GetStatusOk returns a tuple with the Status field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetStatus

`func (o *RunSummary) SetStatus(v SpanStatus)`

SetStatus sets Status field to given value.


### GetTenantId

`func (o *RunSummary) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *RunSummary) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *RunSummary) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.


### GetTotalCost

`func (o *RunSummary) GetTotalCost() Money`

GetTotalCost returns the TotalCost field if non-nil, zero value otherwise.

### GetTotalCostOk

`func (o *RunSummary) GetTotalCostOk() (*Money, bool)`

GetTotalCostOk returns a tuple with the TotalCost field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTotalCost

`func (o *RunSummary) SetTotalCost(v Money)`

SetTotalCost sets TotalCost field to given value.

### HasTotalCost

`func (o *RunSummary) HasTotalCost() bool`

HasTotalCost returns a boolean if a field has been set.

### SetTotalCostNil

`func (o *RunSummary) SetTotalCostNil(b bool)`

 SetTotalCostNil sets the value for TotalCost to be an explicit nil

### UnsetTotalCost
`func (o *RunSummary) UnsetTotalCost()`

UnsetTotalCost ensures that no value is present for TotalCost, not even an explicit nil
### GetTraceId

`func (o *RunSummary) GetTraceId() string`

GetTraceId returns the TraceId field if non-nil, zero value otherwise.

### GetTraceIdOk

`func (o *RunSummary) GetTraceIdOk() (*string, bool)`

GetTraceIdOk returns a tuple with the TraceId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTraceId

`func (o *RunSummary) SetTraceId(v string)`

SetTraceId sets TraceId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
