# ReviewTask

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**CreatedAt** | **time.Time** |  |
**DatasetCaseId** | Pointer to **string** |  | [optional]
**DatasetId** | Pointer to **string** |  | [optional]
**Priority** | **int64** |  |
**ProjectId** | **string** |  |
**QueueId** | **string** |  |
**SpanId** | Pointer to **string** |  | [optional]
**State** | [**ReviewTaskState**](ReviewTaskState.md) |  |
**TaskId** | **string** |  |
**TenantId** | **string** |  |
**TraceId** | **string** |  |
**UpdatedAt** | **time.Time** |  |

## Methods

### NewReviewTask

`func NewReviewTask(createdAt time.Time, priority int64, projectId string, queueId string, state ReviewTaskState, taskId string, tenantId string, traceId string, updatedAt time.Time, ) *ReviewTask`

NewReviewTask instantiates a new ReviewTask object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewReviewTaskWithDefaults

`func NewReviewTaskWithDefaults() *ReviewTask`

NewReviewTaskWithDefaults instantiates a new ReviewTask object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetCreatedAt

`func (o *ReviewTask) GetCreatedAt() time.Time`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *ReviewTask) GetCreatedAtOk() (*time.Time, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *ReviewTask) SetCreatedAt(v time.Time)`

SetCreatedAt sets CreatedAt field to given value.


### GetDatasetCaseId

`func (o *ReviewTask) GetDatasetCaseId() string`

GetDatasetCaseId returns the DatasetCaseId field if non-nil, zero value otherwise.

### GetDatasetCaseIdOk

`func (o *ReviewTask) GetDatasetCaseIdOk() (*string, bool)`

GetDatasetCaseIdOk returns a tuple with the DatasetCaseId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDatasetCaseId

`func (o *ReviewTask) SetDatasetCaseId(v string)`

SetDatasetCaseId sets DatasetCaseId field to given value.

### HasDatasetCaseId

`func (o *ReviewTask) HasDatasetCaseId() bool`

HasDatasetCaseId returns a boolean if a field has been set.

### GetDatasetId

`func (o *ReviewTask) GetDatasetId() string`

GetDatasetId returns the DatasetId field if non-nil, zero value otherwise.

### GetDatasetIdOk

`func (o *ReviewTask) GetDatasetIdOk() (*string, bool)`

GetDatasetIdOk returns a tuple with the DatasetId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDatasetId

`func (o *ReviewTask) SetDatasetId(v string)`

SetDatasetId sets DatasetId field to given value.

### HasDatasetId

`func (o *ReviewTask) HasDatasetId() bool`

HasDatasetId returns a boolean if a field has been set.

### GetPriority

`func (o *ReviewTask) GetPriority() int64`

GetPriority returns the Priority field if non-nil, zero value otherwise.

### GetPriorityOk

`func (o *ReviewTask) GetPriorityOk() (*int64, bool)`

GetPriorityOk returns a tuple with the Priority field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPriority

`func (o *ReviewTask) SetPriority(v int64)`

SetPriority sets Priority field to given value.


### GetProjectId

`func (o *ReviewTask) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *ReviewTask) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *ReviewTask) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetQueueId

`func (o *ReviewTask) GetQueueId() string`

GetQueueId returns the QueueId field if non-nil, zero value otherwise.

### GetQueueIdOk

`func (o *ReviewTask) GetQueueIdOk() (*string, bool)`

GetQueueIdOk returns a tuple with the QueueId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetQueueId

`func (o *ReviewTask) SetQueueId(v string)`

SetQueueId sets QueueId field to given value.


### GetSpanId

`func (o *ReviewTask) GetSpanId() string`

GetSpanId returns the SpanId field if non-nil, zero value otherwise.

### GetSpanIdOk

`func (o *ReviewTask) GetSpanIdOk() (*string, bool)`

GetSpanIdOk returns a tuple with the SpanId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSpanId

`func (o *ReviewTask) SetSpanId(v string)`

SetSpanId sets SpanId field to given value.

### HasSpanId

`func (o *ReviewTask) HasSpanId() bool`

HasSpanId returns a boolean if a field has been set.

### GetState

`func (o *ReviewTask) GetState() ReviewTaskState`

GetState returns the State field if non-nil, zero value otherwise.

### GetStateOk

`func (o *ReviewTask) GetStateOk() (*ReviewTaskState, bool)`

GetStateOk returns a tuple with the State field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetState

`func (o *ReviewTask) SetState(v ReviewTaskState)`

SetState sets State field to given value.


### GetTaskId

`func (o *ReviewTask) GetTaskId() string`

GetTaskId returns the TaskId field if non-nil, zero value otherwise.

### GetTaskIdOk

`func (o *ReviewTask) GetTaskIdOk() (*string, bool)`

GetTaskIdOk returns a tuple with the TaskId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTaskId

`func (o *ReviewTask) SetTaskId(v string)`

SetTaskId sets TaskId field to given value.


### GetTenantId

`func (o *ReviewTask) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *ReviewTask) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *ReviewTask) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.


### GetTraceId

`func (o *ReviewTask) GetTraceId() string`

GetTraceId returns the TraceId field if non-nil, zero value otherwise.

### GetTraceIdOk

`func (o *ReviewTask) GetTraceIdOk() (*string, bool)`

GetTraceIdOk returns a tuple with the TraceId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTraceId

`func (o *ReviewTask) SetTraceId(v string)`

SetTraceId sets TraceId field to given value.


### GetUpdatedAt

`func (o *ReviewTask) GetUpdatedAt() time.Time`

GetUpdatedAt returns the UpdatedAt field if non-nil, zero value otherwise.

### GetUpdatedAtOk

`func (o *ReviewTask) GetUpdatedAtOk() (*time.Time, bool)`

GetUpdatedAtOk returns a tuple with the UpdatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetUpdatedAt

`func (o *ReviewTask) SetUpdatedAt(v time.Time)`

SetUpdatedAt sets UpdatedAt field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
